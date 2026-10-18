# Support and issue routing

ContinuumOS is experimental and has no supported production version, commercial
support contract, service-level agreement, or guaranteed response time.

The [U.S.-only handling hold](EXPORT_CONTROL.md) supersedes the routes below:
do not upload reports or attachments containing project material until the
complete reporting environment is approved. Keep review records local meanwhile.

| Request | Route |
| --- | --- |
| Reproducible source/build defect | Bug-report issue template, with commit, command, hashes, and sanitized output |
| Design change | Proposal issue template; identify the requirement, gate, and alternatives |
| Vulnerability or leaked secret | [SECURITY.md](SECURITY.md); no details in an issue |
| Licensing or commercial permission | [LEGAL.md](LEGAL.md); request a private contact |
| Behavior or harassment concern | [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) |

Read the [current handoff](docs/05-delivery/agent-handoff.md) before reporting a
missing planned feature as a regression. A successful boot does not imply an
interactive desktop or readiness for user data. Use disposable test images and
the explicit platform profile. Do not attach a full environment dump, crash dump,
or disk image without review and an agreed private transfer channel.
