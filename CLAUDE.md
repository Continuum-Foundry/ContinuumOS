# Claude entry point

[AGENTS.md](AGENTS.md) and [docs/README.md](docs/README.md) are the shared source
of project instructions for all coding agents. Read them first. This file adds
only operational notes for Claude Code: commands, layout and pitfalls. Do not
record architecture or decisions here; put those in the specs and ADRs.

## Current state (verify before relying on it)

- Original `no_std` Rust kernel (`kernel/`) cold-boots from a standalone UEFI ISO
  via Limine under QEMU/OVMF and emits versioned serial records. P1-02 (entry) and
  P1-03 (checked boot metadata) have retained evidence.
- P1-04 adds owned fatal exception entry and static stacks; check its
  [story and review status](docs/05-delivery/phase-1/04-exception-handling.md).
  P1-05 adds owned frames/page tables, WP/NX, guards and actual double-fault entry;
  check its [story/review status](docs/05-delivery/phase-1/05-memory-ownership.md).
  P1-06 now adds validated ACPI/xAPIC/HPET, returning interrupts, idle wakeups and delivered-tick uptime; check its runbook/acceptance.
  P1-07's [complete local review passes](evidence/phase-1/P1-07/2026-10-20-acceptance/README.md):
  33 rows and 42 launches. The owner-authorized local development signer and App
  Control policy resolve host execution; see the signing lifecycle runbook.
- No general heap, wall clock, processes, rendering or desktop.
  All ADRs are *Proposed*. Never describe more than the evidence shows.
- Check `git log --oneline -10` and the
  [agent handoff](docs/05-delivery/agent-handoff.md) at session start. Other
  agents (Codex) also commit here, so this summary may be stale.

## Hard rules

- **Never push, open PRs, upload, or send project material to remote services.**
  [EXPORT_CONTROL.md](EXPORT_CONTROL.md) puts a U.S.-only handling hold on all of
  them. `.githooks/pre-push` blocks pushes. Never use `--no-verify` or another
  transport. Past owner exemptions were one-time and are exhausted. A new
  exemption must come explicitly from the owner and be recorded in EXPORT_CONTROL.md.
- Cargo is `offline = true`. Do not fetch crates, install global packages, enable
  WSL, or change Windows settings. Tools are pinned under ignored `work/`.
- Do not change pinned versions (Rust `1.99.1`, Limine 12.9.3, `limine` crate
  `=0.5.0`, QEMU/OVMF in `config/m1-inputs.lock.json`) without a documented experiment.
- Do not edit retained evidence snapshots under `evidence/`. Add a new dated
  directory, and keep failed runs.
- Original material is all rights reserved. Keep third-party notices and record
  provenance for anything copied or adapted ([LEGAL.md](LEGAL.md)).
- Work one story at a time, on a local topic branch.

## Environment

Windows host, MSVC target only (`check_pin` rejects other hosts). Every command
runs from the repo root in a PowerShell session that has dot-sourced:

```powershell
. ./tools/Enter-Dev.ps1   # sets CARGO_HOME/RUSTUP_HOME to work/toolchains, CARGO_TARGET_DIR=build/cargo
```

Without it, `cargo` is not on PATH. Shell state does not persist between tool
calls, so dot-source it inside every PowerShell command.

**PowerShell 7 (`pwsh`) is required.** It is available from the bundled Codex
runtime in the 2026-10-19 session; verify discovery in a fresh environment.
If absent, point `CONTINUUM_PWSH` to an existing executable; do not install globally.

**xorriso (MSYS2) needs execution outside the sandbox** because of a named-object
restriction. ISO packaging, `run` and `test --suite m1` therefore need
the execution tool's per-command sandbox escalation. Never change host settings
to work around it.

## Commands

