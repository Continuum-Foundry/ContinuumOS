# P3-03 — Static-image staging and trusted launch bindings pass locally

**P3-03 is Done under local Codex self-review.** This accepts staged loading,
not independent approval, ring-3 execution, public child launch or the M3 milestone.
Use the [loader runbook](../../../../docs/05-delivery/phase-3/static-loader-runbook.md)
and [criterion review](review.json). Source base: `9553ae3971f60f45501f3f1fd5a69ca8dd4c5c78` plus exact
[dirty input hashes](source-provenance.json). Full raw logs, binaries, ISOs,
source patches/copies and disassembly remain local under ignored `build/`/`work/`.

## Exact runs and artifacts

- Final acceptance: `build/m3/P3-03/acceptance-1792968925381`; all five ordered
  commands pass with no retry or deadline extension.
- Fresh prerequisites: all ten offline/signing/compiler/Phase 0/tool checks pass.
- Complete M1: all 33 rows and 42 launches pass: 20 ready, nine expected fatal,
  eleven expected rejection and two deliberately broken before kernel entry.
  Five consecutive normal ISO boots and the second clean ISO boot pass.
  Clean kernels match; only the permitted 64-byte host-session field differs
  between the two ISOs. This is not release reproducibility acceptance.
- Owned-memory regression: 1000 poisoned create/map/unmap/destroy cycles settle
  all inventories exactly; independent memory ELF/table/alias audit passes.
- Final loader: three independently linked 13216-byte Rust images, each with RX,
  R and RW/BSS loads, stage simultaneously into distinct hardware address spaces.
  The independent reader reconstructs six table pages and all 24 private user
  pages per process (72 total), including every file/BSS/padding byte, zeroed stack,
  synthetic return address, exact startup bytes and R/RX/RW/NX permissions.
  Initial commitment is 98304 bytes per fixture, including stack/startup/return.

Loader build ID: `6f8f5db29de70863575d43d419ccf5033b7959ed9850b8fdc18da2c9b425675f`.
Kernel SHA-256: `501af730a343f42f76fe248e980e62cf5454540febe4e15ef8f244014d077c89`.
Launched loader ISO SHA-256: `a44d62b6ffc42d40012fd5145557fc3b006eacee9b6b178595e9ee14cce4d761`.
Raw loader serial SHA-256: `00e6b6df14a1d8192b430d82dc11d5fd47691cae03431ea56be511489a39fc74`.
The [launch contexts](launches/loader-context.json) identify actual QEMU arguments,
CPU/profile, firmware, payload hashes, build/session, fixed 30000-ms deadline and
confirmed cleanup. The offline guest uses one vCPU and 2 GiB capacity; capacity is
not idle usage. Exact fixture digests/target/compiler/audit-tool pins are in
[user-images.json](user-images.json); actual separate build commands and selected
entry disassembly are in `user-codegen/`.

Normal M1 build ID: `380ab76b3f0caf32e950694e5e53823260a6d4b80fb5ce533aac30ef0667fd5e`.
Normal ISO hashes: `bab092a79219020ebce3237ea67ff6fb6df4cfb96d8f797f4fc59d2f655f3966` and
`7557efd80db807a946be595f6e804fbab90a1d621f399adc50b44477b4d02a24`.
Owned-memory regression build ID: `fbfa0da99fbf55df9d2cabb9fe3bd695a1c7b457dede84724e15c05c2fe51476`.

## Criterion review

| Criterion | Conclusion / retained evidence |
| --- | --- |
| Real ELF contents, entry, RX/R/RW, BSS and guards | Pass: independent source-ELF to actual mapped-page SHA-256 comparison; complete raw user-tree reconstruction checks exact leaf set, ancestors/U-S, no guards/extra mappings, distinct nonreserved ordinary-RAM backing and RO/NX supervisor aliases. |
| Malformed images and transaction rollback | Pass: host header/range/permission/entry/overlap/alias/TLS/relocation/symbol negatives admit no context/frame state; every allocation stage and failing intermediate-table capacity roll back. Guest rejects 19 malformed/truncated cases, every insufficient 0–23-page quota and wrong digest, with exact inventory restoration. |
| Trusted bindings, payload forgery, reuse | Pass: synthetic A/B/service records remain distinct; 15 guest identity/session/class/grant/ceiling edits deny, old process generation cannot resolve/delegate/destroy a reused slot, and host startup-handle/features forgery leaves trusted context unchanged. Grant masks are internal permitted ceilings, not public rights/handles. |
| Explicit startup, lifetime and child ceiling contract | Pass: explicit 64-byte LE encoding, zero bootstrap/features/clock, reserved module lifetime, staged-copy ownership and future supervisor authorization hook are documented in runtime revision 0 and ADR-0019. |
| ISO and M1 regressions | Pass: payload extraction hashes match actual modules/embedded expected digests; all 42 baseline launches plus memory/loader launches are reaped under original bounds; all 28 kernel host tests pass. |

The independent review rejects 14 altered/missing-evidence cases, including content,
startup, W^X/alias, trusted principal, generation reuse, settlement and missing raw
table/required records. This is independent parsing inside local agent self-review,
not approval by another reviewer. [Development findings](development-findings.json)
preserve the failed host timeout, table-test expectation, fixture compile and
initial GNU-OSABI rejection; their original complete runs remain local.

## Limits and next story

No user instruction executes. The synthetic RX return page is deliberately UD2
until **P3-04 installs process_exit and IRETQ**. No syscall IDs, current-thread
binding, user-fault containment, FP/SIMD execution, scheduler or guest UI is accepted.
The bootstrap launcher alone chooses bindings. P3-09/P3-10 must authenticate public
child-launch callers/supervisor delegation and enforce actual grants/aggregate
quotas. Startup identifiers/bytes carry no authority; sessions/principals are
synthetic fixtures, not login/accounts. The coherent trusted ISO/loader supplies
image trust; no general signature, Secure Boot or untrusted package-admission
security guarantee follows. The 256 MiB cold-idle desktop target remains open.
