const {test} = require('node:test');
const assert = require('node:assert/strict');
const vm = require('node:vm');
const fs = require('node:fs');
const source = fs.readFileSync(__dirname + '/cover-images.js', 'utf8');

function fixture(complete, naturalWidth) {
  const fallback = {hidden: false, ariaHidden: 'true',
    setAttribute(name, value) {
      assert.equal(name, 'aria-hidden');
      this.ariaHidden = value;
    }};
  const image = {complete, naturalWidth, hidden: false,
    matches: selector => selector === 'img[data-cover-image]',
    parentElement: {querySelector: () => fallback}};
  const listeners = {};
  let mutation;
  const document = {documentElement: {},
    querySelectorAll: () => [image],
    addEventListener: (name, fn) => listeners[name] = fn};
  vm.runInNewContext(source, {document,
    MutationObserver: class { constructor(fn) {mutation = fn;} observe() {} }});
  return {image, fallback, listeners, mutation};
}

test('cached decode or delivery failure preserves fallback and hides broken image', () => {
  const {image, fallback} = fixture(true, 0);
  assert.equal(image.hidden, true);
  assert.equal(fallback.hidden, false);
  assert.equal(fallback.ariaHidden, 'false');
});
test('successful cached delivery hides fallback', () => {
  const {image, fallback} = fixture(true, 320);
  assert.equal(image.hidden, false);
  assert.equal(fallback.hidden, true);
  assert.equal(fallback.ariaHidden, 'true');
});
test('loading images do not announce a delivery failure', () => {
  const {image, fallback} = fixture(false, 0);
  assert.equal(image.hidden, false);
  assert.equal(fallback.ariaHidden, 'true');
});
test('lazy srcset failure and subsequent success settle without retrying URLs', () => {
  const f = fixture(false, 0);
  f.listeners.error({target: f.image});
  assert.equal(f.image.hidden, true);
  f.image.naturalWidth = 640;
  f.listeners.load({target: f.image});
  assert.equal(f.image.hidden, false);
  assert.equal(f.fallback.hidden, true);
});
test('new runtime cards with cached errors are inspected', () => {
  const f = fixture(false, 0);
  f.image.complete = true;
  f.mutation();
  assert.equal(f.image.hidden, true);
});
