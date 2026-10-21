# Hosted document state and action provider

Status: P2-03 experimental model, 2026-10-21. Action interface **1** (implicit
minor 0), logical state schema **1**, Unicode segmentation data **17.0.0**.
These version axes are independent. No permanent IPC or syscall IDs are assigned.
The [portable Rust crate](../../crates/editor-model/src/lib.rs),
[action schema](schemas/hosted-action.schema.json), [fixtures](../../fixtures/m2/actions-v1/README.md)
and [runbook](../05-delivery/phase-2/document-actions-runbook.md) define this slice.

## Ownership and editing

`Provider` owns a `Model`. Views borrow a model through `Adapter`; destruction of
an adapter cannot destroy its document. Button, shortcut, semantic and automation
entries call the same executor. Their origin is diagnostic, never authority.
No toolkit objects, callbacks, native addresses or threads enter model state.

- Document state: positive u64 logical ID and monotonic content revision, UTF-8
  text, directional anchor/caret byte offsets, saved comparison text, undo/redo
  frames, logical scroll offset and search query. Dirty means current text differs
  from the saved comparison text. There is no save/file operation in this slice.
- Preferences: theme, text size, reduced motion and reduced effects.
- Session: logical focus ID, integer window rectangle, synthetic file scroll and
  selection. These fields are explicitly separate from document state.
- Transient composition, capture, pressed keys and pointer gestures belong to the
  future input/view adapter and are absent from checkpoints. P2-05/P2-06 implement
  their actual routing/reset behavior; this story replays committed operations.

Text is at most 65,536 bytes; each insertion/query is at most 4,096 bytes.
Anchor/caret values must be extended-grapheme boundaries, preserving direction.
Insert replaces selection; delete removes selection (collapsed delete is a no-op).
Backspace/forward-delete key interpretation and visual caret mapping are later
adapter/text work. Insertion/deletion can join neighboring clusters; caret snaps
right to the next boundary in the resulting text. Undo/redo restores text and
selection, increments content revision and recomputes dirty. A new edit clears
redo. Each history holds at most 32 complete frames, evicting the oldest frame;
text itself is never truncated. Search is exact, case-sensitive, non-overlapping
UTF-8 matching; empty query has zero matches. Match spans can be inside graphemes
and must be expanded/rejected before a later selection action. No normalization,
word navigation, shaping, bidi mapping or font conformance is implied.

Every changed operation increments a separate model revision, including selection,
search, settings and session changes. Content revision increments only for an
edit/undo/redo. A no-op returns `Ok(false)` without advancing either revision.
Checked exhaustion returns `Exhausted`; neither counter wraps. A bounded candidate
model is validated before publication; all errors preserve the active model.
This clone-based implementation has bounded but unmeasured allocation/copy costs.

## Action interface 1

Each `Request` carries exact version, caller ID, monotonically increasing per-caller
request ID, provider epoch, document ID, expected document AND model revisions,
monotonic deadline, optional issued context ID, and typed action. Fixture JSON
integers are limited to 2^53-1 for exact cross-language representation; native
counters/checkpoints are u64. JSON schema character limits are supplemented by
the executor's UTF-8 byte limits. Missing/unknown envelope fields or wrong payload
types reject in the host decoder. Typed Rust values cannot contain malformed UTF-8.

| Names (all `@1`) | Payload | Modeled right / effect |
| --- | --- | --- |
| `edit.insert`, `edit.delete`, `edit.select` | UTF-8, null, anchor/caret pair | edit / document |
| `edit.undo`, `edit.redo` | null | edit / document |
| `search.set`, `document.scroll` | UTF-8 query, u32 logical offset <=1,000,000 | edit / document |
| `settings.theme`, `settings.text-size` | light/dark/high-contrast, integer 8–72 | preferences / preference |
| `settings.motion`, `settings.effects` | boolean reduced flag | preferences / preference |
| `files.scroll`, `files.select` | synthetic index 0–9999 | session / session |
| `launcher.open` | null; focus existing editor, preserve document | session / session |
| `session.focus`, `session.window` | registered focus ID, four i32 coordinates | session / session |

Window coordinate magnitudes are <=16384 with positive width/height. Focus IDs are
editor, search, files-window, launch-editor, reduced-effects and reduced-motion.
Input/output schema identity is this draft's action-name/version plus typed payload
and `Outcome { status, document_revision, model_revision }`. Only local logical
effects are supported. There is no preview/compensation, asynchronous execution,
external uncertain result, durable success or workflow engine.

Validation order is bounded payload, supported interface/action, registered caller
and required rights, epoch/document, nonzero request ID, duplicate lookup, high-water
check, cancellation, deadline, revisions, context and action-specific invariants.
Errors are `Bounds`, `Unsupported`, `Denied`, `Conflict`, `Invalid`,
`DuplicateMismatch`, `RetryExpired`, `Cancelled`, `Deadline`, `Expired` or
`Exhausted`. Outcomes always include authoritative revisions at that result.
Host code supplies monotonic time and cancellation status at invocation; the
synchronous mutation is atomic with respect to that host call. Cancellation/deadline
before execution changes nothing. After a completed edit a retry returns its original
result, even after its deadline or a later cancellation; cancellation never undoes
an accepted edit. This does not establish multi-thread preemption or wall time.

