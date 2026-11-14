---
name: feature-development
description: Take a scoped DroidOps feature from issue analysis and specification through architecture, implementation, tests, documentation, and pull request preparation.
---

# Feature development

## Purpose

Deliver one useful, safe feature with acceptance evidence and a focused review.

## When to use

Use for new functionality or a major behavior change. Combine with the relevant
domain/UI/testing skills rather than loading every project skill.

## Required context

Read [AGENTS.md](../../../AGENTS.md), the issue/user request, affected source and
callers, current Git state, and migration status. Do not infer implementation from
README claims, strings, or a proposed crate diagram.

## Implementation workflow

1. Write a brief specification covering Problem, Goal, Scope, User experience,
   Architecture impact, Safety considerations, Acceptance criteria, and Test plan.
   Identify uncertainty and advance independent work while resolving it.
2. Use a focused branch from `dev` and preserve existing user changes. Issue
   creation on GitHub and publication require the relevant authorization.
3. Choose the smallest owning module/service boundary. Keep UI/device logic
   separate and inspect equivalent implementations before adding abstractions.
4. Implement incrementally with behavior tests at external-effect boundaries;
   cover device selection, failure, cancellation, stale results, and confirmation
   when relevant. Avoid unrelated refactoring or dependencies.
5. Validate native behavior against acceptance criteria, update documentation/parity evidence,
   and record unavailable tools and manual platform checks.
6. Prepare a Conventional Commit and review description with problem, behavior,
   test evidence, risks, and acceptance status. Stop at the authorized boundary;
   PR publication, review, merge, and release are distinct actions.

## Coding standards

Apply canonical Rust/UI standards and existing patterns. Use explicit service
errors and background work; document public APIs and only add abstractions with
concrete callers. Preserve existing workflows during incremental migration.

## Safety constraints

Never infer authorization for destructive device actions from a feature request.
Keep real devices out of automated tests, require explicit target and confirmation
for destructive operations, and avoid silent privilege elevation or sensitive logs.

## Validation commands

Always inspect the change and agent infrastructure:

```bash
python3 scripts/check_agent_infrastructure.py
git diff --check
git status --short --branch
```

Run relevant native commands from `AGENTS.md` and feature-specific tests that
actually exist. Root workspace/native checks are available in AGENTS.
Build success alone does not prove feature or cross-platform acceptance.

## Definition of done

The specification's acceptance criteria are addressed, meaningful tests and actual
results are recorded, docs agree with behavior, and the change is ready for review
with remaining risks stated. Do not claim a merge or release that did not happen.

## Relevant documentation

See [native development](../../../docs/development/NATIVE.md) and
[feature parity](../../../docs/migration/FEATURE_PARITY.md) for current evidence.

See [the agent guide](../../../docs/development/AI_AGENTS.md). Feature-specification
templates and contribution/PR guides remain planned; the current parity matrix
is maintained alongside the migration audit.
