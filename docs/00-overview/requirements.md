# Requirements and feature matrix

Status: user intent plus proposed engineering acceptance criteria. Metrics are
targets, not measurements. M0–M8 are defined in [milestones](../05-delivery/milestones.md).

## Core requirements

| ID | Requirement | First acceptance evidence | Gate |
| --- | --- | --- | --- |
| R01 | Original kernel and OS architecture | Build manifest identifies original kernel; no substituted existing OS | M1 |
| R02 | Bootable standalone ISO | Cold boot through declared UEFI firmware, no host-shared root files | M1, M8 |
| R03 | Modern first-party language | Rust source and documented tiny assembly boundary; no first-party C dependency | M0 onward |
| R04 | Live UI resources without OS reboot | Theme/layout updates preserve document and window; boot ID/uptime unchanged | M2, M4 |
| R05 | Live app behavior without OS reboot | New logic used after state migration; old process safely retired | M5 |
| R06 | Selected system services replaceable | Update under load obeys request and resource invariants | M6 |
| R07 | State continuity has explicit limits | Compatibility failures rejected before activation; unsaved text preserved in supported paths | M5 |
| R08 | Exceptional coherent desktop | Design tokens, reference scenes, keyboard behavior, and visual review | M2–M8 |
| R09 | Efficiency is measurable | Recorded CPU/frame/memory traces for fixed workloads and hardware profile | M2 onward |
| R10 | Isolation and bounded resources | User process cannot write kernel/other app memory; quotas and malformed IPC tests | M3 |
| R11 | Persistent user files | Correct flush path and recovery tests survive injected interruption | M6 |
| R12 | Recoverable updates | Fault injection at every transition; last good generation boots | M5–M8 |
| R13 | Secure development boundary | Explicit dev mode, authenticated session, bounded messages, no guest-supplied host commands | M4 |
| R14 | Versioned contracts | Wire compatibility tests, schema migration fixtures, independently built clients | M3–M6 |
| R15 | Accessibility and international text | Semantic tree, keyboard navigation, scale, graphemes, shaping and bidi tests | M2–M8 |
| R16 | Reproducible delivery | Pinned inputs, provenance, checksums, two clean builds with explained differences | M8 |
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
| Devices | Serial, timers/APIC, PCI, VirtIO input/block/console/display | NVMe, USB, Wi-Fi, vendor GPUs are separate driver programs |
| Graphics | One display, software, logical scaling | Acceleration, mixed-DPI multi-display, HDR after correct color pipeline |
| Input | Keyboard, pointer, focus, keymap service | IME and touch staged; secure input path before login |
| Desktop | Launcher, windows, tiling/snap, settings, notifications | Advanced workspaces and scripting after core usability |
| Apps | Editor, file browser, console, inspector | Browser, media suite, compatibility subsystems deferred |
| Text | UTF-8 storage, grapheme editing, shaping hooks | Fonts/script coverage grows against explicit test corpus |
| Accessibility | Semantic model, keyboard, visible focus, reduced motion | Native screen reader, braille and speech drivers staged |
| Files | Read-only system packages, writable data image | Full desktop filesystem after crash-consistency gate |
| Networking | Dev channel first; isolated network service | General TCP/IP, DNS, TLS, IPv6 compatibility stages |
| Audio | Interface reserved; optional single PCM device | Mixer, resampling, microphone permissions later |
| User identity | Single local development user | Authentication, multiple users, encrypted home later |
| Power | Idle halt, timers, orderly shutdown | Physical suspend/resume and battery policy deferred |
| Updates | Resource/app/service classes | Driver resets class-specific; kernel/boot require reboot initially |
| Installation | Live ISO and named VM data disk | Installer only with explicit target selection and recovery design |
| Recovery | Rescue ISO, last good generation, serial logs | Signed recovery and key recovery after release security design |
| Compatibility | Native OS SDK and defined Wasm imports | No implicit POSIX, Win32, Linux ABI, or browser APIs |
| Distribution | Offline coherent package set | Online update repository after trust/time/entropy gates |
| Privacy | Local logs; explicit diagnostic export | No background telemetry by default |
| Typed actions | Versioned operations shared by UI, shortcuts and automation | Optional model proposals through identical authorization |
| AI | Optional extension; absent from required boot/desktop path | Small-model CPU experiment after M5; no assumed NPU/LLM stack |
| Licensing | Inventory and attribution for every dependency/font | Project license remains an open product decision |

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
