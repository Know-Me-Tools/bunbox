# Threat model

## Boundary

Untrusted: submitted code, the Bun process, guest files, package contents, guest-generated events and results. Trusted: host OS/hypervisor, Microsandbox/runtime dependencies, Rust supervisor, host parsers, policy enforcement, storage/extraction, and authentication/ownership enforcement.

Guest messages can lie. Execution IDs, generations and sequence validation reject malformed or stale messages; they do not authenticate the worker against hostile code in the same VM. Guest events cannot authorize host paths, raise limits, reset host deadlines or prove teardown.

## Mandatory controls

- One microVM per session, no host-process fallback.
- Explicit network denial; the upstream default is not offline.
- No host secrets, control sockets or general host mounts inside guests.
- Bounded allocations through the entire ingress path, not only BunBox's final queue.
- Separate cancellation/control path that cannot be starved by output.
- Independently confirm stop/kill and persisted-resource removal.
- Record creation intent before provisioning; discover resources even if creation loses its reply.
- Scope recovery to immutable installation/resource identities, scan all enumeration pages, and never remove unrelated resources.
- Quotas remain active after guest completion; untrusted serialization runs inside the guest budget.
- Artifact destinations are host-generated opaque identities. Stream with quotas and race-resistant extraction; reject special files and path escapes.

## Known blockers

Microsandbox 0.7.2's high-level execution event queue is unbounded; `collect()` accumulates output. The streaming path does not enforce its supplied timeout. Stock rmcp 3.4.0 stdio decoding also needs explicit message length limits. See `compatibility.md` for release-pinned evidence.

The repository therefore does not link the SDK, download a new runtime, create VMs, or accept arbitrary code yet. Pure framing tests cannot certify the virtualization boundary. Fixing one channel in a dependency is not enough unless all upstream buffering and cancellation behavior have also been verified.

## Testing safety

Start with benign fixtures. Adversarial guest fixtures run only after containment and independent termination are proven, with small budgets and a host watchdog. Never run memory/disk/process exhaustion tests directly on the host. Host-side malformed-frame tests use tiny bounded in-memory inputs.

A production release requires real isolation, recovery, cleanup, and soak evidence. Rust memory safety alone is not a sandbox guarantee.

## Proposed shared-service boundary

The [external-tool profile](locus-external-tool.md) adds per-operation authorization, workspace-scoped jobs and artifacts, fenced promotion and independent cloud isolation. Every [MCP transport profile](mcp-tool-server.md), including legacy compatibility, must enforce the same policy. Phase 0 local evidence cannot qualify those boundaries.
