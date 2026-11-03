(() => {
  'use strict';
  // This exact source is the only script admitted by the message document CSP.
  // It has no network, navigation, account or arbitrary-code interface.
  let port = null;
  let nonce = null;
  let revision = 0;
  const bytes = value => new TextEncoder().encode(value).length;
  function mailbox(address) {
    if (!address || bytes(address) > 254 || /[\p{Cc}\p{Cs}]/u.test(address)) return null;
    let separator = -1;
    if (address.startsWith('"')) {
      for (let i = 1; i < address.length; i++) {
        const char = address[i];
        if (char === '\\') { const escaped = address[++i]; if (!escaped || escaped.charCodeAt(0) < 32 || escaped.charCodeAt(0) > 126) return null; continue; }
        if (char === '"') { if (address[i + 1] !== '@') return null; separator = i + 1; break; }
        if (char.charCodeAt(0) > 127 && /\p{White_Space}/u.test(char)) return null;
      }
    } else {
      separator = address.indexOf('@');
      const local = address.slice(0, separator);
      if (!local || local.startsWith('.') || local.endsWith('.') || local.includes('..')) return null;
      for (const char of local) if (char.charCodeAt(0) < 128 ? !/[A-Za-z0-9.!#$%&'*+/=?^_`{|}~-]/.test(char) : /\p{White_Space}/u.test(char)) return null;
    }
    if (separator < 1 || bytes(address.slice(0, separator)) > 64) return null;
    const domain = address.slice(separator + 1);
    if (!domain || /[\p{White_Space}@%/\\?#]/u.test(domain)) return null;
    if (domain.startsWith('[') && domain.endsWith(']')) {
      const literal = domain.slice(1, -1);
      if (literal.startsWith('IPv6:')) { try { if (!new URL(`http://[${literal.slice(5)}]`).hostname.startsWith('[')) return null; } catch { return null; } }
      else { const octets = literal.split('.'); if (octets.length !== 4 || !octets.every(part => /^(?:0|[1-9]\d{0,2})$/.test(part) && Number(part) <= 255)) return null; }
    } else {
      if (/[:[\]]/.test(domain) || domain.endsWith('.')) return null;
      try { const ascii = new URL(`http://${domain}`).hostname; if (ascii.length > 253 || !ascii.split('.').every(label => label.length >= 1 && label.length <= 63 && /^[a-z0-9](?:[a-z0-9-]*[a-z0-9])?$/i.test(label))) return null; } catch { return null; }
    }
    return address.slice(0, separator + 1) + (domain.startsWith('[') ? domain : domain.toLowerCase());
  }
  function mailto(value) {
    if (typeof value !== 'string' || !/^mailto:/i.test(value)) return null;
    try { return mailbox(decodeURIComponent(value.slice(7).split(/[?#]/, 1)[0])); } catch { return null; }
  }
  function textTarget(node, offset, x, y) {
    if (node?.nodeType !== 3 || !Number.isSafeInteger(offset) || offset < 0 || offset > node.length) return null;
    const text = node.data, start = Math.max(0, offset - 256), end = Math.min(text.length, offset + 256);
    const pattern = /(?:"(?:[^"\\\r\n]|\\.)+"|[^\s<>()\[\],;:"\\]+)@(?:\[[^\]\s]+\]|[^\s<>()\[\],;:"\\]+)/gu;
    for (const match of text.slice(start, end).matchAll(pattern)) {
      const candidate = match[0].replace(/[.!?]+$/, ''), left = start + match.index, right = left + candidate.length;
      if (offset < left || offset > right) continue;
      if (candidate.startsWith('//') && left > 0 && text[left - 1] === ':') continue;
      // A bounded search must not turn a longer token into a repaired address.
      if ((left === start && start > 0 && !/[\s<>()\[\],;]/u.test(text[start - 1])) || (start + match.index + match[0].length === end && end < text.length && !/[\s<>()\[\],;]/u.test(text[end]))) continue;
      const address = mailbox(candidate); if (!address) continue;
      const range = document.createRange(); range.setStart(node, left); range.setEnd(node, right);
      // Caret hit testing can return the nearest text on whitespace. Require
      // the pointer to be inside this address's own rendered glyph rectangles.
      if (Array.from(range.getClientRects()).some(rect => x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom)) return address;
    }
    return null;
  }
  function initialize(event) {
    const data = event.data;
    if (port || event.source !== parent || event.origin !== 'null' || data?.type !== 'mw-reader-address-init' || !/^[a-f0-9]{48}$/.test(data.nonce) || !Number.isSafeInteger(data.revision) || data.revision <= 0 || event.ports.length !== 1) return;
    nonce = data.nonce; revision = data.revision; port = event.ports[0]; port.start();
    removeEventListener('message', initialize);
    // HTML is readable before remote images finish. The app's visible window
    // waits two paint frames before uncovering this opaque document.
    if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', parsed, { once: true });
    else parsed();
  }
  function parsed() { if (port) port.postMessage({ type: 'displayed', nonce, revision }); }
  addEventListener('message', initialize);
  document.addEventListener('contextmenu', event => {
    if (!event.isTrusted || event.defaultPrevented || !port) return;
    const target = event.target?.nodeType === 1 ? event.target : event.target?.parentElement;
    const anchor = target?.closest('a,area');
    let address = anchor ? mailbox(anchor.getAttribute('data-mw-contact-address') || '') || mailto(anchor.getAttribute('href')) : null;
    if (!address) {
      const caret = document.caretPositionFromPoint?.(event.clientX, event.clientY);
      const range = caret ? null : document.caretRangeFromPoint?.(event.clientX, event.clientY);
      address = textTarget(caret?.offsetNode ?? range?.startContainer, caret?.offset ?? range?.startOffset, event.clientX, event.clientY);
    }
    if (!address) return;
    event.preventDefault();
    port.postMessage({ type: 'address-context', nonce, revision, address, x: event.clientX, y: event.clientY });
  }, true);
  addEventListener('pagehide', () => { port?.close(); port = null; removeEventListener('message', initialize); document.removeEventListener('DOMContentLoaded', parsed); }, { once: true });
  parent.postMessage({ type: 'mw-reader-address-ready' }, '*');
})();
