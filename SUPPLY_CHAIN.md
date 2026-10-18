# Supply-chain and dependency policy

Inputs must have an exact identity, a known purpose, and retained provenance.
An upstream name, `latest` URL, or SPDX expression alone is not a distribution
approval. [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) distinguishes host tools,
guest inputs, and experiments.

## Admission and updates

Record the source URL, exact version/commit, downloaded digest, upstream digest
or signature verification actually performed, license terms/notices, and selected
features. Explain target support, host build scripts, unsafe/native code, parser
exposure, and required resources before adding privileged guest dependencies.
Keep all Cargo locks, including isolated experiments, under version control.

Use locked inputs for builds. Acquisition is a separate explicit operation;
never hide downloads, installers, global changes, or new host authority behind
doctor. Missing inputs and hash mismatches must stop packaging/launch. Preserve
acquisition failures. Publisher authentication and local hash equality are
different findings and must be reported separately.

License alternatives require an explicit selection for each shipped component.
Do not infer that proprietary first-party code permits combining every upstream
dependency. Slint and other candidate integrations remain experiments until
technical and distribution-license reviews pass. New fonts, images, model weights,
datasets, and generated material need the same provenance review as code.

## Automation and execution

New hosted CI and Dependabot configurations are withheld under the
[U.S.-only handling hold](EXPORT_CONTROL.md). No cloud runner, dependency service,
scanner, AI system, or artifact service is approved. Do not add remote automation
until storage, compute, logs, access, backups, subprocessors, and transfers are
verified against that policy. A self-hosted runner label is not geographic proof;
its control plane and all handling paths need review too.

Keep checks local in an approved environment. Review any acquisition command's
outbound metadata before use; package names and build context can reveal project
information. A local command may still make network requests unless explicitly
offline. Review changed locks, licenses, and features and run relevant compile/
boot/protocol checks before accepting a native dependency or packaging change.

Repository Cargo configuration defaults to offline mode during the hold. This
prevents Cargo's normal dependency/index requests when honored; it is not a
firewall, and build scripts, rustup, acquisition scripts, alternate clients,
overrides, and other software can still contact external services. Do not bypass
offline mode or execute acquisition commands without the handling review.

## Inventory and release

```powershell
. ./tools/Enter-Dev.ps1
pwsh -NoProfile -File tools/Inventory-Licenses.ps1
```

This offline command produces `legal/dependency-inventory.json` from the three
locked Cargo graphs. It records registry declarations and checksums, not a legal
opinion, complete copyright inventory, vulnerability scan, or artifact-level SBOM.
Host firmware/tools and their companion DLLs are separately pinned in
`config/m1-inputs.lock.json`. Re-run and review the inventory after dependency
changes; retain the original evidence snapshots unchanged.

Before distribution, inventory **actual shipped files**, produce an SPDX or
CycloneDX SBOM, preserve full licenses/attributions and required source offers or
corresponding source, and verify packaging includes them. No package is cleared
merely because its registry license field is present. [RELEASE.md](RELEASE.md)
keeps engineering boots distinct from approved releases.
