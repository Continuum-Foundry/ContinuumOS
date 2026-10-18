# ContinuumOS

An original desktop OS project centered on a distinctive, efficient interface and
state-preserving live development inside a running VM. ContinuumOS is the working
project-folder name. Current state: Phase 0 Windows tooling and freestanding
Rust link probes; no boot-tested kernel or ISO.

See the [Phase 0 runbook](docs/05-delivery/phase-0.md) and
[recorded evidence](evidence/phase-0/README.md) for setup, checks and remaining blockers.

Start with [the reconciled plan](docs/00-overview/reconciled-plan.md),
[the documentation index](docs/README.md), and
[the coding-agent handoff](docs/05-delivery/agent-handoff.md).

- Original Rust kernel and isolated services; software graphics first.
- Compare Slint and a minimal custom UI before choosing the toolkit.
- Prove UI/behavior updates while unsaved editor text survives.
- Shared typed actions; optional inference after the live-reload baseline.
- Tight memory targets; independent live updating and boot recovery.
- Build and cold-boot the actual standalone ISO.

Read [AGENTS.md](AGENTS.md) or [CLAUDE.md](CLAUDE.md) before coding. The immediate
work after rechecking Phase 0 is the [M1 ISO boot gate](docs/05-delivery/milestones.md).

Keep docs/, AGENTS.md, CLAUDE.md and README.md together. Use the requested
Documents/ContinuumOS folder as the future coding-project root; the Codex research
workspace and downloadable archive are retained snapshots.
