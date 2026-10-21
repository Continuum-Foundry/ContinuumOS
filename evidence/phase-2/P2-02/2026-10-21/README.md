# P2-02 — shared boundary and corpus acceptance passes

**P2-02 is Done.** This is Codex's local implementation and self-review on
`codex/p2-02-shared-boundaries-fixtures`, based on local commit `54ae8fe`.
It is not independent approval. Changes remain local and uncommitted.

The [review](review.json), [acceptance](acceptance.json) and
[reproduction runbook](../../../../docs/05-delivery/phase-2/shared-fixtures-runbook.md)
record the actual checks and limits. Both candidate stories now require the same
[authored corpus](../../../../fixtures/m2/ui-v1/README.md) and
[draft contract](../../../../docs/06-contracts/hosted-ui-fixtures.md).

- Seven positive fixture/schema pairs cover surfaces, tokens, layout, product
  state/text/data, physical/text/composition/pointer input, variants and paint.
- All **56 negative mutations** produce their declared rejection classes.
  Independent PowerShell schema checks distinguish structural failures from
  structurally valid semantic failures. Two checked integer-overflow and three
  malformed nonfinite-number parse tests also pass.
- All **18 xtask tests** pass, including existing host regressions. Formatting,
  documentation, contract schemas, governance and prepared-index hygiene pass.
- Two separate commands generate identical **10,000-row** synthetic file lists
  and match the authored SHA-256 lock. Generated data stays local.
- All ten offline prerequisite commands pass, including fresh signing/compiler,
  Phase 0 compile/link/path/dependency checks and M1 doctor. No host policy or
  dependency was changed. The complete VM matrix is not rerun for this host slice.

The exact corpus lock SHA-256 is
`674e96d299fe5dfe651c9ca49e6d671d8207a814ed9cc106d6f6e930cfcb50e2`.
The generated file-list SHA-256 is
`ba16344854638caeaaaa34c69696ba4dac54fb4742c6ec41dc947af9a1ac0196`.
The acceptance records the actual signed host executable and harness hashes;
these identify host tooling, not a guest build or boot.

The contract fixes buffer lease serials separately from resource generations,
encoded-sRGB premultiplied color arithmetic, bounded damage, event ordering/reset,
stable logical/semantic IDs and a non-executable token/layout subset. It predeclares
30 size/scale/variant configurations and five capture checkpoints per configuration,
with distinct pixel, geometry and state/semantic rules, rich blur and reduced
effects. Those **150 future captures have not been rendered**. No fabricated
golden images or image-comparison pass is retained.

The [development findings](development-findings.json) preserve observed compile/
harness/fixture failures and their fixes. Complete generated runs, source snapshots
and logs remain under the local paths in the acceptance; selected raw host output
is retained in `traces/`. Source hashes distinguish compiled inputs from the final
reviewed documentation. The checksum manifest covers only this compact bundle.

P2-03 is next: implement the logical state and action provider against the declared
expectations. Font slots remain deliberately unresolved until P2-04 locks exact
licensed font bytes, fallback and Unicode data for both candidates. No renderer,
executed edit/action model, real queue overflow, resource publication, reload,
toolkit choice, guest IPC, accessibility support or performance result is claimed.
ADRs stay Proposed; the 256 MiB guest cold-idle target remains unmeasured.
