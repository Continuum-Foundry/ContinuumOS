# U.S.-only handling and export hold

Effective 2026-10-18 by the project owner's instruction: **project material must
not leave the United States under any circumstances.** This restriction is in
addition to the reservation of rights in [LICENSE](LICENSE). It is an internal
handling requirement, not a government export classification or an export license.

## Immediate hold

### One-time owner authorization — 2026-10-18

After being told that GitHub handling and geographic containment were unverified,
the owner instructed "push still". This authorizes one push of the pending
licensing, governance, and handling-policy changes to the existing GitHub
`origin/main` despite that uncertainty. It is an exception to the administrative
transfer hold, not a finding that the service satisfies U.S.-only handling or
that any legally required authorization exists. It does not waive applicable law.

The authorization is exhausted by the first successful push containing this
record. It does not approve later uploads, other recipients, or other services.
The local push guard may be bypassed for that invocation only and must remain
installed afterward. No standing remote environment approval is created.

### Additional one-time owner authorization — P1-03, 2026-10-18

After being told that the U.S.-only hold blocks new GitHub uploads and the earlier
exception is exhausted, the owner instructed: "override, do the push, another
exemption". This authorizes one push of the completed P1-03 implementation,
sanitized retained evidence and accompanying documentation to the existing
GitHub `origin/codex/p1-03-boot-metadata-validation`, followed by creation of its
pull request against `main`, including the prepared PR text and metadata.

The push authorization is exhausted by the first successful push containing this
record; the PR authorization is exhausted by creation of that one pull request.
The local push guard may be bypassed for that single push invocation and must
remain installed afterward. No other upload, merge, release, remote build or
standing remote approval is authorized. Geographic containment remains unverified;
this administrative exception is not legal clearance or a foreign-transfer
authorization and does not waive applicable law.

### Two scoped owner authorizations — M1 module refactor and README upkeep, 2026-10-18

The owner instructed: "push directly to main (i give you an exemption to do so)
and then update all of the repo ReadMEs as needed then push directly to main
again (i give you an exemption to do so again)". This authorizes two successive
direct pushes to the existing GitHub `origin/main`: first the completed M1 host
module refactor, sanitized verification evidence, accompanying runbook and this
authorization record; then the needed repository README updates.

These are exceptions to the administrative transfer hold and ordinary topic-branch
review procedure. Review is local agent self-review, not independent approval.
Each authorization is exhausted by its corresponding first successful push. The
local push guard may be bypassed for those two push invocations only and must
remain installed afterward. No force push, release, remote build, new recipient,
or standing remote-environment approval is authorized. Geographic containment
remains unverified; these administrative exceptions are not legal clearance and
do not waive applicable law.

### One-time owner authorization — CLAUDE.md upkeep, 2026-10-18

The owner instructed "commit and push to main". After being told the hold blocks
pushes and earlier exemptions are exhausted, the owner chose "Grant exemption,
push". This authorizes one direct push to the existing GitHub `origin/main` of the
expanded `CLAUDE.md` operational notes and this authorization record.

The authorization is exhausted by the first successful push containing this
record. The local push guard may be bypassed for that single push invocation only
and must remain installed afterward. No force push, release, remote build, new
recipient, or standing remote-environment approval is authorized. Geographic
containment remains unverified; this administrative exception is not legal
clearance and does not waive applicable law.

### One-time owner authorization — P1-04, 2026-10-19

After being told that the handling hold blocks new uploads, earlier exemptions
are exhausted, and GitHub's U.S.-only handling remains unverified, the owner
answered "yes" to a one-time exception to push the P1-04 branch and open its PR
against `main`. This authorizes one push of the completed P1-04 implementation,
sanitized retained evidence, accompanying documentation and this authorization
record to the existing GitHub `origin/codex/p1-04-exception-diagnostics`, followed
by creation of its pull request against `main`, including the prepared PR text
and metadata.

The push authorization is exhausted by the first successful push containing this
record; the PR authorization is exhausted by creation of that one pull request.
The local push guard may be bypassed for that single push invocation only and
must remain installed afterward. No merge, force push, release, remote build,
new recipient or standing remote-environment approval is authorized. Geographic
containment remains unverified; this administrative exception is not legal
clearance or a finding of U.S.-only handling and does not waive applicable law.

