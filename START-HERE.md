# Original desktop OS — research handoff

Revision 2, reconciled 2026-10-17. Start with [the reconciled plan](docs/00-overview/reconciled-plan.md),
[the documentation index](docs/README.md), then
[the agent handoff](docs/05-delivery/agent-handoff.md) and
[the implementation milestones](docs/05-delivery/milestones.md).

This package contains **67 files in docs/**: 63 Markdown documents,
two JSON schemas, and two synthetic examples. The library is approximately
35 thousand words, with **83 primary-source catalogue entries, 16 proposed ADRs,
19 tracked requirements, and a 40-task implementation backlog**.

The proposed foundation is an original Rust kernel with isolated services,
versioned live-update boundaries, a shared host/guest UI engine, software rendering
before GPU acceleration, and immutable system generations separate from user data.
Supported UI/app/service changes stay inside the running VM; foundational kernel
changes require reboot initially. ISO packaging is separate from live updates.

Compare Slint/custom UI before selection. Typed actions are core; inference is optional
after state-preserving reload. Keep tight cold-idle targets and separate boot recovery.
Future coding-project root: Documents/ContinuumOS, as requested by the user.

The research covers languages, prior art, layering, boot, kernel/memory/scheduling,
IPC/capabilities, drivers, graphics, UI, apps/Wasm, storage, networking, security,
accessibility/text, power/audio/lifecycle, performance, packaging and recovery.
Every architectural default is distinguished from verified upstream findings and
future implementation evidence.

**No OS implementation or ISO was built in this research task.** The next coding
step is the M0 feasibility/toolchain gate. Root AGENTS.md and CLAUDE.md route future
agents to the same contracts, decisions and acceptance tests.

Validation passed: 281 local links/anchors, all JSON syntax,
requirement/ADR coverage, two positive schema fixtures and nine negative fixtures.
See [the research audit](docs/05-delivery/research-audit.md) and
[machine-readable audit results](audit/doc-audit-results.json).

In the authoring workspace, root docs/ is canonical. This output folder and the ZIP
are deliverable snapshots. When importing into a coding repository, preserve docs/,
AGENTS.md and CLAUDE.md together and maintain a single canonical copy.
