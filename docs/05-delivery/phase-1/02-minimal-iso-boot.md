# P1-02 — Cold-boot a minimal original Rust kernel from ISO

Issue type: Story. Epic: P1. Status: Done. Owner: Codex.
Depends on: P1-01 Done. Suggested labels: `phase-1`, `boot`, `kernel`, `build`.
Backlog: B38, B03, minimal serial slice of B04. Requirements: R01/R02/R03/R16/R17.
Test families: T01/T02/T12, limited to packaging and minimal boot.

## User story

As an OS developer, I need proof that the actual standalone ISO reaches our Rust
kernel through UEFI so later kernel work starts from a demonstrated boot path.

## Implementation scope

1. Create a real kernel package only now; leave the Phase 0 link probes unchanged.
   Implement the pinned loader's minimum handoff, a documented stack/entry boundary,
   required protocol negotiation and fixed-buffer serial output with a timeout.
2. Keep interrupts disabled until their handlers exist. Use no heap, unwinding,
   SIMD, scheduler or firmware boot-service calls after handoff. Audit generated
   code and any assembly against the selected ABI and upstream CPU/loader contracts.
3. Emit a kernel-specific `m1.kernel_entry` checkpoint with build identity after
   reaching first-party code. This is not the full M1 readiness marker.
4. Implement the proposed dev build, M1 ISO packaging and bounded run commands from
   the epic. Stage only pinned loader files, kernel, configuration and notices;
   use the real UEFI optical boot recipe. Capture the final ISO hash after packaging.
5. Launch a new QEMU process from the ISO through pinned OVMF with networking and
   host sharing disabled. Copy the variable-store template per run; never reuse
   mutated firmware state across cold-boot tests.
   Disable automatic guest reboot and use a fresh serial log; unexpected reset
   or a duplicate kernel entry is a failing run even if an entry marker exists.
6. Add the named `iso-smoke` test and an explicit emulator-only exit path. Require
   both the expected guest checkpoint and the decoded test-exit result for success.

## Acceptance criteria

- [x] The real kernel links independently with a manifest identifying first-party
  sources, loader/protocol versions, code model, entry and assembly boundary.
- [x] ISO inspection finds the intended UEFI optical boot entry and expected files.
- [x] A fresh VM boots the exact hashed ISO and logs `m1.kernel_entry` from our kernel.
- [x] Manifest/serial build identities match; an old or unrelated boot log is rejected.
- [x] Network, shares, snapshot restore and direct-kernel loading are absent from
  the captured arguments. No other guest boot disk is attached.
- [x] Success, failure and timeout all leave logs and terminate/reap the VM process.
  An early emulator exit without the marker never passes.

## Tests and evidence

Run one successful `iso-smoke` and repeat it with a fresh variable store. Exercise
missing kernel/loader/firmware input, altered input digest, a staged configuration
referencing a nonexistent kernel, and a guest that never reaches the checkpoint.
Preflight failures and real firmware/loader failures must be reported separately.
Do not substitute a mocked marker for the positive guest case.

Retain the ISO and ELF hashes, artifact input manifest, optical catalog inspection,
exact QEMU invocation, firmware template/run-copy hashes, raw serial/host logs and
exit assertions. A screenshot is supplementary.

## Exclusions and handoff

No allocator, owned page tables, validated framebuffer, stable uptime or complete
boot metadata yet. This passes B03/B38 only for the declared minimal UEFI/ISO route;
BIOS, USB installation and alternate loaders remain unsupported. P1-03 inherits the
working serial/packaging harness and boot handoff, not a mature kernel.
Apply the [epic definition of done](README.md#evidence-and-story-completion).

Implementation revision: working tree on `codex/p1-02-minimal-iso-boot`, source
base `d18bb0a`; exact build inputs/hashes are retained per artifact.
Evidence: [2026-10-18 acceptance review](../../../evidence/phase-1/P1-02/2026-10-18/README.md).
Runbook: [minimal ISO commands and limits](minimal-iso-runbook.md).
Review: Codex checked all six criteria against the retained final acceptance run;
Done for this minimal UEFI/ISO route, not complete M1. No external review is implied.
