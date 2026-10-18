# Handoff for Claude, Codex and other coding agents

## Current state

This workspace contains an architecture/research library and agent entry points.
Phase 0 adds a native Windows Cargo/xtask build system, independent freestanding
kernel/userland link probes and dependency experiments. See the
[runbook](phase-0.md) and [evidence](../../evidence/phase-0/README.md).
There is **no boot-tested kernel, VM demo, boot-tested ISO, guest driver stack or
measured guest performance result**. All ADRs are Proposed. Example
contracts are drafts; sample IDs/digests are synthetic. Do not report implementation
progress from directory names or document completeness.
The user approved the [revision-2 reconciled direction](../00-overview/reconciled-plan.md).
Use it without reopening settled scope; toolkit/inference/loader integrations still
require technical evidence gates.

## Required reading order

1. [Documentation index](../README.md) and [vision](../00-overview/vision-and-scope.md).
2. [Requirements](../00-overview/requirements.md), including deferred scope and metrics.
3. [Architecture summary](../00-overview/architecture-summary.md) and relevant layer specs.
4. [ADRs](../04-decisions/README.md), [live-update invariants](../03-live-update/model-and-guarantees.md),
   and [transaction state machine](../03-live-update/protocol-and-transactions.md).
5. [Milestones](milestones.md), [backlog](backlog.md), [validation](validation.md),
   and [risks](risks-and-open-questions.md).

## Next action

Follow [the Phase 1 epic and seven ordered stories](phase-1/README.md), starting
with P1-02. P1-01 now has [host prerequisite evidence](phase-1/prerequisites-runbook.md).
The remaining stories are local plans, not created Jira issues or completed
implementation. Do not skip their evidence gates or mark all of B06 complete in M1.

Re-run the Phase 0 acceptance commands before extending the work. B01/B02 now have
Windows compile/link evidence; B38 remains open. Run `cargo xtask doctor --profile m1`
against the supplied QEMU/OVMF/ISO inputs, then build the smallest original-kernel ISO in M1.
Follow with the hosted UI comparison in M2. Do not leap into a
general Wasm runtime, full browser, GPU stack or custom UI language first.
Compare Slint/minimal-custom UI through B36 before broad widget investment and
resolve the actual ISO path through B38. Typed actions are required; optional AI
B39 follows the first live-reload demo and never gates the AI-free core.

Rust 1.99.1 is installed only in `work/toolchains/`; dot-source `tools/Enter-Dev.ps1`.
Existing MSVC/SDK were reused. QEMU/OVMF/xorriso are supplied under ignored
`work/tools/` with [input locks](../../config/m1-inputs.lock.json). xorriso's MSYS2
runtime needs execution outside the sandbox's named-object restriction; doctor
reports that restriction as blocked. TCG listings pass; WHPX and virtualization
inspection remain untested/unknown. WSL is not used. No VM has been launched.
Keep scratch files under `work/` in this workspace and research
deliverables under `outputs/`; later source layout is proposed in the build spec.

## Preserve these constraints

- Original kernel, not a rebranded existing OS or web-only shell.
- Modern first-party code, with architecture-specific instructions acknowledged.
- Live updates for declared classes without hidden OS/VM reboot.
- State continuity covers unsaved work, not only cosmetic window geometry.
- ISO generation and live update transport remain separate.
- Versioned boundaries, one writer per epoch, safe old-code retirement and honest rollback limits.
- No assumed `std`, GPU API, network crypto, filesystem durability or runtime port.
- Stable recovery path when shell/app/service updates fail.
- Accessibility, text, memory and observability begin early.
- Keep the 256 MiB cold-idle target; VM capacity is not idle usage.
- Evaluate hosted toolkit preview and guest code reload separately.
- Boot-attempt/known-good recovery does not replace live state transfer.

## How to use proposed decisions

Proceed with reversible experiments using their working defaults. They do not mean
the user has validated every library or implementation property. Record approval
of the reconciled direction accurately. Record gate evidence and update ADR status through explicit
project review. Ask only when a missing choice materially blocks work or a change
would violate core intent; branding and optional future hardware do not block M0.

## Contract changes

Read `docs/06-contracts/` before implementing codecs/IPC. Draft field shapes are
starting points. Add independent codec fixtures and semantic compatibility tests
before freezing ABI IDs. If revising fields, update schemas/examples/specs together.
Never serialize native layouts by casting a struct to bytes. Document handle,
allocator, deadline and crash semantics, not only function signatures.

## Implementation reporting

For every completed slice report what now works, exact artifact/build IDs, tests
actually run, evidence paths and current limits. Mark a compile result, boot result,
render result and live-state result separately. Store failure traces rather than
removing them to make demos look clean. Do not claim performance from a host preview
or third-party benchmark. Do not silently degrade update class or data guarantees.

## Research maintenance

Primary sources are in [the catalogue](../01-research/sources.md), with access limits
and date. Recheck moving-version claims when choosing dependencies. Pin actual
versions/commits/digests; the catalogue's `latest` URLs are not build inputs.
Source-backed findings, proposed design and measured results remain separate.

Maintain requirement→spec→task→test traceability. Add ADRs for new architectural
choices; preserve superseded rationale. Keep `AGENTS.md` and `CLAUDE.md` short and
pointing here so agents share one architecture rather than divergent instructions.