At most 16 caller records and 32 global FIFO result records exist per provider.
Cache keys are caller/request ID; the entire request must match on retry, including
payload, revisions, deadline and context ID. An exact cached retry returns its
original result/revisions after rechecking current caller rights and epoch/document.
A changed request under a retained ID returns `DuplicateMismatch`. A per-caller
high-water ID prevents an evicted/older request from executing again (`RetryExpired`).
Callers serialize IDs; out-of-order lower IDs are not queued. Failures after the
high-water check are retained too. Earlier authorization/version failures do not
consume IDs. Grant updates preserve high-water marks; identity exhaustion needs a
new explicitly managed session, never a counter reset.

## Scoped context model

The trusted host owner issues `Context` descriptors; callers supply only a context
ID. Metadata includes positive ID, caller audience, provider epoch, document ID,
source content revision, owner-supplied provenance (1–64 bytes), effect purpose,
one permitted action name, monotonic expiry, directional source selection and
exact UTF-8 byte size. Selection must be grapheme-aligned and <=4096 bytes.
At most 16 live descriptors are retained; owner revocation releases a slot.
`resolve_context` returns only that excerpt after audience/rights/revision/expiry
checks. Execution additionally checks permitted action and effect. Context is
optional; ordinary callers already granted rights need no redundant context.
Unknown IDs, fabricated registration metadata, wrong audience, purpose/action,
revoked authority, expired lease or stale source reject. Provenance is never taken
from document instructions. Context does not grant or expand rights.

All IDs/rights/callers here are **hosted models, not kernel capabilities**. The
control plane (`grant`, `issue_context`, `restore`) is held by trusted host code;
untrusted same-process Rust code is not isolated by this API. M3 supplies secure
handles, IPC and authenticated caller identity; M5 supplies fencing and provider
replacement. Provider result records/high-water/authority remain outside the logical
checkpoint and are preserved only while that same provider lives. New-provider
replacement, result transfer and production replay safety are deliberately unsupported.

## Logical checkpoint schema 1

This experimental same-schema encoding is explicit little-endian bytes. Integers
have fixed sizes; there is no alignment/padding. `string` means u32 byte length then
exact UTF-8 bytes (no terminator). `frame` means text string, u32 anchor, u32 caret.
A history is u32 count followed by oldest-to-newest frames. Counts/lengths are
checked before allocation; total bytes must be <=16 MiB. Unknown state/Unicode
versions, malformed UTF-8, invalid enum/bool, impossible selections, excessive
bounds, truncation, trailing bytes and arithmetic failures reject.

| Order | Field and encoding |
| --- | --- |
| 1 | 8 ASCII bytes `COSDOC01`, u16 schema version 1 |
| 2 | three u8 Unicode version bytes 17,0,0; u32 exact total length (offset 13) |
| 3 | u64 model revision, u64 document ID, u64 document content revision |
| 4 | current frame, saved comparison string, undo history, redo history |
| 5 | u32 document scroll, query string |
| 6 | u8 theme (0 light, 1 dark, 2 high contrast), u16 text size |
| 7 | u8 reduced motion, u8 reduced effects (booleans exactly 0/1) |
| 8 | four i32 window values x,y,width,height |
| 9 | focus string, u32 synthetic file scroll, u32 selected file |

Dirty and search matches are derived, never independently trusted stored fields.
No authority, dedup records, native layout, toolkit objects, secrets or resources
are serialized. The fixture has an independently authored initial hex checkpoint;
PowerShell independently decodes final Rust checkpoints, including history.
Acceptance records SHA-256 outside the codec. These bytes are not authenticated
or durable and arbitrary semantically valid tampering is not detected by the codec.

`Model::decode` constructs detached state. Owner `Provider::restore` fully validates
before swapping, requires matching document identity, nondecreasing document/model
revisions, and identical state when the model revision is unchanged. Authority and
duplicate records remain with the current provider. Restoring an old checkpoint
cannot rewind accepted work. No migration between schemas is implemented. This
is a logical fixture for later comparison/handoff, not the complete authenticated
M5 transaction/checkpoint envelope or a live app-replacement result.

The pinned `unicode-segmentation` 1.13.3 uses extended grapheme iteration, defaults
off, without `std`. [Upstream API](https://docs.rs/unicode-segmentation/1.13.3/unicode_segmentation/)
and cached source were inspected; checksum/terms are retained in the Cargo lock,
rights inventory and [notices](../../licenses/third-party/unicode-segmentation-1.13.3/).
Portable `no_std`+`alloc` compilation is separate from guest linking/execution and
from P2-04's official text/font/shaping conformance gate.