No new push, upload, publication, external sharing, remote build, cloud dependency
analysis, or transmission of project material is approved. No hosting, processing,
support, backup, AI, or remote-access service is approved for project material
until its full handling has been verified to satisfy this policy, except for the
specific owner-authorized transfers recorded above. The current
local device, storage, AI session, GitHub account, and existing copies have **not**
been verified as meeting that requirement. Do not infer location from a timezone,
IP address, billing address, repository visibility, vendor nationality, or a U.S.
region label. Keep further handling within a separately verified U.S. environment.

Prior GitHub pushes occurred before this instruction. Their storage, processing,
support access, and backups have not been geographically certified. This change
does not recall those copies or prove past containment. An administrator must
review existing copies and service access through an approved channel; do not
delete evidence, revoke accounts, or migrate data without a documented plan.

## Material and handling covered

The restriction covers original source, technical designs, unpublished documents,
history, binaries, ISOs, manifests, logs, screenshots, issue/PR content, build and
dependency information, credentials, and backups under the project's control.
It applies to copies, excerpts, derived artifacts, and processing, not just the
canonical Git repository. Keep project material and project-specific metadata
out of unapproved cloud sync, email, collaboration, telemetry, indexing, AI,
support tickets, hosted runners, and external scanners.

Do not access, carry, send, replicate, or remotely display project material
outside the United States. No non-U.S. recipient or unapproved support operator
may access it remotely. A VPN endpoint or encryption does not waive the project's
absolute handling instruction. Public third-party components retain their own
licenses; this policy does not strip upstream rights, but it governs the project's
confidential bundle and its handling.

## Verification before any remote handling

The administrator must retain an approved record outside unverified services
covering **all** storage, compute, network paths, logging, backups, disaster
recovery, subprocessors, support/admin access, identities, and physical locations.
Use enforceable access controls, restricted egress, audit logs, managed endpoints,
and tested backup/recovery in the approved environment. A provider's marketing
statement or contractual region choice alone is not proof of end-to-end containment.
If the requirement cannot be verified, do not use that service for project material.
No foreign-transfer exception is authorized under the present instruction.

The source-controlled [pre-push hook](.githooks/pre-push) denies all pushes while
no remote environment is approved. It supplies a local guardrail only: Git hooks
can be bypassed and do not stop other tools, existing clones, services, backups,
or network traffic. They do not implement geographic enforcement. A fresh clone
does not automatically install a hooks path. Do not bypass the hold with
`--no-verify`, alternate clients, archives, or another transport, except for the
specific invocations expressly authorized above.

Repository Cargo configuration defaults to offline mode as a second local
guardrail; it does not restrict arbitrary build scripts, other tools, or network
traffic. Do not override it while the transfer hold is active.

New GitHub-hosted CI and Dependabot configurations have been withheld; checks
remain local. No replacement runner or remote transport is approved. This work
does not change firewall, VPN, OS settings, remote visibility, or provider contracts.

## Legal classification remains pending

[The classification record](legal/export-classification.json) intentionally has
no ECCN, EAR99 finding, ITAR/USML determination, license, or exception. Software
does not acquire a government-controlled classification because a file labels
it "export controlled". Qualified export counsel must determine jurisdiction,
classification, encryption implications, access eligibility, applicable reporting,
and any required authorizations from the actual technical facts. Do not claim
ITAR registration, an EAR exception, government approval, or clearance that was
not obtained. A lawful export authorization would not override the owner's
stricter no-foreign-transfer instruction.

Physical U.S. presence alone may not resolve access restrictions: releases of
certain controlled technology or source code to foreign persons can be deemed
exports. Applicable definitions and exceptions require counsel's assessment;
do not collect immigration/citizenship documents in this repository or invent
a blanket legal rule for every software component.

## Sources and outstanding controls

[BIS deemed-export guidance](https://www.bis.gov/learn-support/deemed-exports/what-deemed-export),
[EAR Part 734](https://www.bis.gov/regulations/ear/734),
[BIS encryption guidance](https://www.bis.gov/learn-support/encryption-controls/encryption-items-not-subject-to-ear).
GitHub documents that data-residency offerings may still involve certain
out-of-region storage/transfers; neither a private repository nor a region choice
proves this project's absolute constraint:
[GitHub storage and transfers](https://docs.github.com/en/enterprise-cloud%40latest/admin/data-residency/about-storage-of-your-data-with-data-residency).

The hold, infrastructure verification, and legal review are open blockers in
[repository controls](docs/05-delivery/repository-controls.md) and
[release gates](RELEASE.md). No compliance or geographic-containment guarantee
is made by these files.
