(() => {
  'use strict';
  const fragment = new URLSearchParams(location.hash.slice(1));
  const token = fragment.get('token');
  const origin = fragment.get('origin');
  // The URL retains its origin even though this document's sandbox security
  // origin is opaque. Never let a fragment opt into another parent origin.
  if (!token || !/^[a-zA-Z0-9_-]{16,128}$/.test(token) || origin !== location.origin) return;
  if (document.referrer && new URL(document.referrer).origin !== origin) return;
  let content = document.createElement('iframe');
  content.setAttribute('sandbox', '');
  content.setAttribute('referrerpolicy', 'no-referrer');
  content.setAttribute('title', 'Message content');
  content.style.cssText = 'border:0;width:100%;height:100%;display:block';
  document.body.append(content);
  // A detached workspace window is rendered by the app window that opened it,
  // so the message comes from that window (the opener of the window holding
  // this frame), not from the parent itself. Accept exactly those two windows,
  // and only from the app origin with this frame's random token: another
  // same-origin tab, a sibling frame or a cross-origin opener is refused.
  // `opener` is readable across origins; a closed or severed one is null.
  const trusted = source => {
    if (source === parent) return true;
    try { const opener = parent.opener; return opener !== null && source === opener; } catch { return false; }
  };
  addEventListener('message', event => {
    if (event.origin !== origin || !event.source || !trusted(event.source)) return;
    const data = event.data;
    // The app asks again once it listens on the window holding this frame.
    if (data && data.type === 'mw-reader-ping' && data.token === token) {
      parent.postMessage({ type: 'mw-reader-ready', token }, origin);
      return;
    }
    if (!data || data.type !== 'mw-reader-frame' || data.token !== token ||
        typeof data.html !== 'string' || data.html.length > 8 * 1024 * 1024) return;
    // Sender supplies a sanitized document. The nested sandbox independently
    // forbids scripts, same-origin access, forms and top navigation. Only an
    // authenticated, explicit per-message consent permits isolated popups.
    const next = content.cloneNode(false);
    next.setAttribute('sandbox', data.linksEnabled === true ? 'allow-popups allow-popups-to-escape-sandbox' : '');
    const revision = data.revision;
    next.addEventListener('load', () => {
      if (content === next && Number.isSafeInteger(revision) && revision > 0)
        parent.postMessage({ type: 'mw-reader-displayed', token, revision }, origin);
    }, { once: true });
    // Some engines do not inherit the outer document's image CSP into srcdoc.
    // Install the compatibility restriction before any message markup can load
    // resources. An additional message CSP can narrow this policy, never relax it.
    // The leading doctype preserves standards mode for full documents/fragments.
    const compatible = new URLSearchParams(location.search).get('coep') === 'require-corp';
    next.srcdoc = compatible
      ? '<!doctype html><html><head><meta http-equiv="Content-Security-Policy" content="img-src ' + origin + ' data: blob:">' + data.html
      : data.html;
    content.replaceWith(next);
    content = next;
  });
  parent.postMessage({ type: 'mw-reader-ready', token }, origin);
})();
