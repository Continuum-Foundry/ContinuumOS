# P4-01 initial prerequisite attempt — Blocked

P4-01 started on freshly pulled `main` at
`15e5113b112c179ac9dbe7ad1426a67a2be9f301`. The three preexisting Windows
signing/policy edits were preserved. The fresh strict M3 baseline **failed**;
this bundle is a local diagnostic/self-review record, not passing P4 or M3
acceptance. The earlier strict M3 result remains historical evidence unchanged.

## Observed results

`pwsh -NoProfile -File tools/Test-P3-14.ps1` produced
`build/m3/P3-14/acceptance-1794009093298631100/acceptance.json`.
Host runner/lifecycle preflights, the public supervision guest and independent
review, trace-off stress repetition 0 and independent review, and separate
stress metadata flavors passed. The trace-off guest took 26462 ms within the
original 30000-ms deadline. These partial successes do not pass the full gate.

The first trace-on guest hit its unchanged 30000-ms deadline; the host recorded
30118 ms including stop/reap overhead. QEMU was reaped and the ISO stayed
unchanged. The raw trace completes waves 0–81 and begins wave 82, ending at
sequence 2115 without `m1.route_ready`/`m1.ready`. The existing independent raw
consumer rejects it as `incomplete guest`. No terminal record was synthesized.
This is a guest deadline failure, not a host preflight timeout or an acceptance
interruption. Cause remains unresolved; timing alone does not identify one.
No retry, timeout extension, sample reduction or host-policy change was made.

The nested failure is retained at
`build/m1/sdk-stress-trace-1794009349738308300-29224/sdk-stress-trace-run/`.
The native live observer read 756488 bytes; its recorded read and parse costs are
111215200 ns and 25274600 ns. Those observations alone do not establish why the
guest missed its deadline or imply a desktop performance result.

Complete fresh M1, the remaining M3 matrix and affected hosted M2 acceptance were
not reached by this attempt. The separate read-only
`pwsh -NoProfile -File tools/Recheck-P2Prerequisites.ps1` invocation passed all
ten offline prerequisite commands, including signing/compiler integration,
Phase 0 clean compile/link checks, native tests, doctor negatives and the build
from a source path containing spaces. Its [report](prerequisites.json) is from
`build/m2/P2-01/prerequisites-1794009454400/`. The native suite reports 35 passed
and two preexisting ignored tests; this is not a full affected M2 gate or a new
test waiver. Dependency probes retain their expected missing-allocator final-link
blockers; compile-only dependency feasibility is not a guest runtime port.
This prerequisite pass cannot override the failed M3 guest gate.

## Planning work and limits

The [runbook/hook map](../../../../docs/05-delivery/phase-4/prerequisites-runbook.md),
[candidate profiles](../../../../docs/05-delivery/phase-4/profiles-draft.json) and
[boundary/lifetime plan](../../../../docs/06-contracts/desktop-v0.md) identify
runtime/image/heap/table/credit constraints and owning stories. Profiles remain
unvalidated candidates; draft codecs have no independent byte fixtures or guest
consumers yet. No new ABI ID, runtime capacity, driver or cryptographic selection
is accepted. P4-02–19 stay Not started, and no runtime/device extension proceeded.

Fresh bounded `-device ...,help` calls confirm that the exact hash-verified pinned
QEMU exposes standard VGA, VirtIO PCI keyboard/tablet/serial and a named serial
port. These calls launch no VM. They do not prove firmware mode selection, actual
PCI/IRQ discovery, guest feature negotiation, delivery, reset or DMA confinement.
The proposed dev-disabled profile omits device/port/backend/grants and preserves
separate diagnostic serial; real enforcement remains a later guest gate.

## Provenance and retention

[Diagnostic findings](diagnostic.json) record completed/pending work and the
unresolved blocker. [M3 attempt](m3-attempt.json), [nested stress attempt](stress-attempt.json),
[trace case](trace-case.json), [launch identity](trace-launch.json),
[selected raw tail](trace-tail.txt), [trace-off review](trace-off-review.json) and
[supervision review](supervision-review.json) preserve the observed distinction.
The [baseline source lock](baseline-source-lock.json) still matches every
implementation input at local review; draft documents were prepared separately.
The original dirty patch remains local with the run. No source or binary changed
to explain away the failure.

[Normalization](normalization.json) records original local hashes and path
sanitization; internal references identify original unsanitized local artifacts.
[Draft provenance](draft-provenance.json) identifies the unvalidated planning
bytes. [SHA256SUMS.json](SHA256SUMS.json) covers only the compact retained files.
Full logs, source patches, ISOs, binaries and generated source copies remain local.
Historical missing local-only artifacts were not treated as fresh verification.
No remote publication or human/security certification is claimed.
