'use strict';

// Host research model only. Context, policy and price bounds must come from a
// trusted broker. This module does NOT authenticate callers or invoke providers.
const ATTRS = new Set(['tenant', 'user', 'workspace', 'app', 'task', 'route', 'funder']);
const NAME = /^[A-Za-z0-9_.:-]{1,96}$/;
// Trusted adapters must use the selected funder's meter for financial usage.
// This oracle has one funder per request; mixed financial meters are unsupported.
const RESOURCE_METERS = new Set(['tokens', 'requests', 'memory_bytes', 'compute_ms', 'energy_units', 'network_bytes']);
const copy = value => structuredClone(value);
function name(value) {
  if (typeof value !== 'string' || !NAME.test(value)) throw Error('invalid identifier');
  return value;
}
function integer(value) {
  if (!Number.isSafeInteger(value) || value < 0) throw Error('invalid nonnegative integer');
  return value;
}
function fields(value, allowed = null) {
  if (!value || Object.getPrototypeOf(value) !== Object.prototype) throw Error('invalid record');
  const keys = Object.keys(value);
  if (keys.length > 16) throw Error('record bound');
  for (const key of keys) {
    name(key);
    if (allowed && !allowed.has(key)) throw Error('unknown scope');
    if (allowed) name(value[key]); else integer(value[key]);
  }
  return copy(value);
}
const matches = (scope, context) => Object.entries(scope).every(([k, v]) => context[k] === v);
const canonical = object => JSON.stringify(Object.entries(object).sort(([a], [b]) => a.localeCompare(b)));

