# Observability and performance engineering

Status: proposed instrumentation and targets. R09, R12, R17.

## Instrumentation from the first boot

Log boot ID, kernel build ID, package generation, component instance/epoch,
monotonic timestamp and event kind. Use fixed-capacity ring buffers with loss
counters; diagnostics must not block hard IRQs or allocate indefinitely. Retain
serial output independent of compositor/network health.

Trace input receipt, event dispatch, state update, layout, raster, compose,
presentation submission and available completion timestamps. Trace IPC waits,
scheduling delays, queue depths, allocation pressure, storage flushes and every
update state transition. Correlation IDs span host build and guest activation.
Do not subtract clocks from different machines without calibration and uncertainty.

## Metrics and definitions

| Metric | Meaning | Common misleading substitute |
| --- | --- | --- |
| Frame-time distribution | p50/p95/p99 plus missed deadlines for a named workload | Average FPS alone |
| Guest input-to-present | Guest event arrival to presentation submission/completion where observable | Claimed photon latency |
| Idle work | CPU time, wakeups, rendering count over a fixed interval | “0%” from rounded task-manager output |
| Memory footprint | Resident/committed guest bytes by subsystem and peak transient bytes | Kernel binary size |
| Host cost | QEMU CPU/RSS, display backend and build process costs | Guest-only accounting |
| Reload latency | Build, transfer, quiesce, migration, activation, first visible frame | Only the route-pointer swap |
| Recovery time | Fault injection to usable state, with any lost work described | Process restart timestamp |

GPU timestamps, if introduced, require queue/fence accounting and clock interpretation.
Present success does not prove the physical monitor displayed the result. Power
efficiency needs physical measurements or explicitly named proxies; VM idle metrics
are useful but not battery-life proof.

## Workload suite

Static desktop with clock/caret; rapid pointer/window dragging; resize; long document
scrolling; large virtual list; text typing with complex scripts; many overlapping
windows; launcher/workspace transitions; translucent panels over moving content;
background file activity; and each under repeated live updates. Use fixed input
scripts/seeds plus manual visual review.

Record host CPU/RAM/OS, VM binary/firmware/machine version, accelerator, vCPUs,
guest RAM, resolution/refresh, renderer path, effect settings, package IDs, compiler
profile and warm/cold cache status. TCG, WHPX, KVM and physical hardware are separate
profiles. Use optimized builds for performance conclusions.

## Optimization order

First remove unnecessary work: avoid full layout for paint-only changes, sleep
without activity, virtualize offscreen lists, skip opaque occluded regions. Then
reduce allocations/copies, batch compatible operations and bound caches. Only
after correct CPU-state handling use SIMD; only after a real graphics stack use
GPU effects. Benchmark complexity and invalidation overhead as well as fast paths.

No speculative prediction should change externally visible behavior. Predictive
prefetch is bounded, cancellable and never commits a side effect. Effects can use
lower resolution or cached approximations if visual tests and settings make the
tradeoff explicit.

## Budgets and leak checks

Revision 2 keeps the 256 MiB cold-idle target including bounded caches. The 2 GiB VM
allocation is capacity. Optional inference needs separate cold/warm/active reports
including weights, scratch/attention buffers, shared/device memory and loading/update peaks.

Use [requirements targets](../00-overview/requirements.md) as initial hypotheses.
Account for framebuffer/surface duplication, atlases, caches, migration checkpoints,
both code generations, journal buffers and host copies. Reload peak may exceed
steady state substantially; check headroom before staging a replacement.

Run at least 100 update cycles and compare resources after settling. Track objects,
mapped pages, handles, code images, queued requests and retained checkpoints. An
unbounded old-generation list is a leak, even if visible FPS stays high.

## Debugging tools

Kani is a candidate for focused bounded parser/capability/state-machine harnesses.
Its documented concurrency limits preclude treating it as a complete scheduler/kernel
proof. [S82](../01-research/sources.md#s82)

Use QEMU gdbstub for privileged state, serial logs for early faults, QMP for test
orchestration, and guest inspectors for scenes/routes/resources. Host-test pure
algorithms with fuzz/property tests; use Miri/Loom only where their supported models
apply. They do not validate MMIO or a complete scheduler automatically.
[S35](../01-research/sources.md#s35), [S36](../01-research/sources.md#s36),
[S70](../01-research/sources.md#s70), [S71](../01-research/sources.md#s71)
