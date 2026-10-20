# Phase 1 acceptance

Run from native PowerShell after dot-sourcing `tools/Enter-Dev.ps1`:

```powershell
cargo xtask test --suite m1
# Equivalent wrapper:
pwsh -NoProfile -File tools/Test-P1-07.ps1
```

The pinned xorriso runtime requires the documented execution context outside the
MSYS2 sandbox restriction. All outputs stay local under ignored `build/m1/suite-*`.
The command fails on any missing or failed row; it never retries a failed VM.
The story and its explicit evidence review determine acceptance status.
The [2026-10-20 local review](../../../evidence/phase-1/P1-07/2026-10-20/README.md)
is **Blocked** after Smart App Control refused the xtask host-test binary.
The subsequent [authorized development-signing rollout](../windows-development-signing.md)
is incomplete: its replacement base enforces, but the exact-hash tool supplement
awaits Windows elevation and pinned compiler/Cargo execution remains blocked.
Approval under the existing policy was requested, not achieved. Independent guest
checks and earlier host tests do not waive this row or establish final acceptance.

The aggregator runs formatting, M0 (including harness rejection and process
cleanup tests), dependency probes, kernel library tests, doctor, prerequisite
rejections, documentation links, schema fixtures and governance checks. It builds
the normal kernel into two initially empty Cargo target directories, including a
path with spaces. Five consecutive launches use the first clean kernel and a sixth
uses the second. Each has a freshly packaged optical ISO, distinct host session,
fresh firmware variables and a new bounded offline QEMU process. The 23 existing
named scenarios then cover 36 more launches, including normal idle observation,
metadata variants/rejections and all exception, memory and timer regressions.
Forty-two launches and 33 required rows must be present; every source inventory,
within-flavor ELF identity, session and lifecycle must agree.

Normal acceptance uses the **normal kernel with no Cargo test features** and
`fixture=acceptance-exit`, an explicit emulator-only exit request. This fixture
enables no injected fault or alternative initialization; it exits through port
0xf4 only after the final readiness record. The harness adds `isa-debug-exit` and
requires exit 33. Ordinary `fixture=normal` remains an idle kernel, observed then
stopped/reaped by the host. Test-device exit is not physical-machine shutdown.
Failure paths on a normal build still halt. This bootstrap test configuration is
trusted boot input, not authenticated development authority or a release mode.

Version-5 serial records retain the existing checkpoints and append `m1.ready
stage=ready outcome=ok baseline=boot-memory-timer reclaimed_bytes=0` after
`m1.boot_complete`. The host requires exactly one ordered entry, validated metadata,
exception setup, owned memory, timer epoch, 100 monotonic delivered-tick samples,
timer progress, boot completion and final readiness. Missing readiness, stale IDs,
duplicate stages/entry, malformed records or records after the terminal event fail.
The observer treats expected fatal/rejection outcomes separately from readiness.

The unchanged 30-second VM timeout is now explicit as `vm.deadline_ms=30000` in
the input lock, checked before launch and retained in every manifest. Host commands
have fixed deadlines (600 seconds for suite checks; existing build/package bounds
remain). Timeouts fail positive cases; the deliberate broken-loader/kernel cases
assert bounded failure and cleanup without calling them successful boots.

Packaging fixes volume and file timestamps to `2026102000000000` and the GPT disk
GUID to `61464790-57e2-4a64-a66d-7c6fd0e6770a` (including derived partition GUIDs).
This is the experimental ISO identity, not a unique user-disk identity. Extracted payload
hashes and the UEFI catalog are checked on every package. Comparison requires the
two normal kernels to match exactly. The full ISO images must match after replacing
their explicitly listed 64-byte session fields with zeroes. Every other byte,
including directory/file order, catalog, timestamps and padding, must match.
The actual ISO hashes differ by design and both exact images must boot; no byte
reproducibility or M8 release claim follows from the normalized comparison.

`suite-progress.json` retains completed rows even if the command fails;
`result.json` records the terminal outcome. `suite-inventory.json` contains every
build and launch context. Full disassembly, binaries, source copies and logs stay
local. Retain a reviewed compact bundle under the
[evidence policy](../evidence-retention.md), with selected raw serial output,
per-launch context, one source/artifact inventory per distinct build, checksums,
commands, failures and limitations. Review and update the
[story](07-phase-1-acceptance.md), milestone, backlog, validation and handoff together.

This gate covers the experimental single-core Q35/TCG/OVMF baseline only. It adds
no desktop, heap, isolation, scheduler, persistence, live update, physical-platform
support, Secure Boot, release reproducibility or measured memory/performance claim.
Architectural ADRs stay Proposed. The next bounded workstream is M2's hosted
Slint/custom comparison; B06 and B32 remain partial for later milestones.
