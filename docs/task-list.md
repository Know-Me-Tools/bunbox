# BunBox delivery checklist

Checked items indicate completion of that specific item only. Local-code verification does not imply VM readiness. See `verification.md` for commands and results.

## Baseline
- [x] Establish the single-crate workspace: `bunbox-sandbox` at `crates/sandbox`.
- [x] Record target semantics in `execution-contract.md` and controls in `threat-model.md`.
- [x] Record Mac-first sequencing and current limitations in the five planning documents.
- [x] Complete the dependency-free, fail-closed preflight CLI.
- [x] Complete preallocation-bounded length-prefix framing.
- [x] Validate CLI/framing: 9 integration tests, formatting, Clippy, compilation, 92.31% line coverage.

## Phase 0 — BLOCKED: compatibility and bounded transport
- [x] Record Microsandbox 0.7.2 unbounded event queue and ignored streaming timeout blockers.
- [x] Record that legacy msb 0.2.6 stays untouched.
- [x] Record rmcp 3.4.1 absence and 3.4.0 candidate-not-approved status.
- [ ] Select and explicitly approve a pinned runtime/transport solution; no silent substitution.
- [ ] Verify all upstream buffers, parsing allocations, output collection, and event-count limits.
- [ ] Prove deadline enforcement and cancellation independent of output/backpressure.
- [ ] Review native build/runtime setup with isolated `MSB_HOME` and no automatic downloads.
- [ ] Record evidence and approval before changing Phase 0 status or linking the SDK.

## Phase 0 continued — real Mac VM gate, before worker/storage/MCP
- [ ] Provision a benign VM with an approved pinned runtime in an isolated test installation.
- [ ] Verify explicit network denial, mount/secret exclusion, and enforced VM-wide quotas.
- [ ] Independently confirm termination and persisted-resource removal with a host watchdog.
- [ ] Test lost creation replies, scoped discovery, full enumeration, and cleanup quarantine.
- [ ] Record ARM64 Mac host/runtime evidence; keep other platforms unsupported until tested.
- [ ] Pass and review this gate before starting the execution worker or durable storage.

## Phase 1 — execution worker, after real VM gate
- [ ] Pin guest Bun and verify JS/TS transformation, source mapping, async wrapper, and imports.
- [ ] Implement explicit live state, scoped workspace, bounded output/results, and structured errors.
- [ ] Serialize dispatch; validate execution IDs, generations, sequences, and conservative late output.
- [ ] Validate timeout/cancellation invalidation, residual work quotas, and cleanup-pending behavior.
- [ ] Exercise adversarial guest fixtures only after containment and teardown proof.

## Phase 2 — durable service and artifact lifecycle
- [ ] Add exclusive installation ownership/locking and durable creation/acceptance intent.
- [ ] Enforce idempotency payload matching; never automatically retry uncertain accepted work.
- [ ] Reconcile restarts to `OutcomeUnknown` where appropriate and expose authoritative polling.
- [ ] Implement bounded, race-resistant artifact extraction and independent retention/deletion.
- [ ] Validate crashes, disconnects, quota exhaustion, quarantine, and retained-artifact behavior.

## Phases 3–5 — MCP, hardening and release readiness
- [ ] Approve a specific MCP dependency only after bounded stdio ingress review.
- [ ] Add ownership validation and truthful implemented-capability/exclusion descriptions.
- [ ] Validate transport disconnects without implicit replay or cancellation.
- [ ] Complete real isolation, recovery, cleanup, and soak evidence before release claims.
- [ ] Document remaining limitations; add Linux/Windows only through independent real-host gates.

## Proposed MCP compatibility qualification

- [ ] Validate ordinary agent workflow over bounded rmcp STDIO.
- [ ] Qualify each advertised Streamable HTTP version and mode.
- [ ] Separately qualify authorized two-endpoint HTTP+SSE compatibility; see the [MCP matrix](mcp-tool-server.md).

## Later cloud integration (proposed; not an active gate)

- [ ] Qualify hosted runtime and independent containment/cleanup for its intended placement.
- [ ] Prove the external job, scoped file, cancellation and receipt behavior in the [Locus external-tool specification](locus-external-tool.md).
- [ ] Enable only for approved dots after the two-workspace continuity and access checks; no UAR embedding or first-release dependency.
