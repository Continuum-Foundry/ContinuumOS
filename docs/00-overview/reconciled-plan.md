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
