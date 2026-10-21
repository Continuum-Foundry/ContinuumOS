# Shared hosted UI corpus, version 1

Authored comparison inputs for **both** custom and Slint candidates.
See the [normative experimental contract](../../../docs/06-contracts/hosted-ui-fixtures.md)
and [reproduction runbook](../../../docs/05-delivery/phase-2/shared-fixtures-runbook.md).
These are source fixtures, separate from generated acceptance evidence.

The seven positive JSON files have matching `hosted-*.schema.json` schemas.
`negative.json` applies one replacement/addition at a JSON pointer (empty pointer
replaces the root), or repeats the existing array for excessive-complexity tests.
Its `error` is the required semantic rejection class. This is a test harness
format, never a supported UI asset operation. Malformed nonfinite JSON and checked
integer overflow are tested directly in the Rust harness.

`lock.json` hashes exact fixture/schema/profile bytes and the generated 10,000-row
file list. The generator is in the host-only
[validator](../../../tools/xtask/src/ui_fixtures.rs). Run it twice to reproduce
the same data; no generated 10,000-row copy belongs in Git. Fixture changes require
deliberate lock review and must be supplied to both candidates together. There is
no automatic lock-refresh flag in acceptance.

Fonts are named slots, not a claim of licensed/available glyph assets. P2-04 must
bind exact font bytes/fallbacks and Unicode data once for both candidates. No
rendered golden, model/action implementation, native screen reader, guest IPC,
live reload or performance result is supplied by these fixtures.
