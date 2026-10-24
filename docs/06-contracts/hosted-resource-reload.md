# Hosted resource generation, experimental version 1

This is P2-11's local file boundary for the selected custom UI. It is not a frozen
process ABI, executable update format, durable store or guest endpoint. Internal
Rust structs are never cast into file bytes. The host accepts one bounded JSON
object, then constructs and validates ordinary typed values.

| Field | Meaning and validation |
| --- | --- |
| `schema_version` | Exactly 1; unknown fields reject at every supported object level |
| `build_id` | Nonempty UTF-8 string, at most 64 bytes; diagnostic identity |
| `base_generation`, `generation` | Unsigned integers; target is base + 1, base must equal the actual active resource generation |
| `tokens` | Complete version-1 shared tokens, target generation; seven RGBA8 colors, bounded sizes/spacing/radii, fixed existing effect and motion algorithms, three pinned font slots |
| `variant` | `standard`, same palette as tokens, explicit reduced effects/motion booleans; user preferences still apply |
| `layout` | Complete shared version-1 node object and target generation; bounded tree, explicit IDs/children/rectangles/actions/semantics; authored revision pair is consecutive, publication uses the next local scene revision |
| `assets` | Ordered six `{name, sha256, bytes}` identities matching the actual hash-checked immutable startup fonts; no discovery, arbitrary asset paths, new fonts or image decode |

The canonical JSON SHA-256 covers the whole envelope, including build/base/target
identity and every immutable asset digest. A separate SHA-256 identifies exact
received bytes. Resource files are capped at 128 KiB; bounded parser recursion,
1024 nodes, 64 children per node and depth 32 limit complexity. Node geometry must
be finite and within the existing selected-core bounds. Reused control IDs must
have the specific kinds, focusability, roles and typed actions understood by the
selected adapter. Container/label nodes can be added, removed, reparented and
reordered; the required desktop/editor-window/file-list shell remains present.
Optional controls can disappear, with explicit input retirement/focus fallback.

Only the existing small spacing token directly controls text/control padding;
medium/large tokens remain checked data. The layout subset converts authored
absolute rectangles into parent-local anchored rules. Root extent follows the
host viewport; other sizes/positions are authored data. This is an explicit
reloadable structural subset, not runtime compilation of a general layout DSL.

`resource-template OUTPUT` emits an editable generation-7 starting object at the
selected arranged 1280x720 baseline.
`resource-build SOURCE OUTPUT` validates the source and pinned asset bytes, writes
a complete canonical temporary file and renames it over the completed output.
A failed build does not replace previous completed output. The preview can poll
that output or an editable resource file directly; debounce never makes incomplete
or invalid content eligible. A successful build still needs live base, headroom,
reconciliation, paint and semantic staging checks before publication.

Publication and status follow the [P2-11 runbook](../05-delivery/phase-2/resource-reload-runbook.md).
A single event-loop boundary publishes tokens, arranged tree and semantic revision.
The authoritative model/provider stays in place. Ordered queued input finishes on
old geometry; compatible composition and motion transfer, while changed capture
or focused identity cancels explicitly. Old generation readers own immutable roots;
weak retirement tracking and a three-root budget prevent an accumulating retired
list. Native bitmap copies own their pixels after Rust's surface reader releases.

Status contains actual active generation/digest, bounded request/build identity,
phase timestamps and rejection/cancellation reasons. Receive, validate, stage,
publish and first completed host frame use one monotonic clock. Native presenter
submission and builder durations are measured separately. A lost acknowledgement
requires querying actual active identity; replaying an old base rejects without a
second publication. No boot ID, kernel uptime, guest transport, persistence,
security boundary or native code replacement is claimed.
