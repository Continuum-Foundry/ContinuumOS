# P4-03 bounded device authority acceptance

Status: passed local Codex self-review for the declared device primitives.
This is selected story acceptance, not a new M1/M3/M4 milestone exit, external
approval, IOMMU confinement or a usable guest desktop.

Source: clean `main` fast-forwarded to
`367e046`, followed by the implementation hashes in [source-lock.json](source-lock.json).
The [selection](selection.json) records affected consumers, omissions and the
14-launch ceiling. [Acceptance](acceptance.json) and the
[aggregate review](aggregate-review.json) record fresh final-source checks.
Full execution journals, source snapshots, raw serial, binaries and ISOs remain
local under `build/m4/P4-03/acceptance-1794185451085334400` and its referenced runs.
Only prose, retention and staging change during final host-audit closeout; unchanged
guest artifacts are re-reviewed and reused under the registered resume rules.
[Staging input proof](staging-input-proof.json) checks all 499 implementation
paths: tested working bytes are unchanged; Git text-blob differences are only
listed CRLF/LF normalization, including preexisting files. The raw source lock
does not claim release reproducibility from a normalized fresh checkout.

## What passes

| Coverage | Result |
| --- | --- |
| Kernel host suites | 107 ordinary and 107 device-feature tests; six new broker/DMA test groups |
| Host harness | 18 affected M1 harness tests; one explicit benchmark ignored; 26 selection/resume/budget tests |
| Prerequisites | Ten fresh signing/compiler/Phase 0/tool checks, including clean builds and a source path with spaces |
| Device guest | Optimized original-kernel optical boot; checked modern PCI discovery, trusted grants, two real input queues/device, DMA/reset and owner death |
| Actual queue evidence | 16 used entries across keyboard/tablet, real queue IRQs on GSIs 20/21 and masked-IRQ polling |
| Enforcement | 104 actual CPL3 calls, with trusted process/incarnation bindings, cross-role/range/width/stale-release denials and no calls from pre-entry rolled-back owners |
| Capacity/lifetime | Real three-run admission plus fourth-run backing unwind; two bounded quarantine probes; input-role exit resets a live lease with observed pending receive buffers |
| Physical settlement | Zero retained grants/leases/DMA and exact return of the tested dynamic frame pool to its baseline |
| Independent negatives | 155 missing-criterion mutations rejected by the raw-record oracle |
| Separate builds | All 17 metadata flavors, including ordinary, boundary, scheduler, capability and device subsets |
| Fresh regressions | Ordinary boot, memory, timer, four boundary guests, three scheduler cases including actual missing-SSE2 rejection, capabilities, waits and independent SDK runtime |
| Budget/deadlines | 14 total selected VM launches; no full suites; unchanged 30000-ms guest and inherited local bounds |

Device build ID:
`ab2ac149c37ee04c5fd10a5c0d6cabacc067fc627ef2455f2f5383db6c70ac03`.
Kernel SHA-256:
`e11de1c7e04e20e35ef86641fcec987e90327bf0303753fe93f930c33151b2e3`.
Exact device-test ISO SHA-256:
`32e3b25305dcf1f621c9250d924d88947d60cb8fff25f6c8f1830a3351a96ea8`.
See [device review](device-0-review.json), build/package/case metadata and the
[selected-serial rules](serial-selection.json). Selected excerpts deliberately
retain sequence gaps; they are not substitutes for the complete local streams.

## Supported matrix and limits

| Device | Accepted primitive evidence | Still open |
| --- | --- | --- |
| Q35 keyboard, modern PCI 1af4:1052, 00:04.0 | 64-bit BAR 4, VERSION_1, eight-slot split event/status queues, advertised NumLock feedback, IRQ 20/polling, reset and process-owned lease teardown | Physical key delivery/routing, protected input and isolated driver frontend |
| Q35 tablet, modern PCI 1af4:1052, 00:05.0 | Same bounded queue mechanics, IRQ 21/polling and reset; unsupported LED requests reject locally | Pointer events and tablet feedback features |
| Serial controller, modern PCI 1af4:1043, 00:06.0 | Checked BAR/capabilities, VERSION_1 negotiation and actual reset completion | Port/control-queue delivery, IRQ 22 data delivery, framing, authentication and development transport |
| Boot VGA framebuffer | Existing boot reservation remains protected | P4-04 presentation; no GPU or desktop rendering acceptance |

The actual BARs are above 4 GiB; decoding checks the full 64-bit address, architectural
52-bit bound, selected CPU physical width, capability windows and RAM/reservation
exclusions. New mappings remain supervisor-only RW/NX/UC. The old APIC/HPET helper's
narrower contract is unchanged. Root-slot routing is explicitly Q35-specific.

The tablet's test-only SYN_REPORT sentinels exercise ignored status-queue byte
consumption; they are not advertised tablet feedback or a production fallback.
Keyboard LED requests require the advertised exact code. This distinction preserves
all 16 queue samples while correcting an invalid device-specific assumption.

The temporary kernel mediator and its experimental fixture operations expose
checked descriptions/ranges, not raw user MMIO or a new frozen SDK family. DMA
devices/driver remain trusted for memory integrity without an IOMMU. Only the
existing exact SDK subset is frozen. General drivers, event parsing, input/display/
transport workflows, rendering, live reload and desktop efficiency remain open.
`frames=0` is dynamic-pool settlement, not zero OS memory or a 256 MiB cold-idle pass.

## Failures and review repair

[Development findings](development-findings.json) preserve the initial BAR rejection,
the QEMU tablet-LED assertion and the intermediate reviewer failure. The decoder
had assumed BARs below 4 GiB. After that repair, keyboard queues/IRQs/reset passed,
but sending keyboard-only LED feedback to the tablet triggered an emulator assertion.
Capability checks now reject that unsupported feedback before publication.

The next guest completed successfully. Its first independent review incorrectly
required calls from three owners rolled back before user entry. The repaired oracle
requires those owners to execute no device calls, binds all 104 calls to actual
entered incarnations, and verifies the real input owner at death. That failed
aggregate remains historical; final-source acceptance is a separate fresh run.

The sandbox signer-access failure and default host-test stack overflow are retained.
The approved signer context and existing runbook's serial/16 MiB test-stack settings
pass; no power/security setting, guest deadline, milestone count or standing waiver
was changed. A local evidence-formatting helper correction did not alter guest inputs.

## Reproduction and handoff

Use the [device authority runbook](../../../../docs/05-delivery/phase-4/device-authority-runbook.md)
for exact commands, unsafe/cache/ordering review, supported matrix and extraction
work. Run `tools/Test-P4-03.ps1 -Plan` before execution; review a completed journal
with `tools/p4_device_acceptance.py review --acceptance <journal>` in the same
Enter-Dev/signing environment. Final prose/evidence/index changes rerun host audits
and re-review unchanged guests; they do not restart milestone cascades.

P4-04 is next. P4-06 owns physical input/routing and driver extraction; P4-12 owns
serial port/control/framing; later gates own authentication, UI and live assets.
No account, installer, AI runtime, publication or physical-device claim is added.
