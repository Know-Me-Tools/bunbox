# BunBox execution contract

Status: target contract, **not an implemented interpreter**. Phase 0 is blocked on a bounded Microsandbox transport. The current executable reports readiness; it does not accept or execute code.

## Intended first release

Local single-owner service on this ARM64 Mac, with a Rust control plane and one Microsandbox VM per session running Bun. Linux and Windows require independent real-host validation. MCP connections are not execution sessions. Network/shared-server deployment is deferred.

Support JavaScript and TypeScript, `await` inside an async wrapper, explicit `return`, curated version-pinned dynamic imports, scoped files, structured errors, bounded stdout/stderr, and published artifacts. Static imports are not initially promised. Transformation and source mapping must be tested against the pinned guest Bun version; the host Bun version is not guest evidence.

## Three different persistence lifetimes

| State | Lifetime | Failure behavior |
| --- | --- | --- |
| Explicit `state` object | Healthy live worker only | Lost on reset, hard cancellation, timeout or VM/worker failure |
| Guest workspace | Current configured guest volume | Unexported files may be lost when the VM is destroyed |
| Published artifacts | Independent retention policy | Only finalized, host-stored objects receive durability guarantees |

Top-level declarations are execution-local. A failed execution may already have changed `state`, written files, or produced side effects. Execution is **not transactional**. Resetting runtime memory and deleting retained artifacts are separate operations.

## Limits and cancellation

Enforce source, frame, result, output, event-count, queue, session, CPU, memory, process, disk, artifact and time budgets. Bounds must hold before allocations, including inside third-party transports. A byte cap applied after an SDK buffers output is insufficient.

Queued cancellation prevents dispatch. Running cancellation or timeout invalidates the session and destroys its VM. Cancellation is complete only after host-side termination confirmation. If cleanup cannot be confirmed, expose cleanup-pending and quarantine the capacity. Never interpret dropping a future as stopping a VM.

Only one request is dispatched per session at a time; this does not guarantee absence of residual guest callbacks or subprocesses. Background services are unsupported. VM-wide quotas and idle deadlines remain active between requests, and late output is attributed conservatively.

## Failure and recovery

Persist accepted execution identity before dispatch. Reusing an owner/operation/idempotency key with a different payload is an error. Client disconnects do not implicitly replay or cancel accepted work; polling returns authoritative status.

After host restart, reconcile installation-owned resources before admission. Uncertain executions become `OutcomeUnknown`, never automatic retries or inferred successes. Hold an exclusive installation lock to prevent two control planes from reconciling each other's live resources.

## Exclusions

No arbitrary package installation, public inbound networking, arbitrary host mounts, host commands, host credential inheritance, Docker socket, live-state migration, transparent in-memory recovery, or unsandboxed fallback. Helpers such as `workspace` and `artifacts` improve ergonomics but are not security boundaries.

Expose implemented capabilities and exclusions through the eventual MCP environment description. Do not advertise target capabilities as currently available.

**Separately proposed profile:** [BunBox as an external tool for Locus](locus-external-tool.md) adds authenticated cloud job and file contracts. It does not change this single-owner profile, the no-fallback rule or the local qualification gate.
