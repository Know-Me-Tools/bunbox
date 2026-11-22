# BunBox technical notes

## Verified repository shape and scope
- Workspace member: `crates/sandbox`, package `bunbox-sandbox`, binary `bunbox`.
- Workspace declares Rust edition 2024 and minimum Rust 1.98.
- The package currently declares no external dependencies.
- Implemented foundation covers fail-closed preflight and bounded length-prefix framing.
- Compilation, formatting, Clippy, and 9 integration tests passed; line coverage is 92.31%.
  See `verification.md` for commands and limitations.
- No SDK link, VM, interpreter, durable storage, or MCP endpoint exists at this stage.

## Compatibility stop conditions
Phase 0 is **BLOCKED**. Released Microsandbox 0.7.2 has an unbounded high-level
event queue; collection accumulates output, and streaming ignores its supplied timeout.
An application-side byte cap cannot repair allocations already made by that transport.
Any replacement or patch requires release-pinned review of the full ingress/control path.
Legacy msb 0.2.6 is not the approved runtime and must remain untouched.
rmcp 3.4.1 is absent; 3.4.0 is an unapproved candidate with stdio length-bound work needed.
Do not silently substitute versions or invent API compatibility.

## Safe build setup
1. Check the installed compiler against the manifest requirement before requesting a build.
2. Review manifests, lockfile if present, features, and build scripts before adding dependencies.
3. Keep the initial crate dependency-free; adding runtime dependencies is a separate decision.
4. Do not enable automatic `download-binaries` behavior or run an installer as a build fix.
5. Treat downloaded runtimes/images as explicit, provenance-checked artifacts, not incidental setup.
6. Record exact toolchain/dependency/runtime versions when validation is actually performed.

## Safe runtime setup — future approved probe only
- Assign `MSB_HOME` explicitly to a dedicated BunBox test installation before runtime tooling.
- Do not reuse, migrate, overwrite, or clean the legacy runtime home.
- Isolation of `MSB_HOME` is configuration separation, not proof of guest containment.
- Resolve build-time and runtime paths independently; do not assume an environment variable
  alone controls every download location or native-library lookup.
- Review the pinned runtime's supported Mac configuration before any launch.
- Use benign fixtures, explicit network denial, no secrets/general host mounts, and tiny budgets.
- Establish a host watchdog and independently confirmed stop/resource removal first.
- Leave adversarial guest fixtures disabled until containment and teardown are proven.

## Validation and diagnostics
Local tests should cover prefix rejection before allocation, truncation, valid boundary frames,
fail-closed CLI behavior, and refusal to accept execution. They do not certify virtualization.
Later real-VM tests must cover output floods, timeouts, stale events, partial provisioning,
restart reconciliation, and cleanup failures under controlled budgets.
Never run memory/disk/process exhaustion fixtures directly on the host.
Report missing prerequisites as blockers; report uncertain accepted outcomes as `OutcomeUnknown`.
Unconfirmed teardown is cleanup-pending with quarantined capacity, never a success shortcut.

## Sources of truth
`execution-contract.md` and `threat-model.md` govern all implementation decisions.
`task-list.md` tracks gates; updating a checklist requires evidence, not merely new code.
