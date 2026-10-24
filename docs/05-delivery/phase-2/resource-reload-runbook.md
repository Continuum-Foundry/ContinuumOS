# P2-11 — Hosted token and structural layout reload

Status: [local acceptance/self-review passes](../../../evidence/phase-2/P2-11/2026-10-23/README.md). The selected core is
custom Rust; Slint remains frozen comparison material. This is ephemeral hosted
L0/L1 data publication, with no guest transport or native behavior replacement.

```powershell
. ./tools/Enter-Dev.ps1
cargo build --locked --offline -p xtask --profile m2-comparison
pwsh -NoProfile -File tools/Preview-Custom.ps1 -ResourceReload
# The preview prints its editable resources.json path. Query its current generation
# in the window title; increment base_generation/generation and the token/layout
# generation fields together before saving the next complete edit.
pwsh -NoProfile -File tools/Preview-Custom.ps1 -ResourceReload -Replay
cargo xtask test --suite resource-reload
pwsh -NoProfile -File tools/Test-P2-11.ps1
```

The editable template retains the selected arranged 1280x720 baseline.
The editable template retains the selected arranged 1280x720 baseline.
The preview polls at 100 ms, hashes at most 128 KiB and waits for 100 ms of stable
content. Multiple saves coalesce into one pending hash; unchanged content does not
validate or repaint. A complete JSON object is required. Invalid/truncated saves
leave the last complete view editable. The title reports actual active generation,
outcome and rejection reason. Retrying unchanged rejected content requires an edit
or an explicit reload command; the host query remains authoritative after a lost
acknowledgement. These are local development controls, not authenticated IPC.

The version-1 envelope contains build ID, base/target resource generation, complete
tokens, standard variant, layout and ordered asset identities. Its canonical JSON
hash covers every field and all referenced immutable font hashes. Unknown fields,
versions, bindings, numeric overflow, missing/corrupt assets, duplicate IDs, cycles,
geometry and complexity violations reject. Six hash-checked startup Noto font
buffers are shared unchanged; adding/replacing fonts or image assets is outside
this bounded subset. `icon.missing` is a precise missing-image rejection.

L0 supports palette, spacing, radii and the existing font-size/effect/reduced-motion
settings. L1 supports the existing typed node data: add/remove/reparent/reorder,
rectangles, labels and consistent semantic/action bindings. The data uses absolute
authored geometry converted to parent-local anchored layout. The root follows the
viewport; other rectangles remain authored sizes/positions on resize. This is not
a general layout language. The desktop, editor-window and files-window shell
bindings remain required. The editor/search controls may disappear; their document
state remains owned by the live provider, with explicit focus fallback.

The [version-1 file contract](../../06-contracts/hosted-resource-reload.md) defines
fields, bindings and the atomic `resource-build SOURCE OUTPUT` command.

Publication runs on the serialized host event loop. Queued input finishes against
its old scene before staging. A bounded trial controller copy proves reconciliation
and a full unpublished paint/semantic snapshot; it never replaces the authoritative
provider. Only after success does live reconciliation and whole-scene publication
occur. Compatible composition survives; incompatible/removed focused identity and
moved/replaced capture cancel explicitly. Visual caret affinity is invalidated when
shaped geometry changes. Compatible opacity transitions transfer by stable ID;
caret/drag deadlines are rearmed in their existing monotonic clock domain, and
reduced motion settles transitions. Overflow follows the P2-10 counted reset policy.

At most three generation roots may remain live, including a staged candidate.
Each reserves a conservative 8 MiB for bounded JSON/tree data; 24 MiB is the total
reservation. Old roots are owned by reader handles and tracked through weak
references. Withheld readers cause a headroom rejection instead of early free.
Dead weak entries are collected before another admission. Startup fonts are shared
and counted separately. Staging pixels are capped at 16 MiB, selected paint scratch
at the existing 64 MiB; the bounded trial model/history copy and normal presentation
surfaces are separate host allocations. This is not process-wide OOM recovery,
a measured footprint, or the guest 256 MiB cold-idle result.

Request traces carry receive/validate/stage/publish/first completed host-frame
monotonic samples, build/request/generation IDs, input and generation digests,
cancellation/rejection reasons and resource accounting. Native replay separately
records builder cost, completed-build-to-Form.Update latency, process start times,
PID, session, native window handle/geometry, complete checkpoint hashes, semantics
and before/after PNGs. No cross-process timestamp subtraction or scanout timing is
claimed. Performance and 100-update stability acceptance belong to P2-12.
