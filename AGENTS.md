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
P3-01's prerequisite/contract gate passes local self-review; see
`evidence/phase-3/P3-01/2026-10-25/README.md`. P3-02's owned-memory backend passes
local self-review; see `evidence/phase-3/P3-02/2026-10-25/README.md`. P3-03's staged
static-image loader and trusted launch bindings pass local self-review; see
`evidence/phase-3/P3-03/2026-10-25/README.md`. P3-04 user entry, checked syscalls
and process-local fault containment pass local self-review; see
`evidence/phase-3/P3-04/2026-10-26/README.md`. P3-05 preemption and eager
x87/SSE context pass local self-review; see `evidence/phase-3/P3-05/2026-10-27/README.md`.
Unmasked SSE exceptions, native x87 debug metadata and shared-process threads
remain unsupported/unverified as documented. P3-06 handle rights and retained memory-object lifetimes pass local self-review;
see `evidence/phase-3/P3-06/2026-10-29/README.md`. P3-07's bounded channels and atomic authority transfer pass local self-review; see
`evidence/phase-3/P3-07/2026-10-29/README.md`. P3-08's bounded waits, timers, cancellation and blocking IPC helpers pass local self-review; see
`evidence/phase-3/P3-08/2026-10-30/README.md`. P3-09's bounded user-mode supervision and prepaid subtree ceilings pass local self-review; see
`evidence/phase-3/P3-09/2026-10-30/README.md`. P3-10's ephemeral scoped policy and use-time session checks pass local self-review; see
`evidence/phase-3/P3-10/2026-10-30/README.md`. P3-11's ephemeral routes and writer fences pass local self-review; see
`evidence/phase-3/P3-11/2026-10-31/README.md`. P3-12's independent bounded SDK and nonrendering runtime hooks pass local self-review; see
`evidence/phase-3/P3-12/2026-11-01/README.md`. Only its exact tested contract subset
is frozen. P3-13's bounded SDK stress and resource accounting pass local self-review; see
`evidence/phase-3/P3-13/2026-11-01/README.md`. P3-14 and the strict M3 isolated-runtime/interface exit pass local self-review at the original deadlines; see `evidence/phase-3/P3-14/2026-11-06/README.md`. The earlier owner-accepted exception and failed runs remain historical evidence. M4 guest UI/reload and desktop efficiency targets remain open. No standing test waiver is granted.

P4-01 passes local prerequisite/profile self-review; see
`evidence/phase-4/P4-01/final-1794022944926085700/README.md`. Its two long M3 stress flavors use the explicit
revision-2 60000-ms bound; ordinary 30000-ms limits, counts and 320-ms guest-local
bounds remain. This is not an original-limit pass. P4-02 passes local runtime-admission self-review; see
`evidence/phase-4/P4-02/final-1794169067563019600/README.md`. Its finite seven-role graph,
reclaiming private heaps and bounded teardown/restarts accept no rendering or reload.
P4-03 is next; P4-03 through P4-19 remain Not started. Host codecs/plans accept no guest desktop.

Read `docs/README.md`, `docs/00-overview/requirements.md`, and
`docs/05-delivery/agent-handoff.md` before implementing anything. Claude and other
coding agents should use the same documents. `CLAUDE.md` points to this entry point.

## Development safeguards

Owner direction, 2026-11-08: the later request to slim down current P4-02
validation supersedes the earlier NEXT-time-only direction. Use the reviewed
selection in `config/p4-02-validation.json`: retain and re-review completed
new-behavior components on unchanged implementation, then execute focused affected
regressions. Interrupted/failed full gates remain non-passes. Milestone exits,
new-feature counts and deadlines are unchanged; this is not a standing test waiver.

- Story closeout uses new-behavior tests plus affected regressions selected by
  agent judgment under `docs/05-delivery/validation.md#change-based-story-validation`.
  Record impact, consumers, selected checks and omissions before testing, including
  staged/unstaged/untracked inputs. Do not recursively rerun all prior milestones
  by default. Full suites remain required at milestone exits and for relevant broad
  foundation changes or uncertain impact. This supersedes blanket per-story full
  M1/M3 rerun wording; it changes selection, never deadlines, counts or release exits.
