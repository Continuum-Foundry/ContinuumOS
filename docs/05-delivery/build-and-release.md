# Build, development and release specification

Status: Phase 0 implements `cargo xtask doctor [--json]`,
`cargo xtask build --profile feasibility` and `cargo xtask test --suite m0`.
See [the executable runbook](phase-0.md).
P1-01 adds `cargo xtask doctor --profile m1 [--json]` and explicit workspace-only
[prerequisite supply/validation](phase-1/prerequisites-runbook.md). P1-02 implements
dev kernel build, M1 ISO, bounded baseline run and named ISO smoke/negative tests;
see [its runbook](phase-1/minimal-iso-runbook.md). Other commands remain proposals.

## Proposed repository layout

```text
AGENTS.md / CLAUDE.md
docs/
kernel/             original kernel + architecture HAL
abi/                contracts, codecs, generated SDK inputs
services/           supervisor, brokers, storage, network, input
drivers/            declared device implementations
ui/                 portable layout/scene/paint/semantics
apps/               editor, files, settings, diagnostics
runtimes/           optional Wasm embedding
tools/xtask/        host build/run/update/test/package CLI
assets/             licensed fonts/icons/tokens/reference scenes
tests/              host logic, guest integration, fault models
build/              ignored intermediate artifacts
dist/               ignored verified release artifacts
```

The research workspace uses `work/` for scratch files and `outputs/` for deliverables.
`kernel/` now contains P1-02's minimal original Rust kernel. Other future source
directories are not created to imply implementation.

## Tooling API proposal

| Proposed command | Intended outcome |
| --- | --- |
| `cargo xtask doctor` | Inspect, report, and validate paths/features; do not silently install tools |
| `cargo xtask build --profile dev` | Kernel, selected services/apps and manifests |
| `cargo xtask preview` | Host-rendered portable UI with inspectors |
| `cargo xtask run --profile vm-baseline` | Launch pinned QEMU config; record boot/session metadata |
| `cargo xtask watch --target desktop` | Debounce, build immutable bundle, plan and send supported live updates |
| `cargo xtask update --plan ...` | Validate base/authority; show class and transaction outcome |
| `cargo xtask test --suite ...` | Run named host/guest/fault/benchmark suites |
| `cargo xtask iso --generation ...` | Offline standalone boot image from exact graph |
| `cargo xtask release --generation ...` | Coherent artifacts, checksums, symbols, provenance and notices |

These names are illustrative. Keep machine-readable JSON output for automation and
human-readable diagnostics. Never convert guest-provided strings into shell commands.

## Build profiles

Phase 0 uses separate experimental kernel/userland packages, independent linker
scripts and no global guest target. Its separate dependency workspace prevents
candidate UI libraries entering the kernel dependency graph. All probes are
compile-only; userland entry is experimental and does not freeze syscalls.

Keep Slint/custom prototype features explicit; guest configuration and hosted
preview/interpreter support are separate. Optional inference/model assets have
their own profiles. Core boot/desktop/reload/release commands work without them.
Compare loader routes using actual ISO output, not image extensions.

Development favors incremental builds and diagnostics; measurement uses optimized
code with symbols/tracing configuration recorded. Release disables dev authority and
unneeded debug listeners. Kernel and userland targets/link flags differ. Host and
guest dependencies are separate to prevent accidental `std` or native OS linkage.

Do not enable global LTO/maximum optimization until iteration and compatibility needs
are measured. Whole-program optimization can couple incremental units and change
timing. Record panic behavior and CPU baseline. Unsupported instruction assumptions
must not creep in through host-native compilation flags.
[S05](../01-research/sources.md#s05)

## Watch and update loop

Debounce file writes and compile an identified source revision. Keep last successful
build active after compiler errors. Plans include base/target generation and update
class. Coalesce only queued pre-decision updates; never cancel a decided transaction
to deploy a newer edit. Report compilation separately from guest application latency.

By default watch scoped desktop/app code. Kernel edits create an explicit reboot-required
result rather than secretly restarting QEMU. A user can request a reboot separately.
Do not attach compilation failures to a success notification.

## Reproducibility

Pin Rust/LLVM/linker, dependencies/features, bootloader/protocol/firmware, asset inputs,
QEMU test config and packaging tools. Normalize file order, permissions, timestamps,
paths and locale-sensitive generation. `SOURCE_DATE_EPOCH` is one input, not complete
reproducibility. Compare two clean outputs and explain any nondeterminism.
[S61](../01-research/sources.md#s61)

Build without opportunistic guest-side downloads. A host Linux environment is allowed
for tooling; it does not supply the guest kernel. Record whether Windows-native and
Linux builds are both supported or one is authoritative.

## Release artifact set

ISO; optional VM data image; update bundles; exact generation/package manifests;
SHA-256 checksums; debug symbols/map files; source revision/toolchain lock; boot config;
serial/test/benchmark evidence; license/SBOM; provenance; known limitations; recovery
instructions. Keep secrets/dev credentials outside artifacts.

Verify the exact final ISO, not a pre-packaging kernel. Boot offline, prove no host
share dependency, test rescue paths, inspect dev authority and compare declared
hardware/features to evidence. A successful compile is not a boot test.

## Automation and host safety

Restrict local QMP/GDB/dev endpoints and use explicitly named VM images. Never write
to a physical disk in the initial workflow. Do not silently enable Windows optional
features or change firmware virtualization settings. Preserve logs from failed runs
and provide concrete commands/configuration only after testing this environment.
