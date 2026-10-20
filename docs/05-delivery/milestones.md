# Implementation milestones and gates

Status: gated execution plan. Phase 0 compile/tooling implementation and evidence
are recorded in [the runbook](phase-0.md). P1-02 demonstrates minimal UEFI ISO
kernel entry; M1 as a whole and later milestones remain uncompleted.
Stages are evidence gates, not promised calendar dates. First build feasibility
determines credible estimates. Do not attempt the whole OS in one coding session.
Revision 2 prioritizes M2/M4/M5 state-preserving reload and an AI-free usable desktop.
The [reconciled plan](../00-overview/reconciled-plan.md) records user-approved direction.

## M0 — Feasibility and locked toolchain

Inputs: this library and a clean workspace. Record actual Windows/Linux build options,
Rust/linker/ISO/QEMU/firmware availability and host accelerator. Pin tested versions
and checksums; inspect licenses and guest dependency features. Select explicit
kernel and userland targets, panic/allocation policies and build directory layout.

Experiments: separate freestanding kernel/userland Rust links; candidate
UI/raster/font no-OS feature builds; QEMU/firmware/ISO prerequisite inventory.
The user selected Windows native first and compile/tooling-only Phase 0: actual
ISO boot and VM launch are M1. Local dev transport execution remains M4.
Add Slint/custom target feasibility, x86_64 crate review, and a Limine/Rust OSDev
bootloader comparison tied to an actual ISO recipe.
Do not install OS features, alter virtualization settings or download large toolchains
without reporting the concrete need and respecting environment approval rules.

Exit: two clean probe builds, artifact inspection, source path with spaces and
doctor override checks; locked toolchain, dependency outcomes, and an explicit
packaging/VM prerequisite report with blockers. ISO recipes remain not boot-tested.
Artifacts: environment/build/toolchain manifests and retained logs. Supports R03,
early R16 and R18; no release reproducibility or ABI/isolation claim.

## M1 — Original kernel boots from ISO

Execution is decomposed into [one epic and seven ordered stories](phase-1/README.md).
P1-01 supplies pinned host inputs; P1-02 adds the minimal original-kernel ISO path.
See its [runbook](phase-1/minimal-iso-runbook.md) for evidence boundaries.
P1-03 adds [checked owned metadata and correlated boot records](phase-1/boot-metadata-runbook.md)
with retained acceptance evidence. P1-04 adds
[fatal exception diagnostics](phase-1/exception-diagnostics-runbook.md).
P1-05 implements [owned frames/mappings and guards](phase-1/memory-ownership-runbook.md);
check its acceptance review. P1-06 [interrupt/timekeeping acceptance](../../evidence/phase-1/P1-06/2026-10-20/README.md) passes on the pinned single-core profile. P1-07 remains open; M1 is incomplete.

Implement boot validation, serial output, framebuffer metadata, owned paging,
exception handling, physical allocator and interrupt/timer baseline. Build UEFI ISO.
Initially avoid heap/SIMD until supported; emit boot/readiness checkpoints.

Exit: cold boot from the exact ISO with no host share/network, stable boot ID/uptime,
usable diagnostics on intentional exception and repeatable shutdown/test exit.
Archive config, hash and serial transcript. Supports R01–R03.
Require the packaged ISO's firmware/optical boot path; a working disk image or
direct-kernel emulator launch cannot substitute for it.

## M2 — Hosted UI core and resource reload

After M0, compare Slint and a minimal custom UI with identical editor/file-list
scenes, tokens, text, semantics and input. Audit guest compile/platform hooks,
reload and distribution terms. Select provisionally before expanding a full toolkit;
confirm in M4. Define typed edit/settings actions shared by UI and automation.
Prove hosted token/layout reload preserving document/focus state.

Exit: deterministic renderer fixtures, keyboard workflow, text/semantic test subset,
idle/damage instrumentation and measured hosted baseline. No guest performance claim.
Supports R04, R08, R09, R15 and R19 interface design. Hosted preview does not prove
guest interpreter support or compiled behavior replacement.

## M3 — Isolated processes and stable interfaces

