# BunBox product requirements

## Status and authority
- Phase 0 is **BLOCKED**; no code-execution capability is available.
- This document is a plan, not evidence of an implemented interpreter.
- `execution-contract.md` governs semantics; `threat-model.md` governs boundaries.
- Implemented scope: one dependency-free `bunbox-sandbox` crate, preflight CLI,
  and preallocation-bounded length-prefix framing.
- Local foundation checks passed: 9 integration tests, 92.31% line coverage, formatting,
  and Clippy. See `verification.md`; VM/SDK integration remains unvalidated.

## Product goal
Provide a local, single-owner JavaScript/TypeScript execution service on the
user's ARM64 Mac, with a Rust control plane and one Microsandbox VM per session.
Mac is the first acceptance platform. Linux and Windows need separate evidence.
MCP connections are transports, not execution-session identities.

## Intended first release
- Execute an async wrapper with `await`, explicit `return`, and execution-local declarations.
- Permit only curated, version-pinned dynamic imports; no arbitrary package installation.
- Provide bounded output/results, structured errors, scoped files, and published artifacts.
- Keep explicit `state` only while its live worker remains healthy.
- Distinguish live state, guest workspace, and finalized host-stored artifact lifetimes.
- Serialize requests per session; do not promise transactional rollback or background services.
- Expose only actually implemented capabilities through the eventual MCP description.

## Delivery gates
1. Resolve bounded transport, timeout, and runtime compatibility blockers.
2. Prove real VM provisioning, containment, and independently confirmed teardown on Mac.
3. Only then implement the execution worker and validate guest Bun semantics.
4. Add durable acceptance/recovery and artifacts before exposing accepted execution service.
5. Add bounded MCP transport only after its dependency and ingress review are approved.

## Acceptance requirements
- Reject execution when prerequisites are missing; never fall back to host execution.
- Enforce budgets before allocation throughout the transport, not after SDK buffering.
- Confirm termination before declaring running cancellation complete.
- Report unconfirmed cleanup as cleanup-pending and quarantine affected capacity.
- Report uncertain accepted work as `OutcomeUnknown`; never replay it automatically.
- Demonstrate isolation, restart recovery, cleanup, and bounded-load behavior on real VMs.

## Explicit exclusions
No public/network service, arbitrary host mounts, host credentials or commands,
Docker socket, static-import promise, live-state migration, or unsandboxed fallback.
Preflight and framing validation alone do not establish VM readiness or security.

## Future persistent-dot profile

A user can ask a Locus dot to transform selected files and retrieve a verified result after disconnect or worker replacement. [The external-tool extension](locus-external-tool.md) is optional and separately qualified; the single-owner Mac remains the first release and provides no cloud tenancy, runtime or deployment assurance.
