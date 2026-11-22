# Compatibility evidence — Phase 0

## Status

**Blocked, not an interpreter.** Source review and host prerequisites are recorded; no Bun microVM, hostile-workload containment, guest image, or teardown behavior has been validated. The preflight command reports these known blockers, not a live installation health check.

Local observations: ARM64 macOS 26.7.1 / 25G309, `kern.hv_support = 1`, Rust/Cargo 1.98.1, host Bun 1.4.2, installed `msb 0.2.6`. CLI presence and hypervisor capability are necessary but insufficient.

## Exact release findings

| Item | Evidence | Decision |
| --- | --- | --- |
| Microsandbox 0.7.2 | Published, non-yanked in Cargo sparse index | Research candidate only; not linked |
| Installed msb 0.2.6 | Legacy tag's SDK is 0.1.2, HTTP server architecture | Preserve installation; do not mix APIs |
| rmcp 3.4.1 | Absent from checked registry | Unresolved requested pin |
| rmcp 3.4.0 | Published, non-yanked; official matching release | Proposed replacement, not installed or approved |
| Bun 1.4.2 | Host CLI | Guest version/image unverified |

Registry references: [microsandbox](https://index.crates.io/mi/cr/microsandbox), [rmcp](https://index.crates.io/rm/cp/rmcp). Registry API returned HTTP 403; official sparse-index reads succeeded. GitHub repository and code searches were performed before implementation.

## Blocking source findings

1. [Microsandbox 0.7.2 execution](https://github.com/superradcompany/microsandbox/blob/v0.7.2/sdk/rust/lib/sandbox/exec.rs): `ExecHandle` consumes an unbounded event receiver. The mapper forwards events into an unbounded channel; `collect()` accumulates output. Streaming ignores the supplied timeout. A bounded queue outside the SDK cannot fix hidden allocations.
2. [rmcp 3.4.0 stdio](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.4.0/crates/rmcp/src/transport/async_rw.rs): stock reads need length bounds; the codec's bounded constructor must be explicitly configured and tested, along with outbound limits.
3. [SDK build script](https://github.com/superradcompany/microsandbox/blob/v0.7.2/sdk/rust/build.rs) and [features](https://github.com/superradcompany/microsandbox/blob/v0.7.2/sdk/rust/Cargo.toml): default `download-binaries` can install runtime components during compilation. Do not enable it casually or target the existing runtime home.

A candidate SDK integration must isolate `MSB_HOME` at build and runtime, isolate `MSB_CONFIG_PATH`, explicitly select the local backend, and audit resource paths. Coexistence with the legacy installation remains untested. Runtime installation needs separate approval through the managed CLI route; no shell installer or download-and-run shortcut.

## Actual Phase 0 exit gate (all still outstanding)

- [ ] Select and audit an end-to-end bounded transport, including SDK/protocol buffers and event chunk sizes.
- [ ] Verify an independent termination path cannot be starved by stdout backpressure.
- [ ] Install approved project-isolated runtime components without modifying legacy msb.
- [ ] Pin a Linux ARM64 Bun image by digest and capture runtime identities.
- [ ] Create one VM through Rust, run benign Bun input, check duplex I/O, stderr, EOF and exit.
- [ ] Explicitly disable guest networking and verify denial, including DNS and IPv6 where supported.
- [ ] Verify resource limits; distinguish vCPUs from CPU quota, guest RAM from host overhead, and volume growth from disk size.
- [ ] Observe stop/kill completion, remove persisted resources, and independently enumerate all pages to confirm cleanup.
- [ ] Test lost creation reply, host crash and failed cleanup without touching unrelated resources.

Use a narrow reviewed SDK patch or a supported lower-level bounded API only after examining the whole transport. Do not approximate this gate with mocks or host-process execution. If no acceptable path exists, request a backend decision rather than weakening the contract.

## References to reuse

- [Released SDK stdin tests](https://github.com/superradcompany/microsandbox/blob/v0.7.2/sdk/rust/tests/stdin.rs)
- [Network configuration](https://github.com/superradcompany/microsandbox/blob/v0.7.2/docs/sdk/rust/networking.mdx)
- [Legacy SDK manifest](https://github.com/superradcompany/microsandbox/blob/microsandbox-v0.2.6/sdk/rust/Cargo.toml)
- [RMCP stdio example](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.4.0/examples/servers/src/calculator_stdio.rs)
