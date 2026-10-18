# Draft contract: development update wire format

Status: design draft, version 0.1. This is a framing proposal, not a new cryptographic
protocol. Authentication is a required separately reviewed transport/session layer.
Schema: [update-manifest](schemas/update-manifest.schema.json).
Illustrative fixture: [update example](examples/update-manifest.json).

## Framing

Initial stream header is 32 bytes, encoded little-endian by field, not a Rust struct:

| Offset | Size | Field |
| --- | --- | --- |
| 0 | 4 | Magic ASCII `OSU1` |
| 4 | 2 | Wire major version, initially 1 |
| 6 | 2 | Wire minor version, initially 0 |
| 8 | 4 | Header length, initially 32 |
| 12 | 4 | Payload length, at most 1 MiB |
| 16 | 8 | Session sequence number |
| 24 | 2 | Message kind |
| 26 | 2 | Flags; initially zero |
| 28 | 4 | Reserved; initially zero |

Reject unsupported major versions, oversized lengths, malformed kinds and illegal
flags before allocation. A bounded receiver handles partial reads and deadlines.
Sequence numbers are scoped to an authenticated session; they alone do not prevent
replay across sessions. Payload authentication covers header, payload and context
using the selected reviewed construction, not a plain hash used as a MAC.

## Message kinds (provisional numeric IDs)

`1 Hello`, `2 HelloReply`, `3 BeginPlan`, `4 NeedBlobs`, `5 BlobChunk`, `6 SealBlob`,
`7 Prepare`, `8 Activate`, `9 Abort`, `10 QueryStatus`, `11 Status`, `12 Error`.
The handshake discloses boot/session identity, protocol subset, limits, base generation,
development mode and capabilities. No mutating command is accepted before authority
is established. IDs are placeholders pending independent codec tests.

Manifest metadata uses strict UTF-8 JSON initially. Reject duplicate keys, excessive
nesting, invalid encodings, unknown mandatory fields and oversized strings/arrays.
JSON Schema validation does not establish canonical signing bytes, graph compatibility,
authority or payload integrity. For signed distribution metadata use its selected
standard's canonicalization/verification rules rather than JSON reserialization.

## Blob transfer

Chunks identify transaction, expected digest, total size and offset. Proposed chunk
size ≤64 KiB; object size ≤64 MiB; whole transaction and decompressed sizes have
separate quotas. Check offset+length overflow and coverage. Duplicates must match
previous bytes; reject overlapping inconsistent data. Seal only after complete size
and digest verification. No archive extraction paths from untrusted payloads escape
the staging namespace. Compression is optional and initially disabled.

## Plan semantics

A plan includes base/target generation, transaction ID, authenticated session/boot
binding, source revision, persistence mode, update class, components, dependency
graph digest and resource/checkpoint bounds. Per-component compatibility records
protocol/state versions, old/new epochs and migration/rollback policy. Fields in
the supplied schema are a minimum illustrative MVP subset; cross-field/graph rules
remain semantic validation and may reject a schema-valid plan.

`Prepare` may stage/quiesce under a deadline. `Activate` only applies a prepared
plan with the correct current base and valid authority. `Abort` is rejected after
the durable decision; recovery then uses a new explicit transaction/epoch.
`QueryStatus` is idempotent and authoritative after reconnect. Retries cannot execute
an activation twice. Status distinguishes visible, healthy, finalized, retired,
durable versus ephemeral, and reboot-required outcomes.

## Error payloads and privacy

Return stable error code, phase, transaction/component IDs, retryability and bounded
developer details. Do not leak secrets, raw kernel addresses or full documents by
default. Wrong credentials, digest mismatch, incompatibility, resource exhaustion
and unsafe rollback are distinct errors.

## Fixture limits

The example has synthetic IDs/digests and no executable package. It is for schema
and protocol design only. Passing structural validation is not evidence of a secure
wire implementation, compatible migration or successful live update.
