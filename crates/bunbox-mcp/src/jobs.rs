//! Durable local ledger. The detached worker inherits the sole execution lease
//! over stdin. A process exit drops that lock; a retained marker still reserves
//! the venue until the named VM's removal is independently confirmed.
use crate::model::{Input, RECIPE, ResultRows, Submit, bounded_id, digest};
use crate::venue::Venue;
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::os::unix::process::CommandExt;
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::Write,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Instant,
};

#[derive(Clone)]
pub struct Store {
    root: PathBuf,
    scope: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Job {
    job_id: String,
    vm: String,
    input_sha256: String,
    recipe_sha256: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<ResultRows>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    elapsed_ms: Option<u64>,
}
impl Job {
    fn receipt(&self) -> Result<Value> {
        let mut value = serde_json::to_value(self)?;
        value.as_object_mut().context("invalid_job")?.remove("vm");
        Ok(value)
    }
    fn pending(&self) -> bool {
        matches!(
            self.status.as_str(),
            "accepted" | "running" | "cleanup_unknown"
        )
    }
}
fn private_dir(p: &Path) -> Result<()> {
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(p)?;
    let m = fs::symlink_metadata(p)?;
    ensure!(
        m.file_type().is_dir() && m.permissions().mode() & 0o077 == 0,
        "state_must_be_private"
    );
    Ok(())
}
fn atomic(p: &Path, bytes: &[u8]) -> Result<()> {
    let name = p
        .file_name()
        .context("invalid_record_name")?
        .to_string_lossy();
    let temp = p.with_file_name(format!(".{name}.tmp"));
    let mut f = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .mode(0o600)
        .open(&temp)?;
    f.write_all(bytes)?;
    f.sync_all()?;
    fs::rename(temp, p)?;
    File::open(p.parent().context("invalid_record_parent")?)?.sync_all()?;
    Ok(())
}
fn read_job(dir: &Path) -> Result<Job> {
    serde_json::from_slice(&fs::read(dir.join("receipt.json")).context("unknown_job")?)
        .context("invalid_job")
}
fn record(dir: &Path, job: &Job) -> Result<()> {
    atomic(&dir.join("receipt.json"), &serde_json::to_vec(job)?)
}
fn unlink(p: &Path) -> Result<()> {
    match fs::remove_file(p) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        r => Ok(r?),
    }
}
pub fn valid_job(id: &str) -> bool {
    id.len() == 64 && id.bytes().all(|b| matches!(b,b'0'..=b'9'|b'a'..=b'f'))
}

impl Store {
    pub fn open() -> Result<Self> {
        let root = PathBuf::from(env::var_os("BUNBOX_STATE_DIR").context("state_dir_required")?);
        ensure!(root.is_absolute(), "absolute_state_dir_required");
        private_dir(&root)?;
        let owner = env::var("BUNBOX_OWNER").context("owner_required")?;
        let run = env::var("BUNBOX_RUN").context("run_required")?;
        ensure!(bounded_id(&owner) && bounded_id(&run), "invalid_authority");
        let scope = digest(serde_json::to_vec(&(owner, run))?);
        let root = fs::canonicalize(root)?;
        private_dir(&root.join(&scope))?;
        Ok(Self { root, scope })
    }

    fn path(&self, id: &str) -> Result<PathBuf> {
        ensure!(valid_job(id), "unknown_job");
        Ok(self.root.join(&self.scope).join(id))
    }
    fn lease(&self) -> Result<File> {
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(self.root.join("lease"))?;
        f.try_lock().map_err(|_| anyhow::anyhow!("venue_busy"))?;
        Ok(f)
    }
    fn marker(&self) -> PathBuf {
        self.root.join("active-job")
    }
    fn active_for(&self, id: &str) -> String {
        format!("{}/{}", self.scope, id)
    }

    // Called under the exclusive lease after observing teardown. Release the
    // reservation before publishing a terminal receipt. A crash between these
    // writes leaves a recoverable pending outcome rather than stranded capacity.
    fn finish(&self, dir: &Path, job: &Job, cleaned: bool) -> Result<()> {
        if cleaned {
            unlink(&self.marker())?;
            File::open(&self.root)?.sync_all()?;
        }
        record(dir, job)
    }

    pub fn submit(&self, request: Submit) -> Result<Value> {
        ensure!(bounded_id(&request.request_key), "invalid_request_key");
        ensure!(request.recipe_sha256 == digest(RECIPE), "unreviewed_recipe");
        request.input.validate()?;
        let input = serde_json::to_vec(&request.input)?;
        let input_hash = digest(&input);
        let id = digest(serde_json::to_vec(&(&self.scope, &request.request_key))?);
        let dir = self.path(&id)?;
        let existing = || -> Result<Option<Value>> {
            if !dir.join("receipt.json").exists() {
                return Ok(None);
            }
            let prior = read_job(&dir)?;
            ensure!(
                prior.input_sha256 == input_hash && prior.recipe_sha256 == request.recipe_sha256,
                "request_key_conflict"
            );
            Ok(Some(prior.receipt()?))
        };
        if let Some(v) = existing()? {
            return Ok(v);
        }
        let lease = self.lease()?;
        if let Some(v) = existing()? {
            return Ok(v);
        }
        ensure!(!self.marker().exists(), "venue_reconciliation_required");
        let _verified = Venue::open().context("venue_unavailable")?;
        private_dir(&dir)?;
        let vm = format!(
            "bb-{}",
            &digest(serde_json::to_vec(&(&self.root, &id))?)[..24]
        );
        let mut job = Job {
            job_id: id.clone(),
            vm,
            input_sha256: input_hash,
            recipe_sha256: request.recipe_sha256,
            status: "accepted".into(),
            reason: None,
            result: None,
            result_sha256: None,
            elapsed_ms: None,
        };
        atomic(&dir.join("input.json"), &input)?;
        // Reserve before publishing acceptance. A crash after this marker is
        // always reconciled explicitly; no accepted operation is replayed.
        record(&dir, &job)?;
        atomic(&self.marker(), self.active_for(&id).as_bytes())?;
        let mut command = Command::new(env::current_exe()?);
        command.env_clear().process_group(0);
        for key in [
            "BUNBOX_STATE_DIR",
            "BUNBOX_OWNER",
            "BUNBOX_RUN",
            "BUNBOX_MSB",
            "MSB_HOME",
        ] {
            if let Some(v) = env::var_os(key) {
                command.env(key, v);
            }
        }
        let spawn = command
            .arg("--worker")
            .arg(&id)
            .stdin(Stdio::from(lease.try_clone()?))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        if let Ok(mut child) = spawn {
            std::thread::spawn(move || {
                let _ = child.wait();
            });
        } else {
            job.status = "failed".into();
            job.reason = Some("worker_start_failed".into());
            unlink(&dir.join("input.json"))?;
            self.finish(&dir, &job, true)?;
        }
        job.receipt()
    }

