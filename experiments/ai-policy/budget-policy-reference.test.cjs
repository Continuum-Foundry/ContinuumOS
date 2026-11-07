'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const {BudgetPolicyReference} = require('./budget-policy-reference.cjs');
const rule = (id, limit, scope = {}, meter = 'usd_micros') => ({id, limit, scope, meter, startsAt: 0, endsAt: 1000});
const source = () => rule('os-allowance', 100, {tenant: 'personal', user: 'alice'});
const setup = (extra = {}) => new BudgetPolicyReference({
  budgets: [rule('os-policy', 90), rule('user', 60, {user: 'alice'}), rule('task', 50, {task: 'report'})],
  funders: [source()], ...extra
});
const request = (id = 'call-1', cost = 30, overrides = {}) => ({
  id, now: 10, quote: {usd_micros: cost, tokens: 12, requests: 1},
  context: {tenant: 'personal', user: 'alice', task: 'report', workspace: 'home', app: 'editor', route: 'public', funder: 'os-allowance'},
  ...overrides
});

test('unfunded local calls reject positive unrecognized financial meters', () => {
  for (const meter of ['eur_micros', 'provider_a_credits']) {
    const engine = setup({budgets: [], funders: []});
    const context = {...request().context, route: 'local'}; delete context.funder;
    assert.equal(engine.reserve(request('local', 0, {context, quote: {usd_micros: 0, [meter]: 1}})).reason, 'unfunded_meter');
    assert.deepEqual(engine.snapshot().requests, {});
  }
});
test('one funding entitlement cannot authorize another positive financial meter', () => {
  const engine = setup();
  assert.equal(engine.reserve(request('mixed', 1, {quote: {...request().quote, usd_micros: 1, eur_micros: 2}})).reason, 'unfunded_meter');
  assert.equal(engine.snapshot().ledgers['funder:os-allowance'].held, '0');
});
test('actual usage above the quote blocks new work even below all caps', () => {
  const engine = setup(); engine.reserve(request('small', 10)); engine.dispatch('small', 11);
  const result = engine.settle('small', {usd_micros: 11, tokens: 12, requests: 1});
  assert.deepEqual(result.quoteExceeded, ['usd_micros']);
  assert.equal(engine.snapshot().ledgers['budget:user'].spent, '11');
  assert.equal(engine.reserve(request('next', 1, {now: 12})).reason, 'price_bound_exceeded');
});
test('a quote overrun blocks queued dispatch even when the budget still fits', () => {
  const engine = setup(); engine.reserve(request('first', 10)); engine.reserve(request('queued', 10));
  engine.dispatch('first', 11); engine.settle('first', {usd_micros: 11, tokens: 12, requests: 1});
  assert.equal(engine.dispatch('queued', 12).reason, 'reauthorization_required');
  assert.equal(engine.snapshot().ledgers['budget:user'].held, '10');
});
test('an unexpected charge on a free local call is recorded and closes admission', () => {
  const engine = setup({budgets: [], funders: []});
  const context = {...request().context, route: 'local'}; delete context.funder;
  engine.reserve(request('local', 0, {context})); engine.dispatch('local', 11);
  engine.settle('local', {usd_micros: 2, tokens: 12, requests: 1});
  assert.equal(engine.snapshot().settledUsage.local.usd_micros, 2);
  assert.equal(engine.reserve(request('again', 0, {context, now: 12})).reason, 'price_bound_exceeded');
});
test('route inventory is bounded before constructing a route set', () => {
  assert.throws(() => setup({routes: Array.from({length: 33}, (_,i) => `route-${i}`)}), /route bound/);
});

