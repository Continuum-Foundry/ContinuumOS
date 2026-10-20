# Repository hygiene and compact evidence

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

Existing accepted evidence is preserved byte for byte. Its Git-index fingerprints
are recorded in `audit/repository-hygiene-baseline.json`; that baseline permits
the historical size without allowing new files to accumulate inside old bundles.
Changing it needs an intentional archival/retention decision, not regeneration
on every check. No Git history rewrite or evidence deletion is part of this change.

## Before committing

1. Run acceptance normally and retain complete raw artifacts locally.
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
evidence, enforces new-bundle budgets, protects historical bundle fingerprints
and requires each frozen manifest member to be tracked. A local ignored log cannot
satisfy that last check. Governance separately validates retained bytes against
checksums. The report goes to ignored `work/repository-hygiene-results.json`.
The `.githooks/pre-commit` hook runs this check automatically where the repository's
`core.hooksPath=.githooks` is installed. It needs Node on PATH and fails a commit
when hygiene checks fail. Hooks are local safeguards, not server enforcement;
the separate pre-push handling hold is unchanged.

Deleting tracked logs or adding ignore rules would not remove their old Git
objects from history. Any future archive migration or history rewrite is a separate
task with a recovery plan and the existing handling restrictions still in force.