    pub async fn get(&self, id: &str, cancel: bool) -> Result<Value> {
        let dir = self.path(id)?;
        let job = read_job(&dir)?;
        if !job.pending() {
            return job.receipt();
        }
        if cancel {
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(false)
                .mode(0o600)
                .open(dir.join("cancel"))?;
        }
        // A live worker holds this lease across start, execution and cleanup.
        let Ok(_lease) = self.lease() else {
            return read_job(&dir)?.receipt();
        };
        let mut job = read_job(&dir)?;
        if !job.pending() {
            return job.receipt();
        }
        let marker = fs::read_to_string(self.marker()).ok();
        if marker.as_ref().is_some_and(|m| m != &self.active_for(id)) {
            return job.receipt();
        }
        let cleaned = match Venue::open() {
            Ok(v) => v
                .cleanup(&job.vm, dir.join("creating").exists())
                .await
                .is_ok(),
            Err(_) => false,
        };
        unlink(&dir.join("input.json"))?;
        job.reason = Some("worker_interrupted".into());
        job.status = if cleaned {
            "outcome_unknown"
        } else {
            "cleanup_unknown"
        }
        .into();
        self.finish(&dir, &job, cleaned && marker.is_some())?;
        job.receipt()
    }

    pub async fn worker(&self, id: &str) -> Result<()> {
        let dir = self.path(id)?;
        ensure!(
            fs::read_to_string(self.marker())? == self.active_for(id),
            "wrong_reservation"
        );
        let mut job = read_job(&dir)?;
        ensure!(job.status == "accepted", "job_not_accepted");
        job.status = "running".into();
        record(&dir, &job)?;
        let start = Instant::now();
        let venue = Venue::open()?;
        let cancel = dir.join("cancel");
        // Only the trusted launcher can run workers. A stale named VM must
        // never be reused. Reconciliation owns removal of an earlier resource.
        if venue.present(&job.vm).await? {
            bail!("unexpected_existing_vm")
        }
        let task = async {
            ensure!(!cancel.exists(), "cancelled");
            let bytes = fs::read(dir.join("input.json"))?;
            ensure!(digest(&bytes) == job.input_sha256, "staged_input_changed");
            let input: Input = serde_json::from_slice(&bytes)?;
            ensure!(job.recipe_sha256 == digest(RECIPE), "recipe_changed");
            input.validate()?;
            // Persist the intent before handing creation to an external process.
            // If either process exits here, absence alone cannot establish
            // that an asynchronously starting VM will stay absent.
            atomic(&dir.join("creating"), b"1")?;
            venue.create(&job.vm).await?;
            unlink(&dir.join("creating"))?;
            File::open(&dir)?.sync_all()?;
            ensure!(!cancel.exists(), "cancelled");
            venue.execute(&job.vm, &input, &cancel).await
        }
        .await;
        let cleaned = venue
            .cleanup(&job.vm, dir.join("creating").exists())
            .await
            .is_ok();
        unlink(&dir.join("input.json"))?;
        job.elapsed_ms = Some(start.elapsed().as_millis().min(u128::from(u64::MAX)) as u64);
        if !cleaned {
            job.status = "cleanup_unknown".into();
            job.reason = Some("teardown_unconfirmed".into());
        } else if cancel.exists() {
            job.status = "cancelled".into();
        } else {
            match task {
                Ok(result) => {
                    job.status = "succeeded".into();
                    job.result_sha256 = Some(digest(serde_json::to_vec(&result)?));
                    job.result = Some(result);
                }
                Err(e) => {
                    job.status = "failed".into();
                    job.reason = Some(
                        match e.to_string().as_str() {
                            "guest_output_limit" => "guest_output_limit",
                            "guest_timeout" => "guest_timeout",
                            "guest_failed" => "guest_failed",
                            "create_not_observed" | "lifecycle_command_failed" => {
                                "vm_lifecycle_failed"
                            }
                            _ => "execution_failed",
                        }
                        .into(),
                    );
                }
            }
        }
        self.finish(&dir, &job, cleaned)?;
        Ok(())
    }
}
pub fn failure(e: &anyhow::Error) -> Value {
    let reason = e.to_string();
    let stable = match reason.as_str() {
        "request_key_conflict"
        | "unreviewed_recipe"
        | "venue_busy"
        | "venue_unavailable"
        | "venue_reconciliation_required"
        | "invalid_input"
        | "invalid_row"
        | "source_ref_conflict"
        | "input_limit"
        | "invalid_request_key"
        | "unknown_job" => &reason,
        _ => "invalid_request_or_state",
    };
    json!({"reason":stable})
}
