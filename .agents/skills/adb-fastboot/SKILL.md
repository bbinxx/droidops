---
name: adb-fastboot
description: Implement or diagnose DroidOps ADB and Fastboot discovery, commands, device state, APK operations, transfers, shell, timeouts, and cancellation.
---

# ADB and Fastboot

## Purpose

Provide predictable device services with safe targeting and testable process I/O.

## When to use

Use for discovery/monitoring, device information, package operations, file
management, recovery sideload, shell execution, Fastboot, and scrcpy integration.

## Required context

Read [AGENTS.md](../../../AGENTS.md), crates/droidops-core/src/lib.rs,
crates/droidops-device-services/src/lib.rs, and affected desktop callers. Fastboot
supports discovery/reboot, not flashing/unlocking. APK installation and recovery
ZIP sideload are different workflows. Shell is one-shot, not a persistent terminal.

## Implementation workflow

1. Identify transport, required device state, serial, effect, and supported platforms.
2. Define structured arguments and typed outcomes. Keep missing executable,
   unauthorized/offline device, parse failure, timeout, and empty discovery distinct.
3. Add fixture tests for supported output variants, malformed data, and command
   construction. Include spaces, quotes, Unicode, and option-like paths.
4. Use an injectable process runner with bounded output, timeout, cancellation,
   progress events, and child reaping. Avoid overlapping discovery requests.
5. Bind actions and results to device sessions; handle reconnection and transitions
   between ADB, recovery/sideload, and Fastboot without stale selection.
6. Verify with mocks first; document any separately authorized non-destructive
   device checks. Use recorded history to compare behavior without restoring obsolete tooling.

## Coding standards

Use argument arrays/`Command`, explicit parsing, typed errors, and redacted logs.
Managed Android shell commands need remote quoting/validation even when host
arguments are separate. Free-form shell semantics must be intentional and tested.

## Safety constraints

Require explicit target selection and operation-specific confirmation for deletion,
data clearing, recovery sideload, and any future destructive bootloader operation.
Reject dangerous roots/traversal. Never automatically flash, format, wipe, erase,
unlock, elevate, or operate on real devices in automated tests. Cancellation cannot
guarantee undoing device-side effects.

## Validation commands

Run mock and controlled local process tests without device operations:

```bash
cargo test -p droidops-core -p droidops-device-services --locked
python3 scripts/check_agent_infrastructure.py
git diff --check
```

Physical-device verification is a separately authorized acceptance activity.

## Definition of done

Target/state gates, command arguments, parsing, errors, timeout, cancellation, and
disconnect behavior have mock coverage. Document actual device evidence and known
limitations; an empty result must not conceal an execution failure.

## Relevant documentation

See [native development](../../../docs/development/NATIVE.md) and
[feature parity](../../../docs/migration/FEATURE_PARITY.md) for current evidence.

See [the agent guide](../../../docs/development/AI_AGENTS.md) and
[ADB documentation](https://developer.android.com/tools/adb). Dedicated ADB,
Fastboot, and process-safety guides are planned.
