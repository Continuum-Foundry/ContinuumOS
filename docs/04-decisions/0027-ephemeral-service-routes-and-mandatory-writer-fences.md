# ADR-0027: Ephemeral routes and mandatory synthetic writer fences

Status: Proposed. Date: 2026-10-31. Story: P3-11.

## Context

A logical service name and an IPC endpoint identify destinations; neither proves
current authority to change state. A retained endpoint must not bypass a route
switch. P3-10 already supplies exact process-bound grants and finite sessions.
M3 needs a bounded guest experiment before P3-12 can freeze an independent SDK.

## Proposed decision

Use a separate CPL3 authority owning ephemeral policy, routes, pending requests,
results and a synthetic counter. A separate proxy forwards to two independently
launched private service processes. All mutation passes through the authority;
services receive no writable mapping of its counter. The trusted bootstrap alone
identifies the route administrator. Every commit rechecks the kernel-attributed
service, original caller, current policy/session, exact forwarded request and
writer epoch. The logical service ID is independent of process slots and handles.

Fence first: clear the writer and increment the route epoch. Grant a specific
live instance after checking its own grant. Publish the route separately. Between
these steps requests report unavailable; retaining an old endpoint preserves
transport connectivity but cannot preserve writer authority. Both service
instances remain alive during the tested switch.

Admission reserves one of four request slots and capacity for an immutable result.
Eight results live for one elapsed second. An exact pending duplicate reports
WouldBlock; a retained completed duplicate returns the same result. Changed
duplicates deny. Four non-evicting caller high-water marks prevent reordered or
forgotten IDs from becoming new mutations after result expiry. Exhaustion rejects
admission. This intentionally trades availability for bounded, explicit outcomes.

Cancellation before execution records Cancelled; after commit it returns the
stored result and does not undo the effect. A surviving authority records peer
failure or denial. Authority death loses ephemeral state and results: an in-flight
request may have committed, so the client reports unknown outcome and does not
automatically replay it. Trusted bootstrap creates fresh policy/resource identity.
No checkpoint restores authority. Independent private RW mappings keep their
normal kernel lifetime until owner teardown.

## Alternatives and consequences

Publishing a raw endpoint alone cannot enforce a resource fence. Giving each
service its own writable counter would require a different shared-state protocol.
Durable journaling and generic exactly-once external effects exceed M3; M5/M6
retain state migration, quiescence, retirement and durable activation gates.

This experiment uses first-party allocation-free Rust and existing IPC/waits.
It adds no syscall, guest compiler, UI, filesystem, credential or host policy.
The fixture controller supplies a deterministic failure schedule; it is not a
production discovery or supervision service. The contract remains unfrozen.

## Validation gate

The [story](../05-delivery/phase-3/11-proxy-routes-epochs-and-writer-fencing.md)
requires actual guest IPC, retained-endpoint denial, request/result correlation,
use-time authorization, bounded pressure, failure cuts and exact resource cleanup.
Use the [runbook](../05-delivery/phase-3/routes-runbook.md). Record measured results
separately; this ADR remains Proposed and grants no remote publication authority.
