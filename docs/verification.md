# Foundation verification

## Completed local-code checks

- `cargo generate-lockfile --offline` — passed; no external crates.
- `cargo fmt --all` and `cargo fmt --all -- --check` — passed for initial implementation; final added tests formatted again.
- `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` — passed, including final test additions.
- `cargo test --workspace --locked --offline --test '*'` — initial 7 integration tests passed.
- `cargo llvm-cov --workspace --locked --offline --test '*' --fail-under-lines 80` — initially failed coverage gate at 73.63%; after transport-error and CLI-help tests, **9 integration tests passed and line coverage reached 92.31%** (86.96% regions).
- `cargo run --locked --offline -p bunbox-sandbox --bin bunbox -- preflight` — expected exit **2**, reporting blocked state. This was not a VM execution.

Coverage command automatically installed `llvm-tools-preview` into Rust 1.98.1. This toolchain side effect was not intended and was disclosed. No Microsandbox runtime, guest image, service, or SDK was installed.

Tests exercise public framing APIs and the compiled CLI process. They cover binary round trips, clean versus truncated EOF, fragmented reads, interrupted reads, preallocation oversized rejection, invalid limits, empty frames, transport error propagation, help, blocked preflight, and rejected execution requests.

## Independent review

Read-only Rust/security review found no verified correctness or security defects within this limited scope. It identified stale planning statuses, which were corrected. The review inspected the initial seven-test implementation; two additional integration tests were subsequently added for coverage without changing production code. The reviewer did not rerun tests.

## Explicitly not validated

VM boot, guest Bun semantics, isolation, resource/network enforcement, SDK byte bounds, cancellation, teardown, crash reconciliation, artifacts, MCP, Windows/Linux, fuzzing, soak tests, benchmarks and release builds remain untested. The stock SDK transport issue still blocks Phase 0.

The 92.31% number applies only to this small foundation, not the planned interpreter. No claim of production readiness or completion of the playbook is made.

## Workflow note

Tests were authored before implementation, but no pre-implementation RED run was performed: the loaded Rust workspace workflow calls for batched integration verification at a completed code boundary. Do not report a RED/GREEN cycle as observed.
