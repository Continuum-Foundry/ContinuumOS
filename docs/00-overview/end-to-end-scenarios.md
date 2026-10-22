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
a fresh boot. Live persistence uses a declared data image; the M8 installer must
also establish its separately tested durable installed-disk layout.

## Operate under pressure

Many windows, large text, low memory or a slow app must not starve the whole desktop.
The system bounds queues/caches, refuses updates without headroom and uses a lower-cost
effects profile while retaining clear text/focus. Diagnostics account for both guest
and host costs. A failed app can recover without resetting kernel boot identity.

## Complete everyday file and desktop work — M9

Create a folder, save a document through a picker, copy/rename/move it and open it
through an explicit default-app association. Exercise conflicting names, denied
roots, a missing default app, interrupted copy and cancellation after partial
completion. Delete to trash and restore without silently overwriting another file.
Use the same workflow by keyboard and pointer. Clipboard/drag offers across two
apps expire or revoke without leaking access to unrelated files; notification
actions keep bounded focus/lifetime. Change locale/keymap/time zone, reboot and
verify settings while a wall-clock adjustment leaves monotonic timers correct.

## Manage native applications and ordinary updates — M10

Install an independently built SDK app from a verified offline bundle, inspect
its grants and launch it. An update requesting more authority cannot inherit it
silently. Reject bad publisher/metadata/compatibility and recover interrupted
activation. Stage a kernel/boot change with an explicit reboot-required result.
Uninstall once retaining and once removing selected app-owned data; unrelated user
documents and shared package references survive. Ordinary release updates do not
require enabling the development command endpoint.

## Restore after losing the guest disk — M11

Select document roots and portable settings, back them up to a distinct destination,
then restore into a freshly installed OS after removing the original guest disk.
Authenticate the destination owner, preview conflicts/ownership mapping and verify
the restored content/settings. Credentials, secrets and privileged policy are not
implicitly migrated. Corrupt, truncated, path-escaping, incompatible or unauthorized
restore inputs are refused; interrupted restores report incomplete outcomes without
silently replacing unrelated files. State whether the destination shares the host
failure domain; a second image alone is not host-loss backup evidence.

## Recover from a hung app and shut down — M11

Inspect CPU/memory/disk/service status, terminate a hung app under scoped authority
and reject an unsafe core-service restart. Export bounded diagnostics with private
data excluded by default. Shut down while writes/checkpoints are active; inject a
flush failure and a stuck app to test honest failure/force choices. On the next
boot, restore only compatible authorized app/document references, not old privilege,
and suppress crash loops. Autosave recovery is not falsely labeled explicit save.

## Connect and update without weakening trust — M12

Configure the supported virtual network, resolve DNS and establish validated TLS.
Inject packet loss, malformed traffic, incorrect time, entropy not ready, expired/
untrusted certificates and disconnect during an update download. The same package
verifier handles online and offline content; no bypass permits partial/unverified
activation. Network loss leaves offline apps/files usable and does not enable a
development listener.

## Hear assistive feedback and control capture — M13

Navigate apps by keyboard, observe correct semantic focus/value/actions and hear
validated assistive speech on the supported audio path/language matrix. Play/mix
bounded streams and choose a device. Deny/revoke microphone access and verify both
the trusted indicator and actual stream stop/cleanup. Inject underrun, overload,
device reset and service failure; record accessibility and continuity limits.

## Qualify the declared physical machine — M15

After B56 VM acceptance, inventory and select one named UEFI PC and required
devices. Under a separately explicit disk/write scope, test boot media, installed
boot, input/display/network/audio/power, independent backup and rescue. Unknown
devices/profiles are reported unsupported; a passing VM test is not substituted.

## Explore compatibility without declaring delivery — M16/M17

Inspect malformed and supported x64 PE fixtures, run only the bounded console/
DLL/runtime/GUI probes after their guest prerequisites, and test denied authority
and cleanup. Record missing APIs rather than returning fake success. Compare a
native browser port with the measured compatibility route; no-sandbox launch or
a screenshot cannot demonstrate browser safety. Produce a subset/route proposal
or evidenced stop/replan and request a separate product decision before promising
general application/browser delivery.
