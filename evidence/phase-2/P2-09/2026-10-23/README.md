# P2-09 — Provisional custom Rust UI decision

**P2-09 is Done as a provisional decision gate.** Reviewer: Codex local implementation/self-review, not
independent approval. Source base: `1e84686` after the requested main pull;
local topic branch `codex/p2-09-ui-selection`. Changes are uncommitted.

The [decision review](../../../../docs/05-delivery/phase-2/ui-selection-review.md)
evaluates every comparison axis and mandatory criterion, assigns source/adapter
ownership, freezes Slint comparison inputs and lists required M2 completion work
plus M3/M4/M5 integration. [Exact selection](../../../../legal/ui-selection.json)
pins package/features, fonts, licenses/notices and source. The selected path is
custom-ui/text-core/editor-model 0.1.0 with the existing CPU renderer and Rust
builders plus validated resource/layout data. All ADRs remain Proposed.

The choice favors demonstrated fidelity and explicit ownership. It does not favor
custom frame speed: trace-off p95 was 159.358 ms, with 1500/1500 frames missing
16.667 ms. Those are P2-08 host RAM-submission observations, not a new measurement
or a guest forecast. P2-10 through P2-13 remain required and Not started.

The custom target's 28 upstream packages now have explicit MIT choices except
Apache-2.0 unicode-linebreak and Zlib foldhash. Six unchanged Noto fonts retain
OFL terms; Unicode and supplemental notices remain. The previously missing
rustybuzz ms-use generator MIT notice was copied byte-for-byte from the already
verified cached source, with provenance. No UI implementation, dependency version/feature/lock or guest
artifact changed. Slint terms are unresolved and it remains unselected.
Actual release/handling/ownership and artifact notice delivery gates stay open.

Reproduce using the commands in the decision review. Full prerequisite logs,
patch and local review scratch remain under ignored build/work paths. This
compact bundle records checks and exact input hashes; it does not duplicate
the upstream P2-07/P2-08 evidence or generated sources.

No render, benchmark, guest boot/UI/reload, active behavior route, security or
release clearance is established by this documentation/selection gate.

All ten fresh offline prerequisites pass. [Identity/closure checks](checks.json)
verify 120 upstream manifest members, the unchanged chosen source modulo recorded
CRLF/LF representation, all six local font byte identities and 22 rejected
selection mutations. The [five-criterion review](review.json),
[input hashes](inputs.json) and [development findings](development-findings.json)
record the boundary. Contract checks pass 9 positive / 16 negative fixtures.
Documentation, governance and staged-index hygiene are required final local gates;
their full reports remain in `build/m2/P2-09/validation/` and `work/`.

Chosen target ELF from P2-07: SHA-256
`a0c99bba76e4eccb50963baa60ce109d5cff5ed3d58ed296ae4a985219b57c67`,
25,476,384 bytes. This artifact is final-linked, never executed. The selection
reviewer confirms its accepted identity; no new selected-core binary is built
by this decision. No compiler lock enrollment, certificate or Windows policy changed.
