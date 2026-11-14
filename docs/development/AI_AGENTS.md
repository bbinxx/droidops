# Developing DroidOps with coding agents

[AGENTS.md](../../AGENTS.md) is the single source of project policy.
[CLAUDE.md](../../CLAUDE.md) and
[Copilot instructions](../../.github/copilot-instructions.md) are discovery
adapters. Their presence does not guarantee an agent has loaded linked files.

## Current status

This infrastructure supports the current Tauri/React application and the planned
Rust/egui migration. The application remains unchanged. Device logic is currently
JavaScript; there are no root workspace or native UI packages yet. Read source
before treating README or release notes as evidence of implemented behavior.

## Skill index

| Skill | Use for |
| --- | --- |
| [rust-architecture](../../.agents/skills/rust-architecture/SKILL.md) | Workspace and module boundaries, service contracts, errors, dependencies |
| [egui-development](../../.agents/skills/egui-development/SKILL.md) | Native UI lifecycle, state, rendering, DPI and accessibility |
| [adb-fastboot](../../.agents/skills/adb-fastboot/SKILL.md) | Device discovery, command execution, transfers, shell and device safety |
| [migration-tauri-to-egui](../../.agents/skills/migration-tauri-to-egui/SKILL.md) | Behavior audit, parity, incremental service and UI migration |
| [git-workflow](../../.agents/skills/git-workflow/SKILL.md) | Branches, commits, review preparation and conflicts |
| [rust-testing](../../.agents/skills/rust-testing/SKILL.md) | Domain, process, integration and UI state tests |
| [release-management](../../.agents/skills/release-management/SKILL.md) | Versions, tags, changelog, artifacts and release validation |
| [github-actions](../../.agents/skills/github-actions/SKILL.md) | CI matrices, permissions, caching and release workflow changes |
| [documentation](../../.agents/skills/documentation/SKILL.md) | Guides, specifications, ADRs and source-backed feature documentation |
| [security-review](../../.agents/skills/security-review/SKILL.md) | Process, filesystem, device, privacy and privilege boundaries |
| [performance-optimization](../../.agents/skills/performance-optimization/SKILL.md) | Measured startup, polling, lists, cache and transfer improvements |
| [feature-development](../../.agents/skills/feature-development/SKILL.md) | A feature from specification through review preparation |

Select only the skills relevant to the task. They describe implementation and
review workflows, not permission to execute device operations or publish changes.

## Discovery and portability

Skills follow the [Agent Skills format](https://agentskills.io/specification):
each directory contains a `SKILL.md` with YAML `name` and `description` metadata.
Names match directory names. Relative links resolve from the containing document.

- Codex environments may discover `.agents/skills/`; verify the session's skill
  catalog. If unavailable, ask the agent to read the desired file explicitly.
- Claude environments may use other skill directories. `CLAUDE.md` points to the
  canonical index; explicit file reading avoids requiring a second copy.
- Copilot discovery depends on the editor/agent surface. Repository instructions
  point here, but the agent may need the skill file attached or explicitly read.
- Other tools can read `AGENTS.md` and selected skill files without installation.

Do not assume recursive loading or equivalent permission enforcement across tools.
No global configuration, provider-specific tools, symlinks, or third-party skill
installation are required. Review any proposed external skill's contents and
permissions before separately authorizing installation.

## Example task handoff

Ask an agent to read `AGENTS.md` and the `adb-fastboot` and `rust-testing` skills,
then inspect discovery behavior and propose a mock-tested parser change. State
acceptance criteria and whether implementation is authorized. Hardware testing,
publishing, and destructive operations require their own explicit scope.

## Validation

Requires Python 3.9+ with only the standard library; on Windows use `py -3` if
`python3` is unavailable. Commands work from any current directory when the script
path is supplied correctly; its default root is the script's repository.

```bash
python3 scripts/check_agent_infrastructure.py --self-test
python3 scripts/check_agent_infrastructure.py
git diff --check
```

The checker validates the 12 required skills, the two canonical adapters, required
sections, and local file links in these 16 Markdown files. It deliberately accepts
only two unquoted, single-line YAML fields, `name` and `description`, with simple
string values; it rejects duplicate fields and unsupported YAML constructs rather
than pretending to parse general YAML. Keep metadata in that portable subset or
extend the checker with tests before adopting another format.

Link checking covers inline Markdown file links and reference-style file links,
ignores fenced code and external URLs, and rejects paths outside the repository.
Heading fragments, external URL availability, prose accuracy, and runtime agent
behavior require review. The checker is not a safety enforcement mechanism and
does not execute commands found in documents. It does not validate all repository
documentation. Failures exit nonzero; self-tests use temporary fixtures only.

For application validation, use the scoped commands and limitations in `AGENTS.md`.
No new dependencies or application test claims accompany this infrastructure.

## Planned documentation

Later reviewed phases will add the architecture and device-service guides,
development/build/testing guides, migration audit and parity matrix, ADRs,
security policies, versioning/release guides, and contribution templates. These
are planned documents, not existing links or completed migration deliverables.
Until then, consult [README.md](../../README.md), [RELEASES.md](../../RELEASES.md),
and the source while keeping the audit's documented discrepancies in mind.
