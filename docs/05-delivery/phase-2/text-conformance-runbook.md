# P2-04 — Pinned fonts and bounded text conformance

The [text contract](../../06-contracts/hosted-text.md) defines all support claims,
failure policy and candidate obligations. Raw fonts and official data are locked
but kept local. This is a text experiment, not a toolkit or guest renderer.

```powershell
. ./tools/Enter-Dev.ps1
pwsh -NoProfile -File tools/Supply-TextInputs.ps1
pwsh -NoProfile -File tools/Recheck-P2Prerequisites.ps1
pwsh -NoProfile -File tools/Test-P2-04.ps1
cargo xtask test --suite text
node audit/check-docs.cjs
node audit/check-governance.cjs
node audit/check-repository-hygiene.cjs
```

Supply exact assets before running. Explicit `Supply-TextInputs.ps1 -Download`
fetches only public upstream files using immutable URLs and locked SHA-256; no
project files are sent and no Cargo network setting changes. Existing wrong bytes
are never overwritten. Licenses are retained in `licenses/third-party/`.

Use the existing local Rust/signing/offline cache. Certificate-store access needs
the established unsandboxed execution path. The workspace lock adds only cached
shaping/font/bidi/linebreak dependencies already considered by Phase 0, plus
unicode-bidi/linebreak in the host graph. Review its exact features/terms first,
then explicitly refresh the existing signer lock enrollment with
`tools/windows/Initialize-DevCompiler.ps1`; no new certificate/host policy is used.
The runner now recognizes `text_core-<hash>.exe` in its existing build roots.

The harness checks formatting, Rust tests, freestanding library compilation,
independent native HarfBuzz references and P2-02/P2-03 regression corpora. Two
separate text runs must produce identical structured results, including hashes of
all 45 grayscale PGM previews. Official segmentation tests execute every row from
the supplied locked files. Authored checks cover fallback, bidi visual clusters,
wrapping, all document byte-selection boundaries, caret/hit equivalence, partial
ligatures, composition/relayout, cache eviction/scale changes and invalid bounds.
Inspect selected PNG conversions or the PGM images; passing algorithms alone does
not establish readable text or candidate quality. Both candidates rerun this corpus.

`python tools/Preview-Text.py RUN_DIRECTORY OUTPUT.png` losslessly converts selected
PGMs into a contact sheet using only Python's standard library. Columns are scale
1.0/1.5/2.0; rows are combining, ligature, mono spacing, Arabic, Hebrew, Indic prebase, Indic
conjunct, mixed bidi and CJK wrapping. This conversion adds no font or raster step.

Raw logs, previews, metadata and input hashes stay in ignored
`build/m2/P2-04/acceptance-<timestamp>/` and `run-<timestamp>/`. Acceptance records
compiler, source base/dirty inputs, host CPU/OS, font/data/corpus/dependency locks,
signed host executable and harness hashes. Retain a compact reviewed bundle only.
The first prerequisite run exposed an existing Test-M0 path-with-spaces omission
of `crates/`; it is corrected. A later check hit the intentional lock-signing
guard after dependency edits. Preserve those outcomes with development findings.

Library compile, hosted shaping/raster output, candidate rendering, native IME,
guest execution, actual reload and performance remain distinct. There is no guest
heap/link closure, GUI, full Unicode conformance or recoverable allocator failure.
P2-05/P2-06 receive the same corpus/fonts/model and explicit limitations. The
256 MiB guest cold-idle target remains unchanged and unmeasured.
