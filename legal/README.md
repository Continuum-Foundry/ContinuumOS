# License inventory and notice provenance

Original project material is all rights reserved under [LICENSE](../LICENSE).
The [U.S.-only handling hold](../EXPORT_CONTROL.md) blocks remote transfers and
processing; [export-classification.json](export-classification.json) records
pending legal classification and the lack of approved remote environments.
The owner's scoped push exceptions are recorded in that policy; they authorize
only their named transfers and do not approve a remote environment, release or
standing exemption. The default push guard remains installed.
Upstream notice copies under `licenses/third-party/` retain their own terms and
original bytes; no first-party reservation overrides them.

- [dependency-inventory.json](dependency-inventory.json) resolves the three current
  Cargo graphs for the declared bare-metal target. It includes host tooling and
  candidate experiments; **it is not the shipped guest's SBOM**. Package license
  expressions are upstream declarations, not legal compatibility determinations.
  Source URLs and lock checksums identify inputs, not complete copyright ownership.
- [notice-provenance.json](notice-provenance.json) identifies source artifacts and
  hashes of the verbatim Limine loader/protocol, Rust binding, and bitflags notices.
  These are a starting set for the guest boot path, not a complete toolchain,
  firmware, host tooling, or release notice bundle.

Regenerate the inventory with `pwsh -NoProfile -File tools/Inventory-Licenses.ps1`
after entering the local development environment. It runs Cargo offline and
locked; missing inputs must be supplied explicitly. Review changes and repeat the
governance audit. Do not rewrite historical evidence to make its source hashes
match a later policy change. Existing build identities refer to their recorded inputs.

[ui-selection.json](ui-selection.json) records P2-09's provisional custom UI
configuration, complete active target package/license choices and exact notices,
font/fixture/source pins and frozen Slint prototype. The
[decision review](../docs/05-delivery/phase-2/ui-selection-review.md) explains scoped
OS/app/SDK suitability and delivery obligations. This complements the broader
candidate inventory; `distribution_cleared` remains false. Neither this choice
nor preserved comparison binaries authorize publication or resolve Slint terms.

First-party Cargo packages use `license-file`, not a permissive SPDX identifier,
and remain non-publishable. This records the actual reservation instead of
mislabeling proprietary code as MIT or Apache. Actual artifact notice packaging,
source obligations, and publication permission remain [release gates](../RELEASE.md).
