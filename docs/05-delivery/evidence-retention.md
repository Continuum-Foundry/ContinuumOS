# Evidence retention and bounded local storage

Owner-authorized revision, 2026-11-14. Applies to future story selections;
historical bundles and the existing P4-04 selection are not migrated.
This revision authorizes no deletion, history rewrite or upload. Test counts,
deadlines, milestone exits and independent acceptance reviewers are unchanged.

## Minimum evidence and lifecycle

Keep the smallest set that supports every accepted criterion, identifies exactly
what ran, explains meaningful failures and reconstructs tested inputs. A passing
summary or filename extension cannot determine sufficiency.

| Storage class | Contents | Lifetime |
| --- | --- | --- |
| Committed bundle | Criterion review/outcomes/limits, selection/omissions, build/session/artifact/tool identities, checksums, necessary raw observations and failure diagnostics | Permanent; existing 4 MiB / 200 files / 256 KiB per-file limits |
| Local capsule | Original execution/review journal, complete selected streams and statistical samples, source reconstruction material, meaningful failure records | One canonical compressed capsule per final result; retain while claims are relied upon |
| Active payload | Exact ISOs/ELFs, generated sources, full disassembly, source snapshots and intermediates required by current reviewers/resume | Required cleanup after PR merge and verified evidence; retain documented release/debug/review/resume exceptions |
| Shared caches/tools | Cargo intermediates, dependency caches, pinned compiler/emulator/firmware | Separate inventory/budget; no per-story copies by default |

Record a base commit plus exact dirty patch (staged and unstaged), deleted paths
and necessary untracked inputs. Hashes alone do not reconstruct dirty bytes.
Record tool versions/hashes and acquisition pins. Avoid repeated source trees.
A rebuild recipe does not prove bit-identical output.

Preserve complete selected streams when sequence, absence, counts, timing or
settlement are checked, and all required performance samples and negative outcomes.
Screenshots/excerpts supplement them. Preserve a failure report, relevant raw trace,
context and diagnosis for each materially different failure. Equivalent attempts
can reference canonical diagnostics plus their attempt IDs/outcomes. Unresolved
failures and binary-level investigations retain their necessary exact payload.

New story manifests use schema 2 and a predeclared `retention` object:

```json
{
  "working_set_budget_bytes": 4294967296,
  "capsule_budget_bytes": 67108864,
  "cleanup_after": "Required after PR merge and verified evidence; retain documented release/debug/review/resume exceptions.",
  "required_evidence": "Criterion streams/samples, reviews, commands/context, source reconstruction and meaningful failures.",
  "source_reconstruction": "Base commit plus exact dirty patch and necessary untracked inputs, selected at closeout."
}
```

These are starting budgets (4 GiB active, 64 MiB compressed), not quotas to fill or
a universal evidence-size claim. Choose smaller host-only budgets; justify larger
ones before execution in the impact rationale. Do not silently enlarge budgets
after failure: measure, explain necessity, revise the selection explicitly and
preserve the failed record. Inventory aggregate local payload/cache usage before
each story; record free space and headroom for the declared working budget.
Keep one active attempt's necessary payload and referenced resume dependencies;
superseded payload becomes eligible for reviewed cleanup after preserving unique
evidence. Tools/caches are shared, not permanent per-story evidence.

The runner measures attempt output and referenced artifacts before/after checks
and at finish, stopping on a budget breach. Duplicate paths count once; separate
copies count separately. This is a checkpoint bound, not a filesystem quota or
peak-space guarantee. Shared caches/tools and unreferenced subprocess scratch need
the separate inventory; they are outside the runner's measurement.

## Verified compaction without deletion

`tools/evidence_retention.py` offers `plan`, `pack` and `verify`. It never deletes,
extracts or rewrites original records. Select individual workspace files explicitly;
it writes a new ZIP exclusively under ignored `work/` or `build/`, compresses and
deduplicates by SHA-256, and verifies every retained blob. It rejects missing raw
coverage, streams not declared complete, raw hashes absent from their check's
journal, missing review/provenance, unexplained artifact retention and exceeded
budgets. Semantic sufficiency, truthful completeness and adequate source patches
still require engineering review; these cannot be inferred from an allowlist.

After independent acceptance review, author a local selection JSON:

