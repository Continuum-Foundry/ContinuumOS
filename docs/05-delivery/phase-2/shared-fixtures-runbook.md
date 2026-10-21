# P2-02 — Shared boundary and fixture checks

This is a headless contract/corpus slice. The
[draft specification](../../06-contracts/hosted-ui-fixtures.md) and
[authored fixtures](../../../fixtures/m2/ui-v1/README.md) define common inputs for
P2-03 and both candidates. No window, rendering or guest change is implemented.

## Reproduce locally

Use the existing workspace-local Rust 1.99.1 and enrolled development signer.
Commands do not change enrollment/host policy, install packages or use the network.

```powershell
. ./tools/Enter-Dev.ps1
pwsh -NoProfile -File tools/Recheck-P2Prerequisites.ps1
pwsh -NoProfile -File tools/Test-P2-02.ps1
cargo xtask test --suite ui-fixtures
node audit/check-docs.cjs
pwsh -NoProfile -File audit/validate-contracts.ps1
node audit/check-governance.cjs
# Against the prepared index, after selecting the compact bundle:
node audit/check-repository-hygiene.cjs
```

The prerequisite script retains full offline Phase 0/signing/doctor results under
`build/m2/P2-01/prerequisites-<timestamp>/`. Certificate store access and xorriso
may require the established unsandboxed local execution context. Do not bypass
signing or change policy when blocked. Preserve failures separately.

`Test-P2-02.ps1` independently checks all seven JSON schemas using PowerShell
`Test-Json` (2020-12), then each negative mutation. Shape errors must fail schema
validation; semantic errors must remain structurally valid so the Rust semantic
validator actually exercises them. It then checks formatting, all xtask tests,
and two separate fixture commands. Source commit and dirty-input hashes, compiler,
command exits, corpus lock and independent schema results go to ignored
`build/m2/P2-02/acceptance-<timestamp>/`. Each fixture command writes generated
`files.jsonl` and its result to a fresh `run-<nanoseconds>-<pid>/` beneath P2-02.
The same generated hash is required in both runs against the authored lock.

The Rust host harness supports only the closed schema-keyword vocabulary used by
these fixtures; unsupported schema keywords reject. It is not a general JSON
Schema engine or an independent codec. PowerShell provides independent structural
corroboration. The semantic pass checks storage arithmetic, stride/length/damage,
buffer-lease transitions, graph bounds/identity/references, input sequencing and
balanced key/button/composition/reset traces, UTF-8 selection boundaries and
analytic color vectors. It does not execute editor actions, real event routing,
rendering, overflow queues, asset reload or resource retirement. Those are later
stories. Failures return nonzero and never update the lock or golden baselines.

## Inputs, provenance and retention

No Cargo dependency or feature changes. Existing serde_json 1.0.154 and sha2
0.11.1 remain pinned in the workspace lock and rights inventory. The validator
is host tooling in `xtask`; no `std` support is assumed in a guest UI. Assets are
synthetic strings/colors/2×2 pixels, not third-party images/fonts. The data generator
has an exact specified seed/arithmetic/UTF-8 serialization and SHA-256 lock.

Retain the review, outcomes, source hashes, compiler/tool identity, exact fixture
lock, independent schema records and small selected failure traces. Keep generated
data, full logs and source patches local. Follow the
[retention policy](../evidence-retention.md); checksum only retained bundle members.
Use a temporary Git index for hygiene review when preserving the user's real
staging area. No remote transfer is authorized by this runbook.

## Handoff

P2-03 implements the document/action model against these expected states. P2-04
resolves common font bytes/terms/Unicode data; its absence blocks rendering, not
this data-only story. P2-05 and P2-06 use the same corpus lock and all 30 size/
scale/variant configurations with five capture checkpoints each. They implement
and test the actual adapters, semantic observations, rich blur and fallback.
Unsupported cases stay visible for P2-08. ADR-0008/0011 remain Proposed.
