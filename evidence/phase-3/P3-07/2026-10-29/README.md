# P3-07 - Bounded channels and atomic authority transfer pass locally

**P3-07 is Done under local Codex self-review for its declared experimental subset.**
This is not external approval or the M3 exit. Read the [criterion/unsafe review](review.json),
[channel runbook](../../../../docs/05-delivery/phase-3/channels-runbook.md) and
[ADR-0023](../../../../docs/04-decisions/0023-atomic-queued-authority-and-cycle-collection.md), which remains Proposed.

One fresh offline optical boot runs six tasks in two waves. Real CPL3 clients use
explicit little-endian envelopes, bounded nonblocking calls and typed errors.
Memory and endpoint moves commit bytes and attenuated references together at
enqueue; duplicates preserve their source. Receive copies outputs before atomic
handle installation/dequeue. Sizing, output-copy and full-table failures retain
the complete message. The shared return path now reserves ABI-query's size value
of 32 for that operation; channel sizing errors return scalar zero.

Trusted process-generation/principal/session metadata follows each actual sender
through cross-principal endpoint delegation and two apps of the same user.
Fabricated payload claims 999/777 never replace that metadata. Unknown mandatory
envelope features receive a typed service rejection. Queue count/byte limits,
invalid attachments, attenuation, stale handles and bad output spans are checked.
A receiver dies with accepted memory still queued; both waves receive memory after
sender death. A guest self-cycle and host cross-pair cycles release unreachable
references. Every wave exactly restores object/backing/frame/space/table/stack/pin
inventories, and healthy tasks continue in the same boot.

## Exact artifacts

Final channel run: `${WORKSPACE}/build/m1/channels-1793317358978529200-82332`.
Aggregate: `${WORKSPACE}/build/m3/P3-07/acceptance-1793313272454/acceptance.json`.
Source base: `39a0daafa0d287180959ec7830180a77988e9b88` plus
[dirty input hashes](source-provenance.json). Full binaries, ISOs, source patches,
disassembly and generated logs remain local under ignored build/work.
The tested snapshot includes the owner's three pre-existing Windows signing-policy
edits; these are outside the P3-07 staged change. No host policy was deployed.
Final documentation hashes are recorded separately from the tested code snapshot.

Build ID: `57d862121cebedbacc4d6cec3e90bcece5e161727569b99def0ab843090fb66d`.
Kernel SHA-256: `1a637684bbe9a88b1690e139b5986117a3816a3bb21f7e12dfd218fda02f3d0e`.
Exact launched ISO SHA-256: `343ff19560e834a64e4b7a6557fb12d7908591800cc65f3f34358ecab85232f1`.
Raw serial SHA-256: `559d26433f3b6beba7033861ad6c3063247734af1ce544ddc1b387d02c169937`.

The complete raw channel serial is retained in ordered 200000-byte-budget chunks
`channels-serial-0.txt` and `channels-serial-1.txt`; concatenate their bytes to
recover the exact raw hash above. No records are omitted. Firmware text remains
part of these original bytes. Launch/package context is in [launch-context.json](launch-context.json).

Pinned Q35/TCG/OVMF, one vCPU, 2 GiB VM capacity, unchanged 30000-ms guest deadline,
no NIC/shared filesystem, fresh firmware variables and reaped children. One kernel
entry and exact build/session correlation are checked. Conservative fixed-runtime
precharge is 10559512 bytes; this is not whole-guest idle memory.

## Gate and limits

All ten final-source prerequisites and 47 kernel host tests pass. Complete M1
passes 33 rows/42 launches, including two clean kernel/ISO builds with matching
kernel bytes and only the permitted ISO session-field difference. Current
1000-cycle memory, staged loader, four syscall-boundary groups (36 process runs),
paired scheduler (60 task runs), real missing-SSE2 CPU rejection and original
18-task capability regressions pass raw/artifact audits. Ordinary, boundary,
scheduler, capability and channel flavors build separately.

The channel reviewer checks 298 calls, 40 successful receives,
146 unchanged failure inventories and 601 dispatches.
The combined reviewer rejects 136 evidence mutations. Earlier failed
[development findings](development-findings.json) are retained separately and do
not count as acceptance. This compact bundle follows the retention policy.

The admitted subset precharges eight channel pairs, eight messages per direction,
4096 inline bytes and eight attachments per message, 16384 queued bytes and
sixteen queued references per direction. It retains the 16-owner/128-object/
32-handle registry. No send allocates kernel memory. Boot-wide handle nonce
exhaustion fails closed; host tests force it. Bounded tracing removes unreachable
queue cycles. Receiver-table admission happens at receive, after an accepted move
has already relinquished sender authority. Failed copy outputs are unspecified
until success; no handle is installed and no message consumed on failure.

READ attenuation does not revoke existing handles/mappings or freeze RW backing.
Sender metadata is historical attribution, not current session/policy validity.
One CPU and one owner thread per process are admitted. No blocking IPC, waits,
deadlines, current brokered policy/session enforcement, generic exactly-once,
universal revocation, independent frozen SDK, guest UI/reload or desktop efficiency
acceptance follows. P3-05 FP/debug limitations remain. P3-08 is next.
