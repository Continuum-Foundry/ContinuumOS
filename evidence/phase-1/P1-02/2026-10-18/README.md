# P1-02 — minimal original-kernel ISO acceptance

Recorded 2026-10-18, America/New_York. **Done for minimal x86-64 UEFI ISO entry**.
Reviewed by Codex against all six P1-02 acceptance criteria; no independent human
or external review is implied. ADRs remain Proposed and full M1 remains incomplete.

Final harness: `build/m1/acceptance-1792300105787`, run with the approved execution
context needed by the pinned MSYS2 xorriso runtime. `acceptance/acceptance-results.json`
records all six command outcomes as passed. Source base is `d18bb0a`; builds are
from the dirty `codex/p1-02-minimal-iso-boot` working tree, not a committed release.
Each build's input hashes are verified against current sources when retaining
this snapshot. `artifact-locations.json` identifies the retained ignored binaries.

## Reviewed acceptance criteria

| Criterion | Retained evidence and result |
| --- | --- |
| Original independent Rust kernel and entry boundary | `run/build-inputs.json`, `run/build-manifest.json` and LLVM logs: stable Rust 1.99.1, ELF64 x86-64 ET_EXEC, entry 0xffffffff80000000, static RX/R/RW loads, no unresolved symbols/relocations/dynamic linkage; retained base-3 request region and no FP/SIMD instructions. First-party assembly is limited to documented cli/hlt/in/out. |
| Real UEFI optical ISO and intended contents | `run/iso-manifest.json`, optical catalog and extraction logs: bootable UEFI El Torito entry; extracted kernel/config/loader/notice hashes match staging. The ISO is independently packaged, not a renamed disk image. |
| Fresh boot of the exact ISO | `iso-smoke/cold-boot-a/` and `cold-boot-b/`: new QEMU processes, fresh vars, one matching kernel entry and exit 33 in each; ISO digest unchanged after both runs. Raw firmware logs identify the UEFI QEMU DVD-ROM path. |
| Correlated build identity; false successes rejected | Embedded identity matches both serial records and manifests. `host-tests.log` contains the validator cases rejecting stale/mismatched/duplicate/incomplete markers, marker-only and exit-only results, panic, failure exit 35, timeout and output overflow. Host fixtures are not positive guest evidence. |
| Offline optical-only baseline | Both launch manifests and command arrays use pinned Q35/TCG, qemu64-v1,+apic, one vCPU, 2048 MiB, OVMF, no reboot, no network/monitor/shares, no direct kernel/snapshot/data disk. Only code/vars pflash and the exact read-only CD-ROM are attached. The test device is explicitly enabled for smoke/negative runs only. |
| Failure and timeout cleanup | Normal `run` records host stop/reaping after the checkpoint, not guest shutdown. Smoke processes exit 33 and are reaped. `iso-negative/no-entry-run/` times out without a marker; `bad-config-run/` records Limine's actual missing-executable panic. Both negative runs fail as boots and are reaped; only their expected-failure assertions pass. Host lifecycle tests cover early exit, wrong exit, timeout and log flooding. |

## Exact final artifacts

| Artifact | SHA-256 |
| --- | --- |
| Normal kernel ELF | `fafd790ed87eedc032bd8527f8e53c864f804cae08ec086c3a38d0d74e0983ce` |
| Normal ISO | `d24803ba5084e96006a7b9eae88c41db6edd1a19eec1b7890a0b11e4fc22ff6a` |
| Smoke kernel ELF | `e2356a24dba43705804ea8370efb2f9b0b83258ea5bf66b1a70926c138d9928f` |
| Smoke ISO, booted twice | `2ad7b1d0747b13950b064df72a22b712b9f383848b28033f25126584963a54ae` |

Normal build identity:
`a408c0d4f3bcac075c49754d0f4ca9aa6178b0e7ef2b3dbbec07495a7c08cd2c`.
Smoke build identity:
`d2fbf861323612c8915a96aae9f6629ad6eb4526d2e1a1957ea5d6e39c2d9d38`.
The missing-path test packages the exact proven smoke ELF with a deliberately bad
configuration. Its separate ISO digest and fault identity are recorded. The
no-entry kernel is a distinct build flavor. None of these negative transcripts
substitutes for the two genuine positive guest boots.

Distinct host session IDs and identical fresh-vars source/copy hashes were checked
for the two smoke launches. They are host correlation IDs, not guest identity or
security credentials. `-no-reboot`, exact checkpoints and decoded exits provide
the bounded no-reset acceptance checks; guest uptime is not implemented.

## Regression, prerequisite and failure records

- Baseline M0 acceptance and dependency rechecks ran before extension; copies are
  in `baseline-m0/` and `baseline-dependencies/`. All five dependency libraries
  compile; only architecture links. The other four retain their missing-allocator
  limitation and remain untested guest integrations.
- Final `m0/` and `m0-verification/` contain nine passing host tests, two identical
  clean probe builds, independent LLVM checks, doctor overrides and the source
  path containing spaces. `prerequisites/` retains valid and rejected pinned
  input cases. Host tests also exercise missing kernel/loader/firmware inputs and
  altered digests. No real tool tree was corrupted for those tests.
- Final packaging exercises xorriso staging/extraction paths with spaces and QEMU
  firmware-copy filenames with spaces. `cargo fmt --all -- --check` and
  `git diff --check` pass; local Markdown links are checked separately at delivery.
- `baseline-doctor-sandbox.json` retains the MSYS2 named-object access denial;
  the same pinned files pass in the approved execution context. No Windows
  feature, WSL installation, physical disk or global package was changed.
- `failed-experiments/` preserves the first native-path packaging failure and the
  first firmware-console-line/strict-marker rejection, with their original input
  identities. Their outputs predate the fixes and are never reused as acceptance.

Raw serial bytes, including ANSI sequences and original line endings, are retained
unchanged. Other text logs/manifests redact host workspace/profile paths and use LF.
`SHA256SUMS.json` covers this retained snapshot; original binaries and raw generated
artifacts remain in ignored `build/m1/` at the recorded locations.

## Limits and next gate

This passes B03/B38 only for the declared minimal Limine/OVMF UEFI optical route
and adds the serial slice of B04. P1-03 must validate/normalize boot metadata and
introduce guest session records. Exceptions, memory ownership, interrupts and
timekeeping remain later stories. There is no allocator, validated framebuffer,
desktop, process isolation, persistence, live reload or guest performance result.
BIOS, Secure Boot, alternate loaders, ISO byte reproducibility and release/license
review remain unproven. The 256 MiB desktop cold-idle target is unchanged/unmeasured.
