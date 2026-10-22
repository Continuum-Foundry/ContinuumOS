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
M1's complete experimental VM gate now passes local acceptance. There is no usable
desktop, general driver stack or measured guest performance result. All ADRs are Proposed. Example
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

All [seven ordered Phase 1 stories](phase-1/README.md) now pass local acceptance.
P1-07's [complete review](../../evidence/phase-1/P1-07/2026-10-20-acceptance/README.md)
checks 33 rows and 42 launches: 20 ready, nine expected fatal, eleven expected
rejection and two deliberately broken before entry. Five consecutive normal boots
and the second clean ISO boot pass; kernels match and ISO bytes differ only in
the permitted session field. This is local Codex self-review, not independent approval.
The earlier GPT comparison failure and blocked host execution remain immutable.
The owner authorized a private development signer and replacement App Control
policy on **this Windows installation**; the base and exact-hash tool supplement
are enforcing. Fresh executable and clean Cargo build-script/proc-macro/test/run
preflights pass. See the [signing lifecycle and recovery runbook](windows-development-signing.md).
The runner uses an argument array in the local Cargo home and a fresh signed build
cache. Check enrollment/key expiry and actual tool pins before future acceptance runs.
The next bounded workstream is M2's hosted Slint/minimal-custom comparison; no UI
toolkit is selected by dependency compilation, and no guest reload exists yet.
Follow the [Phase 2 epic and thirteen ordered local stories](phase-2/README.md).
[P2-01's prerequisite/profile recheck](phase-2/01-hosted-prerequisites-and-profiles.md)
now passes [local acceptance](../../evidence/phase-2/P2-01/2026-10-20/README.md):
fresh offline Phase 0/signing/doctor checks and 23 headless lifecycle cases. Use the
[hosted runbook](phase-2/hosted-prerequisites-runbook.md). P2-02's
[local acceptance](../../evidence/phase-2/P2-02/2026-10-21/README.md) now supplies
the shared surface/input/token/layout drafts, locked corpus, 56 negative mutations
and independent schema/determinism checks. Use its [runbook](phase-2/shared-fixtures-runbook.md).
P2-03's [local acceptance](../../evidence/phase-2/P2-03/2026-10-21/README.md)
adds the portable document owner, one typed executor across four headless adapters,
modeled contexts/rights and explicit bounded logical checkpoints. Use the
[action/state runbook](phase-2/document-actions-runbook.md). Segmentation is pinned
to unicode-segmentation 1.13.3 / Unicode 17.0.0; its cached dependency/lock was
reviewed and the existing compiler signer lock enrollment refreshed. Provider
replacement and secure capabilities remain open. P2-05 supplies custom candidate
font/shaping integration and rendering; P2-06 now exercises Slint with that same
explicit text Image adapter, without crediting native Slint shaping.
P2-04's [local text acceptance](../../evidence/phase-2/P2-04/2026-10-21/README.md)
adds six pinned Noto fonts, the bounded custom text pipeline, 45 text/scale
observations, independent native HarfBuzz references, official segmentation tests
and explicit composition/cache/coverage limits. Use the [text runbook](phase-2/text-conformance-runbook.md).
The repaired Phase 0 source-path test includes crates; all ten prerequisites pass.
P2-05's [custom candidate acceptance](../../evidence/phase-2/P2-05/2026-10-21/README.md)
now passes 150 captures/300 workflow states, 45 text and 63 integrated text captures,
28 boundary/failure checks, deterministic reruns and a native Windows preview replay.
Use the [custom runbook](phase-2/custom-ui-runbook.md). Both compiled behavior
variants and freestanding library checking pass; this is not guest link closure,
reload, native IME/accessibility or performance acceptance. Full redraw, copies,
hard rounded coverage and immediate standard transitions stay visible for P2-08.
The owner explicitly approved refreshing the existing signer lock enrollment;
no new certificate or App Control policy was created. Check the retained review.
P2-06 [passes the bounded comparison evidence gate](../../evidence/phase-2/P2-06/2026-10-21/README.md):
Slint 1.18.1 AOT/software output, the common matrix, explicit shared adapters,
real properties/TouchArea probes, compiled text-entry variants and native replay.
Its [runbook](phase-2/slint-ui-runbook.md) records native text setup, rich effects,
rounded ancestor clips, launcher opacity and fractional blend failures. Semantics
remain shared adapter-derived; interpreter input is unavailable offline and
OS/API distribution terms remain unresolved. Existing signer enrollment was
refreshed after reviewing unchanged prior probe pins; no host policy changed.
P2-07 is next; P2-07–P2-13 remain Not started. They cover
freestanding link feasibility, provisional decision, selected hosted
UI/reload and measured acceptance. They do not authorize remote Jira creation.
P1-03's corrected acceptance and superseded/failed experiments are
retained; P1-02's review is unchanged, and P1-01 has
[host prerequisite evidence](phase-1/prerequisites-runbook.md).
The remaining stories are local plans, not created Jira issues or completed
implementation. Do not skip their evidence gates or mark all of B06 complete in M1.

Re-run the Phase 0 acceptance commands before extending the work. B01/B02 now have
Windows compile/link evidence; B03/B38 have P1-02's minimal UEFI optical route.
Run `cargo xtask doctor --profile m1` against the supplied QEMU/OVMF/ISO inputs,
then confirm P1-06's interrupt/timekeeping review and P1-07's complete acceptance. Preserve
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
