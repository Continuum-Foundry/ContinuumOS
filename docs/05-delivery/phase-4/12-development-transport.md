# P4-12 — Transfer bounded bytes over a dedicated development channel

Issue type: Story. Epic: P4. Status: Not started. Owner: Platform/Host tooling.
Depends on: P4-03,P4-11. Backlog: B14.
Requirements: R10/R13/R14/R17. Test families: T04/T06/T08/T11.
Labels: `phase-4`, `dev-transport`.

## User story

As a developer, I need actual bounded host/guest byte transfer independent of
diagnostic serial so transport loss cannot remove the recovery path.

## Implementation scope

1. Implement the selected dedicated VM device and host adapter using exact pinned
   configuration. Restrict the host endpoint to the local approved channel, without
   a public listener, shared root, general network stack or guest-directed commands.
2. Implement framing/partial reads/writes, checked lengths, finite buffers, queue
   pressure, monotonic deadlines, cancellation and disconnect cleanup. Keep protocol
   status independent from logs and reject console output as update frames.
3. Transfer deterministic test blobs in both directions through the real device,
   validating byte identity across arbitrary chunk boundaries. This story admits
   diagnostic test traffic only; mutating updates await P4-13 authentication.
4. Bound host/guest work and process lifecycle, and expose typed errors on absent
   device, negotiation failure, EOF, stalled peer and reconnect. Do not replace the
   sole active transport driver over its own link.

## Acceptance criteria

- [ ] Real guest and host transfer exact bounded payloads through the selected
  device; synthetic in-memory pipes alone cannot pass B14.
- [ ] Split/coalesced/truncated frames, invalid lengths/versions, byte floods and
  full queues reject or backpressure within locked time/memory limits.
- [ ] Disconnect/reset/reconnect and hung peers leave UI/control/serial responsive
  and reclaim or explicitly quarantine device-owned buffers safely.
- [ ] Guest payloads cannot cause host command execution, path traversal, arbitrary
  file reads/writes or compiler invocation; host outputs use fixed approved paths.
- [ ] Dev-disabled configuration exposes no mutating endpoint; all owned host/VM
  processes are reaped in failure probes with precise diagnostics.

## Tests and evidence

Retain actual launch/device arguments, bidirectional digests, sequence/queue data,
reset completion and independent framing oracle results. Inject host death, missing
device, malformed descriptors, transport stalls and early guest exit. Keep secrets absent.

## Exclusions and handoff

Raw bytes are not authenticated updates. P4-13 supplies session authentication and
authority; P4-14 supplies resource staging. Read
[wire framing](../../06-contracts/update-wire-format.md#framing).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
