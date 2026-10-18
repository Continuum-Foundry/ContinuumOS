# Project governance

ContinuumOS is an early, privately administered research and OS implementation
project. These procedures govern repository work; they are not corporate bylaws,
employment terms, or evidence that a company has been formed.

[U.S.-only handling](EXPORT_CONTROL.md) is now mandatory. The remote-transfer hold
supersedes ordinary GitHub collaboration procedures until the full environment
is verified. An earlier push authorization does not waive this newer constraint.

## Responsibility

The verified repository administrator as of 2026-10-18 is
[@hgarg1](https://github.com/hgarg1). The `Continuum-Foundry` organization is the
repository namespace. Administration does not establish ownership of every
copyright or authority to license someone else's work.

The administrator controls access, appoints maintainers, records rights decisions,
and authorizes releases. Maintainers review technical changes within their assigned
scope. Agents may implement and validate authorized work, but are not independent
reviewers or authorized legal signers. No committee, board, security team, or
service organization is implied.

## Decisions and review

- Core intent and the approved revision-2 plan remain the project baseline.
- Architectural decisions remain Proposed until their experiments and review gates
  pass. Evidence, proposals, and measured results must remain distinguishable.
- Changes to rights, licensing, publication, access, release signing, or security
  policy need administrator review; rights changes also need the relevant holder's
  permission. A code-owner review does not replace legal authorization.
- Prepare local review records during the transfer hold; use PRs only after the
  transport and service are verified under the handling policy. Record validation
  and unresolved risks. A sole
  maintainer cannot supply independent self-approval; record that limitation and
  any administrator exception rather than manufacturing a second reviewer.
- Disagreements should identify the requirement, evidence, alternatives, and
  decision. Record the resolution in the issue, PR, or ADR.

Use [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for behavior issues and
[SECURITY.md](SECURITY.md) for vulnerabilities. Decisions concerning an affected
maintainer should be reviewed by another authorized administrator when available;
there is no independent appeals body currently staffed.

## Authority and enforcement

[CODEOWNERS](.github/CODEOWNERS) routes reviews. Enforced approvals, branch
protection, disabled remote automation, organization MFA, and access reviews require actual
GitHub configuration. A policy file cannot enable them. See
[the administrator checklist](docs/05-delivery/repository-controls.md) for observed
state and remaining controls. No SLA, bounty, or commercial support is promised.
