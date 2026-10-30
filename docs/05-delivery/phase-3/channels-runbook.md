# P3-07 bounded nonblocking channels

Experimental, unfrozen, one CPU and one owning thread per process. The [local acceptance review](../../../evidence/phase-3/P3-07/2026-10-29/README.md) passes this bounded subset.
This is not external approval or the M3 exit. Use the approved signer
and xorriso execution context, with the host awake. No host policy changes.

After dot-sourcing `tools/Enter-Dev.ps1`, focused guest bring-up is
`cargo xtask test --suite m1 --case channels`. This builds, packages and launches;
it is not a compile-only command. Full outputs stay under ignored build/work.

Final serial gate and independent review:

```powershell
pwsh -NoProfile -File tools/Test-P3-07.ps1
python tools/Review-P3-07.py --acceptance build/m3/P3-07/acceptance-<run>/acceptance.json
```

The gate runs ten prerequisites, the complete 33-row/42-launch M1 baseline,
owned memory, staged loader, four user-boundary groups, paired scheduler and
CPU-negative boots, the original capability fixture and the channel fixture.
Ordinary, user-boundary-test, scheduler-test and capability-test are separately
built by their cases; channel-test is additional. Host tests run serially with
a 16-MiB host test-thread stack for the precharged registry; this changes no
guest stack, VM capacity, serial bound or deadline. Acceptance requires final
source hashes to match across all these runs.

Two channel waves run six tasks: two separate apps for synthetic user A and an
app for synthetic user B. Negative local exchanges test real user-copy failures,
count/byte/table exhaustion, stale/attenuated authority and a self-referencing
endpoint queue. Cross-owner messages transfer memory and an endpoint, then
attribute user B and the second user-A app as the actual delegated senders.
Unknown mandatory envelope features return a typed service rejection. One wave
reaps a receiver with accepted memory still in flight; both receive queued memory
after sender death. Every wave must settle exactly. Host cases additionally force
handle nonce/pair admission, attachment quota, cross-pair cycles and codec errors.

## Limits and lifetime

The admitted subset precharges eight pairs, two FIFO queues per pair, eight
messages per queue, 4096 bytes and eight attachments per message, 16384 queued
bytes and sixteen queued handles per direction. No send allocates kernel memory.
The fixed storage is charged in full, alongside the existing 16-MiB runtime
envelope. Pair creation also needs two object slots and two handle slots.
Arithmetic and capacities are checked before any authority changes.

Each attachment is source handle u64, attenuated rights u64, mode u32
(0 move, 1 duplicate), reserved-zero u32, explicitly little endian. TRANSFER=512
is required for either mode; duplicate additionally requires DUPLICATE=2.
SEND=1024 and RECEIVE=2048 apply to Channel kind=4. Repeated source slots,
amplification, stale/foreign/wrong-type handles and unknown modes fail atomically.
P3-07-created memory includes TRANSFER; the historical P3-06 profile is unchanged.
Attenuation limits the delivered handle. It does not remove rights from existing
duplicates or mappings, or turn RW backing into an immutable snapshot. A READ
view of mutable backing can observe writes through other legal mappings; only
backing created with an immutable READ ceiling supports the snapshot claim.

Enqueue is the send acceptance point. Bytes and all retained references become
queue-owned together; moves invalidate the source only at that point. Source
handles remain intact on any rejected send. This conveys authority to whoever
legally receives that endpoint, without reserving a particular process table at
send time. Actual destination table/nonce admission occurs on receive. A full
receiver table leaves the message queued; it does not undo an accepted move.

Receive copies staged bytes, predicted fresh handle values and metadata before
installing all handles and dequeuing in one infallible commit. Sizing, table,
nonce and copy failures consume no message or authority. Output bytes may be
partially changed on copy failure and are unspecified unless status is success.
Overlapping output spans are rejected. No pointer, pin or callback survives a
dispatch. There is no separate acknowledgement phase: death before receive
commit retains the queue; death after commit drops installed handles normally.

Sender death preserves already accepted messages and historical metadata.
After peer closure queued messages drain before PeerClosed; an empty open queue
returns WouldBlock. Receiver endpoint loss discards its incoming messages and
their references. Duplicates keep endpoints alive. A bounded tracing pass roots
actual owner tables, mappings and supervisor lifecycle references, traverses
reachable incoming queues and discards unreachable queue cycles. Queue references
alone cannot keep abandoned endpoints alive. Closing one handle is not revocation.

## Draft syscall records

All existing revision/register/return checks apply. IDs remain experimental:

- 13 channel_create(self_process, output_ptr, 16): copy two u64 handles then commit.
- 14 channel_send(endpoint, record_ptr, 32): four u64s: payload pointer/length,
  attachment pointer/count. Copy input once before registry validation.
- 15 channel_receive(endpoint, record_ptr, 48): six u64s: byte pointer/capacity,
  handle pointer/count capacity, metadata pointer/capacity (at least 64 bytes).
- 16 channel_size(endpoint): RDX `(needed_handles << 32) | needed_bytes`.
  This is advisory and consumes nothing; other failures return scalar zero.

Receive metadata is eight explicitly encoded u64s: byte count, handle count,
sender process slot, process generation, bound principal, bound session reference,
service bit, reserved zero. The registry receives this binding only from trusted
launch admission; transferring an endpoint does not transfer caller identity.
Payload claims cannot replace these fields. These are historical references,
valid for attribution after sender death, never an authorization that the session
is currently live. Existing synthetic session references are boot-local and are
not rebound. Current policy/session revalidation remains P3-10.

The 48-byte service envelope stays separate from transport. The bounded codec
validates exact lengths/counts, reserved bits, protocol major/minor and required
features without allocation or native layout casting. Operation-specific payload
and authority policy belong to the service. No network, wait/deadline, universal
revocation, exactly-once execution, SDK freeze or live replacement is implemented.
