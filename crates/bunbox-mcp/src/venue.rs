//! A single disposable VM for the pinned recipe. Lifecycle commands never run
//! caller-selected programs. Raw messages flow on a separately bounded relay.
use crate::model::{Input, MAX_RESULT_BYTES, RECIPE, ResultRows, digest};
use anyhow::{Context, Result, bail, ensure};
use microsandbox_agent_client::{AgentClient, ConnectOptions, TypedMessage};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::Command,
    time::{sleep, timeout},
};

const MSB_PIN: &str = "b7292a901ffc93f84860264c6e0d4636cc9aeeddfb76c952f7c2204ea417ae4b";
const LIB_PIN: &str = "43e36ee2b1f2a7488c25f34193f657568a7733281d856ad267506ccc02993d59";
const IMAGE: &str =
    "docker.io/oven/bun@sha256:3121e24dc54514f0e37bcc996a9e6df64519b4caff03a33bbb9993baca7c403b";
pub struct Venue {
    binary: PathBuf,
    home: PathBuf,
}

fn hash_file(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    std::io::copy(&mut std::fs::File::open(path)?, &mut h)?;
    Ok(hex::encode(h.finalize()))
}

impl Venue {
    pub fn open() -> Result<Self> {
        ensure!(
            cfg!(all(target_os = "macos", target_arch = "aarch64")),
            "unsupported_venue"
        );
        let binary =
            std::fs::canonicalize(std::env::var_os("BUNBOX_MSB").context("msb_not_configured")?)?;
        let home =
            std::fs::canonicalize(std::env::var_os("MSB_HOME").context("msb_home_required")?)?;
        ensure!(
            hash_file(&binary)? == MSB_PIN
                && hash_file(&binary.with_file_name("libkrunfw.5.dylib"))? == LIB_PIN,
            "runtime_pin_mismatch"
        );
        Ok(Self { binary, home })
    }