class BudgetPolicyReference {
  #budgets; #funders; #routes; #ledgers = new Map(); #requests = new Map();
  #revision = 1; #now = 0; #bound;
  #priceBoundExceeded = false;
  constructor({budgets = [], funders = [], routes = ['local', 'private', 'public'], maxReservations = 128}) {
    if (!Array.isArray(budgets) || !Array.isArray(funders) || budgets.length > 128 || funders.length > 32) throw Error('policy bound');
    if (!Array.isArray(routes) || routes.length > 32) throw Error('route bound');
    this.#bound = integer(maxReservations);
    if (this.#bound < 1 || this.#bound > 1024) throw Error('reservation bound');
    this.#routes = new Set(routes.map(name));
    this.#budgets = budgets.map(b => this.#configure(b, 'budget'));
    this.#funders = new Map(funders.map(f => [f.id, this.#configure(f, 'funder')]));
  }
  #configure(input, kind) {
    const rule = copy(input);
    name(rule.id); name(rule.meter); integer(rule.limit);
    rule.scope = fields(rule.scope || {}, ATTRS);
    integer(rule.startsAt); integer(rule.endsAt);
    if (rule.endsAt <= rule.startsAt) throw Error('invalid policy window');
    if (kind === 'funder' && !rule.scope.tenant && !rule.scope.user) throw Error('unscoped funding');
    const id = `${kind}:${rule.id}`;
    if (this.#ledgers.has(id)) throw Error('duplicate policy id');
    this.#ledgers.set(id, {meter: rule.meter, limit: BigInt(rule.limit), spent: 0n, held: 0n});
    return {...rule, ledger: id, enabled: true};
  }
  #tick(now) {
    integer(now);
    if (now < this.#now) throw Error('clock reversal');
    this.#now = now;
  }
  #window(rule, now) { return now >= rule.startsAt && now < rule.endsAt; }
  reserve({id, context, quote, now}) {
    name(id); this.#tick(now);
    context = fields(context, ATTRS); quote = fields(quote);
    for (const key of ['tenant', 'user', 'workspace', 'app', 'task', 'route']) if (!context[key]) throw Error('missing context');
    if (!Object.keys(quote).length) throw Error('empty quote');
    const fingerprint = `${canonical(context)}|${canonical(quote)}`;
    const old = this.#requests.get(id);
    if (old) {
      if (old.fingerprint !== fingerprint) throw Error('duplicate request mismatch');
      return {allowed: false, reason: 'duplicate', state: old.state, id};
    }
    const deny = (reason, policy) => ({allowed: false, reason, policy});
    if (!this.#routes.has(context.route)) return deny('route_disabled');
    if (this.#requests.size >= this.#bound) return deny('reservation_inventory_full');
    const applicable = this.#budgets.filter(b => matches(b.scope, context));
    let selectedSource;
    if (context.funder) {
      const source = this.#funders.get(context.funder);
      if (!source || !source.enabled || !matches(source.scope, context)) return deny('funding_not_permitted');
      selectedSource = source;
      applicable.push(source);
    } else if (context.route !== 'local' || (quote.usd_micros || 0) > 0) {
      return deny('funding_required');
    }
    for (const [meter, amount] of Object.entries(quote)) {
      if (amount > 0 && !RESOURCE_METERS.has(meter) && selectedSource?.meter !== meter) return deny('unfunded_meter');
    }
    for (const rule of applicable) {
      if (!this.#window(rule, now)) return deny('policy_window_unavailable', rule.ledger);
      if (!Object.hasOwn(quote, rule.meter)) return deny('missing_meter_quote', rule.ledger);
      const ledger = this.#ledgers.get(rule.ledger);
      if (ledger.spent + ledger.held + BigInt(quote[rule.meter]) > ledger.limit) return deny('budget_exhausted', rule.ledger);
    }
    if (this.#priceBoundExceeded) return deny('price_bound_exceeded');
    // Synchronous all-or-nothing reservation in this one-process reference.
    for (const rule of applicable) this.#ledgers.get(rule.ledger).held += BigInt(quote[rule.meter]);
    this.#requests.set(id, {fingerprint, context, quote, rules: applicable, revision: this.#revision, state: 'reserved'});
    return {allowed: true, id, policies: applicable.map(r => r.ledger)};
  }
  dispatch(id, now) {
    this.#tick(now);
    const request = this.#get(id);
    if (request.state !== 'reserved') return {allowed: false, reason: 'already_started_or_terminal'};
    const source = request.context.funder && this.#funders.get(request.context.funder);
    if (this.#priceBoundExceeded || request.revision !== this.#revision || !this.#routes.has(request.context.route) ||
        (source && !source.enabled) || request.rules.some(r => !this.#window(r, now) ||
          this.#ledgers.get(r.ledger).spent + this.#ledgers.get(r.ledger).held > this.#ledgers.get(r.ledger).limit)) {
      return {allowed: false, reason: 'reauthorization_required'};
    }
    request.state = 'dispatched';
    return {allowed: true};
  }
  uncertain(id) {
    const request = this.#get(id);
    if (!['dispatched', 'uncertain'].includes(request.state)) throw Error('not in flight');
    request.state = 'uncertain'; // Keep the entire reservation until reconciliation.
  }
  settle(id, actual) {
    const request = this.#get(id); actual = fields(actual);
    if (canonical(Object.fromEntries(Object.keys(actual).map(k => [k, 0]))) !==
        canonical(Object.fromEntries(Object.keys(request.quote).map(k => [k, 0])))) throw Error('meter set mismatch');
    if (request.state === 'settled') {
      if (canonical(actual) !== canonical(request.actual)) throw Error('settlement mismatch');
      return {duplicate: true};
    }
    if (!['dispatched', 'uncertain'].includes(request.state)) throw Error('not in flight');
    const overruns = [];
    const quoteExceeded = Object.keys(actual).filter(meter => actual[meter] > request.quote[meter]);
    if (quoteExceeded.length) this.#priceBoundExceeded = true;
    for (const rule of request.rules) {
      const ledger = this.#ledgers.get(rule.ledger);
      ledger.held -= BigInt(request.quote[rule.meter]);
      ledger.spent += BigInt(actual[rule.meter]);
      if (ledger.spent + ledger.held > ledger.limit) overruns.push(rule.ledger);
    }
    request.actual = actual; request.state = 'settled';
    return {duplicate: false, overruns, quoteExceeded};
  }
  cancelBeforeDispatch(id) {
    const request = this.#get(id);
    if (request.state === 'cancelled') return;
    if (request.state !== 'reserved') throw Error('remote outcome must be reconciled');
    for (const rule of request.rules) this.#ledgers.get(rule.ledger).held -= BigInt(request.quote[rule.meter]);
    request.state = 'cancelled';
  }
  disableRoute(route) { name(route); this.#routes.delete(route); this.#revision++; }
  disableFunder(id) {
    const funder = this.#funders.get(id);
    if (!funder) throw Error('unknown funder');
    funder.enabled = false; this.#revision++;
  }
  #get(id) {
    name(id);
    const request = this.#requests.get(id);
    if (!request) throw Error('unknown reservation');
    return request;
  }
  snapshot() {
    return {
      revision: this.#revision,
      priceBoundExceeded: this.#priceBoundExceeded,
      ledgers: Object.fromEntries([...this.#ledgers].map(([id, l]) => [id, {
        meter: l.meter, limit: l.limit.toString(), spent: l.spent.toString(), held: l.held.toString()
      }])),
      requests: Object.fromEntries([...this.#requests].map(([id, r]) => [id, r.state])),
      settledUsage: Object.fromEntries([...this.#requests].filter(([,r]) => r.state === 'settled').map(([id,r]) => [id, copy(r.actual)]))
    };
  }
}
module.exports = {BudgetPolicyReference};
