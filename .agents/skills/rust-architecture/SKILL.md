---
name: rust-architecture
description: Design DroidOps Rust workspace boundaries, service contracts, errors, and background execution when introducing or restructuring Rust services.
---

# Rust architecture

## Purpose

Create testable Rust boundaries without importing UI or platform concerns into
domain logic or creating empty packages merely to match a diagram.

## When to use

Use for workspace design, shared models, traits, service APIs, dependencies,
configuration, error strategy, and cross-platform background execution.

## Required context

Read [AGENTS.md](../../../AGENTS.md), `src-tauri/Cargo.toml`,
`src-tauri/src/lib.rs`, and relevant callers in `src/lib/adb.js`. The native root workspace now owns domain/device services and the desktop app;
`src-tauri` remains the separate legacy bootstrap.

## Implementation workflow

1. Map current callers, data ownership, side effects, and platform assumptions.
2. Propose the smallest useful boundaries: domain/contracts, device services,
   desktop wiring. Explain why each package needs independent dependencies.
3. Define typed device identity/state, errors, and a testable process boundary.
4. Decide workspace membership, lockfile location, and target paths explicitly;
   preserve legacy build/release paths until their migration is tested.
5. Keep OS-specific tool discovery behind adapters. Bound background requests and
   events; specify cancellation, timeouts, progress, and shutdown ownership.
6. Add pure/model and fake-process tests before wiring UI callers. Record an ADR
   when a decision changes long-term package or runtime boundaries.

## Coding standards

Use documented public types and `Result`, typed library errors, structured
redacted logging, and narrow dependencies. Do not leak Tauri or egui types into
domain APIs. Prefer interfaces at external effects over traits for every helper.

## Safety constraints

Apply canonical safety rules. Capture device identity/session in operations, keep
confirmation policy outside generic process execution, and reject stale results.
Architecture work never authorizes device commands, dependency installation, or
legacy removal.

## Validation commands

Current Rust package, when the toolchain/system libraries are installed:

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked
```

For the existing root workspace, run `cargo metadata --no-deps --format-version 1`,
`cargo test --workspace --locked`, and workspace fmt/Clippy checks. These validate the native workspace; retain scoped legacy checks for legacy edits. Check dependency direction and both applications.

## Definition of done

Boundaries have concrete owners and callers, legacy behavior remains available,
tests exercise failure paths, and dependency/build consequences are documented.
Report actual tool/platform results and unresolved design risks.

## Relevant documentation

See [native development](../../../docs/development/NATIVE.md) and
[feature parity](../../../docs/migration/FEATURE_PARITY.md) for current evidence.

Use [the agent guide](../../../docs/development/AI_AGENTS.md) for available context.
Workspace/device-service guides and ADRs are planned; do not assume they exist.
