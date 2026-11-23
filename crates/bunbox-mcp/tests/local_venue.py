"""Run the real staged STDIO service against a dedicated, provisioned Microsandbox home.

Usage: BUNBOX_MSB=/.../msb MSB_HOME=/... python3 local_venue.py /.../bunbox-mcp
"""
import contextlib
import hashlib
import json
import os
from pathlib import Path
import selectors
import signal
import subprocess
import sys
import tempfile
import time
import uuid

exe = Path(sys.argv[1]).resolve(strict=True)
recipe = Path(__file__).resolve().parents[3] / "runtime/recipes/unanswered-demo-v1.ts"
pin = hashlib.sha256(recipe.read_bytes()).hexdigest()


class Connection:
    def __init__(self, process):
        self.process = process
        self.seq = 0

    def message(self, method, params, notify=False):
        self.seq += 1
        msg = {"jsonrpc": "2.0", "method": method, "params": params}
        if not notify:
            msg["id"] = self.seq
        self.process.stdin.write(json.dumps(msg).encode() + b"\n")
        self.process.stdin.flush()
        if notify:
            return
        with selectors.DefaultSelector() as sel:
            sel.register(self.process.stdout, selectors.EVENT_READ)
            assert sel.select(30), "no MCP response"
            line = self.process.stdout.readline(131073)
            assert 0 < len(line) <= 131072, "invalid MCP response"
        answer = json.loads(line)
        assert answer["id"] == self.seq and "result" in answer, answer.keys()
        return answer["result"]

    def tool(self, name, **arguments):
        reply = self.message("tools/call", {"name": name, "arguments": arguments})
        return reply["structuredContent"]


@contextlib.contextmanager
def connect(root, owner="operator", run="trial-1", msb_home=None):
    env = {**os.environ, "BUNBOX_OWNER": owner, "BUNBOX_RUN": run,
           "BUNBOX_STATE_DIR": str(root)}
    if msb_home is not None:
        env["MSB_HOME"] = str(msb_home)
    with subprocess.Popen([exe], stdin=subprocess.PIPE, stdout=subprocess.PIPE, env=env) as p:
        connection = Connection(p)
        try:
            connection.message("initialize", {"protocolVersion": "2025-06-18",
                "capabilities": {}, "clientInfo": {"name": "bunbox-local-pilot", "version": "1"}})
            connection.message("notifications/initialized", {}, notify=True)
            yield connection
        finally:
            p.stdin.close()
            try:
                p.wait(timeout=5)
            except subprocess.TimeoutExpired:
                p.kill()


def inventory():
    msb = Path(os.environ["BUNBOX_MSB"]).resolve()
    run = subprocess.run([msb, "list", "--quiet"], env={
        "HOME": os.environ["MSB_HOME"], "MSB_HOME": os.environ["MSB_HOME"],
        "PATH": "/usr/bin:/bin", "DYLD_LIBRARY_PATH": str(msb.parent),
    }, capture_output=True, text=True, check=True, timeout=10)
    return sorted(run.stdout.splitlines())


def wait_terminal(client, job_id):
    deadline = time.monotonic() + 100
    while time.monotonic() < deadline:
        result = client.tool("get", jobId=job_id)
        if result.get("status") not in {"accepted", "running"}:
            return result
        time.sleep(0.1)
    raise TimeoutError("job remained pending")


