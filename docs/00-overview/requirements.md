# Requirements and feature matrix

Status: user intent plus proposed engineering acceptance criteria. Metrics are
targets, not measurements. M0–M21 are defined in [milestones](../05-delivery/milestones.md).
The [desktop feature matrix](../05-delivery/desktop-feature-matrix.md) distinguishes
M8 installation, M14 desktop delivery, one M15 PC profile and optional/deferred work.

## Core requirements

| ID | Requirement | First acceptance evidence | Gate |
| --- | --- | --- | --- |
| R01 | Original kernel and OS architecture | Build manifest identifies original kernel; no substituted existing OS | M1 |
| R02 | Bootable standalone ISO | M1 optical boot; M8 exact ISO installs offline and the installed VM boots with ISO detached, no host-shared root | M1, M8 |
| R03 | Modern first-party language | Rust source and documented tiny assembly boundary; no first-party C dependency | M0 onward |
| R04 | Live UI resources without OS reboot | Theme/layout updates preserve document and window; boot ID/uptime unchanged | M2, M4 |
| R05 | Live app behavior without OS reboot | New logic used after state migration; old process safely retired | M5 |
| R06 | Selected system services replaceable | Update under load obeys request and resource invariants | M6 |
| R07 | State continuity has explicit limits | Compatibility failures rejected before activation; unsaved text preserved in supported paths | M5 |
| R08 | Exceptional coherent desktop | Design tokens, reference scenes, keyboard behavior, and visual review; later everyday app/file integration | M2–M8, M9–M14 |
| R09 | Efficiency is measurable | Recorded CPU/frame/memory traces for fixed workloads and hardware profile | M2 onward |
| R10 | Isolation and bounded resources | User process cannot write kernel/other app memory; quotas/IPC tests; M7 account/session and installer-authority enforcement | M3, M7 |
| R11 | Persistent user files | Correct flush/recovery under faults; M9 operations and M11 independent file/settings restore | M6, M9, M11 |
| R12 | Recoverable updates | Fault injection at every transition; last good generation boots; verified ordinary delivery and independent data restore | M5–M8, M10–M14 |
| R13 | Secure development boundary | Explicit dev mode, authenticated session, bounded messages, no guest-supplied host commands | M4 |
| R14 | Versioned contracts | Wire compatibility tests, schema migration fixtures, independently built clients; later desktop service contracts frozen at their gates | M3–M6, M9–M15 |
| R15 | Accessibility and international text | Semantic tree, keyboard navigation, scale, graphemes/shaping/bidi; native assistive speech with declared support | M2–M8, M13–M14 |
| R16 | Reproducible delivery | Pinned inputs, provenance, checksums, two clean builds with explained differences per release | M8, M14–M15 |
| R17 | Honest diagnostics | Update status, precise rejection reason, crash logs, no false success | M4 onward |
| R18 | Agent-ready maintainability | Traceable tasks, ADRs, contracts, tests, dependency inventory | This library; continuous |
| R19 | Shared typed actions and scoped context | UI/automation parity; rights, revision, expiry and reload checks | M2 design, M5 evidence |

## Platform and product coverage

“Initial” means a target for the first credible VM desktop, not available today.

