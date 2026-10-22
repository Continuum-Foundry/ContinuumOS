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

P2-03's portable hosted editor model adds `unicode-segmentation` **1.13.3**,
defaults off, Unicode 17.0.0 tables, under **MIT OR Apache-2.0**. Exact checksum
is in `Cargo.lock`; original copyright and full license texts are preserved in
[its notice directory](licenses/third-party/unicode-segmentation-1.13.3/).
It is linked into host model tests/tools, with freestanding library compilation
checked separately; it is not integrated into the kernel/ISO or cleared for release.

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

P2-04's hosted text experiment now links cached rustybuzz 0.20.1 (MIT),
ttf-parser 0.25.1 (MIT OR Apache-2.0), fontdue 0.9.4 (MIT OR Apache-2.0 OR Zlib),
unicode-bidi 0.3.18 (MIT OR Apache-2.0), unicode-linebreak 0.1.5 (Apache-2.0),
unicode-script 0.5.8 (MIT OR Apache-2.0), and their exact locked transitives.
The [inventory](legal/dependency-inventory.json) records all resolved declarations;
[notice provenance](legal/notice-provenance.json) hashes verbatim cached notice
copies for the workspace graph. These are host experiments and separate library
compile checks, not kernel/ISO dependencies or release clearance.

The [font lock](fixtures/m2/text-v1/fonts.lock.json) pins unmodified Noto Sans,
Mono, Arabic, Hebrew, Devanagari and CJK SC assets under OFL-1.1. Their encoded bytes
remain local; OFL notices are retained for [Noto](licenses/third-party/noto-fonts/OFL.txt)
and [CJK](licenses/third-party/noto-cjk/OFL.txt). Official Unicode 17 test inputs
also remain local, with [Unicode License V3](licenses/third-party/unicode-data/LICENSE.txt)
retained. The independent shaping oracle executes the existing local QEMU
distribution's HarfBuzz DLL; no native library is newly redistributed here.

The isolated dependency experiment includes x86_64 0.15.5, tiny-skia 0.12.0,
rustybuzz 0.20.1, fontdue 0.9.4, and Slint 1.18.1 with locked transitives.
Their exact declared terms are in the inventory. Compilation is not adoption or
distribution approval. Slint's GPL/commercial/royalty-free alternatives need a
specific reviewed choice compatible with the intended product; no selection is
made here. No font, model, or artwork license may be inferred from a renderer's
license. Rejected binding 0.6.5 is retained only as failure/provenance evidence.

P2-06 now links the exact Slint 1.18.1 AOT/software configuration into an optional
host candidate, using the explicit P2-04 text Image adapter. Its GPL-3.0-only,
royalty-free 2.0 and commercial (cached text version 3.0.5) alternatives are
preserved verbatim in [the Slint notice directory](licenses/third-party/slint-1.18.1/),
with archive/file hashes in notice provenance. Active runtime and host compiler
features/checksums are separate in the candidate review. This is not a complete
redistribution bundle for every new transitive dependency. No distribution terms
are selected; OS shell/app versus public toolkit/API exposure, attribution and
applicable commercial/GPL obligations remain review questions before selection
or release. See the [Slint runbook](docs/05-delivery/phase-2/slint-ui-runbook.md).

P2-07 adds independent final-link probes for the actual custom and extracted
Slint AOT/text-image implementations, plus six embedded pinned Noto font inputs.
No registry version/checksum changed. The target runtime/build graphs, archive and
expanded-source verification, compiler-generated code identities and exact notice
hashes are recorded in the P2-07 input review; the dependency inventory now includes
both freestanding graphs. ELFs and font bytes remain local. This is not kernel/ISO
integration, a shipped-code SBOM or license clearance. Slint OS/API distribution
questions and all existing upstream obligations remain open. See the
[freestanding runbook](docs/05-delivery/phase-2/freestanding-ui-runbook.md).

## Reference material and updates

[The research catalogue](docs/01-research/sources.md) identifies upstream sources.
Links, excerpts, license texts, and diagnostic output keep their underlying terms.
Maintainers must investigate attribution/rights concerns through the private
administrator route in [LEGAL.md](LEGAL.md), correct affected future artifacts,
and preserve an accurate audit trail. Release obligations are tracked in
[RELEASE.md](RELEASE.md), not declared satisfied by this index.
