# Transport investigation: recommended next experiment

Status: source-reviewed proposal, not implemented or runtime-verified.

Release examined: Microsandbox 0.7.2, commit `60d4dc8a436fb9365491567ec21d073e924e3c6d`.

## Recommendation

Keep the SDK for VM lifecycle and bypass its high-level execution mapper using the public generic raw agent client. Use one dedicated connection per active session/execution with bounded global concurrency. This avoids maintaining a fork merely to remove one unbounded queue, at the cost of owning more protocol integration and decoding validation.

Do not confuse `microsandbox::agent::AgentClient` (the SDK wrapper around the optimized client) with `microsandbox_agent_client::AgentClient` (the configurable generic protocol client).

Released APIs to verify in a compile-only integration:

- `microsandbox::agent::AgentClient::socket_path(name)`
- Generic `AgentClient::connect_with` and `ConnectOptions::limits`
- `ClientLimits`: frame size, in-flight requests, queued writes/responses, buffered bytes, incomplete-frame timeout and request timeout
- `stream_raw` / `stream_raw_with`, and `RawStream::into_parts`
- `Client::close` for local task shutdown, **not** remote termination

## Why a bounded SDK event channel alone is insufficient

The SDK mapper has an unbounded queue and buffered `collect()` grows vectors. Upstream optimized queues are bounded by counts but decoded payload copies also consume memory. The normal guest's 4-KiB output chunks are not a hostile-peer wire guarantee. Shared-memory arenas are another separately budgeted resource.

The generic client's router checks frame length and reserves its shared byte budget before allocating frame bodies. However:

1. Agent handshake uses the ordinary four-MiB codec before configured routing limits apply.
2. CBOR decoding allocates envelope and payload vectors; malicious lengths/nesting require a dedicated allocation/complexity review and adversarial tests.
3. Transport byte accounting excludes returned data retained by application code and decoded copies.
4. Full response queues can stall inbound processing. Shared inbound/outbound budgets can delay sending a kill signal.

The custom framing helper already in BunBox is a compatibility primitive, **not** this upstream wire protocol or a fix for these issues.

## Cancellation design to prove

Drain and discard output independently from client consumption; signal cancellation concurrently with draining. Bound signal delivery and terminal wait. Guest `Exited` may arrive only after output drains, so it cannot be the sole teardown authority. Escalate through separately controlled whole-VM lifecycle operations and verify stopped/removed identity. If confirmation fails, expose cleanup-pending and keep capacity charged. Dropping/closing the connection does not establish guest termination.

## Installation boundary

For a future dependency experiment, tagged source supports:

```toml
microsandbox = { version = "=0.7.2", default-features = false, features = ["local"] }
```

Do not enable `download-binaries` or `embed-binaries`. Audit the resolved feature graph for feature unification, and separately approve runtime provisioning. A crate pin alone does not verify the installed binary/library version. Use isolated build/runtime/config roots and preserve the old `msb` installation.

## Required experiment tests

Oversized prefixes; incomplete frames; malicious CBOR lengths/nesting; handshake limits; paused consumers; stdout/stderr floods; cancellation with full queues; lost terminal events; VM termination independently from the exec stream. No hostile guest test until benign VM boot and hard termination work.

## Release-pinned source map

All paths below are relative to [the examined commit](https://github.com/superradcompany/microsandbox/tree/60d4dc8a436fb9365491567ec21d073e924e3c6d):

- `sdk/rust/lib/sandbox/exec.rs`: unbounded mapper, collection, ignored streaming timeout.
- `sdk/rust/lib/agent/client.rs`: wrapper identity and local socket path.
- `packages/agent-client/rust/lib/client.rs`: generic public alias.
- `packages/protocol-client/rust/lib/options.rs`: configurable transport limits.
- `packages/protocol-client/rust/lib/router.rs`: frame preallocation budget and dispatch backpressure.
- `packages/agent-client/rust/lib/protocol.rs`: handshake codec exception.
- `crates/protocol/lib/codec.rs`, `message.rs`: framing and CBOR allocation boundaries.
- `crates/agentd/lib/session.rs`: guest output budgets and terminal ordering.
- `sdk/rust/build.rs`, `sdk/rust/lib/runtime/spawn.rs`: installation feature gate and runtime resolution.
