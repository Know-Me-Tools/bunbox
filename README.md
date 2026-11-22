# BunBox

Rust-managed Bun code interpreter with a Microsandbox VM boundary, following the BunBox Playbook.

**Current status: Phase 0 foundation; execution blocked.** This repository does not yet execute code, start VMs, or expose MCP tools. It deliberately has no unsandboxed fallback.

## Intended use

[Read the BunBox Origin story](docs/ORIGIN.md). The [consolidated KnowMe edition](docs/publication/index.html) combines the story, architecture, specification, and implementation plan. Regular agents will be able to use BunBox as a separately qualified [rmcp tool server](docs/mcp-tool-server.md), with STDIO and Streamable HTTP profiles. The old two-endpoint HTTP+SSE protocol requires its own compatibility adapter; it is not a feature of modern rmcp. No transport is implemented today.

## How it fits Locus and UAR

BunBox is a proposed **external, opt-in tool** for persistent Locus dots. Locus owns continuity and the private work state, UAR invokes a scoped tool, and BunBox runs only that job in an isolated VM. The dot can wait without holding a worker and recover a verified artifact by the same job ID. BunBox must not be embedded in UAR or required for the first Locus release.

See the [external-tool specification](docs/locus-external-tool.md) for the proposed cloud API, permissions, recovery, and first user journey. The first BunBox milestone remains a single-owner ARM64 Mac VM; it does not qualify cloud or shared use.

## What exists

- Dependency-free Rust workspace pinned to Rust 1.98.1.
- `bunbox preflight`: reports known compatibility blockers, exiting **2** until resolved. This is a static readiness report, not a live runtime health check.
- Bounded four-byte length-prefix framing with a hard one-MiB payload ceiling; oversized announcements are rejected before payload reads/allocations.
- Public API and CLI integration tests.
- Execution contract, threat model, version research and phased implementation documents.

The framing helper is synchronous and has no timeout. It does not fix unbounded buffering inside Microsandbox or constitute a production transport.

## Run

```sh
cargo run --locked -p bunbox-sandbox --bin bunbox -- preflight
# Expected exit status: 2 (blocked), not an installation failure.
```

## Verify

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked --test '*'
```

Tests cover local framing and executable behavior only. They do not establish VM isolation, cleanup, guest language semantics, or production readiness. No external Rust dependencies are currently linked.

## License

BunBox is open source under the [MIT License](LICENSE). Third-party assets and dependencies retain their own licenses.

## Next gate

Resolve the released Microsandbox SDK's unbounded event queue and validate a separate termination path. Then approve isolated runtime provisioning, run real Bun-in-VM compatibility tests, and advance to the execution worker. Preserve the existing `msb 0.2.6` installation. Do not use guessed SDK calls or automatically download runtimes during builds.

See [compatibility evidence](docs/compatibility.md), [execution contract](docs/execution-contract.md), [threat model](docs/threat-model.md), and [task list](docs/task-list.md).
