# Slint candidate raster baselines

Frozen after local review of the P2-06 exploratory run and the identical common
flat/clip checks at all 150 checkpoints. These are candidate-specific regression
hashes, not cross-toolkit quality acceptance or proof of native Slint text.

The software candidate uses Slint AOT primitives plus explicit P2-04 text Images.
Native blur, rounded ancestor clips and isolated launcher group opacity remain
failing comparison cases. Slint/native accessibility, IME, guest integration and
live replacement remain unproven. The acceptance command compares this file and
never regenerates it. See the [runbook](../../../docs/05-delivery/phase-2/slint-ui-runbook.md).
