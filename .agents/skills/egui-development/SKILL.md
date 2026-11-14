---
name: egui-development
description: Build or review DroidOps egui widgets, eframe lifecycle, UI state, layouts, themes, and repaint behavior during native UI development.
---

# egui development

## Purpose

Implement a responsive native desktop UI while preserving audited user workflows.

## When to use

Use for native screens/widgets, navigation, view models, themes, dialogs, loading
states, repaint scheduling, window scaling, keyboard interaction, and rendering.

## Required context

Read [AGENTS.md](../../../AGENTS.md), the corresponding React screen, its service
calls, and the feature's acceptance criteria. No egui application exists yet.
Verify the selected stable egui/eframe versions and their actual lifecycle API.

## Implementation workflow

1. Describe current screen behavior and the migration acceptance criteria.
2. Separate preferences, navigation, selected device, snapshots, operations, and
   temporary dialogs. Identify a single owner for each state value.
3. Compose small UI functions using consistent style; keep device logic in services.
4. Dispatch work from user intent once; consume bounded events during frames.
   Request repaint on results and schedule bounded polling instead of spinning.
5. Include progress, cancellation, notifications, and error dialogs. Reject results
   from obsolete device sessions; revalidate confirmation before submission.
6. Exercise narrow windows, high DPI, keyboard focus, labels, large lists, shutdown,
   and disconnected/unauthorized devices. Record platform-specific limitations.

## Coding standards

Follow canonical Rust/UI rules. Avoid blocking, image decoding, directory scans,
and process spawning inside rendering. Virtualize large lists when measurement
justifies it. Do not equate a successful spawn with a completed device operation.

## Safety constraints

Never automatically select a target for an action. Confirmation dialogs must name
the device and effect and become invalid when the target changes. The UI must not
invent progress percentages or hide cancellation failures.

## Validation commands

Current infrastructure check:

```bash
python3 scripts/check_agent_infrastructure.py
git diff --check
```

Once the proposed desktop package exists: `cargo test -p droidops-desktop --locked`
for UI-state tests and `cargo run -p droidops-desktop` for manual smoke testing.
These are planned commands; GUI/hardware testing requires an appropriate host
and authorized scope. Record visual, scaling, and keyboard results separately.

## Definition of done

The screen satisfies its parity criteria, errors/loading/cancellation are usable,
state tests pass, no expensive frame work is introduced, and manual evidence is
identified without claiming untested platforms.

## Relevant documentation

See [the agent guide](../../../docs/development/AI_AGENTS.md). UI-state architecture
and feature parity documents are planned for subsequent phases.
