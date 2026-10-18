# Research method and evidence boundaries

Research performed on 2026-10-17 for the architecture described in this library.
The work uses primary documentation, specifications, upstream repositories,
and original research papers. It does not rely on community comments or generated
search summaries as proof of performance or compatibility.

## Method

1. Extract the user's required properties and separate unspecified choices.
2. Examine precedents for live OS evolution and managed-language kernels.
3. Check compiler targets, ABI constraints, runtime requirements and platform APIs.
4. Trace boot, execution, input, rendering, persistence, updates and recovery end to end.
5. Compare alternatives and select provisional defaults with reversal criteria.
6. Convert unverified integration claims into experiments with observable exit gates.
7. Cross-check decisions, interfaces, milestones and test coverage for consistency.

The source catalogue records facts we used and their limits. Directly opened
sources are labeled O; sources inspected through indexed primary-site results
are labeled I. Indexed access is weaker than a complete read. Some later direct
opens failed despite successful indexed results. Those limits are recorded.
No third-party benchmark is presented as a measurement of this OS.

## Confidence model

| Claim | Appropriate evidence |
| --- | --- |
| Language target exists | Official compiler target documentation |
| Existing system supports live changes | Original paper or upstream design documentation |
| Crate can integrate here | Exact pinned crate builds and executes on our guest; not established by `no_std` alone |
| Update preserves state | Fault-injected integration tests and state/epoch traces |
| UI is efficient | Recorded workloads, frame distributions, memory accounting, power proxies |
| Security property holds | Threat-specific review/tests, and formal proof only where actually supplied |
| Distribution is reproducible | Independent clean builds and artifact comparison |

## Findings that refine earlier conversation

- Live evolution is not a first-ever idea. Theseus and MINIX provide directly
  relevant OS precedents; Erlang is a mature code replacement precedent.
- Rust does not supply a stable general native plugin ABI. Serialize protocols
  and replace processes rather than depending on in-memory Rust layouts.
- Modern language syntax cannot remove CPU interrupt/privilege instructions.
  Tiny assembly or compiler-intrinsic boundaries remain necessary.
- `no_std` means freedom from Rust's standard library, not absence of a heap,
  platform hooks, runtime resources, device work or a portability audit.
- GPU-capable virtual hardware does not give a new OS a guest GPU driver.
- Storing an old binary is not enough to guarantee rollback after state changes.
- A Wasm runtime is an optional integration project, not a free built-in service.
- UI fidelity requires shaping, bidi, text boundaries and accessibility semantics;
  a glyph atlas and rounded rectangles are insufficient.

## Source disagreements and moving targets

The Embedded Swift documentation inspected contains an experimental-support note,
while the official getting-started pages describe practical embedded development
and recent maintainers discuss PAL changes. We therefore recommend a pinned
toolchain experiment, not a sweeping release-support claim. `master`, `latest`,
QEMU development documentation, Unicode revisions and Wasm features can move.
Pin exact versions during M0 and record the subset actually supported.

The Redox book direct fetch returned 403, and a few platform documentation URLs
failed direct access. We do not treat inaccessible pages as verified detailed
evidence. Redox remains a comparison candidate with its integration claims left
unresolved. A later agent should re-fetch relevant upstream material if choosing it
as a dependency; this design does not adopt its kernel.

## Stopping rule and remaining research

This is an end-to-end scoped foundation, not a claim that every OS research topic
has been exhausted. Each major subsystem has a design, interface boundaries,
quality criteria, failure cases and a delivery stage. Remaining questions require
prototypes, measurements or user product choices; they are recorded in
[the risk register](../05-delivery/risks-and-open-questions.md). More speculative
browsing should not replace those experiments.

No kernel was compiled, no ISO was built or boot-tested, and no UI benchmark was
run in this research task. The Windows PATH inspection found Node, Git and WSL
launchers; Rust/Cargo/QEMU were not found on PATH. That does not establish whether
they exist elsewhere or whether WSL has a configured distribution.
