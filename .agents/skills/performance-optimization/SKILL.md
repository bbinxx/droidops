---
name: performance-optimization
description: Measure and improve DroidOps startup, idle CPU, memory, list rendering, polling, image caching, transfers, and background scheduling when performance is a demonstrated problem.
---

# Performance optimization

## Purpose

Improve measured responsiveness and resource usage without speculative complexity
or weakening safety and cancellation.

## When to use

Use for slow startup, high idle CPU/memory, large package/directory lists, gallery
loading, thumbnail caches, transfer bottlenecks, and overlapping background work.

## Required context

Read [AGENTS.md](../../../AGENTS.md), affected native code, workload, build
mode, host/platform, and measurement method. Native previews use bounded decoding;
persistent thumbnails remain planned. Historical cache behavior is recorded in
the migration audit. There is no recorded performance baseline yet.

## Implementation workflow

1. Define a reproducible workload and user-visible target before optimizing.
2. Measure startup, idle polling/repaints, CPU/memory, frame latency, command count,
   and cache/transfer sizes relevant to the symptom. Record build mode and hardware.
3. Locate the actual bottleneck using available profiling tools; distinguish
   device latency, process overhead, parsing, image decoding, and rendering.
4. Apply the smallest change: avoid redundant polling, batch requests, virtualize
   lists, bound queues, or decode/resize images off-thread as evidence supports.
5. Key caches by device/session/path/content identity as appropriate; define
   invalidation, atomic writes, quotas, eviction, and concurrency ownership.
6. Repeat the same measurements and regression scenarios, including device switch,
   cancellation, and shutdown. Keep evidence and acknowledge noisy comparisons.

## Coding standards

Separate measurement from implementation. Use bounded work and event-driven
repaints; avoid global caches without ownership or unconditional frame loops.
Choose cache/concurrency limits from measurements, not promises about speed.

## Safety constraints

Never benchmark destructive operations. Do not retain personal photos or package
inventories as fixtures. Optimization must preserve explicit targeting, privacy,
timeouts, failure visibility, and cancellation; cancellation is not rollback.

## Validation commands

Use the native release build for the documented, reproducible workload:

```bash
cargo build -p droidops-desktop --release --locked
cargo test --workspace --locked
python3 scripts/check_agent_infrastructure.py
```

Benchmarks/profilers are not configured. Building is not performance evidence;
record actual measurements and the conditions under which they were collected.

## Definition of done

Before/after measurements use equivalent conditions, regression checks pass, and
the improvement and tradeoffs are documented without unsupported platform claims.

## Relevant documentation

See [native development](../../../docs/development/NATIVE.md) and
[feature parity](../../../docs/migration/FEATURE_PARITY.md) for current evidence.

See [the agent guide](../../../docs/development/AI_AGENTS.md). Benchmark workloads,
performance baselines, and cache architecture documentation are planned.
