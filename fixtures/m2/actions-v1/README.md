# Hosted editor action/state fixture 1

Synthetic data only. [Contract](../../../docs/06-contracts/hosted-editor-model.md)
and [runbook](../../../docs/05-delivery/phase-2/document-actions-runbook.md).

`replay.json` extends the P2-02 scene's initial text/revision/selection with 21
explicit requests and independent expected states. It exercises every registered
operation, including history, settings and synthetic session state. It consumes
committed operations; physical input/composition/gesture mapping is still candidate
work. It does not alter the locked `ui-v1` reference corpus.

`initial.hex` is the exact schema-1 initial checkpoint, authored independently
using Node Buffer integer primitives from the field specification. Rust encoding
and decoding must agree with it. Final checkpoints are independently decoded by
PowerShell. `lock.json` records exact input/schema byte hashes; intentional changes
need review, never automatic regeneration after a failed assertion.
