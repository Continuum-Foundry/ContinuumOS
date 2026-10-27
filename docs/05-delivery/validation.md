# Validation strategy and requirement traceability

Status: acceptance plan with [Phase 0 compile/tooling evidence](phase-0.md).
Host validator/doctor checks, clean freestanding links and dependency compilation
have run. P1-02 adds minimal original-kernel entry from the actual UEFI ISO, bounded
emulator exits and negative boot/cleanup checks; see its
[runbook](phase-1/minimal-iso-runbook.md) and [evidence](../../evidence/phase-1/P1-02/2026-10-18/README.md).
P1-03 adds checked owned descriptors, per-launch guest identity, resource
summaries and bounded rejection cases; see its
[runbook](phase-1/boot-metadata-runbook.md) and
[review](../../evidence/phase-1/P1-03/2026-10-18/README.md).
The corrected matrix passes 18 host tests and 17 VM launches: seven positive
metadata boots, eight expected metadata rejections and two negative ISO boots.
All processes are reaped. This remains a bounded subset of T01/T02/T12 supporting
R01/R03/R17/R18 and preparatory R14 discipline, not process ABI/security completion.
Every result must include source/build IDs, configuration and raw evidence.

P1-04 extends T01/T02 and R01/R03/R17 with the
[fatal exception matrix](phase-1/exception-diagnostics-runbook.md): actual invalid
opcode, non-present read, Rust panic and an invalid opcode with the logger gate
held. Exact symbolized contexts and exit 37 are required. P1-05 adds T03 memory
ownership, zeroing/reuse, checked mapping and actual guard/RO/NX/double-fault cases;
see its [runbook and acceptance status](phase-1/memory-ownership-runbook.md).
This extends R01/R03/R17/R18 and early R10 groundwork. Process containment and
broader M1 acceptance remain later gates. P1-06 adds T01/T02/T04 and
R01/R03/R17/R18 evidence through [accepted timer/uptime tests](../../evidence/phase-1/P1-06/2026-10-20/README.md):
real delivery/acknowledgement, 100 monotonic reads, fresh epochs, masks,
register/DF sentinels, halt continuations and post-delivery diagnostics. This
supports early R09/R10 groundwork without measuring performance or completing
isolation. P1-07 adds the aggregate suite, final readiness and clean ISO comparison,
with [complete local acceptance](../../evidence/phase-1/P1-07/2026-10-20-acceptance/README.md):
33 rows, 28 Rust host tests and 42 launches. Outcomes distinguish 20 ready,
nine expected fatal, eleven expected rejection and two broken-before-entry cases.
Earlier failed/blocked attempts remain separate immutable evidence.

For M1's bounded subset, use the [Phase 1 story acceptance matrix](phase-1/07-phase-1-acceptance.md).
Final local acceptance passes M1's declared experimental VM baseline; broader
product, isolation, performance and live-update requirements remain open.

