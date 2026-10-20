# P1-05 acceptance review — 2026-10-19

**Done for P1-05 on the declared Q35/TCG/OVMF profile. M1 is incomplete.**
Codex performed this local self-review against all six story criteria. This is
not independent approval, user isolation, arbitrary-fault recovery, a release or
a performance result. All architectural ADRs remain Proposed.

Accepted selection: `acceptance-p1-05-final-1792455389828`. All 24 commands in the
[acceptance record](acceptance/acceptance-results.json) returned zero: the 21-command
full matrix plus three early-case repeats after a README input change. The
[artifact review](review.json) checks 27 selected distinct launches, exact ISO/ELF
digests, current source hashes, correlated checkpoints, entry disassembly, owned
root backing and actual double-fault/IST context. The review and retention
procedures are retained in [acceptance](acceptance/).

Source base: `fdf931cca8e0d2165be2b8b2c910da5c960f976f`, dirty, on
`codex/p1-05-memory-ownership`. Every build records a complete input-hash inventory.
The [source snapshot](source/) retains the final input bytes checked against the
working tree during review. The pre-existing `.gitignore` edit is unrelated and
was preserved. Everything remains local; no commit, push or PR is claimed here.

| Validation | Result |
| --- | --- |
| Host checks | 13 xtask + 10 kernel-library tests; formatting passes |
| Phase 0 | Probe links, two clean outputs, provenance, source path with spaces, doctor overrides and cleanup pass |
| Dependency probes | Five libraries compile; allocation-dependent links retain their expected blockers |
| Prerequisites | Locked tool/firmware/loader checks and rejection tests pass in xorriso's documented execution context |
| Complete boots | Nine selected boots reach owned-memory readiness then completion; includes module, serial-only, ownership and explicit exhaustion fixtures |
| Metadata rejections | Eight exact field/reason cases, exit 35, no exception or memory readiness |
| Existing fatal cases | Real #UD, absent-page #PF, Rust panic and held-logger #UD still diagnose correctly after switching mappings |
| New protection cases | Real guard #PF, RO-text store #PF, NX instruction-fetch #PF and actual delivery-failure #DF; exact expected fatal exit 37 |
| Negative ISO cases | Pre-entry halt and missing configured executable fail as expected; neither counts as a boot |
| Lifecycle | All 27 selected sessions are distinct and all processes are reaped; five superseded launches are separately retained |

## Story criteria review

| Criterion | Evidence and conclusion |
| --- | --- |
| Normal readiness on owned tables/stacks | [Normal observation](run/result.json) and [ownership](memory-ownership/result.json) reach `m1.memory_ready`; CR3 differs from loader CR3 and lies inside the reserved image-owned table arena |
| Exclusive, aligned, zeroed frames | Eight distinct physical pages are independently checked against guest usable ranges and reservations. Guest volatile observations verify initial zeroing, dirty/release/reuse zeroing and no leaked live test allocation |
| Explicit failure outcomes | [Kernel tests](acceptance/kernel-host.log) cover unsorted/overlapping maps, inward edges, holes, wrapping ranges, all-reserved memory, descriptor/address/table capacity and finite exhaustion. Guest checks reject invalid/double frees and mapping conflicts; [exhaustion](memory-exhaustion/result.json) uses an explicit eight-live-frame quota |
| Actual permission enforcement | Guest scans every image/direct-map leaf and all six guards after switching, verifies CR0.WP/EFER.NXE and checks absence of kernel aliases. [Text store](write-protect-fault/result.json) reports #PF error 3; [NX fetch](execute-disable-fault/result.json) reports error 17. A live scratch remap with INVLPG observes the new backing |
| Guard and emergency diagnostics | [Guard read](guard-fault/result.json) faults at the runtime low guard. [Actual double fault](emergency-stack/result.json) reports vector 8/error 0 with its frame inside the distinct IST1 stack, saved RSP at the controlled runtime bottom and expected exit 37. Saved #DF RIP is undefined by the architecture and is recorded without a meaning claim |
| Previous checkpoints survive | [Metadata matrix](boot-metadata/result.json), [rejections](boot-reject/result.json), [ISO smoke](iso-smoke/result.json) and four prior fatal cases pass with strict version-4 identity/sequence checks. Reset, missing outcomes, wrong context/exit and timeout cannot count as success |

