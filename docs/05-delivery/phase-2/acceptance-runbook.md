# P2-13 — Aggregate hosted acceptance

Run with the existing enrolled Windows developer signer:

```powershell
pwsh -NoProfile -File tools/Test-P2-13.ps1
```

The ordinary sandbox has a separate certificate view. Use the enrolled context;
do not change enrollment, export keys or broaden host policy. All Cargo commands
remain locked/offline. Full runs, binaries, captures, source patches and LLVM
reports stay under ignored `build/` and `work/`.

The [fixed protocol](../../../config/p2-13-acceptance.json) declares all sixteen
story rows, seed, exact optimized host profile, freestanding target and deadlines.
The unchanged [performance protocol](../../../config/p2-12-performance.json)
fixes scales, sizes, five paired repetitions, 100 file reloads per resolution,
cache/growth budgets and zero-byte damage tolerance before execution. Existing
fixture locks preserve authored quality/text tolerances.

The runner snapshots source, locks, fixtures, notices, fonts and configuration,
runs all ten current Phase 0/signing/tool prerequisites, then invokes P2-12 with
a fresh Cargo target directory under the signer's existing `build/cargo/`
allowlist. It passes the exact executable explicitly to
both native replay adapters; their observations record its signed hash. Final
text, shared fixtures and four-entry typed-action suites run twice against that
same executable and must produce identical result hashes. Native HarfBuzz and
independent checkpoint readers remain separate consumers.

A second fresh target directory builds only the selected custom freestanding
configuration. Its probe retains actual `retained::Frame::draw` and
`Engine::layout_cached` code. Independent ELF/LLVM inspection requires these
symbols, static RX/R/RW mappings, no unresolved symbols/relocations or host
runtime, and thirteen deliberate malformed-image rejections. Real missing-hook
link and accidental-std compilation failures must match their expected reason.
The 64 MiB non-reclaiming probe arena is unchanged. The ELF is never executed.

Current dependency/feature/archive/source/notice inputs are audited. Selected
package versions/features/checksums/terms, all frozen Slint comparison inputs,
and preserved P2-06–09 evidence are checked against the original decision.
P2-09's chosen-source pins remain historical; final selected source is separately
hashed and rebuilt, never silently substituted into the old record. All eight
comparison axes, Slint failures and provisional distribution choices remain
visible. Actual release clearance stays false.

Each subprocess has a fixed deadline and retained command/result/log identity.
Timeout kills only that owned process tree and waits for reaping. Deliberate
early-exit/timeout processes exercise failure handling; the exact final executable
also runs success/nonzero/timeout lifecycle fixtures. Mutated stale, build,
profile, missing, skipped and unreaped matrix records must fail validation.
Missing output, changed source/input sets, changed executable hashes, failed
consumers and incomplete rows abort without a passing aggregate record. Failed
attempts remain in their own directories. No automatic retries occur.

Local documentation, governance and schema checks finish the run. Repository
hygiene additionally runs after the compact reviewed evidence is staged. Review
kernel/build/package/launch inputs against `main`; unchanged inputs require no
new M1 boot claim. An affected M1 path requires its regression gate before
acceptance. Host preview parameter changes do not change the ISO launcher.

The final self-review must distinguish compile/link, render/input/semantic,
same-process host continuity and measured cost. Frame/reload misses remain named
follow-ups, never guest efficiency acceptance. Host PID/monotonic observations
are not kernel boot ID/uptime. M3 runtime/isolation, M4 guest backend/resource
reload, M5 behavior replacement and later native IME/assistive coverage remain
open. ADRs remain Proposed; no publication, remote issue or upload is authorized.
