# P2-11 — Hosted resource reload passes local acceptance

**P2-11 is Done.** This is Codex local implementation/self-review, not independent
approval, completed M2, guest UI/reload or performance acceptance. Source base:
`625243df7969b1b03dc896c1657fad4880298308`, with final dirty inputs recorded in
[tested-inputs.json](tested-inputs.json). No dependency pin, kernel/ISO source,
signing enrollment or Windows policy changed. No push, PR or upload occurred.

The final guarded run is `build/m2/P2-11/acceptance-1792812988032`. All **ten fresh
prerequisite rows and nine acceptance commands pass**, followed by three separate
Python review consumers. The guard verifies final tested source/tool/fixture hashes
before and after acceptance. The existing signer was reused. See
[commands and artifact identity](acceptance.json), [criterion review](review.json)
and [independent state/pixel/lifecycle checks](independent-review.json).

Signed optimized host SHA-256:
`d6747f0ddfbc34a4184c9c6f80044cfb4824ddb05073aeb93e2b90812cdb7620`.
Rust 1.99.1 / LLVM 23.1.1, `m2-comparison`, on the recorded Windows/i7-1165G7 host.

## Accepted behavior

- Bounded versioned file builder/validator, atomic completed-build replacement,
  content-hash polling, 100 ms debounce/coalescing, complete token/layout/asset
  identity and one coherent scene publication. The starting template now matches
  the actual selected 1280×720 baseline.
- **21 successful publications and 20 negative cases**: truncated/version/unknown/
  stale/oversized data, duplicate IDs/cycles, missing/corrupt font/image identities,
  bad bounds/bindings, genuine paint-stage exhaustion, failed builder/output,
  withheld-reader headroom and recovery. Rejections preserve the active generation
  and accepted state. Dead generation roots are collected after reader release.
- Complete model checkpoints match across the first palette/spacing/topology swap:
  unsaved text/revision, selection, undo **and redo**, document/list scroll, search,
  dirty state, preferences, compatible focus and window geometry. The live provider
  remains in place. Ordered typing, moved capture, compatible preedit/commit,
  removed focus/composition, queue overflow and lost-acknowledgement retry pass.
- The real native presenter records **generation 7→8** in one worker PID/start time,
  session, shell process and window handle, with unchanged native geometry and
  increasing host monotonic time. Palette/spacing/panel movement, descendant
  placement, label and semantic parent changes appear in the same window.
  Exact native checkpoints match, then another document edit succeeds. Partial
  writes and stale retries reject. Pixel hashes independently match the retained
  [before](native-before.png) and [after](native-after.png) PNGs.
- Frame and semantic scene revisions agree. Compatible transitions retain their
  clock; caret/drag deadlines survive publication; reduced motion settles.
  The real reader/held-buffer case retains old immutable data until release and
  rejects a fourth live root rather than freeing in-use resources.

The unchanged **150 captures / 300 workflow states**, **45 text cases** and
**63 integrated text captures** pass. The selected 30-configuration suite also
paints its 45/63 text sets, with 21 authored visual-navigation cases and unchanged
independent 64×64 edge tolerances. Rust checks pass **22 custom-ui + 17 editor-model
+ 3 text-core + 25 xtask** tests, plus 22 behavior-feature tests and portable
`x86_64-unknown-none` checking. The pre-existing ignored paint test is reported
honestly; P2-11 separately exercises a genuine transactional paint-pressure failure.

## Timing, ownership and limits

Trace receive/validate/stage/publish/first completed host-frame samples share one
monotonic clock. Builder durations are separate. The single native replay observed
**494.133 ms** from completed file write to bitmap submission/`Form.Update` return,
including debounce and rendering/copies. This is one observation, not p95/p99,
scanout timing, a passed performance target or a guest result.

The local data subset retains six immutable startup fonts, fixed typed controls,
absolute authored rectangles with a viewport-sized root and explicit retirement.
No new font/image loading, universal layout language, native behavior replacement,
durable activation, native IME or assistive consumer is implemented. At most three
resource roots reserve 24 MiB in total; shared fonts, staged pixels (16 MiB cap),
paint scratch (64 MiB cap), trial model and presentation allocations are separate.
These are bounded host policies, not process-wide OOM recovery or the guest
256 MiB cold-idle result. Full redraw, polling, copies, optimization and the
100-reload statistical stability gate remain **P2-12**. ADRs remain Proposed.

[Development findings](development-findings.json) retain the original certificate
view, incomplete-source/prerequisite, fixture, native replay and template failures
and their resolutions. Full logs, binaries, patch/source inventory and raw captures
stay local in ignored `build/` and `work/`. This compact bundle retains normalized
native observations, real pixels, actual result metadata and input/raw-file hashes;
its checksum manifest covers only retained files. See the
[runbook](../../../../docs/05-delivery/phase-2/resource-reload-runbook.md) and
[file contract](../../../../docs/06-contracts/hosted-resource-reload.md).
