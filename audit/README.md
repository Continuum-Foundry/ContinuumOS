# Documentation validation

Run from the extracted package root (the directory containing docs/):

    node audit/check-docs.cjs
    pwsh -NoProfile -File audit/validate-contracts.ps1

The schema checks require PowerShell with Test-Json support for JSON Schema 2020-12;
the recorded run used PowerShell 7.6.5. Neither command tests an operating system.
Both write fresh results under work/. No npm packages are required.

The supplied results describe the authored snapshot. SHA256SUMS.json lists the
packaged files, excluding itself, so copied content can be checked independently.
