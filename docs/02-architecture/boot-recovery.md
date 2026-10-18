# Boot recovery and live-update recovery

Status: proposed. R02, R11, R12. This complements the live transaction specifications.

## Two independent paths

| Property | In-session live update | Boot-generation recovery |
| --- | --- | --- |
| Kernel | Existing kernel continues | Selected kernel boots/reboots |
| Main problem | Transfer live state, requests and authority safely | Select a complete bootable compatible generation |
| Commit evidence | Epoch, route publication and transaction journal | Durable boot target, attempts and health records |
| Failure response | Abort/forward recover/compatible reverse migration | Retry within limits, select known good, or rescue |
| Does not provide | Arbitrary new kernel without reboot | Preservation of volatile app stacks/state without checkpoints |

Passing one path's tests does not establish the other. A/B image layouts are a
possible boot-packaging implementation, not the in-session reload protocol.

## Proposed boot metadata

Store immutable generation references plus versioned, checksummed attempt/selection
records with sequence, candidate generation, previous known-good generation,
attempt count, health criteria/version, and completion result. Persist the candidate
and attempt record before booting it. Limit retries and never overwrite the sole
valid known-good content while staging another generation.

The loader or early supervisor resolves the selected generation according to the
declared boot mode. The kernel ABI/boot modules must be coherent. If using a fixed
ISO base plus writable overlay, an overlay needing a different kernel is rejected
or staged for an explicit new boot image; do not apply its user-space subset blindly.

## Health and fallback

Mark a generation healthy only after declared service/storage/desktop readiness
checks and required observation, not merely reaching kernel entry. Use persistent
attempts so repeated kernel crashes cannot retry forever. A health record includes
the exact generation and policy version. Missing/corrupt packages produce a reason
and a validated fallback, not a mixed graph.

User data remains separate. Before choosing old system code, check whether it can
read the current data/checkpoint schemas. If not, use a compatible rescue reader or
forward fix. Boot fallback never silently rewinds documents to match old code.

## A/B and signed distribution

After the simpler generation selector is reliable, compare dual system slots against
the existing immutable package graph for platform/installer needs. A/B adds slot
metadata, space costs, boot selection and update rules; two partitions alone do not
provide safe recovery. Public distribution requires a separate verified trust-root,
signature/anti-rollback/key-recovery design. Local developer fallback policy must be
distinguished from accepting attacker-supplied old packages.

## Tests

Fail staging/selection writes, interrupt first boot, crash before/after health marks,
exhaust attempts, corrupt one slot/generation, remove a required package, and try a
known-good binary against a newer incompatible user schema. Verify a rescue ISO can
inspect data without automatically changing it. Document persistence limits of
ephemeral live sessions.
