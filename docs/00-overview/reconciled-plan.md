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

The bootloader remains a measured choice: Limine is the working default; Rust OSDev
bootloader is a candidate. Either must cold-boot the actual ISO artifact. A working
disk image is not sufficient evidence for an optical-boot claim.

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
