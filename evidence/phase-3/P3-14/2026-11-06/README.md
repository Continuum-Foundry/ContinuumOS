# P3-14 - strict M3 acceptance and runner repair

The fresh full gate and independent aggregate reviewer pass all **23 matrix
rows and five deliberate failures** under local Codex self-review. The original
30000-ms positive guest, 1000-ms normal Python child, 100-ms negative host-child
and 1-ms negative guest deadlines are retained. No exception, historical
regression reuse or prerequisite reuse is used. The strict M3 isolated-runtime
and interface exit passes; this is not independent human/security certification.

The final run is `build/m3/P3-14/acceptance-1793943120165050100/acceptance.json`.
Its source is commit `254e73b3e589e24278fab00a32aaa6bdfb39435b` plus the retained
local dirty patch and exact [source lock](source-lock.json). Full generated logs,
source snapshots, ISOs and failed/interrupted attempts stay local. The bundle
contains compact reviewed metadata and selected raw records, not complete logs.

## Repairs and verification

- The observer reads appended bytes once, buffers partial records and retains
  validation state. It keeps the same checks, log cap and polling interval;
  final assertions independently reopen the complete log. Differential chunk,
  malformed-record, sequence/identity, terminal-state, output-limit and truncation
  tests pass, along with real child cleanup tests.
- The retained-log benchmark processes 931518 incremental bytes instead of
  106927806 repeated-prefix bytes. This final-source sample takes 5.3826 ms
  incrementally versus 678.8195 ms for full-prefix polling. These are host-parser
  observations, not guest CPU or desktop performance measurements. See
  [benchmark output](observer-benchmark.txt).
- Stress runs before the long M1/runtime regression. Supplied reuse references
  fail before guests if missing or stale. `-StressOnly` produces
  `diagnostic-passed`, which final review rejects as acceptance. Stage start/end
  timestamps and paths stream immediately; reports retain elapsed stage times,
  observer bytes/read/parse time, and child creation/completion time.
- The supervision fixture had assumed five log calls stayed inside one quota
  epoch. A failed trace contains five accepted calls, consistent with crossing
  a boundary. Both guest fixtures now share a bounded clock-interval oracle;
  same-window fifth successes, early quota errors, invalid statuses and reversed
  clocks reject. Three focused host oracle tests and both independently reviewed
  supervision guests pass. No kernel, scheduler, public SDK or wire contract changes.
- All ten focused stress guests pass at the original limit (19250-26243 ms).
  The subsequent fresh full gate separately repeats the complete stress matrix,
  prerequisites, M1 (33 rows/42 launches), runtime/SDK/codec/portable regressions,
  separate build flavors, legacy host lifecycle and deliberate-failure probes.
  The current native suite passes 35 tests; its font-dependent test stays in the
  separate UI gate, and the benchmark is explicitly executed outside that suite.

## Provenance and limits

The [review](review.json), [acceptance](acceptance.json), [development findings](development-findings.json)
and [observer measurements](observer-summary.json) distinguish final acceptance
from earlier failures, interruption and diagnostic-only results. The
[normalization map](normalization.json) records the original compact-file hashes
and workspace-path sanitization. Hash references inside sanitized reports name
the original local artifacts; [SHA256SUMS.json](SHA256SUMS.json) verifies the exact
retained bytes. [Documentation hashes](documentation.json) record the final prose.

The [2026-11-04 owner exception](../2026-11-04/README.md) remains historical;
its failed or interrupted rows are not relabeled passed. No slower-host profile
was needed after the fixes. M4 guest desktop/live assets is next. Guest UI/reload,
M5 state migration and safe retirement, M6 durability, M7 real accounts and all
desktop efficiency targets remain open. Only the existing exact SDK contract
subset is frozen. One vCPU/one thread per process, masked SSE exceptions,
unverified native x87 debug metadata and ephemeral synthetic policy/routes remain
the admitted limits. No remote upload is authorized by this evidence.
