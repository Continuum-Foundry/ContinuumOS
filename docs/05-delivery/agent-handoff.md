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
M2's hosted comparison and selected-core completion now pass local acceptance.
Custom Rust remains provisional pending M4 guest confirmation; no guest reload
exists yet. At the owner's request, M3 now has
[fourteen Phase 3 stories](phase-3/README.md). P3-01 [passes local acceptance](../../evidence/phase-3/P3-01/2026-10-25/README.md): ten current prerequisites, full M1 33-row/42-launch baseline, 42 P3 checks, draft runtime/SDK and locked profiles. Use its [runbook](phase-3/runtime-prerequisites-runbook.md). P3-02 [passes local owned-memory acceptance](../../evidence/phase-3/P3-02/2026-10-25/README.md): 22 kernel host tests, independent ELF/table audits, 1000 exact guest settlement cycles and the complete current M1 gate. Use its [memory runbook](phase-3/owned-memory-runbook.md). P3-03 [passes local staged-loader acceptance](../../evidence/phase-3/P3-03/2026-10-25/README.md): three Rust boot modules, 72 independently audited private pages, transactional failure cleanup and trusted launch-context/reuse denial. Use its [loader runbook](phase-3/static-loader-runbook.md). P3-04 [passes local user-boundary acceptance](../../evidence/phase-3/P3-04/2026-10-26/README.md): four bounded guest launches, 36 process runs, typed syscalls/pinned copies, integer register/flags/IRQ checks, process-local faults/panic/OOM and healthy replacements with exact inventories. Use its [runbook](phase-3/syscall-boundary-runbook.md). Executing launches install generated process_exit; the staged test retains UD2. Next is P3-05. Accept one bounded story at a time; no preemption, user FP/SIMD state or public memory rights are accepted.
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
P2-07 [passes local final-link feasibility](../../evidence/phase-2/P2-07/2026-10-22/README.md):
both real candidate scene/text/raster paths link with an owned bounded probe arena;
target/build inputs, generated AOT code, symbols, segment permissions and deliberate
missing-hook/std failures are audited. The Slint adapter now lives in no-std
`crates/slint-ui`, shared by host and probe. The interpreter input remains unavailable
offline. Use the [freestanding runbook](phase-2/freestanding-ui-runbook.md) and hook
inventory; no guest runtime, UI execution or kernel dependency was added. The owner
explicitly approved the existing signer lock enrollment refresh; no certificate or
Windows policy changed. P2-08's
[local comparison](../../evidence/phase-2/P2-08/2026-10-22/README.md) now passes
the bounded evidence gate: 20 optimized paired blocks, 6,000 frame and 12,000
process samples, independent quality/workflow/semantics checks, 100 direct
token/layout publications per candidate and snapshot-restoring compiled helpers.
Use the [comparison runbook](phase-2/comparable-ui-runbook.md). Custom is recommended
conditionally for P2-09 review, with substantial frame-budget misses. Slint's
per-paint shown components retain references without hide; measured commit grows
beyond 1 GiB. This is a current adapter defect, not an intrinsic toolkit footprint.
Fidelity/native-text/semantic/rights gaps remain explicit. The preliminary unpaced
warm-up is preserved separately; the accepted run uses 50 identical paced frames.
No resource loader/native-window continuity, active helper route, guest UI/reload,
selected toolkit or completed efficiency target follows from P2-08 alone.
P2-09 now records the [provisional custom Rust decision](phase-2/ui-selection-review.md)
and [exact selection/terms/freeze](../../legal/ui-selection.json), with all
eight axes, mandatory criteria and a bounded M2 completion list. Custom is not
selected for speed: measured frame misses remain material. Slint is frozen
comparison/test material; its OS/API terms remain unresolved. Chosen permissive
license alternatives and font/Unicode notices are recorded; distribution remains
uncleared. P2-10 now passes [selected hosted-core acceptance](../../evidence/phase-2/P2-10/2026-10-23/README.md): bounded
layout/invalidation, input retirement and modal/queue policies, semantic hierarchy
and virtual navigation, visual grapheme affinity, rounded coverage and motion.
Use the [selected-core runbook](phase-2/selected-ui-runbook.md). P2-11 now passes [hosted resource reload acceptance](../../evidence/phase-2/P2-11/2026-10-23/README.md): bounded atomic file builds, coherent token/topology publication, reader headroom and native same-window generation 7-to-8 continuity with complete checkpoint preservation. Use the [resource reload runbook](phase-2/resource-reload-runbook.md). P2-12 now passes [hosted damage/idle/stability acceptance](../../evidence/phase-2/P2-12/2026-10-24/README.md): bounded retained damage/layout caching, 108 exact images, 3,000 frames, 6,763 process samples and 100 file reloads per resolution. Its [runbook](phase-2/performance-stability-runbook.md) preserves frame/latency misses and PERF-12-01–03. Opaque-root validation now precedes publication; the original native failure is retained. P2-13 now passes [final aggregate acceptance](../../evidence/phase-2/P2-13/2026-10-24/README.md), including clean linkage of the final retained/cache path. Use its [runbook](phase-2/acceptance-runbook.md). M2's declared hosted subset is accepted locally; M3 now has [fourteen stories](phase-3/README.md), with P3-01/P3-02/P3-03/P3-04 Done and P3-05–P3-14 Not started. Mandatory M2 work is not deferred to guest work. M3 runtime and M4 guest confirmation,
M5 behavior routing/retirement remain pending; ADRs remain Proposed.
P2-01 through P2-13 are Done under local self-review. The owner's subsequent
performance review recommends the [bounded hosted remediation plan](phase-2/performance-remediation-plan.md)
before guest UI integration. The owner authorized implementation; the
[bounded local follow-up](../../evidence/phase-2/performance-remediation/2026-10-24/README.md) now passes with
opaque spans/bounded groups, validated-frame publication, shaped caret damage and
conservative backdrop dependencies. No filtered-backdrop cache was introduced.
Primary 1080p/scale-1 trace-off elapsed frame p95 ranges from 0.789 to 18.642 ms across the twelve workload/effect cells (1,000 samples each). Real file-reload completed-build-to-RAM p95 is 129.527 ms at 720p / 128.116 ms at 1080p. Rich typing/search p95 (18.642 ms) and effects/motion p95 (17.424 ms) still exceed the 16.667 ms period; ten other primary trace-off cells are below it. All misses remain recorded; elapsed/RAM results do not accept renderer-thread CPU, guest input or whole-guest memory targets. M1 passes 33 rows/42 launches; final hosted acceptance and 24,000 primary
tail frames pass their evidence checks. Original P2-13 evidence is unchanged.
No remote Jira creation or transfer is authorized.
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

