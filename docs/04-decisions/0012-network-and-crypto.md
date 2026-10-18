# ADR-0012: Isolated networking and maintained cryptography

Status: Proposed. Date: 2026-10-17.

## Context and decision

General networking is valuable but should not block first live UI updates. Build
an isolated network service after the local dev channel. Evaluate smoltcp and a
maintained TLS stack/provider; implement real clocks, entropy, trust anchors and
resource limits. Do not invent cryptography or disable certificate validation.

## Alternatives

Writing TCP/IP and TLS from scratch adds major risk unrelated to original kernel
identity. Porting an entire Unix network stack imports platform complexity. Exposing
unauthenticated development TCP for convenience violates the trust model.

## Consequences

Socket quotas, timer behavior, provider build dependencies, packet validation and
network-service recovery remain first-party obligations. Replacing the stack can
disconnect sockets; broker ownership only preserves resources that actually survive.

## Gate and reconsideration

After M6, test packet loss, malformed traffic, connection limits and TLS negative
cases on the selected version. A provider that cannot meet the no-OS/CPU/platform
requirements must be replaced or its dependency boundary documented explicitly.

Evidence: [S57](../01-research/sources.md#s57), [S58](../01-research/sources.md#s58),
[S59](../01-research/sources.md#s59), [networking](../02-architecture/networking.md).