```json
{
  "acceptance": "build/story-validation/example/attempt/acceptance.json",
  "failure_disposition": "Select material failure reports/traces below, or explain why none exists.",
  "files": [
    {"source": "build/story-validation/example/attempt/docs.stdout.log", "kind": "raw", "checks": ["docs"], "complete": true, "reason": "Complete command outcome and diagnostics."},
    {"source": "work/example-tested-inputs.patch", "kind": "provenance", "checks": [], "reason": "Exact tested dirty input reconstruction."}
  ]
}
```

Add all necessary streams/reviews/provenance/failures; the two-file example is
illustrative. Each executed check needs complete raw evidence hashed in its
journal. The journal is included automatically. Use `kind: "failure"` for failure
records and `kind: "review"` for additional reviews. `kind: "artifact"` also needs
`artifact_reason` explaining why exact binary bytes remain necessary. Preserve
release/ISO artifacts where required by their gates; this policy waives none.

Keep both command stdout and stderr, even when a test writes its results only to
stderr. Schema-2 execution binds both streams and the command record into artifact
hashes; capsule selection requires both streams. Guest serial and performance data
remain additional required selections whenever the criteria depend on them.

```powershell
python tools/evidence_retention.py plan --selection work/example-retention.json
python tools/evidence_retention.py pack --selection work/example-retention.json --capsule work/evidence/example.zip
python tools/evidence_retention.py verify --capsule work/evidence/example.zip
```

Record the capsule checksum, compressed size, selected-file inventory and review
of sufficiency in the compact bundle. The ZIP stays local, outside the Git checksum
manifest; record its local availability separately. Include the small selection
and verification result in Git. Capsule integrity is **not acceptance revalidation**:
the original passed/failed/interrupted status stays unchanged. Verification opens
no original payload. Fresh artifact review and resume still require original bytes
and unchanged source/tool context. After cleanup, report unavailable original
payload explicitly; its hash or a capsule summary cannot replace a missing ISO/ELF.

## Required cleanup after every merged PR

Owner direction, 2026-11-14: every merged PR requires a cleanup pass before its
post-merge handoff is complete. Opening a PR or finishing validation does not
trigger removal. Record the PR URL, verified merged state, merge commit/time and
verification source. This is an agent obligation, not a background merge watcher
or automatic deletion hook.

1. Reverify retained capsule and compact-bundle checksums and review sufficiency.
   Preserve those evidence records under their retention rules.
2. Inventory the PR's exact local payload paths, including temporary ISOs, extracted
   ISO trees, ELF copies, generated source snapshots and duplicate build outputs.
   Never sweep all `work/` or `build/`; preserve unrelated work and shared tools.
3. Review release, debugging, active review and resume consumers. Retain each needed
   artifact with its reason, consumer/issue or release reference, and recheck event.
   Unresolved investigations and release-required exact ISOs are valid exceptions;
   speculative future usefulness without a consumer is not. Revisit exceptions
   when the named consumer ends.
4. Preview exact resolved workspace paths, hashes and byte totals; classify each
   as `remove-after-merge` or `keep`. Recheck consumers and bytes immediately before
   later authorized deletion. Refuse outside-workspace paths and payload shared
   with an active attempt. Directory removal also requires checking every descendant
   against the reviewed inventory.
5. Remove eligible temporary payload, including temporary ISOs, in the authorized
   cleanup task. Record actual removed/retained bytes, exceptions and availability
   changes in a separate cleanup receipt. A preview is not completion. If nothing
   was generated, record a no-artifacts disposition. If blocked or owner-deferred,
   record pending cleanup and the next action; never silently waive the step.

`tools/evidence_retention.py cleanup-plan --selection <local-cleanup.json>` verifies
capsule identity/integrity and compact-bundle members, requires a recorded merge
and sufficiency review, rejects removal of protected evidence/shared tools or
declared live dependencies, and reports candidate/retained bytes. This is an
offline read-only preview; merge truth and consumer completeness need engineering
review. Input fields are `merge` (`state: "merged"`, `pr_url`, 40-character `commit`,
`merged_at`, `verification_source`), workspace-relative `capsule`, `capsule_sha256`,
`bundle_manifest`, `evidence_sufficiency_review`, and `files`. Each file row has
`path`, `disposition`, `reason` and `consumers` (an explicit list). Removal requires
no remaining consumers. Include kept artifacts' recheck events in their reasons.
The bundle manifest uses existing `files` entries with `path`, `sha256`, `bytes`.