def main():
    before = inventory()
    root = Path(tempfile.mkdtemp(prefix="bunbox-ledger-"))
    def request(key, coverage=True):
        return {"requestKey": key, "recipeSha256": pin, "input": {
            "coverage": {"inbox": True, "sent": coverage}, "cutoff": 500,
            "rows": [{"sourceRef":"s1", "threadRef":"t1", "timestamp":1,
                      "direction":"inbound", "subject":"Can I book a demo?"}]}}
    key = uuid.uuid4().hex
    with connect(root) as client:
        assert {t["name"] for t in client.message("tools/list", {})["tools"]} == {"submit","get","cancel"}
        wrong = dict(request(key), recipeSha256="0"*64)
        assert client.tool("submit", **wrong)["reason"] == "unreviewed_recipe"
        accepted = client.tool("submit", **request(key))
        assert accepted.get("status") == "accepted", accepted
        jid = accepted["jobId"]
        assert client.tool("submit", **request(key))["jobId"] == jid
        assert client.tool("submit", **request(key, False))["reason"] == "request_key_conflict"
    for owner, run in [("other", "trial-1"), ("operator","other-run")]:
        with connect(root, owner, run) as other:
            for operation in ["get","cancel"]:
                assert other.tool(operation, jobId=jid)["reason"] == "unknown_job"
    with connect(root) as client:
        ok = wait_terminal(client, jid)
        assert ok.get("status") == "succeeded", ok
        assert ok["result"] == {"unanswered":[{"threadRef":"t1","sourceRef":"s1"}],
            "unassigned":[],"coverage":"complete","truncated":False}
        partial = client.tool("submit", **request(key+"p", False))
        result = wait_terminal(client, partial["jobId"])
        assert result.get("status") == "succeeded", result
        assert result["result"]["coverage"] == "partial" and result["result"]["unanswered"] == []
        third = client.tool("submit", **request(key+"c"))
        client.tool("cancel", jobId=third["jobId"])
        cancelled = wait_terminal(client, third["jobId"])
        assert cancelled.get("status") == "cancelled", cancelled
        assert client.tool("submit", **request(key))["status"] == "succeeded"
        interrupted = client.tool("submit", **request(key+"i"))
        interrupted_id = interrupted["jobId"]
        # Select only our own service's detached child for this unique job.
        ps = subprocess.run(["ps","-axo","pid=,ppid=,command="], capture_output=True,
                            text=True, check=True, timeout=5)
        children = []
        for line in ps.stdout.splitlines():
            fields = line.strip().split(None, 2)
            if len(fields) == 3 and fields[1] == str(client.process.pid) and fields[2] == f"{exe} --worker {interrupted_id}":
                children.append(int(fields[0]))
        assert len(children) == 1, "expected one worker owned by this test"
        os.kill(children[0], signal.SIGKILL)
        lost = wait_terminal(client, interrupted_id)
        assert lost["status"] == "outcome_unknown", lost
        assert client.tool("submit", **request(key+"i"))["jobId"] == interrupted_id
    # A never-pull request against a private, empty real runtime store cannot
    # start the pinned image. When no creation was acknowledged, the service
    # must retain capacity and refuse later admissions even after reconnect.
    missing_ledger = Path(tempfile.mkdtemp(prefix="bunbox-uncertain-"))
    with tempfile.TemporaryDirectory(prefix="bb-",dir="/private/tmp") as empty_home:
        with connect(missing_ledger, msb_home=empty_home) as client:
            uncertain = client.tool("submit", **request(key+"u"))
            final = wait_terminal(client, uncertain["jobId"])
            assert final["status"] == "cleanup_unknown", final
        with connect(missing_ledger, msb_home=empty_home) as client:
            assert client.tool("get", jobId=uncertain["jobId"])["status"] == "cleanup_unknown"
            assert client.tool("submit", **request(key+"new"))["reason"] == "venue_reconciliation_required"
            assert client.tool("submit", **request(key+"u"))["jobId"] == uncertain["jobId"]
            assert (missing_ledger/"active-job").exists()
            assert not list(missing_ledger.rglob("input.json"))
    assert before == inventory()
    assert not (root/"active-job").exists() and not list(root.rglob("input.json"))
    print(json.dumps({"ledger":str(root),"profile":"Darwin arm64 local VM","recipeSha256":pin,
        "firstResultSha256":ok["resultSha256"],"statuses":[ok["status"],result["status"],cancelled["status"]],
        "workerInterrupted":lost["status"],"unconfirmedCreation":final["status"],
        "unknownCapacityHeldAfterRestart":True, "unknownLedger":str(missing_ledger),
        "ownerAndRunDenied":True,"stdioRestartRecovered":True,"inventoryRestored":True}, indent=2))


if __name__=="__main__":
    main()
