# Privacy and diagnostic handling

This document covers repository collaboration and the current development tools.
It is not a privacy notice for a deployed service or finished operating system;
no such product is provided by this repository today.

[EXPORT_CONTROL.md](EXPORT_CONTROL.md) prohibits new project-material uploads,
remote processing, or sharing through any unverified environment. Existing GitHub
storage and this AI session have not been certified U.S.-only. The descriptions
below identify existing data flows, not approval to continue them.

## Current data flow

GitHub hosts repository content, commit identity, issues, PRs, workflow metadata,
and any uploaded artifacts under the account's actual access controls and
[GitHub's privacy statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement).
GitHub and any explicitly used development or AI service have their own terms
and data handling. Repository policy cannot promise how those services behave.

Local tooling writes build manifests, source identities, process/command records,
serial logs, and prerequisite diagnostics under `build/` and `work/`. Inventory
may inspect available tools, Windows platform/SDK, disk capacity, and CPU/
virtualization information. Builds and explicit supply commands may contact
upstream package/tool hosts. Local logs are not automatically a reviewed export.

Retained evidence under `evidence/` is committed and shared with anyone granted
repository access; a later public release would expose it publicly. Public README
badge images also request content from an external badge provider when rendered.
The current README no longer embeds those external images under the handling hold.
Do not treat a private repository or a local log directory as a secret vault.

## Minimize and review

- Include only diagnostic data needed to reproduce or validate a result.
- Remove tokens, passwords, private keys, personal identifiers, customer content,
  user documents, machine names, and unrelated absolute paths before export.
- Inspect filenames, commands, environment fragments, JSON, serial bytes, screenshots,
  and archives. Automatic path substitution alone is not a privacy review.
- Keep credentials outside the repository and output archives. Rotate any exposed
  secret and use [SECURITY.md](SECURITY.md) for incident handling.
- Do not upload confidential project or third-party data to another service without
  authorization. Obtain permission before publishing someone's name or report.

The administrator should review diagnostic retention at each milestone and remove
unnecessary live copies. Published Git commits, forks, caches, and external backups
may retain copies; complete deletion or a specific retention period is not promised.
Privacy concerns use a private administrator contact requested as described in
[GOVERNANCE.md](GOVERNANCE.md). A deployed product needs its own reviewed privacy
notice, contact, retention schedule, and applicable data-rights process.
