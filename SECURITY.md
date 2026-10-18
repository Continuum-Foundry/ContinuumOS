# Security policy

## Supported state

There are **no supported production releases** and no security support window.
The repository currently demonstrates a minimal original kernel boot in a declared
QEMU profile. Isolation, secure updates, persistent storage, and production trust
boundaries have not passed their later gates. No security certification, response
SLA, bounty, or legal safe-harbor commitment is offered.

Reports about current source, host tooling, dependency acquisition, packaging,
credentials, or repository configuration are welcome. Record the commit and exact
input hashes so the issue can be evaluated against the affected state.

## Report privately

The [U.S.-only handling hold](EXPORT_CONTROL.md) takes priority: no report containing
project material may be sent through an unverified channel, including GitHub or
this AI session. The contact-request procedure below must contain no project
technical information and does not itself approve a reporting service.

Do not put exploit details, credentials, private code, or personal data in issues
or PRs. The repository is private as observed on 2026-10-18. GitHub private
vulnerability reporting was not available through the checked API; do not assume
the `/security/advisories/new` page is a working reporting channel.

Contact [@hgarg1](https://github.com/hgarg1), the verified administrator, through
an existing trusted private channel. If you do not have one, create an issue titled
**Private security contact requested**, with no technical details, and wait for
an agreed confidential channel. Do not send vulnerability information to an
unverified address or another person's account. Establishing and testing a durable
private contact is an outstanding readiness item in
[repository controls](docs/05-delivery/repository-controls.md).

Once the channel is established, provide the affected commit/artifact, prerequisites,
impact, minimal reproduction, and sanitized logs. Avoid accessing unrelated data,
running destructive tests, or affecting infrastructure you are not authorized to
test. Ask before sharing affected third-party confidential material.

## Handling and disclosure

The administrator should acknowledge receipt, triage severity and affected versions,
coordinate any upstream report, prepare a fix and regression evidence, and agree
on disclosure with the reporter where practical. Timeframes depend on capacity;
none are guaranteed. Do not promise confidentiality beyond the access and handling
controls actually available. Credit reporters with their consent.

Revoke and rotate exposed credentials before cleanup; deleting a log or commit
does not invalidate a secret. Check artifacts and caches for copies. Preserve a
restricted incident record containing scope, response, validation, and follow-up
without placing secrets in this repository.

## Development boundaries

Use the declared offline VM profile without physical disks, host shares, exposed
management ports, or a general guest-supplied host command channel. Downloaded
tools and Cargo build scripts execute with host authority. Hash pinning detects
input changes; it does not prove publisher identity or that code is trustworthy.
The [supply-chain policy](SUPPLY_CHAIN.md) describes the review requirements.
