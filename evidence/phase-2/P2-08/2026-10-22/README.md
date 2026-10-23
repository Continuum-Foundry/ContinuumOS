# P2-08 — Hosted comparison accepted locally

**P2-08 is Done as a bounded comparison evidence gate.** Candidate performance,
toolkit selection, native-window/file-driven hosted reload and guest execution
are not accepted by this result. Review is Codex's local implementation/self-review,
not independent approval. Source base is `4dae2c537442983016749a8d97a19da5e16391cc`
on `codex/p2-08-ui-comparison`, with exact dirty inputs in
[compiled-inputs.json](compiled-inputs.json). Changes are local and uncommitted.

The [recorded review](review.json), [comparison/recommendation](comparison-report.md),
[measurement summary](measurement-summary.md), [quality review](quality-review.md)
and [runbook](../../../../docs/05-delivery/phase-2/comparable-ui-runbook.md) define
the boundary. Full binaries, PPMs, pretty traces, command logs and failed source
copies remain local under ignored paths in [local artifacts](local-artifacts.json)
and [development findings](development-findings.json).

- All **ten prerequisite rechecks** pass, including signing, fresh Phase 0 links/
  tests, offline dependency probes, source paths with spaces and M1 doctor.
  **51 Rust unit tests pass**, with one existing font-dependent default ignore;
  the candidate regression paths exercise its failed-paint boundary.
- Both optimized candidates replay **30 configurations, 150 captures, 300 workflow
  states, 45 text cases and 63 integrated text captures each** against unchanged
  goldens. Independent state/semantic equality and 150 full image differences
  are retained. The existing independent pixel reviewer also checks every capture.
  Nine text observations per candidate retain 12 missing-cluster counts across
  emoji/intentional-missing probes; native IME/assistive export remains unavailable.
- **20 sequential paired blocks** retain all **6,000 frames and 12,000 process
  samples** as frame/process CSV pairs. Each has 50 identical paced warm-up frames,
  all 180 geometry/workload/variant cells, 300 active frames and 30-second eventless
  idle sampling. No final block times out. Active overruns and deadline misses
  are retained. Raw first-process and warmed distributions remain separate.
- Custom's observed peak private commit is about **166.5–167.1 MiB**, and its idle
  private commit is near **159 MiB**. Slint's observed peak is about **1,132.7–1,134.4
  MiB**, with about **871–873 MiB** added during active sampling. The current
  bridge shows a new component on each paint without hiding it, retaining Slint's
  documented extra reference. This is an adapter lifetime defect, not an intrinsic
  toolkit memory requirement. Both paths miss frame budgets frequently.
- **100 alternating direct token/layout publications per candidate** change output
  while exact owner checkpoint bytes remain unchanged. Two compiled helper variants
  restore that same snapshot and insert `Continuum` versus `CONTINUUM`. Owner PID,
  binary hashes, model state, selection, focus, window and monotonic samples are
  checked. Helpers are restarts/proposals; no active behavior route or native unload
  is claimed. Build/start/restore/returned-checkpoint failures and forced timeout
  preserve the owner; all ten helpers are reaped. Active composition/capture blocks
  launch, and removed focus cancels transient input in a separate fixture.
- The independent sample reviewer validates nearest-rank aggregation and rejects
  **15 corrupted sample/mode cases**. Contract schema checks pass **9 positive /
  16 negative fixtures**. Documentation, governance and Git-index retention checks
  are required before completion; their final local outputs remain under `work/`.

| Exact signed host artifact | SHA-256 |
| --- | --- |
| v1, `slint-candidate` | `9b4c0a6692d780bb093ea8889454c443194c398e356c9973962395197c3e4a49` |
| v2, `slint-candidate,behavior-v2` | `c2a4ddaf34d9fb1c653d24e6e72728f457ebb22aa4ab04ea462aa4551f2f20ed` |

Both are 3,660,176-byte combined harness executables. They are not minimal
candidate-specific production binaries or resident-memory measures. Rust 1.99.1,
Windows NT 10.0.26300.0 and Intel i7-1165G7 identify the host. Uncontrolled laptop
power/load/cache variation makes wall-clock paired deltas noisy, sometimes
negative; none is silently subtracted. `0` idle CPU deltas are quantized counter
observations, not proof of zero work. CSV blanks denote unavailable counters.
See [accounting](accounting.json), [paired deltas](paired-overhead.json) and raw
samples for buffer/cache/trace/allocation scope. Combined helper/compiler peaks,
OS scheduler wakeups and native clock/caret/drag workloads are unsupported.

The preliminary unpaced warm-up and its unequal frame counts remain separate;
the entire accepted matrix was rerun with equal work. Provider rollback rejection,
an existing host-process test startup stall, optional Pillow absence and their
corrections remain in development findings. No Cargo lock, dependency version,
certificate, signer enrollment or Windows policy changed. Source newline
reconstruction metadata explains working-tree CRLF inputs where applicable.

The recommendation is **conditional custom consideration at P2-09**, with bounded
raster/damage/cache/motion/layout completion and remeasurement. Slint's measured
speed benefits do not waive retained-component, fidelity, native adapter or OS/API
terms gaps. No toolkit is selected and all ADRs remain Proposed. No guest heap,
loader, input, UI, live update, security guarantee, efficiency target or release
clearance follows. The 256 MiB guest cold-idle target is unchanged.
