# BunBox system design

## Status
Target design only; Phase 0 is **BLOCKED**. Dependency-free preflight and
length-prefix framing are implemented and locally validated (see `verification.md`).
VM/SDK integration remains unvalidated. No guest executes today.
Refer to `execution-contract.md` for behavior and `threat-model.md` for required controls.
The first runtime acceptance gate is a real VM on the user's ARM64 Mac.

## Planned request lifecycle
1. Validate ownership, source size, operation identity, policy, and admission capacity.
2. Persist accepted identity before dispatch; reject key reuse with a different payload.
3. Persist creation intent before provisioning any new installation-owned VM.
4. Admit only one dispatched request per session and enforce a host-owned deadline.
5. Validate bounded guest events against execution identity, generation, and sequence.
6. Finalize a bounded result without assuming guest completion proves host-side cleanup.

This lifecycle is not implemented by the preflight CLI. Do not expose accepted work
until durable acceptance and recovery exist. An initial VM probe is not an interpreter.

## Framing and resource budgets
- Check a declared length before payload allocation; reject oversized and truncated frames.
- Apply independent source, result, stdout/stderr, frame, event-count, and queue budgets.
- Cover upstream transport buffers and collection paths, not only application queues.
- Bound sessions, CPU, memory, processes, disk, artifacts, and execution/idle time separately.
- Guest-side serialization consumes the guest budget; host deadlines cannot be reset by events.
- Final wire schema and numerical budgets require implementation review and measured tests.

## Cancellation and uncertain outcomes
Queued cancellation prevents dispatch. Running cancellation or timeout invalidates the
session and requires VM destruction through a separate, non-starvable control path.
Only independent host-side confirmation completes cancellation; dropping a future is not stop.
If stop or resource removal cannot be confirmed, expose cleanup-pending and quarantine capacity.
A disconnected client neither implicitly cancels nor replays accepted work.
After restart, uncertain accepted executions become `OutcomeUnknown`, not inferred success.
Polling will return authoritative status once the durable service exists.

## Recovery and persistence
Hold an exclusive installation lock before reconciliation or admission. Discover resources
by immutable installation/resource identity even if provisioning lost its reply; scan all pages.
Never remove unrelated resources or reuse quarantined capacity without confirmed cleanup.
Explicit `state` dies with worker/reset/failure; guest files may die with the guest volume.
Only finalized host-stored artifacts have independent retention guarantees; reset is not deletion.
A failed execution can leave mutations and side effects; execution is not transactional.

## Artifact and guest boundaries
Use host-generated opaque artifact destinations, quota-bounded streaming, and race-resistant
extraction that rejects special files and path escapes. No guest path authorizes a host path.
Deny networking explicitly; do not inherit host secrets, control sockets, or general mounts.
Residual callbacks/subprocesses remain possible, so quotas and idle deadlines survive completion.

## External adapters

The [MCP server profiles](mcp-tool-server.md) expose the same core job semantics without putting an executor in a transport process. The [Locus profile](locus-external-tool.md) adds durable handoff and independently authorized artifact promotion. Neither replaces the local VM or transport gates.
