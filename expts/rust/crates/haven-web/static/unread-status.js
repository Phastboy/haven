// Unknown status stays hidden; failure must never prevent page use.
(() => {
  const indicators = document.querySelectorAll('[data-unread-indicator]');
  if (!indicators.length) return;

  fetch('/unread-status', {
    credentials: 'same-origin', cache: 'no-store', redirect: 'error',
    headers: {Accept: 'application/json'},
  }).then(response => {
    if (!response.ok) throw new Error('Unread status unavailable');
    return response.json();
  }).then(status => {
    if (typeof status.has_unread !== 'boolean') return;
    for (const indicator of indicators) indicator.hidden = !status.has_unread;
  }).catch(() => {});
})();
