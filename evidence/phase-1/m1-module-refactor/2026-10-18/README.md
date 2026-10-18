# M1 host module refactor — 2026-10-18

The host harness was split into build, package, launch and observe modules under
`tools/xtask/src/m1/`. The 1,030-line `m1.rs` is now a 14-line entry point.
Existing cases and records moved into that directory; their orchestration and
serial contract remain in place. Shared inputs, process and support modules keep
pin validation, bounded commands and artifact operations reusable.

Cases now pass the expected rejection into launch. Observation still spawns and
owns the child, using the shared owning guard to kill and reap it on early errors.
Existing tests moved beside their implementations. Phase 0's input inventory now
lists every M1 module. CLI commands, manifest schemas, deadlines, log limits and
artifact naming are preserved. Build IDs changed because xtask sources are hashed.

This is a host maintenance change, not a new guest feature or completion of M1.
The previous P1-02/P1-03 evidence remains unchanged. Verification was local;
no remote publication, system settings or tool installations were involved.

## Verification

Full regression run: `acceptance-p1-03-1792348291638`.
All 11 commands in [acceptance-results.json](acceptance-results.json) returned
zero. The [review](review.json) independently checks the retained run reports,
identity matrix, exact rejection outcomes, image bounds and process cleanup.
Its [procedure](review-procedure.ps1) is adapted from the existing P1-03 review.
This is local self-review, not external review or a performance measurement.

| Check | Result |
| --- | --- |
| Formatting | `cargo fmt --all --check` passed |
| Host tests | [11 xtask tests](host-tests.log) and [7 kernel-library tests](kernel-host.log) passed |
| Phase 0 | [Acceptance](phase-0-results.json) passed, including clean builds, doctor overrides and source path with spaces |
| Dependencies | [All five library compiles passed](dependency-probes.log); allocation-dependent links remain blocked as documented |
| M1 prerequisites | Doctor and P1-01 rejection checks passed |
| Ordinary metadata boots | Seven passed, including normal observation, smoke, module and serial-only cases |
| Metadata rejections | Eight exact field/reason outcomes and emulator exit 35 |
| ISO negative cases | Two expected failures; these are not positive boot evidence |
| Lifecycle | All 17 launches had distinct host sessions, unchanged ISO hashes and reaped processes |

The standalone `cargo xtask build --profile dev` also passed at
`build/m1/build-1792348285469529100-34020`; it produced the same normal build ID
as the full suite. Exact kernel digests, source commit/dirty state and per-launch
ISO/session identities are retained in the manifests and review.

| Flavor | Build ID |
| --- | --- |
| Normal | `5fdec03978227c1539bf3a40066f2a0f2453638fa0094b8be6c2dd1db69d54eb` |
| Smoke / metadata | `773a6b53c9ef9c15e6980bca9e574f14ffa1e4bb80bb8cbdf4c7bb2cc5fa1348` |
| Owned-input injection | `b57e821a3377e378943884f0802a8ae3097548110addd3135040d305e8299ef1` |
| Pre-entry halt | `88aae4cbdc763251cd96d2232ab017fc7d708c35feba696e3c7c079c6cd31fdd` |

## Baseline and retention

Before edits, Phase 0 doctor, the feasibility build, clean-build comparison,
11 host tests and dependency compiles passed. The first acceptance attempt's
source-path-with-spaces build was blocked by Windows Application Control
(`os error 4551`); its [failure trace](baseline-sandbox-failure.log) is retained.
The unchanged baseline [passed outside the sandbox](baseline-phase-0-results.json).
The final suite used that same execution context for the documented xorriso
MSYS2 restriction. No product-code workaround was introduced.

The [retention procedure](retention-procedure.ps1) checked every current hashed
source against [build-inputs.json](build-inputs.json), including all nine modules.
The [index](evidence-index.json) records raw and retained hashes of the compact,
sanitized verification artifacts. These are summaries, not a full source or ISO
snapshot. Raw ISO images, serial transcripts, manifests and command logs remain
in the workspace artifact directories recorded by the acceptance result.
