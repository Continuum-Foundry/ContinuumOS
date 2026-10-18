# Networking and the development channel

Status: proposed. R06, R13, R14. Decisions: [ADR-0007](../04-decisions/0007-development-transport.md),
[ADR-0012](../04-decisions/0012-network-and-crypto.md).

## Development first

Bring up a bounded VirtIO console/serial byte stream or equivalent explicitly
configured local transport before a full TCP/IP stack. This lets us test live UI
updates without making DNS, TLS, certificate time and general networking prerequisites.
The transport adapter has frame lengths, maximum sizes, timeouts, checksums/content
digests and an authenticated session design; local reachability is not authentication.

Host binds management endpoints to local-only addresses/pipes with restricted
permissions. Do not expose QMP or GDB to untrusted networks. Guest messages never
name arbitrary host files or shell commands. The host sends only approved built
artifacts; pulling guest diagnostics requires an explicit bounded protocol.

Use a tested authentication construction/library, not a hand-rolled cipher. Early
trusted local tests may use an explicitly labeled development token or inherited
transport authority, with replay/sequence protection. Do not describe this as a
secure public network protocol. Before network exposure, implement the trust and
entropy gates in the security specification.

## General network architecture

An isolated network service owns TCP/IP state and driver access. Apps receive
restricted socket/service capabilities through policy. A stack such as smoltcp is
a candidate, but device polling, timers, buffers, routes and API integration remain
our responsibility. [S57](../01-research/sources.md#s57)

Initial feature order: virtual Ethernet → ARP/IPv4 → static configuration → UDP/TCP
→ DHCP/DNS → TLS/time/trust → IPv6 and broader compatibility. Every stage has packet
and malformed-input tests. No claim of internet readiness before robust limits,
retransmission behavior, entropy and certificate validation exist.

Driver IRQ work wakes the network service; no packet parsing or TLS handshake in
hard IRQ context. Bound packet pools, connection counts, retransmission storage,
DNS cache, handshake concurrency and per-app bandwidth. Timers use a monotonic
clock. Wall-clock changes must not corrupt transport timeout calculations.

## TLS and cryptography

Reuse maintained TLS and cryptographic implementations. Audit the exact provider's
build/architecture dependencies: a Rust TLS API does not mean every provider is
pure Rust or requires no platform services. Secure randomness, trust anchors and
time validation must be real. Do not bypass verification to make a demo connect.
[S58](../01-research/sources.md#s58), [S59](../01-research/sources.md#s59)

## Live replacement boundaries

Application service replacement can preserve a connection when a stable network
broker owns it and the protocol permits reassignment. Replacing the TCP stack itself
usually loses sockets unless its entire connection/timer/buffer state is migrated
and compatible. The initial policy is reconnect with an explicit interruption,
not a promise of invisible TCP continuity.

An update must not replace the sole transport required to deliver its own recovery
commands. Keep a serial rescue/control path, or classify that update as a restart/
reboot operation. Downloaded packages are verified before staging and cannot gain
authority merely by arriving through a trusted host.

## Acceptance

Test truncated frames, slow sender, reconnect, duplicate/reordered requests, quota
exhaustion, bad credentials, revoked dev authority, stack crash, and transport loss
during activation. Verify the OS remains usable offline and that a release image
does not silently listen for development updates.