This task changes requirements and previews only: **delete nothing yet**. Existing
artifacts stay intact. Later cleanup must respect that deferral until the owner
authorizes execution. Preserve historical bundles and availability notices; this
does not authorize a retrospective bulk migration, history rewrite or upload.

## Historical repository-hygiene findings

Audit date: 2026-10-20, after merged P1-06 (`8581f87`). The tracked working tree
contains approximately 76 MiB, including 62 MiB of logs. P1-06 contributes about
37 MiB across 1762 evidence files. Full LLVM disassemblies and repeated prerequisite
reports dominate. About 8.3 MiB of evidence bytes are exact duplicates; other
large reports differ only in paths or build context. Git already compresses and
deduplicates this strongly: the observed packed object database is about 5.1 MiB.
The main current costs are checkout size, file count and review noise, not a
76 MiB clone transfer. These figures describe this local repository snapshot.

## Retention policy

Keep generated runs, full disassembly/symbol dumps, tool output, repeated source
trees, binaries, ISOs, firmware variables and failed-development scratch under
ignored `work/` or `build/`. `.gitignore` now excludes logs, evidence source copies,
VM/build products and common caches by default. It does not untrack existing files.
Do not ignore all JSON, lockfiles or fixtures: many are required source inputs.

Promote a compact, sanitized acceptance bundle deliberately. Preserve the review,
case outcomes, exact source/tool/artifact hashes, launch arguments, selected raw
serial records, necessary failure traces and small relevant disassembly excerpts.
Record a source commit plus patch/hashes for dirty inputs; do not copy the entire
source tree into every run. Share one artifact audit per identical kernel build
and one prerequisite report per identical input set, with explicit references.
Do not discard a failure to produce a passing narrative, truncate raw records
silently, or replace measured context with an unsupported summary.

For new evidence bundles the default review budget is **4 MiB, 200 files, and
256 KiB per file**. These are repository maintenance limits, not OS performance
targets. If essential evidence does not fit, revise its representation or document
an explicit policy change before adding it; do not silently raise the limit.
Large local artifacts remain local; this policy grants no remote upload permission.

The owner's subsequent cleanup removes every previously tracked file matching
`.gitignore`: 2774 logs/source-copy files, about 63 MiB. Local copies and the earlier
Git revision are preserved; see the [availability notice](../../evidence/README.md).
Historical READMEs/checksum files remain original snapshots and can reference
paths absent from a fresh checkout. The baseline records this one explicit migration
and fingerprints the remaining metadata. Do not regenerate it to admit new bulk.
No physical file deletion or Git history rewrite is performed.

## Before committing

1. Run acceptance normally and retain full payload while review/resume needs it.
   Schema-2 story closeout also requires a reviewed, verified minimal local capsule.
2. Select the compact evidence and create its checksum manifest over the files
   actually being retained in Git. Use `git add -f -- <specific-file>` only for
   selected ignored diagnostics, after sanitization/review; never force-add a run tree.
3. Stage the intended files, then run:

   ```powershell
   node audit/check-repository-hygiene.cjs
   node audit/check-governance.cjs
   node audit/check-docs.cjs
   ```

The hygiene check reads the **Git index**, rejects generated output outside
evidence, enforces new-bundle budgets, protects historical metadata fingerprints
and requires each new frozen manifest member to be tracked. The declared historical
migration is the only exception; an ignored local log cannot satisfy a new bundle's
manifest. Governance separately reports local-only historical members and verifies
their bytes when present. The report goes to ignored `work/repository-hygiene-results.json`.
The auditor batches staged blob reads and checks each identical blob once per run;
every manifest reference still receives its checksum/length check. No historical
fingerprint, new-bundle limit or missing-member rejection is waived by batching.
The `.githooks/pre-commit` hook runs this check automatically where the repository's
`core.hooksPath=.githooks` is installed. It needs Node on PATH and fails a commit
when hygiene checks fail. Hooks are local safeguards, not server enforcement;
the separate pre-push handling hold is unchanged.

Deleting tracked logs or adding ignore rules would not remove their old Git
objects from history. Any future archive migration or history rewrite is a separate
task with a recovery plan and the existing handling restrictions still in force.
