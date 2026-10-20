# Handoff for Claude, Codex and other coding agents

## Current state

This workspace contains an architecture/research library and agent entry points.
Phase 0 adds a native Windows Cargo/xtask build system, independent freestanding
kernel/userland link probes and dependency experiments. See the
[runbook](phase-0.md) and [evidence](../../evidence/phase-0/README.md).
P1-02 adds a **minimal original Rust kernel cold-booted from a standalone UEFI ISO**
with serial entry and emulator-only test exits. See the
[runbook](phase-1/minimal-iso-runbook.md) and [evidence](../../evidence/phase-1/P1-02/2026-10-18/README.md).
P1-03 adds checked, kernel-owned boot descriptors, retained backing reservations,
version-2 session-correlated diagnostics and guest rejection cases. See the
[metadata runbook](phase-1/boot-metadata-runbook.md) and
[corrected acceptance review](../../evidence/phase-1/P1-03/2026-10-18/README.md).
All loader-reclaimable memory remains live. P1-03 captures the base-3 ACPI locator
as a physical byte address without dereferencing it; P1-06 validates and maps its
table chain separately. Bootstrap pointer readability still trusts
the pinned firmware/loader. Image bounds now include GOT/BSS and are audited on
every build. P1-04 adds owned exception tables, static runtime/double-fault/NMI
stacks and fatal diagnostics; see the [exception runbook](phase-1/exception-diagnostics-runbook.md)
for its acceptance status and CPU/unsafe review. P1-05 implements bounded physical
frames, owned page tables, guarded runtime/IST stacks and enforced WP/NX; check the
[memory runbook and story review](phase-1/memory-ownership-runbook.md) before continuing.
P1-06 adds [ACPI/xAPIC/HPET interrupts and delivered-tick uptime](phase-1/interrupts-timekeeping-runbook.md);
its [local acceptance passes](../../evidence/phase-1/P1-06/2026-10-20/README.md).
The current serial contract is version 5. No general heap or scheduler exists yet.
There is no complete M1 readiness, usable desktop, guest driver stack or measured
guest performance result. All ADRs are Proposed. Example
contracts are drafts; sample IDs/digests are synthetic. Do not report implementation
progress from directory names or document completeness.
The user approved the [revision-2 reconciled direction](../00-overview/reconciled-plan.md).
Use it without reopening settled scope; toolkit/inference/loader integrations still
require technical evidence gates.

## Required reading order

1. [Documentation index](../README.md) and [vision](../00-overview/vision-and-scope.md).
2. [Requirements](../00-overview/requirements.md), including deferred scope and metrics.
3. [Architecture summary](../00-overview/architecture-summary.md) and relevant layer specs.
4. [ADRs](../04-decisions/README.md), [live-update invariants](../03-live-update/model-and-guarantees.md),
   and [transaction state machine](../03-live-update/protocol-and-transactions.md).
5. [Milestones](milestones.md), [backlog](backlog.md), [validation](validation.md),
   and [risks](risks-and-open-questions.md).

## Next action

Repository rights and governance are defined in [LEGAL.md](../../LEGAL.md),
[CONTRIBUTING.md](../../CONTRIBUTING.md), and [SECURITY.md](../../SECURITY.md).
Original material is all rights reserved; external patches are paused. Preserve
upstream terms and immutable evidence. File-based policies do not establish
corporate ownership, a security guarantee, or enforced GitHub protections.

The owner's [U.S.-only handling policy](../../EXPORT_CONTROL.md) now places remote
pushes, uploads, sharing, and processing on hold. Earlier push authorization does
not override it. No current GitHub, AI, host, or backup environment is certified
as satisfying it. Keep changes local; do not bypass the installed pre-push guard.

Follow [the Phase 1 epic and seven ordered stories](phase-1/README.md). P1-06 is
accepted, but P1-07 is **Blocked**. Its initial blocker was Windows Smart App
Control refusing the rebuilt xtask host-test executable (4551), including outside the sandbox.
The [blocked local review](../../evidence/phase-1/P1-07/2026-10-20/README.md)
records the failed first ISO comparison, fixed GPT recipe and independent guest
regressions. The owner subsequently authorized a private development signer and
replacement development App Control policy on **this Windows installation**; see
the [setup and lifecycle runbook](windows-development-signing.md). Check its local
deployment/preflight results before assuming execution is unblocked. The replacement
base is enforcing; Windows canceled UAC for the prepared exact-hash tool supplement,
so pinned Cargo/compiler execution and the clean-build preflights remain blocked.
The compiler wrapper is prepared but uncompiled/untested. Do not treat partial guest
checks as a passing gate. Resolve authorized execution, then rerun the complete
`cargo xtask test --suite m1` and explicitly review all 33 rows/42 launches.
P1-03's corrected acceptance and superseded/failed experiments are
retained; P1-02's review is unchanged, and P1-01 has
[host prerequisite evidence](phase-1/prerequisites-runbook.md).
The remaining stories are local plans, not created Jira issues or completed
implementation. Do not skip their evidence gates or mark all of B06 complete in M1.

