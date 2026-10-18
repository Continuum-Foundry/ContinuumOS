# ADR-0007: Bounded local development transport before TCP/IP

Status: Proposed. Date: 2026-10-17.

## Context and decision

UI live development should not wait for TLS/DNS/general networking. Use a tested
local QEMU byte-stream transport such as VirtIO console/serial, behind a transport
adapter with bounded framing, content verification, explicit dev authority and
session replay protection. Verify actual Windows host integration in M0/M4.

## Alternatives

Virtual Ethernet is broadly useful but adds stack and security dependencies. A
host-shared filesystem expands the guest/host interface and is not a complete
activation protocol. Reattaching an ISO cannot change running code.

## Consequences

Some configurations require a guest transport driver and host pipe/socket adapter.
Local-only is not synonymous with authenticated. The development channel is disabled
in release images. Guest input must never become host shell commands or arbitrary paths.

## Gate and reconsideration

M4 tests transfer, malformed lengths, replay, reconnect and credential revocation.
If the platform path is impractical, select a constrained virtual network transport
without weakening the update transaction/security contract.

Evidence: [S32](../01-research/sources.md#s32), [network design](../02-architecture/networking.md).
