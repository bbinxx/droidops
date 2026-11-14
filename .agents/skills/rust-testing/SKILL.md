---
name: rust-testing
description: Test DroidOps Rust domain behavior, process adapters, device simulations, error paths, cancellation, and native UI state without physical devices.
---

# Rust testing

## Purpose

Establish evidence for device behavior through deterministic tests at real effect
boundaries, without requiring a connected phone in ordinary CI.

## When to use

Use for unit/integration test design, mock ADB/Fastboot runners, reconnection,
configuration, UI state, timeouts, cancellation, and cross-platform test planning.

## Required context

Read [AGENTS.md](../../../AGENTS.md), the affected service and callers, and its
acceptance criteria. Native tests cover domain/process contracts and UI selection/rendering. The legacy
Tauri tests alone do not cover those services.

## Implementation workflow

1. Identify observable behavior and the regression each test should catch.
2. Use fixtures for parsing, domain states, configuration, and exact command
   arguments. Cover unauthorized/offline/recovery/Fastboot and malformed output.
3. Inject a fake process runner for service tests. Simulate spawn failures, exit
   errors, stderr, timeout, cancellation, partial transfers, and reconnection.
4. Where process behavior itself matters, use controlled local helper processes
   with temporary directories; never depend on tools found accidentally on PATH.
5. Test UI state independently: selection changes, stale results, confirmation
   expiry, loading/progress/errors, navigation, and shutdown.
6. Keep timers deterministic where practical. Run applicable existing tests and
   cross-platform jobs; document separate manual GUI/hardware acceptance results.

## Coding standards

Assert results, state, arguments, and effects rather than mirroring implementation
text or checking mocks exist. Use realistic complete output fixtures without
personal device data. Keep synchronization bounded to prevent hanging CI.

## Safety constraints

Dangerous command tests use fakes that record requests only. No flash, erase,
format, unlock, wipe, privileged installer, or physical-device dependency in CI.
Do not infer rollback from cancellation or success from exit status alone when
the protocol exposes additional failure output.

## Validation commands

Current infrastructure self-tests and existing Rust package checks:

```bash
python3 scripts/check_agent_infrastructure.py --self-test
cargo test --manifest-path src-tauri/Cargo.toml --locked
```

The Cargo command requires Rust and native dependencies and may currently run zero
tests. For native code run `cargo test --workspace --locked`. Add package/test
filters only when the named packages/tests exist; report skipped platform checks.

## Definition of done

Tests detect the named regressions, cover relevant failures, run without devices,
and have recorded results. State explicitly which UI/platform behavior remains
manual or unverified.

## Relevant documentation

See [native development](../../../docs/development/NATIVE.md) and
[feature parity](../../../docs/migration/FEATURE_PARITY.md) for current evidence.

See [the agent guide](../../../docs/development/AI_AGENTS.md). A testing strategy
and per-platform acceptance checklist are planned documentation.
