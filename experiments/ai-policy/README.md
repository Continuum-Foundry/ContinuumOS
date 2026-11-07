# AI budget policy reference

Retained on 2026-11-07 from the owner's AI planning review. First-party host
research fixture for [ADR-0031](../../docs/04-decisions/0031-ai-policy-and-funding.md)
and B68. No guest service, financial integration or production acceptance is claimed.

Run from repository root:

```text
node --test experiments/ai-policy/budget-policy-reference.test.cjs
```

The 28 cases cover scoped mandatory ceilings, sponsored versus personal payment,
atomic holds, separate units, child/task caps through supplied trusted scope,
deduplication, use-time policy/expiry/revocation, unknown outcomes, idempotent
settlement, honest overruns, terminal storage bounds and invalid input/clock checks.
Synchronous calls model atomic single-process admission, not distributed concurrency.

Inputs must already be authenticated and scope/price-bound by a trusted broker.
This model does not validate identity, permissions, source-use rules or tool effects.
It has no persistence, network, payments, authoritative sponsor ledger, period reset,
distributed consensus or guest resource accounting. Terminal records never silently
evict to permit duplicate dispatch. A production Rust service needs independent
contract/negative tests and durable/distributed recovery gates. Generated test
results stay under ignored `work/` or `outputs/`; these sources are reusable fixtures.

The audit adds six regressions for unfunded foreign/credit meters, unsupported
mixed financial meters, quote violations below caps, queued work after violation,
unexpected free-local charges and route inventory. The original 22 cases still
run. A fixed resource-meter vocabulary and one selected funding meter are the
reference's supported subset; other positive meters require matching funding.
Actual usage remains recorded even without a matching ledger. Any verified quote
violation closes new/queued admission; no production recovery/finance claim follows.
