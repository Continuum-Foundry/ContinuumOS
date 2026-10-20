# P1-06 acceptance review — 2026-10-20

**Done for P1-06 on the pinned single-core Q35/TCG/OVMF profile. M1 is incomplete.**
Codex performed this local self-review against all six story criteria. This is
not independent approval, physical-platform certification, scheduling, precision
measurement, wall time, user isolation, a desktop or a live-update demonstration.
All architectural ADRs remain Proposed.

Accepted selection: `acceptance-p1-06-final-1792470634864`. Its [acceptance record](acceptance/acceptance-results.json)
contains 31 successful commands, including the full 29-command matrix and
source-consistency repeats. The [artifact review](review.json) checks 36 selected
distinct launches, current build-input inventories, exact ELF/ISO hashes, ordered
identity/sequence checkpoints, original exception entry and returning timer entry,
owned memory/protection regressions and timer behavior. Procedures are retained
under [acceptance](acceptance/).

Source base: `0311184` (accepted P1-05), dirty, on
`codex/p1-06-interrupts-timekeeping`. The [source snapshot](source/) retains exact
final build-input bytes. The unrelated pre-existing `.gitignore` edit was preserved
and is not a build input. No commit, push, PR or remote processing is claimed here.

| Validation | Result |
| --- | --- |
| Host checks | 13 xtask + 14 kernel-library tests; formatting passes |
| Phase 0 | Independent probe links, clean comparisons, provenance, source path with spaces, doctor overrides and cleanup pass |
| Dependencies/prerequisites | Five library probes retain their previous link limits; pinned M1 doctor and input-rejection checks pass in xorriso's documented execution context |
| Complete boots | 14 selected runs reach memory, ACPI/calibration/timer progress and boot completion; ordinary observation is host-stopped after its checkpoint |
| Original fatal regressions | Eight real exception/panic/protection cases, including actual guard-delivery #DF on IST1; expected exit 37, no completion |
| Post-interrupt fatal | Real #UD after at least 100 delivered interrupts, exact ELF RIP, saved IF set, owned frame and exit 37 |
| Rejections | Eight original metadata cases and three new platform cases; exact reasons and exit 35, no false completion |
| Negative ISO cases | Two deliberately broken boots fail before kernel entry; they never count as successful boots |
| Lifecycle | All 36 selected sessions distinct; every VM reaped; no timeout passes as a successful boot |

## Story criteria review