| Property | Initial scope | Later scope / deliberate limit |
| --- | --- | --- |
| Firmware | OVMF UEFI x86-64 | Physical UEFI; BIOS optional, not required |
| CPU | One core, baseline x86-64 | SMP, other architectures after portability gate |
| Memory | Paging, isolation, quotas, demand-zero or eager mapping | Swap, memory compression, NUMA deferred |
| Scheduling | Preemption, fairness, bounded priority policy | Deadline/hard real-time contracts deferred |
| Devices | Serial, timers/APIC, PCI, VirtIO input/block/console/display | M15 selects required drivers for one PC; broad NVMe/USB/Wi-Fi/vendor-device coverage remains separate |
| Graphics | One display, software, logical scaling | M18 optional acceleration; mixed-DPI multi-display/HDR require separate evidence |
| Input | Keyboard, pointer, focus, keymap service; M4 trusted setup/login prerequisites, validated before M7 credentials | IME and touch staged |
| Desktop | Initial launcher/windows/settings; M9 cross-app clipboard/drag/defaults/notifications and regional settings | Advanced workspaces and scripting beyond declared desktop scope |
| Apps | Editor, file browser, console, inspector; M10 native SDK and install/update/uninstall | M16 EXE research and M17 browser study do not promise shipping compatibility/browser; media suite deferred |
| Text | UTF-8 storage, grapheme editing, shaping hooks | Fonts/script coverage grows against explicit test corpus |
| Accessibility | Semantic model, keyboard, visible focus, reduced motion | M13 native assistive consumer/validated speech required for M14; broad language/device/braille coverage separately gated |
| Files | M6 durable data; M9 complete operations/trash/pickers; M11 independent documents/portable-settings backup | Complete-machine/credential backup and broader filesystem semantics separately gated |
| Networking | Dev channel first; M12 virtual Ethernet/addressing/DHCP/DNS/TCP/TLS and time/trust required for M14 | IPv6/broader protocols and physical wireless remain explicit later coverage |
| Audio | Interface reserved for M8 | M13 VM playback/capture/mixing/device/permission gate required for M14 |
| User identity | One development user through M6; M3 authority foundations; M7 local accounts/login/scoped RBAC and first-user setup required for M8 | Encrypted homes need a separate key/recovery gate; network identity/roaming deferred |
| Power | Idle halt/timers; M11 bounded shutdown/session recovery; M15 named-PC poweroff | Physical suspend/resume and battery policy deferred |
| Updates | Resource/app/service classes; M10 verified ordinary offline delivery and M12 online client | Driver resets class-specific; kernel/boot require reboot initially |
| Installation | M6 preparation; M7 graphical offline VM installer/accounts/repair/preserving reinstall required for M8 | M15 one named physical PC; dual boot/resizing/encryption/Secure Boot remain further gates |
| Recovery | Rescue ISO, last good generation, serial logs | Signed recovery and key recovery after release security design |
| Compatibility | Native SDK first; M20 bounded Wasm; M16/B59 x64 Windows study and probes | No implicit POSIX/Win32/Linux ABI, managed runtime or browser APIs; no foreign-app reload guarantee |
| Distribution | Offline coherent package set; M10 verified bundle lifecycle | M12 verified online client; hosting/publication independently authorized |
| Privacy | Local logs; explicit diagnostic export | No background telemetry by default |
| Typed actions | Versioned operations shared by UI, shortcuts and automation | Optional model proposals through identical authorization |
| AI | Optional extension; absent from required boot/desktop path | Small-model CPU experiment after M5; no assumed NPU/LLM stack |
| Licensing | Inventory and attribution for every dependency/font | Original project material is all rights reserved; upstream terms remain separate; distribution needs rights review |

