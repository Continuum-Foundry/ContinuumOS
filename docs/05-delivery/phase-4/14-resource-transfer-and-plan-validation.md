# P4-14 — Stage and validate bounded resource plans

Issue type: Story. Epic: P4. Status: Not started. Owner: Updates/Security/UI.
Depends on: P4-13. Backlog: B15/B17 staging.
Requirements: R04 preparation/R10/R13/R14/R17. Test families: T05/T06/T07/T08/T11/T13.
Labels: `phase-4`, `staging`, `validation`.

## User story

As a guest desktop, I need complete validated resource generations before activation
so malformed or incompatible transfers leave the current UI usable.

## Implementation scope

1. Implement explicit bounded manifest/chunk/seal/status codecs under P4-13 auth.
   Check total/chunk/decompressed bytes, offset arithmetic, exact coverage/digest,
   duplicate consistency and transaction quota before allocations. Disable compression
   initially and use a private staging namespace without archive path authority.
2. Validate strict metadata: UTF-8, duplicate keys, nesting, mandatory fields,
   protocol/layout/resource schema and base/target generation. Translate prototype
   examples into actual supported codecs; schema validity alone is not compatibility.
3. Permit only declared L0/L1 tokens/layout/assets for existing behavior. Reject
   executable/helper images, callbacks/pointers, unknown actions and attempts to
   mutate grants. Bound tree/layout complexity, image/font parser work and references.
4. Prepare complete paint/semantic/input reconciliation before publication, reserving
   old/new/checkpoint-free scratch and presentation headroom. Use process-local
   validation isolation and bounded deadlines; failure releases candidate state.

## Acceptance criteria

- [ ] Valid host-built generations arrive over the actual authenticated channel,
  seal by exact digest and prepare as a complete coherent candidate.
- [ ] Corrupt/truncated/inconsistent chunks, missing assets, wrong base/schema,
  unknown mandatory features, invalid geometry and quota pressure reject atomically.
- [ ] Schema-valid but semantically unsafe layout/action/authority plans and code
  payloads deny before publication; current document/window/visible generation remain.
- [ ] Full old/new live headroom is admitted; validator death/OOM/timeout and auth
  expiry cleanly discard staging without starving UI or serial recovery.
- [ ] Independent codec/semantic cases and mutation checks validate actual rejection
  reasons; protocol status distinguishes received, prepared and visible success.

## Tests and evidence

Exercise malicious lengths/depth, NaN/infinity where parsers admit them, cycles,
missing fonts/images, wrong target, duplicate/reordered chunks and resource floods.
Retain exact manifest/resource hashes, expected denials and before/peak/settled bytes.

## Exclusions and handoff

No executable behavior replacement or durable blob store. P4-15 receives an immutable
validated candidate and explicit lifetime/authority/status rules. Read
[update draft](../../06-contracts/update-wire-format.md) and
[hosted publication](../../06-contracts/hosted-resource-reload.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
