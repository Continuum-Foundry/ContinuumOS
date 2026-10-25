# Agent entry point

This workspace contains research/design, Phase 0 Windows build/link probes and
the minimal original Rust kernel ISO with checked metadata, identity, fatal
diagnostics, P1-05 memory ownership and P1-06 timer/uptime. P1-07's complete local
acceptance passes; see `evidence/phase-1/P1-07/2026-10-20-acceptance/README.md`.
Check the story's review before continuing. M1's experimental VM baseline and M2's declared hosted UI subset pass local acceptance; see
`evidence/phase-2/P2-13/2026-10-24/README.md`. Guest UI/reload and efficiency targets remain open;
this is not a usable OS. See
`docs/05-delivery/phase-1/interrupts-timekeeping-runbook.md` and retained evidence.
The user approved the reconciled revision-2 roadmap. See
`docs/00-overview/reconciled-plan.md`; individual library integrations still need validation.

Read `docs/README.md`, `docs/00-overview/requirements.md`, and
`docs/05-delivery/agent-handoff.md` before implementing anything. Claude and other
coding agents should use the same documents. `CLAUDE.md` points to this entry point.

Before retaining new evidence, follow `docs/05-delivery/evidence-retention.md`.
Keep full generated logs/source copies local; stage a compact reviewed bundle and
run `node audit/check-repository-hygiene.cjs`. Historical logs/source copies are
local-only after the owner's cleanup; see `evidence/README.md` before verifying them.

Preserve the user's central intent: an original desktop OS, modern languages,
excellent and efficient UI, live development without VM reboot for supported
changes, and a separately buildable bootable ISO. Do not replace the project with
a Linux distribution, a web desktop, or an existing kernel.

Architecture decisions are proposed engineering defaults until their documented
experiments pass. They are not claims of implementation or user approval. Work
through the milestone gates; update the relevant specification and ADR when
evidence changes a decision. Keep source-backed findings, proposals, and measured
results distinguishable. Never claim a VM boot, safe update, security guarantee,
or performance result without its evidence.

No guest compiler, GPU stack, third-party crate port, Linux application
compatibility, or universal kernel hot reload may be assumed to exist. First-party
kernel code should be Rust with narrowly documented architecture-specific assembly.
Use explicit protocol schemas across replaceable boundaries, not Rust memory layout.
Compare Slint with a minimal custom UI before selecting the toolkit. Typed actions
are core interfaces; inference is optional after the first state-preserving guest
reload. Keep the 256 MiB cold-idle target. Boot recovery and live state transfer
are separate; a booting disk image does not prove the required ISO boots.

Phase 0 implements compile/tooling feasibility; actual ISO boot starts M1 in
`docs/05-delivery/milestones.md`. Re-run the Phase 0 checks before extending it.
Rust is workspace-local under ignored `work/toolchains/`; existing MSVC/SDK were reused.
Pinned QEMU/OVMF/ISO tools are workspace-local under ignored `work/tools/`.
P1-02/P1-03 launch bounded offline QEMU processes; no WSL, system setting, physical
disk or global package was changed. xorriso needs execution outside the sandbox's
MSYS2 named-object restriction. Do not attempt the entire backlog at once.