    async fn cli(&self, args: &[&str], seconds: u64) -> Result<String> {
        let mut child = Command::new(&self.binary)
            .args(args)
            .env_clear()
            .env("MSB_HOME", &self.home)
            .env("HOME", &self.home)
            .env(
                "DYLD_LIBRARY_PATH",
                self.binary.parent().context("invalid_runtime_path")?,
            )
            .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
            .current_dir(&self.home)
            // The worker's stdin owns its execution lease. The create process
            // retains that lease if the worker exits while boot is in flight.
            .stdin(if args.first() == Some(&"create") {
                Stdio::inherit()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;
        let out = child.stdout.take().context("no_stdout")?;
        let err = child.stderr.take().context("no_stderr")?;
        async fn bounded(mut r: impl AsyncRead + Unpin) -> Result<Vec<u8>> {
            let mut v = vec![];
            (&mut r).take(32769).read_to_end(&mut v).await?;
            ensure!(v.len() <= 32768, "lifecycle_output_limit");
            Ok(v)
        }
        let run = timeout(Duration::from_secs(seconds), async {
            let (stdout, _stderr, status) = tokio::try_join!(bounded(out), bounded(err), async {
                Ok(child.wait().await?)
            })?;
            ensure!(status.success(), "lifecycle_command_failed");
            Ok(String::from_utf8(stdout)?)
        })
        .await;
        match run {
            Ok(Ok(s)) => Ok(s),
            other => {
                let _ = child.kill().await;
                match other {
                    Ok(r) => r,
                    Err(_) => bail!("lifecycle_timeout"),
                }
            }
        }
    }

    pub async fn present(&self, name: &str) -> Result<bool> {
        // The independent list only takes names from this installation's registry.
        Ok(self
            .cli(&["list", "--quiet"], 10)
            .await?
            .lines()
            .any(|n| n.trim() == name))
    }

    pub async fn cleanup(&self, name: &str, creating: bool) -> Result<()> {
        let present = self.present(name).await?;
        ensure!(present || !creating, "creation_unconfirmed");
        if present {
            self.cli(&["remove", "--force", "--quiet", name], 20)
                .await?;
        }
        ensure!(!self.present(name).await?, "cleanup_unknown");
        // The guest relay is a second host observation, separate from registry state.
        match std::os::unix::net::UnixStream::connect(self.socket(name)) {
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
                ) => {}
            _ => bail!("cleanup_unknown"),
        }
        Ok(())
    }

    fn socket(&self, name: &str) -> PathBuf {
        self.home
            .join("run/sandboxes")
            .join(&digest(name)[..24])
            .join("agent.sock")
    }

    pub async fn create(&self, name: &str) -> Result<()> {
        self.cli(
            &[
                "create",
                IMAGE,
                "--name",
                name,
                "--cpus",
                "1",
                "--memory",
                "256M",
                "--root-disk",
                "1G",
                "--pull",
                "never",
                "--no-net",
                "--quiet",
            ],
            60,
        )
        .await?;
        ensure!(self.present(name).await?, "create_not_observed");
        Ok(())
    }

    pub async fn execute(&self, name: &str, input: &Input, cancelled: &Path) -> Result<ResultRows> {
        let path = self.socket(name);
        let work = guest_call(path, input);
        let cancel = async {
            while !cancelled.exists() {
                sleep(Duration::from_millis(75)).await;
            }
        };
        tokio::select! {
            result = timeout(Duration::from_secs(30),work) => result.context("guest_timeout")?,
            _ = cancel => bail!("cancelled"),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Data {
    #[serde(with = "serde_bytes")]
    data: Vec<u8>,
}
#[derive(Deserialize)]
struct Exit {
    code: i32,
}

async fn guest_call(socket: PathBuf, input: &Input) -> Result<ResultRows> {
    let options = ConnectOptions::default()
        .setup_timeout(Duration::from_secs(4))
        .limits(|mut l| {
            l.max_frame_size = 65536;
            l.max_in_flight = 1;
            l.queued_writes = 2;
            l.queued_responses = 8;
            l.buffered_bytes = 262144;
            l.incomplete_frame_timeout = Some(Duration::from_secs(3));
            l.request_timeout = Some(Duration::from_secs(4));
            l
        });
    let agent = AgentClient::connect_with(socket, |_| options).await?;
    let request = json!({"cmd":"/usr/local/bin/bun", "args":["--eval",RECIPE],
        "env":["HOME=/tmp","PATH=/usr/local/bin:/usr/bin:/bin","BUN_INSTALL_CACHE_DIR=/tmp"],
        "cwd":"/tmp","tty":false,
        "rlimits":[{"resource":"nproc","soft":64,"hard":64},{"resource":"nofile","soft":64,"hard":64},
        {"resource":"cpu","soft":20,"hard":25},{"resource":"fsize","soft":65536,"hard":65536}]});
    let stream = agent
        .stream(TypedMessage::new("core.exec.request", request))
        .await?;
    let (writer, mut reader) = stream.into_parts();
    let bytes = serde_json::to_vec(input)?;
    let send = async {
        for part in bytes.chunks(8192) {
            writer
                .send(TypedMessage::new(
                    "core.exec.stdin",
                    Data {
                        data: part.to_vec(),
                    },
                ))
                .await?;
        }
        writer
            .send(TypedMessage::new("core.exec.stdin", Data { data: vec![] }))
            .await?;
        Ok::<(), anyhow::Error>(())
    };
    let receive = async {
        let mut out = Vec::new();
        let mut total = 0;
        let mut events = 0;
        loop {
            let m = reader.recv().await?.context("guest_missing_exit")?;
            events += 1;
            ensure!(events <= 512, "guest_event_limit");
            match m.t.as_str() {
                "core.exec.started" => {}
                "core.exec.stdout" | "core.exec.stderr" => {
                    let data: Data = m.payload()?;
                    total += data.data.len();
                    ensure!(total <= MAX_RESULT_BYTES, "guest_output_limit");
                    if m.t == "core.exec.stdout" {
                        out.extend(data.data);
                    }
                }
                "core.exec.exited" => {
                    ensure!(m.payload::<Exit>()?.code == 0, "guest_failed");
                    let decoded: ResultRows = serde_json::from_slice(&out)?;
                    input.check_result(&decoded)?;
                    return Ok(decoded);
                }
                _ => bail!("unexpected_guest_event"),
            }
        }
    };
    let (_, result) = tokio::try_join!(send, receive)?;
    Ok(result)
}
