# Custom candidate raster baselines

The [shared corpus](../ui-v1/README.md), profile and [fonts/text corpus](../text-v1/README.md)
remain the common inputs. This directory adds only reviewed custom-candidate
RGBA and semantic hashes. It changes no common scene, palette, action, state
expectation or tolerance. Slint uses its own reviewed raster baseline later.

`goldens.json` fixes all 150 comparison checkpoints after local Codex visual and
analytic review. `cargo xtask test --suite custom-ui` compares exact same-build
RGBA and semantic hashes and fails without updating this file. A future intended
pixel change needs a separate explicit review, retained failure and new hashes;
never regenerate the baseline to hide a mismatch. See the
[runbook](../../../docs/05-delivery/phase-2/custom-ui-runbook.md).
