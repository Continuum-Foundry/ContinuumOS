# Documentation validation

Run from the repository root (the directory containing `docs/`):

    node audit/check-docs.cjs
    node audit/check-governance.cjs
    pwsh -NoProfile -File audit/validate-contracts.ps1

The schema checks require PowerShell with Test-Json support for JSON Schema 2020-12;
the recorded run used PowerShell 7.6.5. These three commands check documentation,
governance consistency and synthetic schema fixtures. They write fresh results
under `work/` and do not test an operating system. No npm packages are required.

The governance check validates policy/link coverage, the remote-transfer hold,
Cargo license-file declarations, inventory identities/checksums against current lockfiles,
and frozen evidence checksum indexes. These are repository consistency checks,
not legal clearance, a vulnerability scan, or enforcement of GitHub settings.
Checks remain local under [EXPORT_CONTROL.md](../EXPORT_CONTROL.md). Hosted workflows and Dependabot
are withheld. Passing audits do not prove the host, AI, backups, or existing
GitHub copies are inside the United States. Do not upload their reports to an
unverified service.

The supplied results and root SHA256SUMS.json describe the original authored
research snapshot, not the current implementation tree. Phase 0 and Phase 1
results retain their own evidence records. Re-run the commands above for current
documentation checks.

Host and guest behavior checks are separate. The
[current P1-03 acceptance harness](../tools/Test-P1-03.ps1) runs formatting,
Phase 0 checks, dependency probes, kernel tests, prerequisites and the named ISO
cases. The [module-refactor review](../evidence/phase-1/m1-module-refactor/2026-10-18/README.md)
retains its latest regression results and an `evidence-index.json` with raw and
retained hashes. The governance audit checks `SHA256SUMS.json` evidence indexes;
that separate retention index must also be verified when handling its artifacts.
Historical evidence and its READMEs remain snapshots of their recorded runs.
