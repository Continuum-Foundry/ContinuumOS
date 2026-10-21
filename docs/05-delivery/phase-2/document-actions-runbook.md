# P2-03 — Document state and typed actions

The [model contract](../../06-contracts/hosted-editor-model.md) describes exact
semantics/bounds, [Rust core](../../../crates/editor-model/src/lib.rs) owns state,
and [authored fixtures](../../../fixtures/m2/actions-v1/README.md) supply the replay.
This is a headless host model; actual candidate widgets/input rendering start later.

## Reproduce locally

```powershell
. ./tools/Enter-Dev.ps1
pwsh -NoProfile -File tools/Recheck-P2Prerequisites.ps1
pwsh -NoProfile -File tools/Test-P2-03.ps1
cargo xtask test --suite editor-model
node audit/check-docs.cjs
pwsh -NoProfile -File audit/validate-contracts.ps1
node audit/check-governance.cjs
node audit/check-repository-hygiene.cjs
```

Use existing local Rust/signing/offline cache. No tool installation, policy
enrollment, guest build, network, window or VM is part of this harness. Established
unsandboxed execution is needed for certificate-store access. The new lock adds
only cached `unicode-segmentation` 1.13.3 (MIT OR Apache-2.0), defaults off, with
Unicode 17.0.0 data, no runtime transitive dependencies/build script. After reviewing
the exact lock diff, the existing signing runbook requires explicit
`tools/windows/Initialize-DevCompiler.ps1` to refresh lock enrollment. It retains
the signer/policy. `DevSigning.ps1` permits the narrow `editor_model-<hash>.exe`
test executable name in the already declared build roots.

The acceptance script independently validates 21 positive action envelopes and
seven structural mutations through PowerShell JSON Schema 2020-12. It checks
formatting, editor/xtask tests, `x86_64-unknown-none` library compilation, the locked
P2-02 corpus, and two separate replay commands. The latter must produce identical
structured results and checkpoint hashes. Each command replays 21 operations
through button/shortcut/semantic/automation entries and checks all declared states,
roundtrip snapshots after every operation, view destruction/recreation and 56
negative entry/case pairs. Adapter origin cannot bypass authority.

The independent PowerShell decoder validates all four final checkpoints from both
runs against authored final state and historical text. The initial `initial.hex`
was authored with separate Node Buffer little-endian primitives from the contract,
not captured from Rust output; the Rust suite checks its encoder and decoder
against those exact bytes. This is bounded independent codec corroboration, not
formal parser verification or an IPC compatibility guarantee.

Unit cases additionally cover cache eviction/mismatched retries, undo/redo branching,
dirty restoration, directed selection, UTF-8 and extended-grapheme interiors,
joining clusters, context audience/provenance/scope/expiry/revision/revocation,
authority limits, arithmetic exhaustion, full text/history bounds, every checkpoint
truncation with both original and adjusted length headers, and byte mutations.

Complete logs, generated checkpoints and source hashes stay under ignored
`build/m2/P2-03/acceptance-<timestamp>/` and `run-<nanoseconds>-<pid>/`. Each acceptance
records source base/dirty hashes, compiler, OS/CPU, lock/harness/signed executable
identities and exact command exits. Promote only a reviewed compact bundle; run
hygiene against a prepared temporary index without changing user staging.

## Limits and next gate

R19 is an M2 interface/model result. IDs/rights and host-supplied time/cancellation
are modeled. No kernel-backed capabilities, trusted caller IPC, provider
replacement, in-flight fencing, durability, actual GUI semantics, rendering,
input dispatch, font conformance or guest performance is claimed. Snapshots do
not transfer grants/dedup records into a new provider. B09/B37 remain partial;
ADRs remain Proposed and the 256 MiB guest target remains unmeasured.

P2-04 receives the model, action API, pinned segmentation version and fixtures.
It must validate fonts/Unicode/shaping/bidi; P2-05/P2-06 must connect real candidate
widgets and input paths to this executor and rerun the common corpus observations.
