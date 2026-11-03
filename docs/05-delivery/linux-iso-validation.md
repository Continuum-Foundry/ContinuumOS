# Linux ISO and guest validation

The x86-64 GNU/Linux host uses the same original freestanding Rust kernel, Limine
loader, ELF audits, ISO payload verification, serial-record assertions and bounded
QEMU observer as Windows. PowerShell 7 remains the cross-platform orchestration
language. This is a host tooling port; Linux is not part of the guest OS.

## Supply tools explicitly

The measured host is Debian 13 x86-64 with GCC 14.2.0, Python 3.12, Node 24 and
Rust 1.99.1. Use the existing checkout; a cloud task is already isolated and does
not require a worktree. All downloaded/extracted/build files stay under ignored
`work/` or `build/`; no system package, signing policy, network interface or power
setting is changed. Run acquisition only under the owner's applicable handling
authorization in [EXPORT_CONTROL.md](../../EXPORT_CONTROL.md).

From the repository root:

```bash
bash tools/linux/Install-Rust.sh
python3 tools/linux/supply.py --download --build --fetch
work/tools/powershell/pwsh -NoProfile
```

Inside PowerShell:

```powershell
. ./tools/Enter-Dev.ps1
cargo xtask doctor --profile m1 --json
```

`config/m1-inputs.linux.lock.json` records Linux runtime hashes and acquisition
checksums separately from the unchanged Windows lock. The supplier never rewrites
expected hashes. Rebuilding QEMU uses its exact upstream commit, four jobs and
locked Meson/Ninja wheels; build/runtime prerequisites come from checksum-pinned
Debian archives whose initial hashes were checked against signed repository
metadata. TLS, Cargo checksums and frozen lockfiles remain enabled. `--fetch`
explicitly supplies the public Cargo cache; subsequent validation is offline.
Without flags, the supplier only verifies retained runtime files.

The runtime lock describes the measured outputs on this compiler/base OS. A build
on another base image or source path can produce different executable bytes and
must fail closed until separately reviewed and pinned. Do not regenerate the lock
to silence a mismatch. Restoring the prepared cloud filesystem retains the exact
measured tools; live processes are not needed or assumed to survive.

Linux QEMU is built from release 11.1.0; Windows retains its original pinned
11.1.0 development build. The machine remains `pc-q35-11.1`, TCG, `qemu64-v1,+apic`,
one vCPU and 2048 MiB RAM. Both firmware files match the Windows lock byte for
byte. Linux xorriso 1.5.6 is separately pinned and must pass the same boot-catalog
and extracted-payload checks. Native wrappers pin their emulator/ISO executable,
shared-library and ROM inputs; QEMU's wrapper supplies only its local ROM data
path in addition to the recorded locked VM arguments.

## Run acceptance serially

Keep the execution host available throughout acceptance. Do not change guest
limits or host deadlines to make a run pass. Host preflight timeouts and guest
deadline failures are different outcomes; preserve their logs separately.

```powershell
. ./tools/Enter-Dev.ps1
$env:RUST_TEST_THREADS='1'
$env:RUST_MIN_STACK='16777216'

# Prerequisite identity, bad overrides, firmware corruption and missing runtime.
./tools/Test-P1-01.ps1

# Complete M1: host checks, two clean builds, ISO and all original guest cases.
./tools/Test-P1-07.ps1

# Complete prior runtime/SDK regressions, then public supervision and ten stress
# guests (five repetitions with tracing off/on), including independent reviews.
./tools/Test-P3-13.ps1
```

The P3-13 command runs the complete P3-12 acceptance first, which itself reruns
M1 and ten prior runtime cases, SDK and SDK-runtime guests, codec conformance,
separate metadata build flavors and the selected portable UI/model link checks.
Each runner prints its artifact directory and writes a status/result JSON. A zero
exit code alone is insufficient: inspect those current-run reports and independent
reviews. Full logs, ISOs, binaries and firmware-variable copies remain local;
promote only compact evidence under the repository retention policy.

Windows continues to require its original Authenticode certificate/compiler
runner checks when enrolled. Linux records Authenticode as not applicable and
runs native executable and clean build-script/proc-macro/test/argument/exit-code
preflights. It does not pretend that an ELF binary has a Windows signature.
Windows native acceptance must still be rerun on a supported Windows host after
changes; a Linux pass does not establish Windows runtime parity.

Hosted Windows native-window and private-commit/performance measurement commands
remain outside this ISO/guest port; Linux rejects unsupported process accounting
explicitly. No cross-platform desktop-performance equivalence is claimed.
