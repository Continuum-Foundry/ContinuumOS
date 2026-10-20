# P1-05 publication preparation — 2026-10-19

The owner requested the PR and approved a one-time exemption, recorded in
[EXPORT_CONTROL.md](../../../../EXPORT_CONTROL.md). This preparation does not
add a new guest result or claim independent approval.

The pre-push governance check found that the P1-05 checksum manifest used the
algorithm label `SHA256`, while the repository contract requires `SHA-256`.
All 1,298 listed file digests matched. Only the label in the active
[manifest](../2026-10-19/SHA256SUMS.json) was corrected; its file list and hashes,
every retained payload byte, and all 34 build-source inputs remain unchanged.
The checksum manifest excludes itself, so no payload digest needed regeneration.

The [original manifest](original-SHA256SUMS.json) and
[failed check](governance-label-failure.json) are retained here. The
[passing check](governance-passed.json) verifies all 2,501 frozen payload files
across the existing evidence bundles, including P1-05's 1,298. The original
freeze-procedure snapshot remains a historical record of the procedure used;
replaying it requires the same canonical-label correction afterward.

Documentation/link and whitespace checks also passed. The kernel/ISO acceptance
remains the [existing local self-review](../2026-10-19/README.md): 23 host tests,
27 selected fresh ISO launches and the unchanged exact build/artifact identities.
The unrelated `.gitignore` edit is excluded from the implementation commit.
