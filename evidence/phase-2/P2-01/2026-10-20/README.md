# P2-01 — local prerequisite and profile acceptance passes

**P2-01 is Done.** Codex implemented and self-reviewed this slice locally on
`codex/p2-01-hosted-prerequisites`; this is not independent approval. The existing
uncommitted Phase 2 planning changes were preserved. No push or PR was made.

The [review](review.json) checks fresh offline prerequisites, current source and
signed executable hashes, and all 23 headless lifecycle cases. See the
[runbook](../../../../docs/05-delivery/phase-2/hosted-prerequisites-runbook.md) for
real commands, measurement rules, dependencies and limits.
Documentation, synthetic contract schemas, governance and the prepared Git-index
repository-hygiene checks pass. A temporary review index preserves the user's
real staging area; the changes remain uncommitted.

## Accepted results

- All ten prerequisite commands pass: fresh signed execution; clean offline Cargo
  compiler/build-script/proc-macro/test/run fixture; workspace Windows-filtered
  dependency metadata; both doctor forms; feasibility build; M0 clean/link test;
  Test-M0 (including source/output paths with spaces and override rejection);
  dependency probes; and M1 doctor against pinned inputs.
- Fifteen xtask tests pass, including all existing M1 host tests and the new profile
  validator test. Two fresh freestanding M0 probe builds match and receive
  independent LLVM inspection. These probes are not booted.
- Both optimized and diagnostic builds pass all six resolution/scale combinations:
  1280×720 and 1920×1080, scales 1.0/1.5/2.0, with a fixed 60 Hz target.
- The 23 lifecycle cases comprise **13 successful launches**, **one expected
  nonzero exit**, **one forced timeout**, and **eight rejections before launch**.
  All **15 launched children are reaped**. Paths with spaces pass. Invalid
  size/profile/scale/deadline, missing configuration/input, changed input checksum
  and unknown fields reject clearly. Expected failures retain a failed runner
  status and nonzero command exit; they are never relabeled as successful runs.

[prerequisites.json](prerequisites.json) records the host CPU/OS, enrolled signing
expiry, executable runner config, exact resolved package features/terms and cached
facility/notice hashes. [m0-results.json](m0-results.json) records current probe
inputs/artifacts; [m0-acceptance.json](m0-acceptance.json) and
[doctor-tests.json](doctor-tests.json) record the path/override gate.
[acceptance.json](acceptance.json) indexes full local result paths; the review maps
them to retained `cases/` records with the same hashes. Selected raw lifecycle
traces, two raw clock-pair sample sets, negative configs and sanitized prerequisite
output are retained. The checksum manifest covers only retained files.

## Failure history and limits

The initial sandbox attempt could not see the enrolled current-user certificate.
[Its trace](traces/sandbox-signing-failure.txt) and
[command exits](sandbox-results.json) remain separate. A read outside the sandbox
confirmed the existing private key and validity through 2027-10-20. Both subsequent
prerequisite runs pass with that signer in the established local execution context;
no host policy, trust enrollment, tool install or network override was changed.

All five candidate libraries compile. Only the architecture probe finally links;
the other four still stop for lack of a guest allocator. The
[dependency results](dependency-results.json) and selected allocator failure retain
that boundary. No library is selected by a compile pass. Cached native-window,
presentation, tracing and image packages remain inventory, with no new dependency
or selected hosted feature graph. Fonts/text assets still require P2-04 provenance.

The timer-call samples describe only this Windows host clock-call baseline; zero
minimum/median readings can occur and are retained. They do not establish UI
instrumentation overhead, display timing, host-to-guest performance or the 256 MiB
guest cold-idle target. Workload families and measurement lengths are fixed;
scene/input scripts and pixel/semantic tolerances belong to P2-02.

P1-07's accepted 33-row/42-launch M1 evidence remains the VM boundary. Its
historical raw records are local-only where documented in
[evidence/README](../../../README.md). This slice changes no kernel behavior and
does not rerun the complete VM matrix. There is no window, rendered scene,
toolkit selection, guest UI, state-preserving reload or new guest performance
result. P2-02 is the next bounded story; ADRs remain Proposed.

Full generated output stays in the ignored `build/m2/P2-01/` and nested Phase 0
run paths recorded by the review. Source commit/dirty hashes distinguish compiled
inputs and final reviewed documents. No raw source tree, binary, key or certificate
is promoted into this compact bundle.
