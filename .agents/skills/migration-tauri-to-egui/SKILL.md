---
name: migration-tauri-to-egui
description: Audit and incrementally migrate DroidOps Tauri and React behavior into Rust services and egui while tracking feature parity against the historical legacy baseline.
---

# Tauri to egui migration

## Purpose

Move one verifiable behavior at a time without replacing the current application
without claiming unverified feature parity or acceptance.

## When to use

Use for migration audits, parity mapping, Rust service ports, UI replacements,
Tauri dependency analysis, compatibility testing, and eventual legacy removal.

## Required context

Read [AGENTS.md](../../../AGENTS.md), the migration audit/parity matrix, and
current Rust services/screens. The removed web application's reference is commit
`08414ac`; inspect it through Git when needed, for example
`git show 08414ac:src/lib/adb.js`. Do not restore Node/Tauri just to read history.

## Implementation workflow

1. Inventory the feature's UI, data, commands, dialogs, filesystem effects, and
   platform dependencies. Separate present behavior from incomplete claims.
2. Define acceptance criteria and fixtures; record parity as Not started, Planned,
   In progress, Implemented, Tested, Verified, or Blocked with supporting evidence.
3. Port pure models/parsers and service behavior behind process interfaces first.
   Preserve behavior except for explicitly documented safety corrections.
4. Implement the corresponding egui state and screen against the service contract.
5. Compare legacy/native behavior using the same scenarios and mock data; record
   OS/hardware checks separately and update the parity matrix when it exists.
6. Keep one major feature per branch. The user explicitly approved legacy removal;
   retain historical comparison evidence and unresolved acceptance gaps.

## Coding standards

Keep UI-independent Rust services, small components, minimal dependencies, and
testable errors. Do not couple visual redesign to service migration. Record
workspace/lockfile/target-path consequences before moving Cargo packages.

## Safety constraints

Do not reproduce known unsafe auto-selection or broad process permissions as
desired behavior. Do not silently drop ZIP sideload, mirroring, or diagnostics.
Do not add flash/unlock operations merely because unused UI strings mention them.

## Validation commands

Validate the native application; obsolete web build commands no longer apply:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --locked
python3 scripts/check_agent_infrastructure.py
```

Build success alone does not prove feature parity or hardware acceptance.

## Definition of done

Each migrated feature has source-backed acceptance criteria, service/state tests,
recorded compatibility evidence and updated parity status. Historical source
remains the reference; the removed UI is not a runnable fallback in this tree. Unverified behavior is explicitly listed.

## Relevant documentation

See [native development](../../../docs/development/NATIVE.md) and
[feature parity](../../../docs/migration/FEATURE_PARITY.md) for current evidence.

See [the agent guide](../../../docs/development/AI_AGENTS.md) and
[migration audit](../../../docs/migration/TAURI_TO_EGUI.md),
[cleanup record](../../../docs/migration/LEGACY_REMOVAL.md), and
[README.md](../../../README.md).
