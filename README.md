# ContinuumOS

An original desktop OS project centered on a distinctive, efficient interface and
state-preserving live development inside a running VM. ContinuumOS is the working
project-folder name. Current state: research/design only; no implemented OS or ISO.

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
work is [M0 feasibility](docs/05-delivery/milestones.md) with actual tools and evidence.

Keep docs/, AGENTS.md, CLAUDE.md and README.md together. Use the requested
Documents/ContinuumOS folder as the future coding-project root; the Codex research
workspace and downloadable archive are retained snapshots.