test('bundled allowance funds a call without a personal payment account', () => {
  const engine = setup(); assert.equal(engine.reserve(request()).allowed, true);
  assert.equal(engine.dispatch('call-1', 11).allowed, true);
  engine.settle('call-1', {usd_micros: 20, tokens: 10, requests: 1});
  assert.equal(engine.snapshot().ledgers['funder:os-allowance'].spent, '20');
});
test('all applicable ceilings bind and denial leaves every ledger unchanged', () => {
  const engine = setup(); const before = engine.snapshot();
  assert.equal(engine.reserve(request('large', 51)).reason, 'budget_exhausted');
  assert.deepEqual(engine.snapshot(), before);
});
test('parallel admissions cannot each spend the same available task allowance', async () => {
  const engine = setup();
  const results = await Promise.all([Promise.resolve().then(() => engine.reserve(request('a'))),
    Promise.resolve().then(() => engine.reserve(request('b')))]);
  assert.equal(results.filter(r => r.allowed).length, 1);
  assert.equal(engine.snapshot().ledgers['budget:task'].held, '30');
});
test('switching payer does not evade the independent user ceiling', () => {
  const other = {...source(), id: 'enterprise', limit: 1000};
  const engine = setup({funders: [source(), other]});
  assert.equal(engine.reserve(request('a', 40, {context: {...request().context, task: 'one'}})).allowed, true);
  const changed = {...request().context, task: 'two', funder: 'enterprise'};
  assert.equal(engine.reserve(request('b', 30, {context: changed})).reason, 'budget_exhausted');
});
test('funding cannot cross the declared owner boundary', () => {
  const engine = setup();
  assert.equal(engine.reserve(request('other', 1, {context: {...request().context, user: 'bob'}})).reason, 'funding_not_permitted');
});
test('a free local route needs no payer; an unconfigured cloud route does', () => {
  const engine = setup();
  const local = {...request().context, route: 'local'}; delete local.funder;
  assert.equal(engine.reserve(request('local', 0, {context: local})).allowed, true);
  assert.equal(engine.reserve(request('remote', 0, {context: {...local, route: 'public'}})).reason, 'funding_required');
});
test('tokens and money are separate dimensions with separate ceilings', () => {
  const engine = setup({budgets: [rule('tokens', 10, {}, 'tokens')]});
  assert.equal(engine.reserve(request()).reason, 'budget_exhausted');
  assert.equal(engine.snapshot().ledgers['funder:os-allowance'].held, '0');
});
test('missing meter quotes fail closed', () => {
  const engine = setup({budgets: [rule('energy', 20, {}, 'energy_units')]});
  assert.equal(engine.reserve(request()).reason, 'missing_meter_quote');
});
test('identical request IDs never reserve twice; changed context is rejected', () => {
  const engine = setup(); engine.reserve(request());
  assert.equal(engine.reserve(request()).reason, 'duplicate');
  assert.equal(engine.snapshot().ledgers['budget:user'].held, '30');
  assert.throws(() => engine.reserve(request('call-1', 31)), /mismatch/);
});
test('revocation after reserve blocks dispatch without pretending to refund remote work', () => {
  const engine = setup(); engine.reserve(request()); engine.disableFunder('os-allowance');
  assert.equal(engine.dispatch('call-1', 11).reason, 'reauthorization_required');
  engine.cancelBeforeDispatch('call-1');
  assert.equal(engine.snapshot().ledgers['budget:user'].held, '0');
});
test('route disable blocks queued calls and new calls', () => {
  const engine = setup(); engine.reserve(request()); engine.disableRoute('public');
  assert.equal(engine.dispatch('call-1', 11).allowed, false);
  assert.equal(engine.reserve(request('call-2', 1, {now: 12})).reason, 'route_disabled');
});
test('cancellation after dispatch retains the reservation pending actual outcome', () => {
  const engine = setup(); engine.reserve(request()); engine.dispatch('call-1', 11);
  engine.uncertain('call-1');
  assert.throws(() => engine.cancelBeforeDispatch('call-1'), /reconciled/);
  assert.equal(engine.reserve(request('call-2', 25, {now: 12})).reason, 'budget_exhausted');
  engine.settle('call-1', {usd_micros: 10, tokens: 5, requests: 1});
  assert.equal(engine.reserve(request('call-3', 25, {now: 13})).allowed, true);
});
test('duplicate receipts do not double charge and conflicting receipts reject', () => {
  const engine = setup(); engine.reserve(request()); engine.dispatch('call-1', 11);
  const actual = {usd_micros: 10, tokens: 3, requests: 1};
  engine.settle('call-1', actual); const before = engine.snapshot();
  assert.equal(engine.settle('call-1', actual).duplicate, true);
  assert.deepEqual(engine.snapshot(), before);
  assert.throws(() => engine.settle('call-1', {...actual, usd_micros: 11}), /mismatch/);
});
test('provider overrun is recorded honestly and blocks subsequent admission', () => {
  const engine = setup(); engine.reserve(request()); engine.dispatch('call-1', 11);
  assert.ok(engine.settle('call-1', {usd_micros: 70, tokens: 20, requests: 1}).overruns.includes('budget:user'));
  assert.equal(engine.snapshot().ledgers['budget:user'].spent, '70');
  assert.equal(engine.reserve(request('next', 1, {now: 12})).reason, 'budget_exhausted');
});
test('window expiry does not reset ledgers or allow stale held calls to start', () => {
  const engine = setup(); engine.reserve(request());
  assert.equal(engine.dispatch('call-1', 1000).allowed, false);
  assert.equal(engine.reserve(request('expired', 1, {now: 1000})).reason, 'policy_window_unavailable');
  assert.equal(engine.snapshot().ledgers['budget:user'].held, '30');
});
test('clock reversal, fractional prices and unsafe integer quotes reject', () => {
  const engine = setup(); engine.reserve(request());
  assert.throws(() => engine.reserve(request('reverse', 1, {now: 9})), /clock reversal/);
  assert.throws(() => engine.reserve(request('fraction', 0.5)), /integer/);
  assert.throws(() => engine.reserve(request('overflow', Number.MAX_SAFE_INTEGER + 1)), /integer/);
});
test('terminal deduplication inventory is bounded and never silently evicted', () => {
  const engine = setup({maxReservations: 1}); engine.reserve(request()); engine.cancelBeforeDispatch('call-1');
  assert.equal(engine.reserve(request('new', 1)).reason, 'reservation_inventory_full');
});
test('caller mutation and diagnostic mutation cannot alter stored policy or reservations', () => {
  const funder = source(); const engine = setup({funders: [funder]}); funder.limit = 999;
  const input = request(); engine.reserve(input); input.quote.usd_micros = 0;
  const view = engine.snapshot(); view.ledgers['budget:user'].held = '0';
  assert.equal(engine.snapshot().ledgers['budget:user'].held, '30');
  assert.equal(engine.snapshot().ledgers['funder:os-allowance'].limit, '100');
});
test('an overrun in one call also stops previously reserved work before dispatch', () => {
  const engine = setup({budgets: [rule('user', 60)]});
  engine.reserve(request('first', 20)); engine.reserve(request('queued', 20));
  engine.dispatch('first', 11);
  engine.settle('first', {usd_micros: 50, tokens: 12, requests: 1});
  assert.equal(engine.dispatch('queued', 12).reason, 'reauthorization_required');
});
test('omitting a scoped attribute cannot make its policy disappear', () => {
  const engine = setup(); const context = {...request().context}; delete context.workspace;
  assert.throws(() => engine.reserve(request('omitted', 1, {context})), /missing context/);
});
test('provider credits are their own meter and cannot be substituted for dollars', () => {
  const credits = {...source(), id: 'promo', meter: 'provider_a_credits', limit: 4};
  const engine = setup({funders: [credits]});
  const context = {...request().context, funder: 'promo'};
  assert.equal(engine.reserve(request('missing', 0, {context})).reason, 'missing_meter_quote');
  const quote = {usd_micros: 0, tokens: 12, requests: 1, provider_a_credits: 3};
  assert.equal(engine.reserve(request('credit-a', 0, {context, quote})).allowed, true);
  assert.equal(engine.reserve(request('credit-b', 0, {context, quote})).reason, 'budget_exhausted');
});
test('a zero personal-charge ceiling permits sponsorship without allowing personal charges', () => {
  const wallet = {...source(), id: 'personal-wallet', limit: 100};
  const engine = setup({funders: [source(), wallet], budgets: [
    rule('personal-charge', 0, {user: 'alice', funder: 'personal-wallet'})
  ]});
  assert.equal(engine.reserve(request('sponsored', 5)).allowed, true);
  const context = {...request().context, funder: 'personal-wallet'};
  assert.equal(engine.reserve(request('self-paid', 5, {context})).reason, 'budget_exhausted');
});