The [identity and authorization plan](../02-architecture/identity-and-authorization.md)
separates M3's synthetic multi-principal enforcement tests from real account/login
support. The 2026-10-21 [roadmap amendment](reconciled-plan.md#installer-and-accounts-amendment--2026-10-21)
requires M7 accounts and installation readiness for M8; an earlier live-only preview
does not satisfy that release gate.
The later desktop/platform expansion preserves M8 and adds M9–M15 delivery and
M16–M21 separately gated research. B56/M14 requires desktop/network/audio/backup
acceptance; B58/M15 requires that VM release and one named PC. Neither release
requires browser/EXE/GPU/SMP/Wasm/AI experiments. No new feature is implemented by this plan.

## Performance targets and accounting

Provisional minimum test profile: 2 GiB guest RAM, one vCPU, 1280×720 or 1920×1080,
software composition, declared host CPU and accelerator. First feasibility at
1280×720 is separate from later 1080p quality targets. Enhanced profile: 4 GiB,
two vCPUs after SMP exists, 1080p, proven GPU path. Host memory is reported separately.

- At 60 Hz, frame period is 16.67 ms; at 120 Hz it is 8.33 ms. Start with 60 Hz.
- Target at 1080p/60: p95 render+composition CPU time below 8 ms and p99 below
  16 ms for the named ordinary desktop workload; adjust only with recorded evidence.
- Target p95 guest input-event-to-present submission below 25 ms on the reference
  profile. This excludes unknown display scanout/host timing and is not end-to-end latency.
- No perpetual rendering on a static desktop. Exclude the clock/caret only by
  specifying their timers; do not hide them from measurements.
- Provisional **cold-idle** desktop budget: 256 MiB resident/committed guest memory,
  including kernel, services, app shells, framebuffers and bounded reclaimable caches.
  Account separately for genuinely free RAM; caches cannot disappear from total usage.
- The 2 GiB VM configuration is capacity, not a 2 GiB idle allowance. If inference is
  enabled, report cold idle (model absent), warm idle (model resident), and active
  desktop+inference with weights, scratch/attention caches and shared/device bytes.
- Record replacement peaks including old/new instances, checkpoint and queued work;
  reject staging when safe headroom is unavailable. Targets remain unmeasured.
- At least 100 consecutive compatible reloads without unexplained monotonic
  resource growth; compare after quiescence with a defined tolerance.
- Target resource-update latency below 500 ms from completed host build to visible
  guest commit. Compilation, transport, quiescence and activation are separate metrics.

GPU acceleration and native hardware results cannot be inferred from emulator
measurements. TCG and WHPX results are different test profiles. See
[performance accounting](../02-architecture/observability-and-performance.md).

## Nonnegotiable correctness properties

At most one active writer for an owned state epoch. No old code freed while any
old execution can reach it. No new authority granted solely because a package is
newer. Failed pre-activation changes preserve the old visible generation. A
post-activation external effect is not silently undone by rolling back code.
Persistent success requires durable activation metadata; ephemeral sessions say
they will lose changes at reboot. Kernel changes that need reboot say so directly.

## Traceability

The [validation matrix](../05-delivery/validation.md) maps every requirement to
test families. The [backlog](../05-delivery/backlog.md) maps implementation work to
milestones and requirement IDs. An implementation agent must maintain both when
changing behavior or adding support claims.

P1-03's [checked metadata acceptance](../../evidence/phase-1/P1-03/2026-10-18/README.md)
adds boot-specific evidence for R01/R03/R17/R18 and preparatory R14 version/boundary
discipline. The internal BootInfo is not a process or persisted ABI; this does not
complete update diagnostics, isolation, release reproducibility or efficiency gates.

P1-04's [accepted fatal diagnostics](../../evidence/phase-1/P1-04/2026-10-19/README.md)
extend R01/R03/R17 evidence with actual kernel exceptions and bounded panic/emergency
output, plus R18 source/evidence review. This is early R10 groundwork; guard and
double-fault integration were later gates at that review. User containment and
broader M1 remain later gates.

P1-05's [accepted memory ownership](../../evidence/phase-1/P1-05/2026-10-19/README.md)
extends R01/R03/R17/R18 and early R10 evidence with owned CR3 mappings, frame
reservation/zeroing/reuse, enforced WP/NX, guards and actual double-fault/IST entry.
It does not establish user isolation, a general heap, measured desktop efficiency
or complete M1 readiness.

P1-06's [accepted timer baseline](../../evidence/phase-1/P1-06/2026-10-20/README.md) extends R01/R03/R17/R18 evidence with validated ACPI/device mappings, returning interrupts, calibrated delivered-tick uptime and idle continuation. Fresh-epoch and failure cases are retained. R09/R10 remain early groundwork; no measured performance, isolation, scheduler, wall-time or live-update guarantee follows.

P1-07 adds an ordered final `m1.ready` checkpoint and aggregate acceptance harness
for R01/R02/R03/R17/R18. Its [complete local review passes](../../evidence/phase-1/P1-07/2026-10-20-acceptance/README.md):
33 rows and 42 launches close the declared experimental M1 baseline. Two clean
kernels match; both actual ISOs boot and differ only in the declared session field.
The earlier GPT and host-execution failures remain preserved. This does not complete
R16 release reproducibility or any broader product requirement.

P2-01's [local prerequisite acceptance](../../evidence/phase-2/P2-01/2026-10-20/README.md)
adds R03/R18 host infrastructure and early R09/R16 profile/provenance setup.
Fresh offline compile/link/signing/doctor checks and 23 headless lifecycle cases
do not establish rendered UI, reload continuity or guest performance.

P2-02's [accepted shared corpus](../../evidence/phase-2/P2-02/2026-10-21/README.md)
prepares R04/R08/R15 with bounded surface/input/token/layout drafts and separate
visual/semantic/state expectations; R18 and early R14 receive schema, negative
fixture and deterministic hash evidence. These are contract checks, not UI,
accessibility, action-execution, IPC, continuity or performance acceptance.

P2-03's [accepted hosted document/actions](../../evidence/phase-2/P2-03/2026-10-21/README.md)
adds R19's M2 interface/model result, R04/R15/R18 logical-state preparation and early
R07/R14 bounded schema discipline: shared entry execution, modeled rights/context,
grapheme-aligned editing, history and explicit snapshots. It does not establish
rendered accessibility, guest continuity, secure IPC or provider replacement.

P2-04's [accepted bounded text experiment](../../evidence/phase-2/P2-04/2026-10-21/README.md)
adds R08/R15/R18 preparation and R07 composition groundwork: six pinned fonts,
independent shaping references, official segmentation rows, shared text/scale
fixtures and logical/visual caret, selection, wrap and quota checks. Text-only
rasters are retained separately from semantic expectations. There is no candidate
UI, native IME, full Unicode, guest memory/performance or actual reload claim.

P2-05's [accepted bounded custom candidate](../../evidence/phase-2/P2-05/2026-10-21/README.md)
extends hosted R08/R15/R18 and R04 preparation with common software surfaces,
rendered reference scenes, shared action workflows/semantic observations, bounded
virtualization/caches and native Windows presentation. R09 receives instrumentation
only. Immediate standard transitions, full redraw/copies and text/native-accessibility
limits remain explicit comparison findings. Guest execution, actual continuity,
measured efficiency and toolkit selection remain later gates.

P2-06's [bounded Slint evidence](../../evidence/phase-2/P2-06/2026-10-21/README.md)
extends hosted R08/R15/R18 comparison inputs and R04 preparation with actual Slint
AOT/software surfaces, the same logical workflows and an explicit P2-04 text Image
adapter. Native text setup, rich effects, rounded ancestor clips, launcher opacity,
fractional blending and native semantics remain failing/unavailable cases; none
is waived. R09 receives instrumentation only. Guest links/execution, actual reload,
measured efficiency and distribution/toolkit selection remain later gates.
