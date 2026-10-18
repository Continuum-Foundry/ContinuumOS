# Power, audio and whole-system lifecycle

Status: staged design. R06, R09, R12, R15.

## Idle and clocks

An idle scheduler uses the architecture's safe halt/wakeup path; it does not busy
poll for UI or device work. Services wait on events and bounded timers. Coalesce
nonurgent timers where it does not violate semantics. A clock widget, caret and
animation each declare their cadence. Measure wakeups rather than describing a
desktop as idle merely because its wallpaper is static.

Separate monotonic time for deadlines, wall time for human timestamps/certificates,
and display timing for presentation. Handle clock adjustments explicitly. Boot ID
and uptime are kernel-owned so a UI/service restart cannot reset the evidence of
machine continuity.

## Shutdown and reboot

Orderly shutdown blocks new updates, resolves or aborts the active transaction,
requests app checkpoints/saves within deadlines, flushes authoritative storage,
stops devices safely, and invokes the platform poweroff path. Do not mark unsaved
work saved if a flush failed. A stuck service cannot hang shutdown forever without
an explicit force option and clear consequences.

A kernel/boot update stages a new coherent generation and reports “reboot required.”
This is an honest supported update class. User-space live updating remains the
normal development loop, not a reason to pretend every change is safe live.

## Suspend/resume

Deferred for physical hardware. Suspend changes timer, device, GPU, storage and
network assumptions. A future design must freeze updates, quiesce devices, preserve
or discard sessions deliberately, reinitialize resources on resume and reconstruct
display/input state. A VM pause/snapshot is not proof of laptop suspend support.
ACPI is a platform reference, not an implementation of that policy.
[S29](../01-research/sources.md#s29)

## Audio

Reserve a versioned audio service interface, but do not block the first desktop on
it. A later implementation supports bounded PCM streams, explicit sample format,
rate/channel negotiation, shared ring ownership, underrun/overrun accounting and
per-app gain. A mixer and resampler have CPU and latency budgets. Real-time audio
needs scheduling analysis beyond “high priority.” VirtIO sound exists in the
device specification, but host/version support and our driver are separate gates.
[S32](../01-research/sources.md#s32)

Audio update policy initially drains/reopens streams and reports a discontinuity.
No claim of glitch-free mixer/driver replacement until tested under load. Microphone
capture needs explicit permission and a trusted visible indicator. Speech output
for accessibility depends on audio services and a chosen synthesis engine; a
semantic tree alone is not a working screen reader.

## Session recovery

Persist app identity, document references, geometry and compatible checkpoints with
quotas and privacy controls. On boot, restore only compatible trusted apps and
resources; missing packages or revoked permissions create recoverable placeholders.
Autosaved unsaved text is distinct from a user-saved document. Crash-looping apps
are not relaunched indefinitely.

## Later hardware programs

Battery/thermal policy, CPU frequency states, display power management, lid events,
hotplug, hibernation, Bluetooth and USB audio each need drivers, ownership rules,
failure tests and release support statements. They are recorded here so that an
experimental VM desktop is not confused with a complete laptop OS.
