// One controller per document; no person-specific status survives a full load.
(() => {
  if (window.havenUnread) { window.havenUnread.sync(); return; }
  let person = '', value = null, active = null, revision = 0;
  let thread = null, message = '', suspended = null, lastReturn = Date.now();
  const identity = () => document.querySelector('[data-unread-person]');
  const paint = () => {
    for (const indicator of document.querySelectorAll('[data-unread-indicator]')) {
      const hidden = value !== true;
      if (indicator.hidden !== hidden) indicator.hidden = hidden;
    }
  };
  function clear() {
    revision++;
    active?.controller.abort();
    active = null;
    value = null;
    paint();
  }
  function refresh(invalidate = false) {
    if (!person || suspended || document.visibilityState === 'hidden') return;
    if (invalidate) { revision++; value = null; paint(); }
    if (active) return; // Its completion schedules one follow-up if invalidated.
    const request = {person, revision, controller: new AbortController()};
    active = request;
    fetch('/unread-status', {
      credentials: 'same-origin', cache: 'no-store', redirect: 'error',
      signal: request.controller.signal, headers: {Accept: 'application/json'},
    }).then(response => {
      if (!response.ok) throw new Error('Unread status unavailable');
      return response.json();
    }).then(status => {
      if (active !== request || request.revision !== revision) return;
      // A cookie may have changed in another tab before this document updates.
      value = identity()?.dataset.unreadPerson === person && status?.person_id === person && typeof status.has_unread === 'boolean'
        ? status.has_unread : null;
      paint();
    }).catch(() => {
      if (active === request) { value = null; paint(); }
    }).finally(() => {
      if (active !== request) return;
      active = null;
      if (request.revision !== revision) refresh();
    });
  }
  function sync() {
    const marker = identity();
    if (suspended && marker === suspended) return;
    suspended = null;
    const nextPerson = marker?.dataset.unreadPerson || '';
    const nextThread = document.querySelector('[data-conversation-id]');
    const nextMessage = document.getElementById('thread-messages')?.lastElementChild?.id || '';
    const changed = nextPerson !== person;
    if (changed) { clear(); person = nextPerson; }
    // A server-rendered thread has marked messages read. Live insertions may
    // change unread state too; refresh without assuming the server marked read.
    const readOrMessage = nextThread && (nextThread !== thread || nextMessage !== message);
    thread = nextThread;
    message = nextMessage;
    paint();
    if (changed) refresh();
    else if (readOrMessage) refresh(true);
  }
  function returned(event) {
    if (document.visibilityState === 'hidden') { clear(); lastReturn = -Infinity; return; }
    if (event?.persisted) { clear(); lastReturn = -Infinity; }
    sync();
    const now = Date.now();
    // visibility/pageshow/focus often describe the same return to the app.
    if (now - lastReturn < 1000) return;
    lastReturn = now;
    refresh(true);
  }
  document.addEventListener('visibilitychange', returned);
  window.addEventListener('pageshow', returned);
  window.addEventListener('focus', returned);
  document.addEventListener('submit', event => {
    const form = event.target;
    if (!form.action) return;
    const action = new URL(form.action, window.location.href);
    if (action.origin === window.location.origin && action.pathname === '/auth/sign-out') {
      suspended = identity();
      clear();
      person = '';
    }
  }, true);
  new MutationObserver(sync).observe(document.documentElement, {
    childList: true, subtree: true, attributes: true,
    attributeFilter: ['data-unread-person', 'data-last-message-id'],
  });
  window.havenUnread = {sync};
  sync();
})();