```powershell
# Fast checks (no VM)
cargo fmt --all -- --check
cargo test --locked -p xtask                     # host harness unit tests
cargo test --locked -p continuum-kernel --lib    # allocation-free kernel library on host
node audit/check-docs.cjs                        # doc links/structure; must report "errors": []
node audit/check-governance.cjs
pwsh -NoProfile -File audit/validate-contracts.ps1

# Kernel / ISO / VM (bounded, offline QEMU; artifacts go to build/m1/<label>-<time>-<pid>/)
cargo xtask doctor --profile m1
cargo xtask build --profile dev                  # Cargo profile kernel-dev
cargo xtask iso --profile m1
cargo xtask run --profile vm-baseline
cargo xtask test --suite m1 --case iso-smoke     # also: iso-negative, boot-metadata, boot-reject

# Full story acceptance (slow; writes build/m1/acceptance-*)
pwsh -NoProfile -File tools/Test-M0.ps1
pwsh -NoProfile -File tools/Test-P1-07.ps1       # complete declared M1 gate
```

`cargo xtask` is an alias for `run --locked --package xtask --` (`.cargo/config.toml`).
The full `test --suite m1` exists. Partial guest checks cannot waive a blocked host
row; do not disable policy or silently skip tests. Review the P1-07 evidence first.

## Layout

| Path | What it is |
| :--- | :--- |
| `kernel/src/main.rs` | Entry point (`kernel_entry`), Limine requests, serial, test exits |
| `kernel/src/boot.rs`, `lib.rs`, `tests.rs` | Owned `BootInfoV1` validation; host-testable library |
| `kernel/link.ld` | Kernel linker script; request markers, RX/R segments |
| `tools/xtask/src/main.rs` | CLI, Phase 0 build/link probes, pin checks |
| `tools/xtask/src/m1.rs` + `m1/` | M1 harness: `build` (compile + ELF/disasm audits), `package` (ISO), `launch` (QEMU), `observe` (process supervision), `records` (serial contract), `cases`, `inputs`, `process`, `support` |
| `tools/xtask/src/elf.rs` | Independent ELF inspection |
| `tools/*.ps1` | Doctor, supply, acceptance and evidence-retention scripts |
| `experiments/` | Phase 0 link/dependency probes (`dependencies` is excluded from the workspace) |
| `config/m1-inputs.lock.json` | Pinned QEMU/OVMF/Limine/xorriso inputs and hashes |
| `docs/` | Specs, ADRs (`04-decisions`), contracts (`06-contracts`), delivery/stories (`05-delivery`) |
| `evidence/` | Immutable sanitized evidence per story/date |
| `work/`, `build/` | Ignored: toolchains, tools, scratch, build output |

## Kernel conventions

- Features: `emulator-test` (writes 0x10 → QEMU exit 33 on success, 0x11 → exit 35
  on rejection via isa-debug-exit port 0xf4), `fault-before-entry`, `boot-inject`.
  Ordinary builds must reject test fixtures.
- Serial records are a versioned contract (currently version 3: build ID, session
  ID, sequence, stage, outcome, emergency flag) validated by `m1/records.rs`. If you change the
  format, bump the version and update the runbook and records validator together.
- No heap, no formatting of panic arguments, bounded loops only. Each `unsafe`
  block and each piece of assembly gets a comment stating its assumption. Assembly
  stays narrowly scoped and documented (see `kernel/README.md`).
- Build audits reject FP/SIMD code in the kernel. Use explicit schemas at
  replaceable boundaries, never Rust memory layout.
- Every xtask source file is hashed into the build identity, so harness edits
  change build IDs. That is expected.

## Finishing a story

1. Implement against the story's acceptance criteria. Add host tests and named
   positive and negative VM cases (success requires the exact marker sequence plus
   the expected exit code; timeouts, panics and early exits never count).
2. Add `tools/Test-P1-0N.ps1` modelled on `Test-P1-03.ps1`.
3. Update the runbook, story file, `kernel/README.md`, the agent handoff, and any
   affected spec/ADR in the same change. Keep ADRs Proposed.
4. Retain sanitized evidence under `evidence/phase-1/P1-0N/<date>/`, with a README
   and `SHA256SUMS.json`.
5. Run `node audit/check-docs.cjs` and `node audit/check-governance.cjs`.
6. Commit locally. Do not push.
