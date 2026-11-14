---
name: migration-tauri-to-egui
description: Audit and incrementally migrate DroidOps Tauri and React behavior into Rust services and egui while tracking feature parity and retaining the legacy application.
---

# Tauri to egui migration

## Purpose

Move one verifiable behavior at a time without replacing the current application
before feature parity and acceptance.

## When to use

Use for migration audits, parity mapping, Rust service ports, UI replacements,
Tauri dependency analysis, compatibility testing, and eventual legacy removal.

## Required context

Read [AGENTS.md](../../../AGENTS.md), `src/App.jsx`, the affected screen and helpers,
and both frontend/Tauri manifests. Legacy device logic is JavaScript; native services now live in
`crates/droidops-device-services`. The legacy bootstrap had no device core to extract. Inspect source rather than copying release
claims about icons, gallery thumbnails, interactive shell, or per-app cast.

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
6. Keep one major feature per branch. Remove legacy dependencies only in a later
   approved change after all required acceptance gates pass.

## Coding standards

Keep UI-independent Rust services, small components, minimal dependencies, and
testable errors. Do not couple visual redesign to service migration. Record
workspace/lockfile/target-path consequences before moving Cargo packages.

## Safety constraints

Do not reproduce known unsafe auto-selection or broad process permissions as
desired behavior. Do not silently drop ZIP sideload, mirroring, or diagnostics.
Do not add flash/unlock operations merely because unused UI strings mention them.

## Validation commands

Current checks, with legacy dependencies installed for the build:

```bash
python3 scripts/check_agent_infrastructure.py
npm run build
git diff --check
```

Use the scoped Rust commands in `AGENTS.md` for Rust changes. Workspace/native tests now exist; use the root commands in AGENTS. A browser build does not prove parity.

## Definition of done

Each migrated feature has source-backed acceptance criteria, service/state tests,
recorded compatibility evidence, updated parity status, and an available legacy
fallback. Unverified behavior is explicitly listed.

## Relevant documentation

See [native development](../../../docs/development/NATIVE.md) and
[feature parity](../../../docs/migration/FEATURE_PARITY.md) for current evidence.

See [the agent guide](../../../docs/development/AI_AGENTS.md) and
[README.md](../../../README.md). Migration audit, plan, and parity documents are
planned; do not treat them as existing artifacts.
