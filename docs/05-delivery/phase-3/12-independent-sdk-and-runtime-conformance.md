# P3-12 — Prove independent SDK interoperability and runtime closure

Issue type: Story. Epic: P3. Status: Done. Owner: Unassigned.
Depends on: P3-11 Done. Suggested labels: `phase-3`, `sdk`, `interoperability`.
Backlog: B02/B11/B12 ABI closure, B13 conformance, B15/B37 handoff.
Requirements: R03/R10/R14/R18; preparatory R05/R06/R19.
Test families: T02/T03/T06/T11/T13; T14 contract subset.

## User story

As an SDK consumer, I need to build without private kernel/service code and still
interoperate so compiler layout or shared implementation mistakes cannot define the ABI.

## Implementation scope

1. Package the smallest public no-std Rust startup/syscall/object/IPC/route SDK and
   its contract source, generated bindings and byte fixtures. Build client and
   service in separate clean output roots against declared SDK versions, without
   private kernel structs/modules or shared native-layout definitions.
2. Supply a separately implemented reference encoder/decoder and semantic checks
   that do not call the production codec. Test every supported operation/message
   and error/ownership rule, version discovery, compatible minor/features and explicit
   incompatible major/mandatory-feature outcomes. Compare exact canonical bytes.
3. Execute independently built clients/services inside the actual ISO guest,
   including handle transfer, trusted sender context, waits, policy checks and
   proxy route switch/fencing. Keep a prior compatible client artifact unchanged
   while rebuilding the service; record both input/artifact identities.
4. Close M3's selected runtime hook obligations using real reclaiming allocation,
   zeroing, startup/termination, preempted math/FP state and bounded process failure.
   Use a nonrendering portable text/model/math smoke workload under these hooks;
   review the exact selected dependency instruction/features. Do not run a display
   backend or imply the hosted raster pipeline has passed guest confirmation.
5. Freeze only the tested syscall numbers, explicit startup/message encodings,
   error/version/ownership rules and SDK subset after these tests. Synchronize
   specs, schemas/examples, generated bindings and compatibility fixtures. Reserve
   unimplemented families as unsupported rather than freezing invented behavior.
6. Record M4/M5 handoff: dev-session binding, real clock/wait usage, presentation/
   input/immutable-resource obligations, provider-side scope/revision/deadline checks,
   replacement reauthorization and checkpoint authority exclusion. Distinguish
   monotonic deadlines from wall-clock values in public types; later file/control/
   diagnostic service seeds are interfaces only.

## Acceptance criteria

- [x] Client and service build independently from clean locked inputs using public
  contracts only, then interoperate in the guest without private native-layout sharing.
- [x] An independent codec/semantic consumer verifies every supported message and
  malformed fixture; changing one production codec cannot make both sides agree falsely.
- [x] Unchanged compatible clients work across a fixture service rebuild/route switch;
  incompatible versions/features fail before mutation without a stale-writer bypass.
- [x] Actual runtime hook tests demonstrate reclaiming allocation, FP/context
  isolation and bounded failure for the nonrendering portable workload; remaining
  M4 presentation/text/font/input/resource obligations are itemized explicitly.
- [x] The tested ABI subset and compatibility policy are reviewed/frozen coherently;
  no untested family, credential format or desktop-service implementation is claimed.
- [x] Public SDK types and M4/M5 authority/lifetime handoff distinguish process,
  session, policy and route epochs, monotonic/wall time and data versus authority.

## Tests and evidence

Retain clean build recipes, public dependency graph, byte fixtures, independent
codec outcomes and exact kernel/client/service/SDK hashes. Test truncation, all
length/rights/enum/reserved-bit classes, required-feature mismatch, old/new minor
versions and deliberate producer/consumer byte mutations. Use separate guest
artifacts for nonrendering runtime stress; absent presentation is not a stub success.
Rerun affected M2 portable/link checks if public build/shared code changes.

## Exclusions and handoff

No general native application SDK release, UI guest backend, dev transport or real
behavior reload. P3-13 stresses the final interoperable runtime; M4/M5 receive the
tested bounded subset and open obligations. Sources:
[contracts index](../../06-contracts/README.md),
[objects contract](../../06-contracts/ipc-and-objects.md),
[actions contract](../../06-contracts/actions-and-context.md),
[runtime hooks](../../../experiments/ui-link/hooks.json),
[desktop service seeds](../../06-contracts/desktop-services.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).

## Local acceptance — 2026-11-01

P3-12 passes local Codex self-review for the bounded independent SDK 0.1.0 subset. Three clean public-only build roots per ISO preserve one unchanged client across minor-0/minor-1 service builds. The route guest completes 20 version exchanges, two accepted immutable moves, two incompatible-message handle closures and the full nine-incarnation writer-fence matrix. A separate six-process ISO passes reusable 64 KiB heap/32-slot limits, zero/copy/reallocation checks, selected model/grapheme/libm work, preempted FP state, normal return, panic, fallible OOM and exact teardown. The independent reference passes 4871 byte cases and 2411 separately supplied producer cases, including paired field-swap rejection. All 93 kernel host tests, ten fresh prerequisites, complete M1 (33 rows/42 launches), ten prior runtime regressions, eleven separate metadata build flavors and affected selected portable link/model checks pass. Only the exact tested contract hash is frozen; supervision and later desktop families remain experimental. No guest display/font/shaping/raster backend, dev transport, state migration, safe old-code retirement, M3 exit or desktop efficiency target is accepted. P3-13 is next.

See [criterion/unsafe review and exact artifact identities](../../../evidence/phase-3/P3-12/2026-11-01/README.md), [SDK contract](../../06-contracts/sdk-v1.md) and [runbook](sdk-runbook.md). ADR-0028 remains Proposed pending project review.