Re-run the Phase 0 acceptance commands before extending the work. B01/B02 now have
Windows compile/link evidence; B03/B38 have P1-02's minimal UEFI optical route.
Run `cargo xtask doctor --profile m1` against the supplied QEMU/OVMF/ISO inputs,
then confirm P1-06's interrupt/timekeeping review and P1-07's current blocker. Preserve
the owned runtime/IST stacks, GDT/TSS/IDT, exception text mappings and the six
memory/protection cases, including actual double-fault entry. Only eligible
ordinary RAM has a direct mapping; ACPI, modules, framebuffer and loader backing
remain physically reserved. P1-06 explicitly maps validated
ACPI/device ranges with explicit RO/WB and RW/UC policy before use. Preserve its
masked controller startup, all-GPR returning entry, exact version-5 timer records
and eight timer/discovery cases. Uptime counts delivered interrupts and can lag
elapsed time; no wall time or scheduling guarantee exists.
Preserve P1-03's internal BootInfo lifetime rules, exact build/session
correlation and the `boot-metadata`/`boot-reject` regression cases. Do not reclaim
loader memory or dereference ACPI tables merely because descriptors were copied.
Follow with the hosted UI comparison in M2. Do not leap into a
general Wasm runtime, full browser, GPU stack or custom UI language first.
Compare Slint/minimal-custom UI through B36 before broad widget investment and
preserve the actual ISO path demonstrated through B38. Typed actions are required; optional AI
B39 follows the first live-reload demo and never gates the AI-free core.

Rust 1.99.1 is installed only in `work/toolchains/`; dot-source `tools/Enter-Dev.ps1`.
Existing MSVC/SDK were reused. QEMU/OVMF/xorriso are supplied under ignored
`work/tools/` with [input locks](../../config/m1-inputs.lock.json). xorriso's MSYS2
runtime needs execution outside the sandbox's named-object restriction; doctor
reports that restriction as blocked. TCG listings pass; WHPX and virtualization
inspection remain untested/unknown. WSL is not used. P1-02/P1-03 launch bounded offline
TCG VM processes and reaps them after each test.
Keep scratch files under `work/` in this workspace and research
deliverables under `outputs/`; later source layout is proposed in the build spec.

## Preserve these constraints

- Original kernel, not a rebranded existing OS or web-only shell.
- Modern first-party code, with architecture-specific instructions acknowledged.
- Live updates for declared classes without hidden OS/VM reboot.
- State continuity covers unsaved work, not only cosmetic window geometry.
- ISO generation and live update transport remain separate.
- Versioned boundaries, one writer per epoch, safe old-code retirement and honest rollback limits.
- No assumed `std`, GPU API, network crypto, filesystem durability or runtime port.
- Stable recovery path when shell/app/service updates fail.
- Accessibility, text, memory and observability begin early.
- Keep the 256 MiB cold-idle target; VM capacity is not idle usage.
- Evaluate hosted toolkit preview and guest code reload separately.
- Boot-attempt/known-good recovery does not replace live state transfer.

## How to use proposed decisions

Proceed with reversible experiments using their working defaults. They do not mean
the user has validated every library or implementation property. Record approval
of the reconciled direction accurately. Record gate evidence and update ADR status through explicit
project review. Ask only when a missing choice materially blocks work or a change
would violate core intent; branding and optional future hardware do not block M0.

## Contract changes

Read `docs/06-contracts/` before implementing codecs/IPC. Draft field shapes are
starting points. Add independent codec fixtures and semantic compatibility tests
before freezing ABI IDs. If revising fields, update schemas/examples/specs together.
Never serialize native layouts by casting a struct to bytes. Document handle,
allocator, deadline and crash semantics, not only function signatures.

## Implementation reporting

For every completed slice report what now works, exact artifact/build IDs, tests
actually run, evidence paths and current limits. Mark a compile result, boot result,
render result and live-state result separately. Store failure traces rather than
removing them to make demos look clean. Do not claim performance from a host preview
or third-party benchmark. Do not silently degrade update class or data guarantees.

## Research maintenance

Primary sources are in [the catalogue](../01-research/sources.md), with access limits
and date. Recheck moving-version claims when choosing dependencies. Pin actual
versions/commits/digests; the catalogue's `latest` URLs are not build inputs.
Source-backed findings, proposed design and measured results remain separate.

Maintain requirement→spec→task→test traceability. Add ADRs for new architectural
choices; preserve superseded rationale. Keep `AGENTS.md` and `CLAUDE.md` short and
pointing here so agents share one architecture rather than divergent instructions.