The owner's M3 planning clarification adds [identity/ownership/authorization foundations](../02-architecture/identity-and-authorization.md)
to local B11–B13 and aligns B15/B37's handoff. Include trusted caller/session
provenance, scoped role-to-capability grants, cross-owner/same-user app denial and
brokered invalidation tests when implementing the Phase 3 stories. Product scope remains
one development user; two synthetic users are enforcement fixtures, not implemented
accounts at M3. The approved installer/accounts amendment now makes M7 B41–B44
mandatory for M8. The [Phase 3 stories](phase-3/README.md) now include these
foundations and required guest enforcement cases. P3-01 accepts prerequisites/contracts, P3-02 accepts the internal memory backend and P3-03 accepts staged static-image loading/trusted launch bindings; P3-04 accepts the bounded integer user-entry/syscall/fault boundary; P3-05–P3-14 remain Not started.

The owner's installer planning clarification seeds [M6 installer prerequisites](installer-preparation.md)
in B22–B24/B40, with B33/B34 packaging/recovery handoff. Include a non-writing
VM target/layout/payload planner and interrupted first-generation cases when M6
is decomposed. M6 selects and experimentally validates the boot layout/payload and
reviews entropy/credential feasibility. M4 seeds trusted setup/login input; M5
excludes secrets/privileged handles from app checkpoints and blocks active installer
writer replacement. M7 adds real accounts and graphical keyboard-complete offline
installation, first-installed-boot owner setup, repair and preserving reinstall.
The exact M8 ISO must pass B44's independent installed-disk lifecycle with accounts,
files and ownership preserved. Live-only previews have separately scoped reviews
and cannot satisfy M8. The four new broad B41–B44 tasks are not P-level stories;
P3 is now decomposed; decompose P4–P21 only at the owner's request before each
milestone. No installer code, disk write, physical-PC operation or publication is
authorized by these edits.

The subsequent approved [desktop/platform expansion](desktop-feature-matrix.md)
preserves M8 and adds M9–M21 with broad B45–B60 (60 total tasks). M7 now focuses on
installation/accounts. B56/M14 accepts M9–M13 desktop/app/update/backup/admin/network/
audio/assistive delivery; B58/M15 qualifies one named PC after the accepted VM.
M16/B59 and M17/B60 are sourced compatibility/browser studies with bounded probes,
not promised ports or release dependencies. B29/B30/B27/B39 now have optional
M18/M19/M20/M21 homes; their actual prerequisites can pass before M15. Keep the
original kernel and avoid universal Windows-app, browser, driver or foreign-reload
claims. Backup covers documents/portable settings with explicit authenticated-owner
mapping and no default credential/policy migration. M3–M6 seed interfaces only;
do not import later features into their exits. P3 is planned; no P4–P21 stories exist yet.

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
