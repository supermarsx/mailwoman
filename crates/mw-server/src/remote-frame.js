(() => {
  'use strict';
  const fragment = new URLSearchParams(location.hash.slice(1));
  const token = fragment.get('token');
  const origin = fragment.get('origin');
  // The URL retains its origin even though this document's sandbox security
  // origin is opaque. Never let a fragment opt into another parent origin.
  if (!token || !/^[a-zA-Z0-9_-]{16,128}$/.test(token) || origin !== location.origin) return;
  if (document.referrer && new URL(document.referrer).origin !== origin) return;
  const content = document.createElement('iframe');
  content.setAttribute('sandbox', '');
  content.setAttribute('referrerpolicy', 'no-referrer');
  content.setAttribute('title', 'Message content');
  content.style.cssText = 'border:0;width:100%;height:100%;display:block';
  document.body.append(content);
  addEventListener('message', event => {
    if (event.source !== parent || event.origin !== origin) return;
    const data = event.data;
    if (!data || data.type !== 'mw-reader-frame' || data.token !== token ||
        typeof data.html !== 'string' || data.html.length > 8 * 1024 * 1024) return;
    // Sender supplies a sanitized document. The nested sandbox independently
    // forbids scripts, same-origin access, forms, popups and top navigation.
    content.srcdoc = data.html;
  });
  parent.postMessage({ type: 'mw-reader-ready', token }, origin);
})();
