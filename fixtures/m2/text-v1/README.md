# Locked text comparison inputs

P2-04's [contract](../../../docs/06-contracts/hosted-text.md) defines scope,
versions, fallback order, mapping policy and bounds. `lock.json` freezes authored
corpus/composition fixtures, exact font/data manifests and independent native
HarfBuzz shaping expectations. Font/official test bytes remain local in
`work/text-inputs/`; no machine font discovery is allowed.
Git preserves these JSON input bytes verbatim (`.gitattributes`), so newline
conversion cannot invalidate their locked identities on a fresh checkout.

Run `pwsh -NoProfile -File tools/Supply-TextInputs.ps1` to verify supplied assets.
An explicit `-Download` retrieves only the public locked URLs; it does not override
Cargo's offline mode. A mismatched existing file fails rather than being replaced.
Use the [runbook](../../../docs/05-delivery/phase-2/text-conformance-runbook.md)
for complete acceptance. Both P2-05/P2-06 must run every case at every scale,
reporting shaping/bidi/fallback separately. Those candidate results are pending.
