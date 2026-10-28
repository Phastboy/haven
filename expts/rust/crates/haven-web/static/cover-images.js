(() => {
  const selector = 'img[data-cover-image]';
  function settle(image, failed) {
    image.hidden = failed;
    const fallback = image.parentElement.querySelector('.cover-fallback');
    if (fallback) {
      fallback.hidden = !failed;
      fallback.setAttribute('aria-hidden', String(!failed));
    }
  }
  // Capture also handles lazy images and cards inserted by the runtime.
  document.addEventListener('error', event => {
    if (event.target.matches?.(selector)) settle(event.target, true);
  }, true);
  document.addEventListener('load', event => {
    if (event.target.matches?.(selector)) settle(event.target, !event.target.naturalWidth);
  }, true);
  function inspect(root) {
    root.querySelectorAll(selector).forEach(image => {
      if (image.complete) settle(image, !image.naturalWidth);
    });
  }
  inspect(document);
  new MutationObserver(() => inspect(document))
    .observe(document.documentElement, { childList: true, subtree: true });
})();
