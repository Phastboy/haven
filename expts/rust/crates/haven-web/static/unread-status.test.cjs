const {test} = require('node:test');
const assert = require('node:assert/strict');
const vm = require('node:vm');
const fs = require('node:fs');
const source = fs.readFileSync(__dirname + '/unread-status.js', 'utf8');

function browser(fetch, count = 2) {
  const listeners = {};
  let mutate;
  const state = {
    marker: {dataset: {unreadPerson: count ? 'alice' : ''}}, thread: null,
    lastMessage: null, indicators: Array.from({length: count}, () => ({hidden: true})),
    now: 10000,
  };
  const document = {
    documentElement: {}, visibilityState: 'visible',
    querySelectorAll: () => state.indicators,
    querySelector: selector => selector === '[data-unread-person]' ? state.marker : state.thread,
    getElementById: () => ({lastElementChild: state.lastMessage}),
    addEventListener: (name, callback) => {listeners[name] = callback;},
  };
  const window = {
    location: {href: 'https://haven.test/offers', origin: 'https://haven.test'},
    addEventListener: (name, callback) => {listeners[name] = callback;},
  };
  const context = vm.createContext({document, window, fetch, URL, AbortController,
    Date: {now: () => state.now},
    MutationObserver: class {constructor(callback) {mutate = callback;} observe() {}},
  });
  vm.runInContext(source, context);
  return {state, document, window, listeners, mutate: () => mutate(),
    rerun: () => vm.runInContext(source, context)};
}
function fixture(fetch, count = 2) {return browser(fetch, count).state.indicators;}
const response = (has_unread, person_id = 'alice') => ({
  ok: true, json: async () => ({has_unread, person_id}),
});
const settled = () => new Promise(resolve => setImmediate(resolve));

test('a delayed unread request leaves both indicators hidden until resolved', async () => {
  let resolve;
  let requests = 0;
  const indicators = fixture((url, options) => {
    requests++;
    assert.equal(url, '/unread-status');
    assert.equal(options.credentials, 'same-origin');
    assert.equal(options.cache, 'no-store');
    assert.equal(options.redirect, 'error');
    return new Promise(done => {resolve = done;});
  });
  assert.equal(requests, 1);
  assert.ok(indicators.every(indicator => indicator.hidden));
  resolve(response(true));
  await settled();
  assert.ok(indicators.every(indicator => !indicator.hidden));
});

test('read state keeps indicators hidden', async () => {
  const indicators = fixture(async () => response(false));
  await settled();
  assert.ok(indicators.every(indicator => indicator.hidden));
});

test('network, HTTP, malformed JSON and invalid state fail without showing a dot', async () => {
  for (const fetch of [
    async () => {throw new Error('offline');},
    async () => ({ok: false}),
    async () => ({ok: true, json: async () => {throw new Error('bad JSON');}}),
    async () => ({ok: true, json: async () => ({has_unread: 'true'})}),
    async () => ({ok: true, json: async () => null}),
  ]) {
    const indicators = fixture(fetch);
    await settled();
    assert.ok(indicators.every(indicator => indicator.hidden));
  }
});

test('pages without signed-in message navigation make no unread request', () => {
  fixture(() => {assert.fail('unexpected fetch');}, 0);
});


test('runtime navigation and duplicate script loads reuse status without extra requests', async () => {
  let requests = 0;
  const app = browser(async () => {requests++; return response(true);});
  await settled();
  app.state.indicators = [{hidden: true}, {hidden: true}];
  app.state.marker = {dataset: {unreadPerson: 'alice'}};
  app.mutate();
  app.rerun();
  await settled();
  assert.equal(requests, 1);
  assert.ok(app.state.indicators.every(indicator => !indicator.hidden));
});

test('reading or sending refreshes once and mutations during an active check queue one follow-up', async () => {
  const pending = [];
  const app = browser(() => new Promise(resolve => pending.push(resolve)));
  app.state.thread = {};
  app.state.lastMessage = {id: 'message-1'};
  app.mutate();
  app.mutate();
  assert.equal(pending.length, 1);
  pending[0](response(true));
  await settled();
  assert.equal(pending.length, 2);
  assert.ok(app.state.indicators.every(indicator => indicator.hidden));
  pending[1](response(false));
  await settled();
  app.state.thread = {}; // Redirect/rerender after sending or opening the thread.
  app.state.lastMessage = {id: 'message-2'};
  app.mutate();
  assert.equal(pending.length, 3);
  pending[2](response(false));
  await settled();
  app.state.lastMessage = {id: 'message-3'}; // SSE insertion.
  app.mutate();
  assert.equal(pending.length, 4);
  pending[3](response(true));
  await settled();
  assert.ok(app.state.indicators.every(indicator => !indicator.hidden));
});

test('return events coalesce, clear hidden-tab state and revalidate bfcache restores', async () => {
  let requests = 0;
  const app = browser(async () => {requests++; return response(true);});
  await settled();
  app.document.visibilityState = 'hidden';
  app.listeners.visibilitychange({});
  assert.ok(app.state.indicators.every(indicator => indicator.hidden));
  app.state.now += 2000;
  app.document.visibilityState = 'visible';
  app.listeners.visibilitychange({});
  app.listeners.focus({});
  app.listeners.pageshow({});
  await settled();
  assert.equal(requests, 2);
  app.listeners.pageshow({persisted: true});
  assert.ok(app.state.indicators.every(indicator => indicator.hidden));
  await settled();
  assert.equal(requests, 3);
});

test('account switch aborts and ignores the old response, including delayed completion', async () => {
  const pending = [];
  const app = browser((_, options) => new Promise(resolve => pending.push({resolve, options})));
  app.state.marker = {dataset: {unreadPerson: 'bob'}};
  app.mutate();
  assert.equal(pending.length, 2);
  assert.equal(pending[0].options.signal.aborted, true);
  pending[0].resolve(response(true));
  pending[1].resolve(response(false, 'bob'));
  await settled();
  assert.ok(app.state.indicators.every(indicator => indicator.hidden));
});

test('sign-out clears cached/in-flight state and guest/account transitions never reuse it', async () => {
  let requests = 0;
  const app = browser(async () => {requests++; return response(true);});
  await settled();
  app.listeners.submit({target: {action: 'https://haven.test/auth/sign-out'}});
  assert.ok(app.state.indicators.every(indicator => indicator.hidden));
  app.mutate();
  assert.equal(requests, 1);
  app.state.marker = {dataset: {unreadPerson: ''}};
  app.mutate();
  assert.equal(requests, 1);
  app.state.marker = {dataset: {unreadPerson: 'bob'}};
  app.mutate();
  await settled();
  assert.equal(requests, 2);
  assert.ok(app.state.indicators.every(indicator => indicator.hidden)); // Response belongs to Alice.
});

test('failed refresh clears prior status and a later return can retry', async () => {
  let requests = 0;
  const app = browser(async () => {
    requests++;
    if (requests === 2) return {ok: false, status: 401};
    return response(true);
  });
  await settled();
  app.state.now += 2000;
  app.listeners.focus({});
  await settled();
  assert.ok(app.state.indicators.every(indicator => indicator.hidden));
  app.state.now += 2000;
  app.listeners.focus({});
  await settled();
  assert.equal(requests, 3);
  assert.ok(app.state.indicators.every(indicator => !indicator.hidden));
});
