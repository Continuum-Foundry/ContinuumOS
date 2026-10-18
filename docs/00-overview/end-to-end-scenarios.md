# End-to-end scenarios and expected outcomes

These scenarios connect the subsystem specifications into product behavior.
They are acceptance journeys, not descriptions of an existing implementation.

## Cold boot and first work

The user launches a declared VM profile from the standalone ISO. Firmware/loader
handoff reaches our kernel, serial logs identify the build/boot, services start with
granted authority, and the desktop becomes interactive. The UI clearly indicates
ephemeral or persistent storage mode. The editor opens a document through granted
file-service authority; saving reports actual durable success or a useful error.

## Change appearance while working

With unsaved text open, the developer edits colors/spacing and saves source/assets.
Host validation/build completes, the guest validates a resource generation, and the
compositor/UI commits it at a safe boundary. Window, document, focus and caret remain.
No OS reboot occurs. A malformed theme leaves the previous theme visible with a
diagnostic. Reduced-motion/high-contrast overrides continue to work.

## Change app behavior while typing

Host builds a new editor instance and plan. The stable proxy stops new mutations,
old work drains, a checkpoint captures the accepted text revision, candidate imports
it with restricted authority, and compatible routes/resources switch to a new epoch.
Buffered input resumes in order. Old execution retires after no references remain.
The developer sees new behavior and precise build/epoch status. Kernel uptime persists.

## Reject a fundamentally incompatible edit

A changed schema has no supported migration, or a component now requires a different
kernel ABI. The planner rejects before activating and reports the missing migration
or reboot requirement. Existing work remains usable. The system does not silently
restart the VM or discard the document to produce a clean-looking demo.

## Recover from failed activation

Before a durable activation decision, failure returns to the old owner. After a
decision, recovery honors the decided epoch and state journal. If the new service
fails after irreversible writes, preserve that data and use a compatible forward fix
or recovery UI. Retaining an old binary is not sufficient justification for rollback.

## Disconnect the host

The OS continues ordinary use offline. Incomplete staged transfers expire; decided
transactions finish/recover under guest supervision. On reconnect, the host queries
transaction status instead of blindly retransmitting activation. A release image
does not accept development commands without explicit dev authority.

## Package the current system

The developer selects a coherent generation. The host packages an ISO with exact
content, build metadata, notices and recovery tools. A fresh VM boots it without host
shares/network. User documents are not bundled by accident. ISO identity and package
graph can be compared to the live development session; a different kernel may require
a fresh boot. Persistence is supplied by a separate declared data image.

## Operate under pressure

Many windows, large text, low memory or a slow app must not starve the whole desktop.
The system bounds queues/caches, refuses updates without headroom and uses a lower-cost
effects profile while retaining clear text/focus. Diagnostics account for both guest
and host costs. A failed app can recover without resetting kernel boot identity.