Add static user ELF loader, user stacks, syscall entry/return, handles, channels,
waits, preemptive task scheduling, quotas and supervisor. Implement proxy routes
and epochs. Establish FP/SIMD context policy before running UI math in user tasks.

Exit: process isolation/forged-handle/malformed-IPC/deadlock/resource tests, independent
SDK client/service build, user fault containment and no stale-writer bypass.
Supports R10, R14 and prerequisites for R05/R06.

## M4 — Guest desktop and live assets

Bring the same UI core to guest framebuffer/surfaces; add input driver, minimal
compositor/shell and controlled host→guest dev transport. Keep serial recovery.
Transmit validated resource/layout updates and publish at scene boundaries.

Exit: live resource demo inside running guest, kernel boot ID/uptime unchanged,
correct focus/gesture handling, rejected corrupt update, disconnect/retry tests and
guest rendering metrics. Persistence can remain explicitly ephemeral.
Confirm the selected toolkit's actual guest backend and surface/input/semantics.
Raster buffer submission is sufficient; custom scene export is not mandatory.
Supports R04, R08, R09, R13, R17.

## M5 — Stateful app behavior replacement

Implement coordinator, quiescence, restricted candidate startup, identity-schema
checkpoint, route/lease handover, health test and safe retirement. Editor is the
first meaningful case. Add deliberate incompatible schema rejection and migration fixture.

Exit: edit unsaved text → change behavior → new instance takes over with text,
selection and window intact; at least 100 cycles without unexplained growth. Inject
failure in every in-memory phase. Demonstrate old code is not freed while reachable.
Show UI/layout and compiled behavior changes, unchanged boot ID/uptime, no VM reset,
and equivalent typed UI/automation actions through replacement. This first meaningful
product demo precedes optional inference.
Supports R05, R07, R12, R14, R17, R19. Not yet a power-loss durability claim.

## M6 — Durable state and selected service updates

Add VirtIO block, real flush semantics, tested record store or revised filesystem
choice, immutable packages/generations and activation journal. Implement durable
settings/file operations, idempotency records and recovery selection. Replace a
simple settings/service process under load before attempting complex storage drivers.
Separately implement boot-attempt/health/known-good selection (B40), with bounded
failed boots and schema-aware fallback. A/B slots remain a later packaging option.

Exit: fault matrix proves durable outcomes; full disk/flush errors are honest; accepted
mutations are preserved; old/new writer overlap never occurs. Rescue ISO can recover
the last good coherent generation. Supports R06, R11, R12, R14.

## M7 — Extensions behind independent gates

Evaluate Wasm interpreter, broader network/TLS, VirtIO GPU display/acceleration,
audio, SMP, richer text/assistive services and additional hardware. Optional small-model
CPU inference may start after M5 via typed actions; it does not gate core release.
Report cold/warm/active memory and inference-worker failure. Each extension has its own
dependency, security, performance and recovery experiment. None is declared shipped
because another succeeds. Defer broad real-PC support until device matrix is explicit.

Exit per extension: integration evidence, workload comparison, supported feature
subset and limitations. This stage may contain multiple releases; it is not a
single “finish all drivers” task. Software fallback remains available.

## M8 — Coherent experimental release

Build standalone ISO and optional data image from a coherent locked generation.
Run requirement matrix, two clean-build comparison, offline boot, recovery, release
dev-authority checks, visual/accessibility review, license/SBOM and provenance audit.

Exit: source/build instructions, exact tested binaries, checksums/symbols, VM profile,
known limitations and reproducible evidence bundle. Call it an experimental VM OS,
with declared support; do not imply daily-driver laptop readiness. Supports R02,
R08–R19 within the agreed matrix. An AI-free release can pass all core gates.

## Stop/replan criteria

A gate failure creates a bounded issue and evidence, not silent scope substitution.
If the host cannot build/run the selected path, complete portable work and report
the exact missing capability. If resource budgets fail, profile before dropping
correctness. If a user requires zero upstream boot code or full kernel live reload,
revise scope/ADRs explicitly before promising an implementation timeline.
