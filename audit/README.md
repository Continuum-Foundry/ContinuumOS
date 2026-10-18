# Documentation validation

Run from the extracted package root (the directory containing docs/):

    node audit/check-docs.cjs
    node audit/check-governance.cjs
    pwsh -NoProfile -File audit/validate-contracts.ps1

The schema checks require PowerShell with Test-Json support for JSON Schema 2020-12;
the recorded run used PowerShell 7.6.5. Neither command tests an operating system.
Both write fresh results under work/. No npm packages are required.

The governance check validates policy/link coverage, the remote-transfer hold,
Cargo license-file declarations, inventory identities/checksums against current lockfiles,
and frozen evidence checksum indexes. These are repository consistency checks,
not legal clearance, a vulnerability scan, or enforcement of GitHub settings.
Checks remain local under EXPORT_CONTROL.md. Hosted workflows and Dependabot
are withheld. Passing audits do not prove the host, AI, backups, or existing
GitHub copies are inside the United States. Do not upload their reports to an
unverified service.

The supplied results and root SHA256SUMS.json describe the original authored
research snapshot, not the current implementation tree. New Phase 0 results and
their own checksum index are under evidence/phase-0/. Re-run the commands above
for current documentation checks.