- Before guest validation, preview the selected plan with an explicit VM-launch
  budget and full-suite opt-in; see `docs/05-delivery/validation-run-control.md`.
  Keep one active writer per checkout. Stop at the first failure, diagnose it and
  rerun affected work instead of restarting earlier milestone cascades. Reuse
  completed commands only with the required input/context/artifact proof; final
  prose/evidence/staging changes rerun host audits, not unchanged guest tests.
- When instructed to work on `main`, inspect the working tree, switch safely and
  run `git pull --ff-only` before assuming it is current. Preserve unrelated edits;
  do not discard changes or force a divergent branch to match the remote.
- Keep test policy in the registered hooks in `kernel/src/user_fixture.rs`.
  Syscall operation arms share dispatch and return validation; switch capture and
  finish use the fixture-installed observer, never a hardcoded fixture address.
  Public operations must not select return IP, selectors or privileged flags.
- For shared boundary/scheduler changes, validate ordinary, `user-boundary-test`,
  `scheduler-test` and `capability-test` builds separately. An all-features build
  does not prove narrower subsets compile. Prefer early per-flavor compile checks
  where tooling supports them, then run the required guest regressions. The
  current xtask case commands combine building, packaging and guest execution;
  do not claim a compile-only matrix command exists.
- Run acceptance serially in the approved signer/xorriso execution context.
  Advise the owner to keep the host awake during long runs; do not change power or
  security settings. A host preflight timeout and a guest deadline failure are
  distinct failures. Preserve both logs, keep deadlines unchanged and never count
  an interrupted run as acceptance. Do not infer a root cause from a timeout alone;
  per-subprobe timestamps are a diagnostic improvement, not a proven fix.
- For an owner-exempted push, make the exact payload, remote destination, target
  branch and any one-time guard bypass explicit in the approved scope and record
  it in `EXPORT_CONTROL.md`. Use existing explicit authorization without asking
  again; clarify only missing scope required to proceed. An exemption is not a
  standing waiver. Keep unrelated edits and local artifacts out of the upload,
  preserve the pre-commit check and leave the push guard installed afterward.

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

Linux host tooling and serial ISO/guest acceptance use `docs/05-delivery/linux-iso-validation.md`.
Windows signing remains Windows-specific; Linux uses native execution preflights.

## AI planning amendment — 2026-11-07

The owner requested [the AI amendment](docs/05-delivery/ai-roadmap.md): Gemma local
default, qualified flagship cloud/private endpoints, automatic editable preferences,
review final send/publish with scoped standing rules, OS allowance policy foundation,
enterprise federation and cooperating models. Read R20–R27,
[AI architecture](docs/02-architecture/ai-platform.md), ADR-0030–0033 and the
unfrozen AI task/policy contract before AI work. B39/M21 follows M5 plus B67/B68;
B67–B82 and M23–M26 are separately gated broad planning tasks, Not started.
Keep the current ordered Phase 4 gate, all existing release exits/evidence, 256 MiB model-unloaded cold
idle including dormant AI, and fully counted resident/active/default/peak profiles.
No native model/server port, funded allowance or production policy engine is accepted.
Decompose later milestone stories only at the owner's request.

## Authorization planning amendment — 2026-11-08

Read [authorization roadmap](docs/05-delivery/authorization-roadmap.md),
[identity architecture](docs/02-architecture/identity-and-authorization.md) and
ADR-0034 before account/policy work. Cedar is an evaluation candidate; account
presets, scoped roles/elevation and richer controls have separate qualification.
B83–B89/M27/M28 are Not started; enterprise OS identity and data-center workload
authorization are independent of AI federation/server feasibility. Keep current
Phase 4 order, B01–B82 dependencies, existing M7 scope and all release requirements
unchanged. No engine port, native server capability or security result is accepted.
