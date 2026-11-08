# Reconciled implementation direction

Revision 2. Date: 2026-10-17. The user approved incorporating the reviewed external
plan with the changes below. This authorizes the revised roadmap; it does not mean
candidate integrations have been implemented, measured or technically validated.

## Decisions carried forward

Keep an original Rust kernel, hardware-isolated services, capability-controlled
interfaces, software rendering first, a VM-first platform, immutable system content
and separately buildable ISO. Reuse portable components only after exact-target
integration tests. Existing-kernel substitution and hidden VM reboot remain out of scope.

## Six reconciled changes

| Change | Implementation consequence | Evidence gate |
| --- | --- | --- |
| Live reload is an early product goal | M2 proves hosted continuity; M4 proves guest resource reload; M5 proves app behavior replacement before optional AI | Unsaved editor text, selection and window geometry survive; same kernel boot ID/uptime |
| Slint competes with a custom UI core | Compare two bounded prototypes through the same app/surface/input/action boundary; do not build a full custom widget suite before the comparison | M2 provisional selection; M4 confirms exact guest configuration |
| Typed actions are platform infrastructure | Buttons, shortcuts and automation call the same versioned operations; an optional model can propose those operations later | R19, ADR-0015 and the actions contract |
| AI is optional after the baseline | Inference is a separate unprivileged service experiment; no model in the kernel or mandatory boot/desktop/reload path | B39 / E10 only after M5; does not block core release |
| Memory goals remain tight | 256 MiB provisional cold-idle target for the declared baseline; account for all caches and report model/active/reload peaks separately | T09 with named profile; VM RAM allocation is not idle footprint |
| Boot recovery and live updating are independent | Implement boot-attempt/known-good generation selection separately from in-session state transfer; evaluate A/B packaging later | B40 and M6/M8 recovery tests |

## Installer and accounts amendment — 2026-10-21

The owner approved a VM-first graphical, keyboard-accessible offline installer,
real local accounts and install/repair/data-preserving reinstall as mandatory M8
outcomes. Account creation occurs on first installed boot before ordinary desktop
access. This supersedes the preliminary live-only-M8 and optional-account/installer
scope; earlier live technology previews retain separate scoped release reviews.

M3 reserves trusted identity and narrowly scoped installer/account authority; M4
adds trusted setup/login input prerequisites; M5 constrains credential checkpoints
and active installer-writer replacement. M6 selects and experimentally validates
installed-boot/storage/payload foundations and reviews entropy/credential feasibility.
M7 implements B41–B44, all required by B35/M8. Optional AI and other independent
extensions remain outside that new release dependency chain. See
[milestones](../05-delivery/milestones.md) and the [installer plan](../05-delivery/installer-preparation.md).

Existing M0–M2 results, current P2 stories and historical evidence remain unchanged.
P3–P8 are decomposed only at the owner's request before each milestone. This
amendment approves roadmap direction, not an implemented installer, authentication
result, dependency selection, physical-disk operation or publication authorization.

The bootloader remains a measured choice: Limine is the working default; Rust OSDev
bootloader is a candidate. Either must cold-boot the actual ISO artifact. A working
disk image is not sufficient evidence for an optical-boot claim.

## Desktop and application-platform expansion — 2026-10-21

The owner approved preserving M0–M8, including installation/accounts, and extending
the roadmap through M21. M9–M13 deliver everyday files/desktop, software management,
independent documents/portable-settings backup and administration, networking and
audio/assistive integration. M14 accepts that usable VM desktop; M15 qualifies one
named PC after the VM release. Broader hardware support remains outside that profile.

M16 researches scoped x64 EXE compatibility and compares native-service/Wine routes;
M17 studies native versus Windows-route browser feasibility. Neither promises
existing app/browser support or blocks M8/M14. GPU, SMP, Wasm and AI retain separate
M18–M21 experiment gates. Their prerequisite graphs, not milestone numbers, determine
when they can start; AI remains optional after M5, independent of Wasm.

B45–B60 bring the broad backlog to 60, preserving B01–B44. B56 and B58 are the new
VM/PC release roots. Backup excludes credentials/secrets/privileged policy by default
and restores through explicit ownership mapping into an authenticated account.
No accepted M0–M2 result/P2 story changes. P3–P21 decomposition waits for the owner's
request before each milestone. See the [feature matrix](../05-delivery/desktop-feature-matrix.md)
and [desktop service responsibilities](../06-contracts/desktop-services.md).
The original kernel, supported live development, independent ISO and 256 MiB
baseline target remain central. This amendment authorizes planning edits only,
not ports, physical writes, executable downloads or publication.

## Display expansion amendment — 2026-11-01

The owner requested sequenced 4K and multi-monitor delivery with a dedicated display
milestone. M4 seeds output/scale/ownership contracts; M9–M13 prepare settings, SDK,
portable preferences/session recovery and semantic coordinates. M22 follows M14
and precedes M15 qualification: single 4K, two equal-scale outputs, mixed/fractional
scaling, hot-plug/independent pacing and exact-image acceptance, in that order.
B61–B66 expand the broad backlog to 66; existing IDs and M8/M14 exits remain intact.
M22 is required display delivery, independent of optional M18 acceleration or other
research tracks. M15 repeats the supported cases on the selected physical profile.

