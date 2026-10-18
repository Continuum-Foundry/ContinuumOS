# Windows M1 prerequisites

P1-01 supplies host inputs only. No ISO has been packaged or booted.
The reviewed pins are in [the input lock](../../../config/m1-inputs.lock.json).
All acquisition and extraction stays in ignored `work/`; no persistent PATH,
Windows feature, WSL, installer, VM or physical disk change is performed.

```powershell
. ./tools/Enter-Dev.ps1
# Explicit acquisition, normally needed only on a fresh checkout:
pwsh -NoProfile -File tools/Supply-M1Prerequisites.ps1
cargo xtask doctor --profile m1
cargo xtask doctor --profile m1 --json
pwsh -NoProfile -File tools/Test-P1-01.ps1
cargo check --locked --manifest-path experiments/boot-bindings/Cargo.toml --target x86_64-unknown-none
```

The supply script prints URLs, sizes and destinations, downloads only absent
archives, verifies every archive against its SHA-256 pin before extraction, and
keeps mismatches for inspection. It extracts the QEMU NSIS archive with portable
7-Zip; it never executes the QEMU installer. Windows `tar.exe` extracts the
MSYS2 packages. Package runtime DLLs stay together under `work/tools/iso/usr/bin`.
The lock records 7-Zip's acquisition inputs too. Package archives retain their
metadata and included notices; they are not global package-manager installations.

M1 doctor verifies required file sizes and hashes **before** executing any
version/capability query. It then checks QEMU's versioned machine, explicit CPU,
TCG and HPET listings and xorriso's version. It does not acquire tools. M0 doctor
retains compile-only behavior. ISO/run commands remain unavailable until P1-02;
their implementation must call the same M1 preflight and stop on any failure.

`CONTINUUM_QEMU_SYSTEM_X86_64`, `CONTINUUM_XORRISO`, `CONTINUUM_OVMF_CODE` and
`CONTINUUM_OVMF_VARS` are strict overrides with no fallback. Their contents must
match the lock. Executables must remain in their locked runtime directory;
relocating an executable alone does not supply its DLLs. A different tool tree
requires a reviewed lock update. Paths may contain spaces.

## Tested inputs and limits

- QEMU 11.1.0, build `v11.1.0-12130-ge470268ff4`; package
  `qemu-w64-setup-20260811.exe`. Its SHA-512 matched the publisher's companion
  file. Both come from the same upstream origin; this is not an independently
  authenticated signature check.
- xorriso 1.5.8.pl02 from MSYS2 package `1.5.8.2-1`, with libbz2 1.0.8-4,
  libiconv 1.19-1, libreadline 8.3.006-1, ncurses 6.6-2, zlib 1.3.2-1,
  msys2-runtime 3.6.11-4 and gcc-libs 15.3.0-1. SHA-256 values are measured
  acquisition pins; MSYS2 package signatures were not independently verified.
- The runtime's `NtCreateDirectoryObject` call is denied by the Codex sandbox.
  Version and acceptance probes pass outside that sandbox, using the same local
  files. Sandbox doctor correctly reports blocked; this is an execution permission
  limitation, not evidence of a missing DLL. Later packaging needs that execution
  context too. No sandbox relaxation or Windows setting is persisted.
- QEMU's `60-edk2-x86_64.json` pairs raw `edk2-x86_64-code.fd` (3,653,632 bytes)
  with `edk2-i386-vars.fd` (540,672 bytes). The combined flash is 4 MiB. This is
  the non-Secure-Boot profile; Secure Boot remains unsupported. Copy the variable
  template per run. Descriptor/format/hash verification does not prove boot.
- Limine 12.9.3's binary ZIP matches the previously reviewed upstream SHA-256.
  Keep `LICENSE` and `3RDPARTY.md` with the binaries. QEMU's GPL notices,
  `COPYING.LIB`, `edk2-licenses.txt`, and the runtime packages' GPL/LGPL/GCC
  exception and other included notices remain with the local inputs. Distribution
  source/notice obligations need release review; no project license is selected.
- Protocol commit `3a0526b700e356f0eac1b71a77697b3fd1c707a3` pins the specification
  and C reference header. Rust `limine =0.5.0`, defaults off, with locked bitflags
  2.13.2 compiles on stable Rust 1.99.1 for `x86_64-unknown-none`. The narrow probe
  requests base revision 3 and revision-0 framebuffer/memory-map/RSDP structures;
  their IDs/layout were compared with the reference header. Limine 12.9.3's
  source accepts base revisions through 6, including 3. No broader binding/API or
  guest handoff compatibility is claimed. Version 0.6.5 was rejected because
  its `ptr_metadata` feature needs nightly; the failure is retained.

## Proposed launch profile

The lock contains an argument array with explicit placeholders for the exact ISO,
locked firmware code, fresh variable-store copy and new serial log. It is a
configuration for P1-02, not an implemented launcher or a boot result:

`pc-q35-11.1,hpet=on`, TCG, `qemu64-v1,+apic`, one vCPU, 2048 MiB, standard VGA,
headless display, COM1 file log, optical-only strict boot, no automatic reboot.
Network, monitor and parallel ports are explicitly disabled. QMP/GDB are omitted
and disabled by default. There are no host shares or data disks. P1-02 must bound
the process lifetime, copy vars, substitute paths as argument-array values and
revalidate every input. APIC/HPET registers, timing, framebuffer and ISO boot
remain guest experiments.

## Sources

[QEMU Windows packages](https://qemu.weilnetz.de/w64/),
[MSYS2 packages](https://repo.msys2.org/msys/x86_64/),
[7-Zip](https://www.7-zip.org/download.html),
[Limine 12.9.3](https://github.com/Limine-Bootloader/Limine/releases/tag/v12.9.3),
[loader revision handling](https://github.com/Limine-Bootloader/Limine/blob/v12.9.3/common/protos/limine.c),
[pinned protocol](https://github.com/Limine-Bootloader/limine-protocol/tree/3a0526b700e356f0eac1b71a77697b3fd1c707a3),
[Rust binding 0.5.0](https://crates.io/crates/limine/0.5.0).
