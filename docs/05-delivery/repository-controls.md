# Repository controls and administrator checklist

Observed 2026-10-18 through authenticated read-only GitHub API calls. These are
repository-control findings, not guest security results. No remote setting was changed.

**U.S.-only handling hold:** [EXPORT_CONTROL.md](../../EXPORT_CONTROL.md) now
prohibits new uploads, pushes, sharing, and remote processing until all handling
paths are verified. The observations below predate that instruction and do not
prove containment. No further remote inspection is needed to add these local
policies. Legal classification and existing-copy review remain pending.

The owner subsequently authorized one GitHub push of the pending policy changes
despite disclosed unverified containment. [EXPORT_CONTROL.md](../../EXPORT_CONTROL.md)
records its scope and exhaustion rule. This is not remote-environment approval,
geographic verification, or legal export clearance; the local guard remains installed.

| Control | Observed state |
| --- | --- |
| Repository visibility | Private; keep private pending [publication review](../../LEGAL.md) |
| Default branch | `main` |
| Verified collaborator | `hgarg1`, administrator |
| Issues | Enabled |
| Private vulnerability reporting API | HTTP 404; no working private reporting endpoint verified |
| Rulesets API | HTTP 403; enforcement could not be inspected; plan/permission restrictions may apply |
| Security/analysis settings | Not exposed in the returned repository response; enablement unverified |
| CODEOWNERS | Supplied locally; remote activation/required-review enforcement not verified |
| New hosted workflow and Dependabot | Withheld under the handling hold; no new remote automation is approved |
| Current GitHub/AI/device/backups | U.S.-only containment not verified; existing copies need administrator review |
| Local push guard | `core.hooksPath=.githooks` installed in this checkout; simulated hook invocation rejected without remote contact. Not a network/DLP control; fresh clones require installation |

## Enable and verify in GitHub

The administrator should record the date, actor, actual setting, and a successful
test for each control. GitHub availability depends on the account/repository plan;
do not claim a control exists based on a file or an unavailable API.

- [ ] Require organization MFA, minimize write/admin roles, remove unused tokens,
  and establish a tested recovery method. Do not place recovery codes in Git.
- [ ] Verify the physical device, network, storage, backup, AI, hosting, support,
  identity, and all subprocessors/transfers before any project remote handling.
  Review existing GitHub and other service copies without claiming they can
  automatically be recalled. Obtain legal export-classification/access review.
- [x] Install and test the local push-deny hook in this checkout without remote contact.
- [ ] Independently enforce egress and device/data access in the approved environment;
  the local hook is bypassable and does not provide geographic containment.
- [ ] After the holding environment is verified, protect `main` from deletion and
  force push and require recorded review. Do not introduce cloud status checks
  until their full processing path is approved.
- [ ] Enable CODEOWNERS review where supported. Add an independent qualified
  reviewer before requiring an approval that a sole maintainer cannot supply.
  Review administrator bypasses explicitly and record any exception.
- [ ] Protect release tags and publishing environments, establish signing-key
  custody, and require approval before publishing artifacts. No publishing
  credentials belong in the documentation-check workflow.
- [ ] Review and suspend any existing unapproved remote automation through an
  approved administration channel. No new cloud scans, dependency services, runners,
  or data uploads may be introduced merely to satisfy a repository checklist.
- [ ] Establish and test a private vulnerability/conduct contact; update SECURITY.md
  and CODE_OF_CONDUCT.md only after the channel works. Do not publish a guessed email.
- [ ] Review issue/artifact/log access and retention; verify backups and restore
  procedures without exposing secrets or changing immutable evidence.
- [ ] Only after remote handling approval, test permitted collaboration and controls
  without foreign transfers. Until then run the local audits only.

These items remain open until verified. This change does not enable a paid plan,
create a team, change visibility, invite collaborators, or enforce branch rules.

## Reference instructions

[Code owners](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners),
[private vulnerability reporting](https://docs.github.com/en/code-security/how-tos/report-and-fix-vulnerabilities/configure-vulnerability-reporting),
[secure Actions usage](https://docs.github.com/en/actions/reference/security/secure-use),
[rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/about-rulesets).