The [display roadmap](../05-delivery/display-roadmap.md) defines decomposition
slices, matrix, continuity/failure tests and memory gates. Keep the 256 MiB baseline
target; lock and review extended-profile budgets before accepting measurements.
HDR, VRR, rotation, mirroring and arbitrary GPUs remain separate future gates.
This amendment authorizes planning edits, with no P-level stories, new implemented
support, changed historical evidence or physical execution/publication approval.

## First meaningful product demonstration

1. Boot the original kernel/desktop from the ISO in the declared QEMU profile.
2. Open the editor, create unsaved text, select a range and position its window.
3. Change both a UI resource/layout and app behavior on the host.
4. Build, validate and send updates into the still-running guest.
5. Show the new appearance/behavior while retaining accepted document state,
   selection, window position and usable focus.
6. Prove unchanged kernel boot ID/uptime and no VM reset/snapshot substitution.
7. Reject an incompatible update cleanly; inject a candidate failure and recover
   according to the declared class without falsely claiming universal rollback.

A crashed optional inference worker is a later resilience test, not a substitute
for this live-development demonstration. An AI-free first OS is fully consistent
with this reconciled scope.

## UI boundary clarification

Our OS always owns compositor/window policy, input routing, capabilities, application
lifecycle and live state transfer. A selected toolkit may own app layout/widgets and
rasterization. Slint need not export our custom scene graph: it can present buffers
through the common surface contract. Keep the custom retained-scene protocol optional
for toolkit internals until the comparison chooses the integration path.

## Status and navigation

[UI comparison](../01-research/ui-toolkit-comparison.md),
[typed actions and optional AI](../02-architecture/actions-and-optional-ai.md),
[boot recovery](../02-architecture/boot-recovery.md),
[milestones](../05-delivery/milestones.md) and
[backlog](../05-delivery/backlog.md) contain the operative details. New ADRs and
revised existing ADRs remain Proposed until their technical gates pass. No new
implementation result is implied by this reconciliation.

## AI platform amendment — 2026-11-07

The owner requested incorporation of the AI review and selected Gemma as the
default local family, flagship GPT/Claude/Grok/Gemini public options, enterprise
federation and cooperating models. This amends the historical small-model-only
scope above; isolation, native typed actions and independent core exits remain.
The assistant is available by default in qualified AI profiles and can be turned
off. It need not continuously infer, observe or keep a model resident.

Owner choices: final external send/publish review with scoped standing rules;
automatic ordinary preference learning with editable memory; a foundation for
OS-sponsored allowance and a deterministic policy engine enforcing applicable
organization/device/workspace/user/task/sponsor budgets. No allowance amount,
personal-charge fallback or funded service is promised.

[R20–R27](requirements.md#conditional-ai-profile-requirements),
[ADRs 0030–0033](../04-decisions/README.md),
[AI architecture](../02-architecture/ai-platform.md) and
[the AI roadmap](../05-delivery/ai-roadmap.md) retain all 21 concerns and acceptance
gates. B39/M21 now qualifies local Gemma after M5; B67–B82 add contracts, policy,
durable personal work, verified packages, UX, private/public routes, cooperation,
federation and native-server feasibility. M23–M26 name the new independent gates.
All new broad tasks are Not started; the host budget oracle is preparation only.

Keep 256 MiB model-unloaded cold idle including dormant AI, and separately report
fully counted resident/active/actual-default/loading/replacement profiles. Preserve
R01–R19, M8/M14/M22/M15 exits, P4-02 as the next implementation step, accepted
evidence and all existing deadline/security/remote-handling rules. No new runtime,
GPU, guest network/credential stack or native server capability is assumed.
Native data-center execution needs a separately approved program after M26 study.
Dates remain unestimated pending exact-target feasibility; early contract work may
start now, and M21 cannot start before the first M5 reload prerequisite. No P5–P26
stories are generated until the owner's milestone-decomposition request.

## Accounts and authorization amendment — 2026-11-08

The owner requested the [authorization plan](../05-delivery/authorization-roadmap.md):
Cedar-first evaluation, Administrator/Standard/Guest presets, distinct workload
identities, scoped roles and inheritance/delegation ceilings, explicit elevation,
permissions explanations and separate enterprise/data-center authorization gates.
[ADR-0034](../04-decisions/0034-native-authorization-and-scoped-roles.md) remains
Proposed; no engine port, credential mechanism or security result is accepted.

B83–B89 bring the backlog to 89 broad tasks, all new tasks Not started. M27 owns
enterprise OS identity/fleet authorization; M28 owns only the named workload and
data-center authorization profile. They are independent of AI M25/M26 and do not
establish a native server runtime or production data-center platform.

Preserve current P4-03–P4-19 order, B01–B82 dependencies, M7's existing local-account
scope, M8/M14/M22/M15 exits, accepted evidence, deadlines and the 256 MiB cold-idle
target including dormant policy services. Expanded presets/UX and engine selection
are separately gated additions, not new requirements for those releases. Detailed
later stories await an owner decomposition request.
