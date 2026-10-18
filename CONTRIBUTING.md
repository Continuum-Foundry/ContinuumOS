# Working on ContinuumOS

Original project material is [all rights reserved](LICENSE). **External patches
are not currently accepted.** Report defects or discuss proposals without
submitting code, copied text, or artwork you are not authorized to share. Opening
a PR does not establish a license, assignment, or contributor agreement.

Under [EXPORT_CONTROL.md](EXPORT_CONTROL.md), new pushes, issues containing project
material, PR uploads, sharing, and remote processing are on hold. Workflows below
describe local procedures only until the environment is geographically verified.

## Authorized implementation work

Repository access is not itself a rights grant. Before incorporating another
person's work, the administrator must verify the appropriate written permissions.
Once authorized, read [AGENTS.md](AGENTS.md), the
[handoff](docs/05-delivery/agent-handoff.md), and the ordered story's acceptance
criteria. Keep one milestone gate in scope and record evidence for behavior claims.

Use a local topic branch; do not push or open a PR during the handling hold.
Prepare a review record stating the problem, resulting behavior,
validation, limitations, and requirement/story references. Update contracts,
specifications, and proposed ADRs together when a boundary changes. Kernel changes
need an explanation of unsafe/assembly assumptions and independent artifact checks.

## Validation

From PowerShell 7 at the repository root:

```powershell
node audit/check-docs.cjs
node audit/check-governance.cjs
pwsh -NoProfile -File audit/validate-contracts.ps1

# For source/build changes, after explicit local tool setup:
. ./tools/Enter-Dev.ps1
cargo fmt --all -- --check
pwsh -NoProfile -File tools/Test-M0.ps1
```

For packaging or boot changes, follow the
[M1 runbook](docs/05-delivery/phase-1/minimal-iso-runbook.md) and run the relevant
P1-01/P1-02 acceptance harness. A passing documentation workflow is not a VM boot.
Use the required execution context for MSYS2; do not change Windows features or
install global packages to bypass a failure.

Keep downloaded tools, binaries, and scratch output in ignored `work/` and `build/`.
Retain compact sanitized evidence and failed runs without rewriting old snapshots.
Never submit credentials, customer data, raw environment dumps, or unrelated
personal paths. Report vulnerabilities through [SECURITY.md](SECURITY.md).

## Provenance and AI-assisted work

Record copied/adapted material's origin, version, exact changes, license, and
permission. Preserve upstream notices. AI-assisted submissions need the same
human review, provenance checks, rights review, and validation as other work.
Do not claim human review or rights clearance that did not occur. Do not upload
confidential project material to an external service without authorization.

License, ownership, contribution, and publication decisions follow
[LEGAL.md](LEGAL.md) and [GOVERNANCE.md](GOVERNANCE.md). There is no CLA or DCO
requirement purporting to resolve rights that have not been agreed in writing.
