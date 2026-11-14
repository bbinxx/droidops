---
name: security-review
description: Review DroidOps process execution, device confirmations, filesystem paths, privilege escalation, dependencies, and sensitive-data handling before changing those boundaries.
---

# Security review

## Purpose

Find concrete ways a change could target the wrong device, execute unintended
commands, expose sensitive data, or modify host/device files without consent.

## When to use

Use for reviews of process adapters, shell/filesystem operations, destructive
actions, privilege elevation, capabilities, dependencies, and diagnostics/logging.

## Required context

Read [AGENTS.md](../../../AGENTS.md), affected callers, domain validation,
and crates/droidops-device-services/src/lib.rs. Inspect process supervision,
remote quoting, canonical-parent guards, upload preflight, confirmation binding,
and image/runtime limits. The Tauri capability surface no longer exists.

## Implementation workflow

1. Map inputs and trust boundaries: frontend, process executable/arguments,
   Android shell, device output, local cache, downloaded files, and privileged host.
2. Check explicit target selection, required device state, and confirmation bound
   to the exact operation/session. Revalidate immediately before execution.
3. Check host argument separation and remote shell interpretation independently.
   Test option injection, quotes, metacharacters, and malformed device output.
4. Check root/traversal/symlink/overwrite risks, local versus remote paths, safe
   temporary files, cache isolation, quotas, and partial-transfer cleanup.
5. Review timeout, cancellation, output bounds, shutdown, privilege disclosures,
   executable discovery, logs/redaction, artifact provenance, and dependencies.
6. Report findings with source location, preconditions, impact, severity, and
   minimal remediation. Add mock/fixture regression tests for implemented fixes.

## Coding standards

Keep typed errors and narrow service interfaces. Use allowlisted managed actions
instead of exposing a generic command endpoint. Keep raw shell an explicit user
surface. Do not call a capability scope or shell quoting alone complete security.

## Safety constraints

Review is read-only unless fixes are authorized. Never prove a finding by wiping,
flashing, unlocking, privileged installation, secret extraction, or unsafe writes
to a real host/device. Use controlled fixtures and fakes; redact all evidence.

## Validation commands

Run affected native safety tests and scoped code validation:

```bash
cargo test -p droidops-core -p droidops-device-services --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
python3 scripts/check_agent_infrastructure.py
git diff --check
```

Dependency vulnerability/license automation is not configured yet. Syntax or
compilation alone does not establish security review completion.

## Definition of done

Trust boundaries and safety gates are reviewed, implemented fixes have regression
evidence, and residual risks/platform limitations are explicit. No destructive
test or sensitive fixture is introduced.

## Relevant documentation

See [native development](../../../docs/development/NATIVE.md) and
[feature parity](../../../docs/migration/FEATURE_PARITY.md) for current evidence.

See [the agent guide](../../../docs/development/AI_AGENTS.md). Dedicated ADB,
Fastboot, process-execution, and dependency/security policies are planned.