| Criterion | Evidence and conclusion |
| --- | --- |
| Normal readiness and 100 events | [Normal run](run/result.json), [timer progress](timer-progress/result.json) and raw serial records reach `m1.timer_ready`, 100 samples and `m1.timer_progress`. One EOI per timer interrupt permits repeated hardware delivery. No IRQ-context logging or allocation occurs |
| 100 monotonic uptime reads | [Clock reads](clock-read/result.json) include two launches with 100 strictly increasing samples each. Review independently checks count × calibrated resolution against each uptime value. Calibration inputs and effective resolution are recorded; this is delivered-interrupt time, not elapsed/wall-time accuracy |
| Stable identity and fresh epoch | Every selected record has exact build/session and sequential indices. Clock-read launches share ELF/build but have distinct host sessions/configuration ISOs and each initializes ticks/uptime to zero. QEMU `-no-reboot`, single-entry checks and process records exclude a reset as success |
| Masking suppresses delivery | [Masked timer](timer-masked/result.json) polls live HPET for at least the requested 60 ms reference interval while IF is enabled and LVT masked; before/after delivery counts match, and an actual restored event advances the count |
| Idle continuation and diagnosable exceptions | [Idle wakeup](idle-wakeup/result.json) performs 100 real STI/HLT continuations with distinct sentinels in all GPRs and DF, checked after IRETQ. [Post-delivery #UD](timer-exception/result.json) preserves fatal diagnostics. All older exception/guard/IST cases remain passing |
| Malformed/unsupported/calibration rejection | Host fixtures cover RSDP/SDT length/checksum/ranges, unsupported topology/GAS, clock overflow and synthetic counter wraps. Guest [corrupt RSDP](acpi-corrupt/result.json), [unsupported HPET discovery](timer-unsupported/result.json) and [zero calibration](timer-calibration/result.json) require exact rejection before timer readiness; there is no guessed fallback |

The [runbook](../../../../docs/05-delivery/phase-1/interrupts-timekeeping-runbook.md)
documents explicit ACPI copy/MMIO mappings, WB/UC policy, controller/vector ownership,
register/stack/unsafe assumptions, overflow behavior and reviewed public CPU/ACPI/HPET
contracts. Every build independently audits LLVM output for 15 GPR saves/restores,
CLD, SysV alignment, exact vectors and IRETQ, along with existing image bounds,
symbol closure and FP/SIMD rejection. [ADR-0017](../../../../docs/04-decisions/0017-bootstrap-timekeeping.md)
records this limited baseline as Proposed.

| Build flavor | Exact build ID |
| --- | --- |
| boot-inject | `3e2f55ac8ab65d736961e95b32c88763d3fb17f1e0f8a01558043bc9471910ec` |
| exception-test | `79c361caeda17b2089d0d9bb8f50a1da15964b889c0ccdd01fc30932e02efbf9` |
| fault-before-entry | `6b72437200942cbb8a5f32ee6ced7b99a0544841ee46d548cbd45f6b23b803be` |
| iso-smoke | `6703b00ce061ad3b061259811f0e708284191d666dcb17e5f406d54f2f9fc4ec` |
| memory-test | `6bf3d10c6227a2c683dcc8b8645c8d9f708447d08ca116da73c4b8b5bfab1207` |
| normal | `8e3a87240bd3251384becb50f0fa129d63f30fa69c5d61d2f7ea10a1bb2f68f6` |
| timer-test | `38efb200d062a4bbf94c08e811b1e57aba28b368d89d21fe289cf6c7f87be899` |

Per-run manifests retain full kernel/ISO/firmware/configuration digests, arguments,
host exits and logs. Binaries and ISOs remain in ignored `build/m1/`.

## Failed and superseded development

The first timer ISO reached owned memory then rejected HPET GAS width zero with
`field=hpet reason=unsupported`, before enabling interrupts. Its raw record is
retained under [development experiments](development-experiments/). QEMU leaves GAS
width unspecified; the corrected route independently checks the hardware's 64-bit
main-counter capability and period. Three subsequent passing development cases
use earlier source inventories and are also excluded from final selected evidence.

The original sandboxed doctor encountered the documented MSYS2 named-object
restriction. One formatting attempt hit Windows error 1224, and an initial binary
audit implementation had a Rust error-type mismatch. Corrected retries passed;
these are explicitly transcribed in [observations](development-experiments/observations.txt),
not presented as raw logs. No Windows feature, security setting or global package changed.

Final source review tightened the full APIC-base mask, explicit PAT capability and
exact six-LVT support while early matrix cases were running. Every stale selected
case was repeated against the final frozen inventory. The original matrix and
superseded runs are retained under [superseded](superseded/); review rejects any
stale selected source or unexplained within-flavor ELF/build difference.

## Limits and handoff

The locked profile remains `pc-q35-11.1`, `qemu64-v1,+apic`, TCG, one vCPU, 2 GiB,
pinned OVMF/Limine, no network/shares/user disk/snapshot/direct-kernel boot. ACPI is
bounded to revision-2 RSDP/XSDT, tables up to 64 KiB, 128 root entries, one IOAPIC,
xAPIC and 64-bit HPET. Device pages are explicit supervisor UC/NX; no loader/ACPI
memory is reclaimed. Existing six guards and WP/NX remain in force.

Uptime is initialized once and counts delivered timer events at its calibrated
resolution (approximately 10 ms). It can lag elapsed time under masking/coalescing;
there is no TSC interpolation, UTC, drift/deadline guarantee or performance result.
Negative fixtures alter owned/copied values; they do not claim removal of real
hardware. Spurious handling is code/artifact reviewed; spontaneous spurious delivery
is not claimed. No heap, scheduler, process/FP context switching, SMP, suspend,
rendering, persistence or live update exists. The 256 MiB desktop target is unchanged
and unmeasured. Next: P1-07's final M1 acceptance; B06 remains partial for M3.

`evidence-index.json` distinguishes raw and retained hashes; serial/source bytes
are verbatim, while host text is path-sanitized. `SHA256SUMS.json` freezes the bundle.