M2's planned hosted subset is decomposed into the
[Phase 2 stories](phase-2/README.md) and [required acceptance matrix](phase-2/13-phase-2-acceptance.md#required-acceptance-matrix).
P2-02/P2-04/P2-05/P2-06/P2-10 cover T05/T10 scenes, text, input and semantics;
P2-03 supplies R19/T14 interface/model fixtures, not secure IPC or replacement.
P2-07 covers exact T02 target compile/link feasibility; P2-08 compares candidates
before P2-09's decision. P2-11/P2-12 cover hosted L0/L1 T07 continuity, T09
idle/damage/memory/latency and 100 compatible reloads. P2-13 reviews final builds
and T13 consistency. P2-01's [local review](../../evidence/phase-2/P2-01/2026-10-20/README.md)
adds fresh T02/T13 prerequisites and T09 profile setup: 15 xtask tests and 23
lifecycle cases, with all 15 launched children reaped. P2-02's
[local review](../../evidence/phase-2/P2-02/2026-10-21/README.md) adds T05/T10
corpus preparation and T13 contracts: seven positive inputs, 56 negative
mutations, two overflow checks, three nonfinite parse checks, independent schemas,
18 xtask tests and two exact 10,000-row generations. The 30 visual configurations
and 150 captures are declared future coverage, not rendered passes.
P2-03's [local review](../../evidence/phase-2/P2-03/2026-10-21/README.md) adds
T05/T07 logical continuity and T14 hosted action/model checks: 21 operations across
four entries, 56 negative entry/case pairs, bounded snapshot/history/retry tests,
independent schema/codec checks and portable library compilation. This does not
test actual widgets or provider replacement. P2-04's
[local text review](../../evidence/phase-2/P2-04/2026-10-21/README.md) adds T05/T10
text preparation: 45 text/scale observations, six independent native shaping
references, 766 official grapheme and 1944 word cases, six modeled composition
scenarios and 14 bounds/cache checks. P2-05's
[accepted custom candidate](../../evidence/phase-2/P2-05/2026-10-21/README.md)
adds hosted T05/T10 rendering/routing/semantics: 150 checkpoints, 300 workflow
states, all 45 text observations, 63 integrated text captures, 28 boundary/failure
checks and a native Windows replay. T09 receives counters and deterministic idle
decisions only; full workloads, process/guest memory and efficiency acceptance
remain later gates. P2-06
[adds Slint evidence](../../evidence/phase-2/P2-06/2026-10-21/README.md) for the same
matrix, 29 shared/adapter boundary checks, native primitive/API probes and Windows
replay. Shared text Images pass; native Text setup, rich effects, rounded ancestor
clips, launcher opacity, fractional blending and native semantics remain gaps.
P2-09 records the [provisional custom Rust selection](phase-2/ui-selection-review.md)
under T13 review: target closure, all comparison axes, exact licenses/notices,
source ownership, Slint freeze and required P2-10–P2-13 completion cases. No
new render/measurement/guest result follows from selection. P2-10 now passes
[selected hosted-core acceptance](../../evidence/phase-2/P2-10/2026-10-23/README.md): layout/input/semantic/motion tests,
30 selected configurations, all 45 text/63 integrated captures, full checkpoint
parity, independent edge/semantic assertions, pressure rejection and native replay.
P2-11 now passes [hosted T07 continuity and T05/T10/T14 publication/input checks](../../evidence/phase-2/P2-11/2026-10-23/README.md): 21 publications, 20 negatives, full checkpoints and native same-window generation 7-to-8 proof. P2-12 now passes [hosted T09 damage/idle/metrics and T07 repeated-update checks](../../evidence/phase-2/P2-12/2026-10-24/README.md): 108 exact images, 300 cache probes, 3,000 frame / 6,763 process samples and 100 compatible file publications at each resolution, plus headroom and opaque-root rejection. Frame/resource-latency misses remain explicit. P2-13 now passes [final T02/T05/T07/T09/T10/T13/T14 hosted acceptance](../../evidence/phase-2/P2-13/2026-10-24/README.md), including clean selected links and current-source regressions;
P2-07/P2-08 evidence is described below.
Hosted R04/R08/R09/R15 and R19 design remain distinct from full
guest/product requirements, M3 isolation, M4 guest resources and M5 behavior transfer.

P2-07's [local final-link acceptance](../../evidence/phase-2/P2-07/2026-10-22/README.md)
adds T02/T13 and R03/R18 evidence: both real candidate APIs close target symbols
with bounded probe allocation, independent ELF/codegen inspection, 26 rejected
mutations and per-candidate missing-hook/std failures. All ten prerequisites,
51 Rust unit tests and both unchanged hosted matrices pass. Active dependency
source/feature/notice identities and M3/M4 obligations are explicit. This is early
R14/R16 discipline, not a process ABI, release reproducibility, guest UI or runtime
acceptance. The separate interpreter input is unavailable offline.

P2-08's [local comparison review](../../evidence/phase-2/P2-08/2026-10-22/README.md)
adds hosted T05/T07/T09/T10/T14 comparison observations: 20 paired blocks,
6,000 frames, 12,000 process samples, independent image/state/semantic review,
100 alternating direct token/layout publications per candidate and compiled
snapshot-restoring helper variants/failures. Raw distributions, damage, allocation
traffic, observer idle CPU and process peaks remain separate from guest metrics.
Both candidates miss frame budgets; Slint retains shown components and exceeds
1 GiB of host commit. Those are findings, not performance acceptance. Native
clock/caret/drag/assistive workloads, file-driven resource publication and active
behavior routing remain unsupported here. P2-09 now records the provisional
selection; P2-10–P2-13 now pass the final hosted subset. Guest gates remain M3–M5.

[P1-01 host prerequisites](phase-1/prerequisites-runbook.md)
pass version/capability/hash checks and rejection tests; this adds no ISO boot result.

M3 has [fourteen Phase 3 stories](phase-3/README.md). P3-01 [passes local acceptance](../../evidence/phase-3/P3-01/2026-10-25/README.md): ten current prerequisites, full M1 baseline, 42 P3 contract/profile/host lifecycle checks and eight reaped children. This adds T02/T13 prerequisite and T09 profile setup only. P3-02 [passes local owned-memory acceptance](../../evidence/phase-3/P3-02/2026-10-25/README.md): 22 kernel host tests, independent ELF/raw-table audits, 11 evidence mutations, 1000 exact guest settlements and the complete current 33-row/42-launch M1 gate. This adds the internal T02/T03 memory subset and T09 inventory groundwork; ring-3 enforcement remains P3-04/P3-06. P3-03 [passes staged-loader acceptance](../../evidence/phase-3/P3-03/2026-10-25/README.md): 28 kernel host tests, three independently linked ELFs/72 raw-table-audited pages, exact BSS/startup/stack bytes, 19 malformed/truncated guest cases, 24 insufficient page quotas, wrong digest, forged authority/stale generation denial and 14 reviewer mutations. The complete current M1 and 1000-cycle owned-memory regression pass. This adds the loader/launch staging portions of T02/T03/T11; ring-3 execution and public child authorization remain later gates. P3-04 [passes local user-boundary acceptance](../../evidence/phase-3/P3-04/2026-10-26/README.md): 30 kernel host tests, four bounded guests/36 process runs, checked pointer/error/flags/IRQ cases, sixteen contained offenders with immediate healthy replacements, inventories and independently parsed/mutation-tested evidence. This adds the syscall/runtime/fault portions of T02/T03/T11 while preserving the full 33-row/42-launch M1 gate. P3-05–P3-14 remain Not started.
Its [required final matrix](phase-3/14-phase-3-acceptance.md#required-acceptance-matrix)
covers T02/T03 user entry, loader, CPU state and isolation; T06/T11 handles, IPC,
trusted provenance, scoped ownership/grants, invalidation and writer fencing;
and T09 runtime resource/fairness measurements. P3-12 requires separate clean SDK
client/service builds and an independent codec consumer. P3-13 owns sustained
adversarial lifecycle/accounting; P3-14 aggregates final-source guest tests and
complete M1 regressions. These are future tests, not new passing evidence for
R10/R14 or guest UI, live updating, accounts, durability or desktop efficiency.

## Requirement matrix

The approved M9–M21 expansion has a [feature/release matrix](desktop-feature-matrix.md)
mapping owner, milestone, backlog and explicit future acceptance. B35/M8 retains
its installer/accounts scope; B56/M14 adds the declared usable desktop; B58/M15
requires actual named-PC qualification after the VM release. M16–M21 research is
not a hidden prerequisite of either VM release. No new feature is claimed by this plan.

| Requirement | Test families | Passing evidence |
| --- | --- | --- |
| R01 original OS | T01 boot/provenance | Original kernel and declared upstream boundaries |
| R02 ISO | T01, T12 release | Exact ISO cold boot offline without host shares |
| R03 language | T02 toolchain/ABI | Freestanding builds; audited assembly/first-party C inventory |
| R04 live resources | T05 UI, T07 reload | Scene updates, preserved work, unchanged kernel boot ID/uptime |
| R05 live app logic | T07 | New behavior/build instance, preserved logical state, safe old retirement |
| R06 service updates | T06 IPC, T07, T08 failure | Fenced writers, coherent routes, accepted request preservation |
| R07 continuity limits | T07, T08 | Schema rejection, migration fixtures, honest partial-continuity outcomes |
| R08 exceptional UI | T05, T10 visual review | Reference scenes, interaction review, scale/focus/clipping checks |
| R09 efficiency | T09 workload | Frame distributions, memory/idle/peak/reload costs on declared profiles |
| R10 isolation | T03 memory, T04 device, T06, T11 security | User faults contained; trusted principal/session provenance, cross-owner and same-user app grants enforced; permissions/limits and revocation scope tested; DMA trust stated |
| R11 persistent files | T04, T08 storage | Acknowledged durable writes survive modeled faults |
| R12 recovery | T07, T08, T12 | Known outcome after every transition; last good/rescue path |
| R13 dev boundary | T11 | Bad authority/replay/host-command requests rejected; release disabled |
| R14 contracts | T02, T06, T07 | Independent clients/codecs, versions and migration compatibility fixtures |
| R15 accessibility/text | T05, T10 | Keyboard/semantic/scaling/script corpus; exact support limitations |
| R16 delivery | T12 | Locks, provenance, license inventory and explained clean-build comparison |
| R17 diagnostics | T01,T07,T08,T11 | Accurate visible/error states, correlated traces, no false success |
| R18 maintainability | T13 documentation | Links, schemas, traceability and status/evidence consistency |
| R19 typed actions | T06,T07,T14 | UI/automation parity, scoped context, expected revisions and reload-safe authority |

## Test families

**T01 — Boot and recovery:** repeated cold ISO boot; malformed/missing boot response;
controlled user/kernel exceptions; rescue generation selection; no network/shared-folder
dependency. Readiness markers and serial output are mandatory; screenshots supplement them.

**T02 — Toolchain/loader/ABI:** kernel/user code models, section alignment, W^X/NX,
entry/return registers, invalid ELF ranges/segments, unsupported relocations, panic
policy and FP/SIMD save/restore. Compare independently built SDK clients.

**T03 — Memory/process isolation:** invalid user pointer, unmap race, guard fault,
cross-process mapping denial, resource exhaustion, process death and cleanup. Validate
zeroed pages and register state so one process cannot read remnants of another.

**T04 — Devices/storage path:** bad descriptors/features, DMA lifetime/reset under
load, lost interrupts, block error/timeout, flush failure, torn/reordered write model.
Separate VM kill tests from a real fault-injecting block model.

**T05 — UI interaction/rendering:** deterministic paint/layout scenes, old/new damage,
occlusion/alpha/filter invalidation, buffer lifetime, idle pacing, keyboard/focus/capture,
gesture interruption and text composition. Keep screenshot and semantic assertions separate.

**T06 — IPC/authority:** malformed frames/messages, length arithmetic, queue saturation,
deadlines/cancel/peer death, rights attenuation, stale handles, handle-transfer rollback,
route epochs, duplicate/reordered requests and independent protocol versions.
M3 also requires the [identity/authority matrix](../02-architecture/identity-and-authorization.md#m3-work-items-and-acceptance):
two synthetic users and a service principal, forged caller/role rejection,
cross-owner denial/explicit sharing, same-user app least authority, scoped role
administration, child launch/transfer ceilings, delegated-channel attribution,
broker restart and session/policy invalidation. Hosted fixtures do not replace
guest enforcement. M5 adds replacement reauthorization; full login and durable
account/RBAC tests belong to mandatory M7 B41 after M6. M3 also rejects unauthorized
disk/account requests using synthetic operations; it does not implement the installer.

**T07 — Live update:** L0/L1/L2/L3 separately; compiler failure; corrupt transfer;
wrong base/schema/protocol; quiescence timeout; candidate start/import failure;
headroom shortage; active-input reload; incompatible dependency graph; 100-cycle leak test.
Prove changed behavior and unchanged kernel boot identity. Keep an external host trace
of VM lifecycle as additional evidence that reset/snapshot restore did not fake continuity.

**T08 — Update/persistence faults:** deterministic failure before/after checkpoint,
prepare record, decision flush, writer fence, route publication, health record and ACK.
Crash old/new/coordinator, disconnect host, fill disk, fail flush. Check single writer,
accepted state revision, dedupe history, coherent generation and available recovery blobs.
For irreversible effects require forward-recovery tests, not a fictional rollback pass.

**T09 — Performance:** fixed scripts/seeds for idle, drag/resize, editor scroll/type,
large list, overlap, effects over changing content and concurrent file work. Use optimized
builds, warm/cold distinction, p50/p95/p99, missed deadlines, resource peaks and trace overhead.
Never equate frame submission with display-photon latency.
Cold idle targets 256 MiB including bounded caches; VM capacity is not an idle
allowance. Optional inference adds separate warm-idle/active, load/unload and
weights/attention/scratch/device/shared-memory/peak accounting.

**T10 — Accessibility/international text:** semantic names/actions/focus, keyboard-only
workflows, contrast/non-color states, reduced motion, high scale, bidi, graphemes,
shaping/fallback and long translations. A semantic tree pass is not a screen-reader pass.

**T11 — Security:** replay/invalid session, malformed packages/fonts/images, capability
escalation, excessive imports, guest-directed host commands/paths, dev endpoint in release,
entropy not-ready and TLS/certificate failures once networking exists. Perform threat-specific
review before handling valuable credentials or exposing public services.
M4 tests protected setup/login focus, restricted capture and synthetic sensitive-
event exclusion from logs/inspectors. M7 B41 validates actual guest entropy,
credential verification/storage and authorized recovery before real credentials.
M5 tests exclusion of credentials/privileged installation handles from ordinary
app checkpoints and refusal to replace a modeled active installer writer.

**T12 — Release:** compare two clean builds, verify package/ISO checksums, boot the exact
output offline, validate notices/SBOM/provenance, run rescue/last-good selection and
confirm the support matrix matches actual test evidence.
At M6 add the [installer-preparation matrix](installer-preparation.md#m6-preparation):
non-writing target/layout/payload validation and first-generation storage faults.
For M8, require mandatory B41–B44 and the [installer gate](installer-preparation.md#later-installer-gate):
confirmed target/revalidation, untouched sentinel disks, interrupted-install outcomes,
ISO-detached cold boots with fresh firmware variables, persisted user data and rescue.
Graphical installation and restricted first-owner setup must pass keyboard-only and
pointer workflows, login/logout/lock, scoped role administration and cross-user
denial. Required repair and preserving reinstall retain files, settings, account/
credential records and ownership on the supported format; unknown formats are
refused without writes. Inject interrupted account/role/setup and credential-policy
writes to reject duplicate privilege or login bypass. Validate M6's chosen boot
layout/discovery separately from this full lifecycle. These future tests are distinct
from live ISO boot and ordinary VM data-disk tests. Earlier live-only preview reviews
do not satisfy T12's M8 account/installer gate.

**T13 — Documentation/contracts:** local links/anchors, JSON parsing/schema fixtures,
all requirement IDs covered, ADR status/evidence, no proposed command represented as
existing, and source citations for externally derived claims.

**T14 — Typed actions:** compare button/shortcut/automation invocation of the same
operation; reject fabricated context, stale revision/preview, extra authority and
unsupported versions. Test duplicate requests and action-provider replacement.

**T15 — Optional inference:** exact supported model/operators/backend, FP/SIMD,
quotas, invalid proposals, malicious context, worker crash and unload/reload. Measure
editor responsiveness and all memory profiles. This is not a core-release gate.

## Later desktop and research acceptance

M9/B45–B47 extends T05/T06/T08/T10/T14 with real file/picker and cross-app exchange
workflows: name conflicts, denied roots, interrupted/cancelled copy/move, trash
restore, missing default apps, expired/revoked offers, notifications and complete
keyboard use. Persist regional/keymap settings and adjust wall time without
affecting monotonic deadlines or file identity.

M10/B48/B49 extends T06/T07/T08/T11/T12/T14 with independent SDK apps, scoped
install/update/uninstall, permission-increase review, explicit data retention,
publisher/trust rejection, interrupted activation and reboot-required outcomes.
Offline/online delivery share one verifier and remain separate from dev authority.

M11/B50–B52 extends T03/T08/T09/T11/T12 with a coherent document/portable-settings
backup on a separate destination and verified fresh-install restore after source
guest-disk loss. Reject corrupt/truncated/path-escaping input and unauthorized
ownership mappings; exclude secrets/credentials/privileged policy by default.
Test hung apps, unsafe service-control denial, failed flush/forced shutdown,
revoked session grants, crash-loop suppression and bounded diagnostic export.

M12/B28/B53/B54 extends T04/T06/T09/T11/T12 with declared virtual network protocol
coverage, packet loss/malformed traffic, DHCP/DNS changes, trust/time/entropy and
certificate failures, interrupted downloads and offline usability. Verification
bypass is a failure. M13/B31/B55 extends T04/T05/T09/T10/T11 with audio negotiation,
capture permission/indicator/revocation, underrun/reset cleanup and validated
assistive speech for declared language/device profiles.

M14/B56 re-runs the exact combined ISO/installed lifecycle, clean-build, fault,
keyboard/accessibility and 256 MiB baseline/full-resource matrices. M15/B57/B58
requires profile-specific physical boot/device/flush/power/install/backup/rescue
evidence, not VM substitution or permission inferred from a roadmap document.

M16/B59 follows the [Windows evidence ladder](../01-research/windows-compatibility-feasibility.md#proposed-m16-evidence-ladder):
PE bounds/ABI/import/runtime/GUI probes, negative authority/resource cases and an
explicit subset or stop/replan outcome. M17/B60 follows the
[browser procedure](../01-research/browser-feasibility.md#m17-procedure-and-acceptance),
including sandbox/maintenance feasibility; no-sandbox launch cannot pass. M18–M21
retain separate acceleration/SMP/Wasm/AI evidence and resource accounting.

The documentation audit checks 60 unique B IDs, M0–M21, dependency cycles/known
references, required release closures and optional-track exclusion. It also checks
every feature row's owner/classification, valid milestone/task references and
acceptance outcome. These metadata checks do not execute any future OS acceptance.

Revision-2 T05 also compares Slint/custom target configuration and records text/effect/
semantic limits. T01/T12 require the actual ISO and boot-attempt/health/fallback cases;
live transaction tests cannot substitute for boot recovery.

## Test placement and limits

Pure state machines, codecs, layout and storage models run on host. Guest integration
tests validate privilege transitions, IRQ/device behavior and actual rendering. Use
property/fuzz tests for untrusted parsers and migration bounds. Loom/Miri are helpful
within supported models; they cannot prove full hardware correctness.

Do not create implementation-mirroring tests for trivial cosmetic changes. Focus new
tests on behavior, boundaries and previously uncertain risks. Once relevant checks
pass, do not repeat unrelated suites without a reason.

## Evidence bundle layout

Proposed `evidence/<run-id>/`: `config.json`, input revision/locks, artifact hashes,
serial log, guest events, host lifecycle log, metrics, screenshots, fault script/seed,
assertion results and limitations. Include exact test/harness revision. Any manual
intervention is recorded. Redact secrets and document content before sharing.


The [post-P2-13 performance follow-up](../../evidence/phase-2/performance-remediation/2026-10-24/README.md)
re-runs complete M1 (33 rows/42 launches), all sixteen hosted acceptance rows and
24,000 primary tail frames. Separate raw consumers validate exact before/after
pixels, 108 incremental/full images, 200 file reloads and six malformed-profile
mutations. These hosted elapsed/RAM observations leave all guest CPU/input/memory
targets unaccepted; the original P2-13 bundle remains immutable.
