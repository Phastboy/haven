const {test} = require('node:test');
const assert = require('node:assert/strict');
const vm = require('node:vm');
const fs = require('node:fs');
const source = fs.readFileSync(__dirname + '/unread-status.js', 'utf8');

function fixture(fetch, count = 2) {
  const indicators = Array.from({length: count}, () => ({hidden: true}));
  vm.runInNewContext(source, {
    document: {querySelectorAll: () => indicators}, fetch,
  });
  return indicators;
}
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
  resolve({ok: true, json: async () => ({has_unread: true})});
  await settled();
  assert.ok(indicators.every(indicator => !indicator.hidden));
});

test('read state keeps indicators hidden', async () => {
  const indicators = fixture(async () => ({ok: true, json: async () => ({has_unread: false})}));
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
