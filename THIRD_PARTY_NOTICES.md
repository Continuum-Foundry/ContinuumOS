# Third-party components and notices

Original project material is all rights reserved under [LICENSE](LICENSE).
That reservation does **not** relicense, restrict, or claim ownership of upstream
software or third-party reference material. Preserve their notices and terms.

This is a provenance index, not a complete redistribution license bundle. Exact
Cargo package declarations/checksums are in
[the generated inventory](legal/dependency-inventory.json); exact native-tool
inputs and companion DLLs are in [the M1 lock](config/m1-inputs.lock.json).
Neither list establishes that every resolved package is shipped.

## Guest boot path

| Component | Pinned identity | Scope and notice location |
| --- | --- | --- |
| Limine bootloader | 12.9.3 | ISO loader; BSD-2-Clause plus bundled third-party terms. Preserve `LICENSE` and `3RDPARTY.md` together. Copies: [loader notice directory](licenses/third-party/limine-12.9.3/). |
| Limine protocol | Commit `3a0526b700e356f0eac1b71a77697b3fd1c707a3` | Specification/C reference header used for comparison; preserve [upstream LICENSE](licenses/third-party/limine-protocol/LICENSE). No first-party C guest dependency is introduced. |
| Rust limine bindings | 0.5.0, defaults off | Kernel dependency; MIT OR Apache-2.0. Full upstream terms: [binding notice directory](licenses/third-party/limine-0.5.0/). |
| bitflags | 2.13.2 | Binding dependency; MIT OR Apache-2.0. Full upstream terms: [bitflags notice directory](licenses/third-party/bitflags-2.13.2/). |
| Rust core/compiler-builtins | Rust 1.99.1 distribution | Target/runtime code needs an exact release review, including applicable Rust/LLVM/compiler runtime exceptions and notices; compiler version alone is not a complete runtime attribution inventory. |

The retained P1-02 packaging evidence demonstrates that Limine loader and binding
notices were staged. It does not establish a complete release bundle: first-party
rights notice, bitflags/runtime notices, actual linked-code inventory, and binary
distribution permission must be reviewed before a distributable release. The
historical ISO is an internal engineering artifact, not a cleared customer release.

## Host-side tools

| Component | Purpose | Terms and source of detail |
| --- | --- | --- |
| Rust/rustup/Cargo/LLVM | Compile, link, inspect | Mixed upstream licenses and exceptions; [Phase 0 toolchain provenance](evidence/phase-0/toolchain-manifest.json) and installed distribution notices |
| QEMU 11.1.0 Windows package | Emulator | GPL and separately licensed components; preserve package `COPYING`, `COPYING.LIB`, library/module notices, and required source material if redistributing |
| Bundled OVMF/EDK II | Firmware executed by QEMU | Multiple upstream notices in `work/tools/qemu/share/edk2-licenses.txt`; not covered by the first-party reservation |
| xorriso 1.5.8.pl02 and MSYS2 runtime | ISO construction | GPL/LGPL and other bundled terms; package versions, license declarations, and dependency metadata retained in [P1-01 evidence](evidence/phase-1/P1-01/2026-10-18/package-metadata/) |
| 7-Zip 26.04 | Portable extraction | Preserve the release's license texts, including applicable LGPL/BSD/unRAR conditions for its actual components; `7zr` and full 7-Zip have different notices |
| MSVC and Windows SDK | Existing Windows host linker/SDK | Microsoft terms govern installed tools; no redistribution right is inferred |
| Node.js, Git, PowerShell, GitHub Actions | Repository automation | Their upstream terms apply; these tools are not automatically guest dependencies |

Native tool archives and installed packages remain under ignored `work/`; their
presence there is not permission to publish them. Never copy an executable without
its runtime and corresponding notice review. This repository does not redistribute
the downloaded tool archives as tracked source.

## Candidate experiments

The isolated dependency experiment includes x86_64 0.15.5, tiny-skia 0.12.0,
rustybuzz 0.20.1, fontdue 0.9.4, and Slint 1.18.1 with locked transitives.
Their exact declared terms are in the inventory. Compilation is not adoption or
distribution approval. Slint's GPL/commercial/royalty-free alternatives need a
specific reviewed choice compatible with the intended product; no selection is
made here. No font, model, or artwork license may be inferred from a renderer's
license. Rejected binding 0.6.5 is retained only as failure/provenance evidence.

## Reference material and updates

[The research catalogue](docs/01-research/sources.md) identifies upstream sources.
Links, excerpts, license texts, and diagnostic output keep their underlying terms.
Maintainers must investigate attribution/rights concerns through the private
administrator route in [LEGAL.md](LEGAL.md), correct affected future artifacts,
and preserve an accurate audit trail. Release obligations are tracked in
[RELEASE.md](RELEASE.md), not declared satisfied by this index.
