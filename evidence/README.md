# Evidence retained in Git

Future stories follow the [2026-11-14 retention revision](../docs/05-delivery/evidence-retention.md):
compact committed evidence plus a verified local capsule, with temporary validation
payload retained only while needed by active consumers. Capsule integrity does not
revalidate acceptance or replace missing original artifacts for resume. This policy
revision deletes nothing and does not migrate the historical bundles below.

On 2026-10-20 the owner requested removal of files matching `.gitignore` from
the repository. **2774 historical log/source-copy files (about 63 MiB) are now
untracked.** Their local copies were preserved byte for byte; they are also present
in Git revision `7e4df4212378bd2ae35efa85386bbf301b0483b4` and its history.
This removal does not purge old Git objects or shrink existing history.

The current checkout retains the reviews, outcomes, manifests, artifact hashes,
launch configuration and other nonignored metadata. Historical READMEs and frozen
checksum/index files remain original snapshots: their links to `.log` files and
`source/` trees describe the original retained bundle, and those paths are no
longer provided by a fresh checkout. Do not claim fresh raw-evidence verification
from a passing metadata check. Restore from the recorded revision into ignored
local storage when the original bytes are needed; do not re-add the full run tree.

`audit/repository-hygiene-baseline.json` records the authorized removal, its count,
byte total and path/blob fingerprint, and protects the remaining tracked metadata.
The governance audit reports local-only members separately and verifies their
checksums only when a preserved local copy is available. New bundles still must
include every member of their committed checksum manifest and fit the
[compact-evidence policy](../docs/05-delivery/evidence-retention.md).
