# BunBox architecture

## Current boundary
Phase 0 is **BLOCKED**. The workspace contains only `bunbox-sandbox` at
`crates/sandbox`; its `bunbox` binary is a readiness/preflight surface.
Dependency-free preflight and bounded framing are implemented and locally validated; see `verification.md`.
There is no linked Microsandbox SDK, VM integration, interpreter, durable store, or MCP service.
`execution-contract.md` and `threat-model.md` remain authoritative.

## Intended dependency layers
| Layer | Responsibility | Admission gate |
| --- | --- | --- |
| Bounded framing primitive | Reject oversized inputs before payload allocation | Local integration tests passed; not an SDK transport |
| Sandbox adapter | VM lifecycle, network denial, quotas, host-confirmed cleanup | Phase 0 blockers resolved |
| Mac VM probe | Benign provisioning and containment evidence | Approved runtime and isolated home |
| Execution worker | Pinned guest Bun, wrapper, state, bounded results | Real VM gate passed |
| Supervisor | Serialized dispatch, deadlines, cancellation, ownership | Validated worker/lifecycle |
| Durable storage | Accepted identities, reconciliation, finalized artifacts | Real VM gate passed |
| MCP adapter | Validated bounded requests, capability description | Core semantics and dependency approved |

These are responsibility boundaries, not additional crates already created.
Outer adapters depend on core policy/contracts; core policy must not depend on MCP.
The sandbox adapter alone owns runtime-specific integration. Guest helpers must not
be treated as host authorization or isolation mechanisms.

## Trust and control
- Submitted code, guest events/files/results, and guest packages are untrusted.
- The host OS/hypervisor, runtime dependencies, supervisor, parsers, and storage are trusted.
- IDs, generations, and sequences reject stale/malformed events; they do not authenticate
  a worker against hostile code sharing its VM.
- Keep cancellation/control independent of output flow so floods cannot starve teardown.
- Host policy owns paths, budgets, deadlines, resource identities, and cleanup decisions.
- All queues and upstream buffers need verified bounds; a bounded final queue is insufficient.

## Runtime and ownership
Use an explicitly isolated `MSB_HOME`, never the existing legacy installation.
Record creation intent before provisioning and reconcile immutable installation-owned
identities under an exclusive lock; enumerate all resource pages without touching others.
Normal completion does not disable VM-wide quotas or idle deadlines.
Running timeout/cancellation invalidates the session; unknown teardown quarantines capacity.

## Optional cloud integration

The [Locus external-tool specification](locus-external-tool.md) defines the proposed cloud boundary. Keep the BunBox supervisor and sandbox out of UAR. Locus and its workflow engine own sleep, wake, approvals and the owner's durable state. Cloud operation ownership, file promotion, tenant isolation and receipts require independent qualification. The local installation lock and Mac verification do not establish these properties for a replicated cloud service.

## Deferred choices
Do not select an unreviewed SDK patch or alternate transport merely to unblock builds.
Microsandbox 0.7.2 has unbounded event buffering and unenforced streaming timeout.
Legacy msb 0.2.6 remains untouched. rmcp 3.4.1 is absent; 3.4.0 is an unapproved candidate.
Storage engine, guest image, dependency APIs, and wire payload schema remain undecided.
