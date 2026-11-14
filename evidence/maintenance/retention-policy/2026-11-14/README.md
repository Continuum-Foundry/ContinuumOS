# Retention policy and required post-merge cleanup

Passed local self-review for host retention tooling, 2026-11-14. No guest or
milestone acceptance is claimed. No deletion is performed or authorized now.

New story selections declare working/capsule budgets; the runner accounts for
referenced payload and binds both command streams plus execution metadata.
Explicit local capsules compress/deduplicate selected complete evidence and
verify its integrity without substituting for fresh artifact review or resume.
Every merged PR requires reviewed cleanup, including temporary ISOs after
verified evidence, with release/debug/review/resume exceptions and a receipt or
pending owner deferral. The offline cleanup planner deletes nothing and does
not independently authenticate merge declarations or discover active consumers.

[Checks](checks.json) retain 11 retention and 29 runner tests, documentation,
governance and original-index hygiene outcomes with complete output streams (CRLF normalized to LF, with both hashes recorded).
[Source identity](source-lock.json) matches the tested working implementation;
only the base commit advanced to P4-04's merge with an identical tree. Final
staging audits are separate host-only checks. No new guest test is needed.

The required source reconstruction and original journal remain in the verified
local capsule identified by [availability](local-availability.json). Exact source
patch/untracked bytes and earlier interruption records stay local. Capsule
integrity is distinct from acceptance review. Full payload and all pre-existing
artifacts remain intact. No automatic merge watcher, remote workflow or new
dependency is introduced. Shared-cache and transient peak storage are outside
the runner's checkpoint accounting; evidence sufficiency needs engineering review.

Reproduce with `python tools/story_validation.py plan --manifest
config/retention-policy-validation.json`, then the corresponding `run` and
`review` actions. Refer to the [retention policy](../../../../docs/05-delivery/evidence-retention.md)
for capsule and cleanup-preview inputs. Historical evidence is unchanged.
