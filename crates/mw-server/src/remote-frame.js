(() => {
  'use strict';
  const fragment = new URLSearchParams(location.hash.slice(1));
  const token = fragment.get('token');
  const origin = fragment.get('origin');
  const expectedOrigin = __MW_PARENT_ORIGIN__;
  const addressHandler = __MW_ADDRESS_HANDLER_SOURCE__;
  const addressHash = __MW_ADDRESS_HANDLER_HASH__;
  // The URL retains its origin even though this document's sandbox security
  // origin is opaque. Never let a fragment opt into another parent origin.
  if (!token || !/^[a-zA-Z0-9_-]{16,128}$/.test(token) || origin !== (expectedOrigin || location.origin)) return;
  if (document.referrer && new URL(document.referrer).origin !== origin) return;
  let content = document.createElement('iframe');
  content.setAttribute('sandbox', '');
  content.setAttribute('referrerpolicy', 'no-referrer');
  content.setAttribute('title', 'Message content');
  content.style.cssText = 'border:0;width:100%;height:100%;display:block';
  document.body.append(content);
  let addressPort = null;
  let documentRevision = 0;
  let displayedRevision = 0;
  function displayed(revision) {
    if (revision === documentRevision && revision > displayedRevision) {
      displayedRevision = revision;
      parent.postMessage({ type: 'mw-reader-displayed', token, revision }, origin);
    }
  }
  function closePort() { if (addressPort) addressPort.close(); addressPort = null; }
  addEventListener('pagehide', closePort, { once: true });
  // Preserve the already sanitized layout; remove executable nodes/attributes
  // independently before admitting our one fixed, hash-authorized handler.
  function addressDocument(html, compatible) {
    const doc = new DOMParser().parseFromString(html, 'text/html');
    doc.querySelectorAll('script').forEach(node => node.remove());
    for (const node of doc.querySelectorAll('*')) for (const attribute of Array.from(node.attributes))
      if (/^on/i.test(attribute.name)) node.removeAttribute(attribute.name);
    let policies = Array.from(doc.querySelectorAll('meta[http-equiv]')).filter(node => node.httpEquiv.toLowerCase() === 'content-security-policy');
    if (!policies.length) { const policy = doc.createElement('meta'); policy.httpEquiv = 'Content-Security-Policy'; policy.content = "default-src 'none'; style-src 'unsafe-inline'"; doc.head.prepend(policy); policies = [policy]; }
    for (const policy of policies) {
      const directives = policy.content.split(';').filter(part => !/^\s*script-src(?:-elem|-attr)?(?:\s|$)/i.test(part));
      policy.content = directives.join(';') + "; script-src 'sha256-" + addressHash + "'; script-src-attr 'none'";
    }
    if (compatible) { const policy = doc.createElement('meta'); policy.httpEquiv = 'Content-Security-Policy'; policy.content = 'img-src ' + location.origin + ' data: blob:'; doc.head.prepend(policy); }
    const handler = doc.createElement('script'); handler.textContent = addressHandler; doc.head.append(handler);
    return '<!doctype html>' + doc.documentElement.outerHTML;
  }
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
    if (event.source === content.contentWindow && event.origin === 'null' && event.data?.type === 'mw-reader-address-ready' && documentRevision > 0 && !addressPort) {
      const channel = new MessageChannel();
      const nonce = Array.from(crypto.getRandomValues(new Uint8Array(24)), byte => byte.toString(16).padStart(2, '0')).join('');
      const current = content, revision = documentRevision;
      addressPort = channel.port1;
      channel.port1.onmessage = received => {
        const value = received.data;
        if (content !== current || documentRevision !== revision || value?.nonce !== nonce || value.revision !== revision) return;
        if (value.type === 'displayed') { displayed(revision); return; }
        if (value.type !== 'address-context' || typeof value.address !== 'string' || value.address.length > 254 || !Number.isFinite(value.x) || !Number.isFinite(value.y) || value.x < 0 || value.y < 0 || value.x > current.clientWidth || value.y > current.clientHeight) return;
        const bounds = current.getBoundingClientRect();
        parent.postMessage({ type: 'mw-reader-address-context', token, revision, address: value.address, x: bounds.left + value.x, y: bounds.top + value.y }, origin);
      };
      channel.port1.start();
      current.contentWindow.postMessage({ type: 'mw-reader-address-init', nonce, revision }, '*', [channel.port2]);
      return;
    }
    if (event.origin !== origin || !event.source || !trusted(event.source)) return;
    const data = event.data;
    // The app asks again once it listens on the window holding this frame.
    if (data && data.type === 'mw-reader-ping' && data.token === token) {
      parent.postMessage({ type: 'mw-reader-ready', token }, origin);
      return;
    }
    if (!data || data.type !== 'mw-reader-frame' || data.token !== token ||
        typeof data.html !== 'string' || data.html.length > 8 * 1024 * 1024 || !Number.isSafeInteger(data.revision) || data.revision <= documentRevision) return;
    // Sender supplies a sanitized document. The nested sandbox independently
    // forbids scripts, same-origin access, forms and top navigation. Only an
    // authenticated, explicit per-message consent permits isolated popups.
    const next = content.cloneNode(false);
    next.setAttribute('sandbox', data.linksEnabled === true ? 'allow-scripts allow-popups allow-popups-to-escape-sandbox' : 'allow-scripts');
    const revision = data.revision;
    closePort(); documentRevision = revision;
    next.addEventListener('load', () => {
      if (content === next && Number.isSafeInteger(revision) && revision > 0)
        displayed(revision);
    }, { once: true });
    // Some engines do not inherit the outer document's image CSP into srcdoc.
    // Install the compatibility restriction before any message markup can load
    // resources. An additional message CSP can narrow this policy, never relax it.
    // The leading doctype preserves standards mode for full documents/fragments.
    const compatible = new URLSearchParams(location.search).get('coep') === 'require-corp';
    next.srcdoc = addressDocument(data.html, compatible);
    content.replaceWith(next);
    content = next;
  });
  parent.postMessage({ type: 'mw-reader-ready', token }, origin);
})();