All 32 exception wrappers/common entry instructions are reviewed in retained LLVM
disassembly, along with CR0/CR3/CR4, EFER and INVLPG operations. Every build independently
rejects FP/SIMD, undefined symbols and incomplete image extents. The
[memory runbook](../../../../docs/05-delivery/phase-1/memory-ownership-runbook.md)
documents the unsafe assumptions, exact capacities, cache/mapping policy and
CPU source review. Hardware A/D entry updates use `UnsafeCell` and volatile IO.

| Build flavor | Exact build ID |
| --- | --- |
| Normal | `ff868a8da4de569087d0ab88a65e3984b4efc8e6d345e1f214c45a55005bfb99` |
| Emulator smoke | `438a295560e825e91d2ef5a021bbad745b6eb227689a75253fdb8e2caa3cf52d` |
| Memory cases | `f85c04a0987bb03c2d0c3c1f059f07792138991022d7627cedbef59d0b2f152f` |
| Existing exception cases | `746046ab64039155c65d7ef0537ea20e1c8418a4d31af118b5d09f4717522e3b` |

Full ELF, ISO, firmware, configuration and input digests, including the rejection
and pre-entry flavors, are in the per-run manifests and review.

## Retained failures and superseded evidence

- The initial normal build compiled but failed its symbol audit because link
  garbage collection removed the otherwise unused NX probe data page. Ordinary
  mapping inspection now also checks that page; the [failed audit](failed-experiments/build-1792454033627736200-28776/result.json)
  and captured logs are retained.
- Windows Application Control blocked the rebuilt host test executable before
  execution on the first acceptance attempt (error 4551). An unchanged retry ran
  successfully. The [original failure](failed-experiments/application-control/host-tests.log)
  remains retained; no host security setting changed. Earlier source-formatting
  file-mapping errors are separately described in the explicitly transcribed
  [development observations](failed-experiments/development-observations.txt).
- Updating `kernel/README.md` during documentation work changed a hashed build
  input. The affected normal/smoke/negative cases were repeated against the final
  README. The original full matrix and five original launches are retained under
  [superseded](superseded/), including the previous README bytes. They do not count
  toward the final 27 selected launches. The final review checks matching current
  inventories and identical IDs/ELF bytes within each flavor.

## Limits and handoff

The profile remains `pc-q35-11.1`, `qemu64-v1,+apic`, TCG, one vCPU, 2 GiB capacity,
pinned OVMF and Limine, with no network/shares/user disk/snapshot/direct-kernel boot.
Physical frame metadata covers addresses below 4 GiB and at most 512 descriptors;
the static table arena has 2304 pages (9 MiB plus metadata/padding). This is a
bounded bootstrap policy, not a general allocator or a measured desktop footprint.

All loader memory stays physically reserved; zero loader bytes are reclaimed.
Only eligible ordinary RAM is directly mapped. Kernel physical aliases, loader,
modules, framebuffer and ACPI backing are absent. Their copied descriptors remain
available. P1-06 must explicitly map validated ACPI/device ranges before access.
The runtime, double-fault and NMI stacks each have two absent guard pages.

IF remains clear. There is no clock, external IRQ baseline, general heap, process
isolation, DMA safety, SMP, rendering or live update. Controlled fatal diagnostics
do not establish arbitrary-corruption recovery, NMI timing behavior or physical
shutdown. The 256 MiB desktop cold-idle target is unchanged and unmeasured.

The [evidence index](evidence-index.json) distinguishes raw/retained hashes and
verbatim source/serial bytes. `SHA256SUMS.json` freezes the completed bundle.
Binaries, ISOs, firmware-variable copies and Cargo caches stay under ignored
`build/m1/`. Historical evidence is unchanged. Next: P1-06, then P1-07's final M1 gate.
