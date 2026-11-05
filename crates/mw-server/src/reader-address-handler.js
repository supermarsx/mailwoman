(() => {
  'use strict';
  // This exact source is the only script admitted by the message document CSP.
  // It has no network, navigation, account or arbitrary-code interface.
  let port = null;
  let nonce = null;
  let revision = 0;
  let imageObserver = null;
  let imagesClosed = false;
  let prefixes = [];
  let restoreScroll = null;
  const emptyImage = 'data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7';
  const imageStates = new Map();
  const imageCounts = { loaded: 0, failed: 0, pending: 0, total: 0 };
  let progressScheduled = false;
  function reportImageProgress() {
    if (imagesClosed || progressScheduled || !port) return;
    progressScheduled = true;
    queueMicrotask(() => {
      progressScheduled = false;
      if (!imagesClosed && port) port.postMessage({ type: 'image-progress', nonce, revision, ...imageCounts });
    });
  }
  function countImage(image, state) {
    const source = image.getAttribute('src') || image.getAttribute('srcset') || '';
    const current = state && document.body.contains(image) && source && source !== emptyImage && !/^mw-(?:rc|original):/i.test(source)
      ? state === 'loading' ? 'pending' : state === 'error' ? 'failed' : 'loaded' : null;
    const previous = imageStates.get(image);
    if (previous === current || !previous && !current) return;
    if (previous) { imageCounts[previous]--; imageCounts.total--; imageStates.delete(image); }
    if (current) { imageCounts[current]++; imageCounts.total++; imageStates.set(image, current); }
    reportImageProgress();
  }
  function imageState(image, pending = false) {
    if (imagesClosed || image?.tagName !== 'IMG') return;
    if (!image.getAttribute('src') && !image.getAttribute('srcset')) {
      image.removeAttribute('data-mw-image-state');
      image.removeAttribute('data-mw-image-unsized');
      countImage(image, null);
      return;
    }
    const state = pending || !image.complete ? 'loading' : image.naturalWidth > 0 ? 'loaded' : 'error';
    image.setAttribute('data-mw-image-state', state);
    countImage(image, state);
    if (state === 'loading' && !image.hasAttribute('width') && !image.hasAttribute('height') && !image.style.width && !image.style.height && !image.style.aspectRatio && !image.getBoundingClientRect().width && !image.getBoundingClientRect().height) image.setAttribute('data-mw-image-unsized', '');
    else if (state !== 'loading') image.removeAttribute('data-mw-image-unsized');
  }
  function imageSettled(event) { imageState(event.target); }
  function imageAdded(node) {
    if (node?.nodeType !== 1) return;
    if (node.tagName === 'IMG') imageState(node);
    node.querySelectorAll('img').forEach(image => imageState(image));
  }
  function imageRemoved(node) {
    if (node?.nodeType !== 1) return;
    if (node.tagName === 'IMG') countImage(node, null);
    node.querySelectorAll('img').forEach(image => countImage(image, null));
  }
  function observeImages() {
    if (imagesClosed || imageObserver) return;
    document.querySelectorAll('img').forEach(image => imageState(image));
    reportImageProgress();
    imageObserver = new MutationObserver(records => {
      if (imagesClosed) return;
      for (const record of records) {
        if (record.type === 'attributes') {
          const image = record.target;
          imageState(image, true);
          // A cached resource can already be complete before the mutation
          // observer runs; settle it without waiting for another load event.
          queueMicrotask(() => imageState(image));
        } else { record.removedNodes.forEach(imageRemoved); record.addedNodes.forEach(imageAdded); }
      }
    });
    imageObserver.observe(document.body, { subtree: true, childList: true, attributes: true, attributeFilter: ['src', 'srcset'] });
    document.addEventListener('load', imageSettled, true);
    document.addEventListener('error', imageSettled, true);
  }
  // In-place image patching. The bridge relays parent-requested operations over
  // this nonce-bound port; every operation is validated here again, and the
  // whole batch is refused if any single one is not acceptable.
  const dataImage = /^data:image\/(?:gif|png|jpeg|webp|avif);base64,[A-Za-z0-9+\/=]*$/;
  const cssForbidden = /[\x00\x5c]|@import|image-set|cross-fade|-webkit-image|element\(|\bimage\(|expression\(/i;
  const patchAttrs = ['src', 'srcset', 'background', 'style'];
  function urlAllowed(raw) {
    if (typeof raw !== 'string' || raw.length > 4096) return false;
    if (raw.length > 5 && raw.startsWith('data:')) return dataImage.test(raw);
    return prefixes.some(prefix => raw.startsWith(prefix) && /^[A-Za-z0-9_.-]{1,512}$/.test(raw.slice(prefix.length)));
  }
  function cssAllowed(value) {
    if (value.length > 65536 || cssForbidden.test(value)) return false;
    const all = (value.match(/url\s*\(/gi) || []).length;
    let seen = 0, ok = true;
    value.replace(/url\(\s*(?:"([^"]*)"|'([^']*)'|([^)"'\s]*))\s*\)/gi, (_m, a, b, c) => { seen++; ok = ok && urlAllowed(a ?? b ?? c ?? ''); return ''; });
    return ok && seen === all;
  }
  function valueAllowed(attr, value) {
    if (value === null) return attr !== 'text';
    if (typeof value !== 'string') return false;
    if (attr === 'src' || attr === 'background') return urlAllowed(value);
    if (attr === 'srcset') {
      const parts = value.split(',').map(part => part.trim());
      return parts.length > 0 && parts.length <= 64 && parts.every(part => { const [url, descriptor, ...rest] = part.split(/\s+/); return rest.length === 0 && urlAllowed(url) && (descriptor === undefined || /^\d+(?:\.\d+)?[wx]$/.test(descriptor)); });
    }
    return (attr === 'style' || attr === 'text') && cssAllowed(value);
  }
  function patchTarget(op) {
    if (!op || typeof op !== 'object' || !Number.isSafeInteger(op.i) || op.i < 0) return null;
    if (op.a === 'text') {
      const node = (op.s === 'h' ? document.head.querySelectorAll('style') : op.s === 'b' ? document.body.querySelectorAll('style') : [])[op.i];
      return node && node.tagName === 'STYLE' ? node : null;
    }
    if (op.s !== 'b' || !patchAttrs.includes(op.a)) return null;
    return document.body.querySelectorAll('*')[op.i] || null;
  }
  function applyPatch(ops) {
    if (!Array.isArray(ops) || ops.length > 4096) return false;
    // Body <style> text is addressed by its position among all body elements.
    const targets = [];
    for (const op of ops) {
      if (!op || !valueAllowed(op.a, op.v)) return false;
      const node = op.a === 'text' && op.s === 'b' ? document.body.querySelectorAll('*')[op.i] : patchTarget(op);
      if (!node || (op.a === 'text' && node.tagName !== 'STYLE')) return false;
      targets.push(node);
    }
    ops.forEach((op, index) => {
      const node = targets[index];
      if (op.a === 'text') node.textContent = op.v;
      else if (op.v === null) node.removeAttribute(op.a);
      else node.setAttribute(op.a, op.v);
    });
    return true;
  }
  function onPortMessage(received) {
    const value = received.data;
    if (!port || !value || value.type !== 'patch' || value.nonce !== nonce || value.revision !== revision) return;
    port.postMessage({ type: 'patch-result', nonce, revision, id: Number.isSafeInteger(value.id) ? value.id : 0, ok: applyPatch(value.ops) });
  }
  let scrollScheduled = false;
  function reportScroll() {
    if (scrollScheduled || !port) return;
    scrollScheduled = true;
    setTimeout(() => { scrollScheduled = false; if (port) port.postMessage({ type: 'scroll', nonce, revision, x: scrollX, y: scrollY }); }, 100);
  }
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
    nonce = data.nonce; revision = data.revision; port = event.ports[0]; port.onmessage = onPortMessage; port.start();
    if (Array.isArray(data.prefixes)) prefixes = data.prefixes.filter(value => typeof value === 'string' && value.length < 512).slice(0, 4);
    const at = data.scroll;
    if (at && Number.isFinite(at.x) && Number.isFinite(at.y) && at.x >= 0 && at.y >= 0 && at.x < 1e7 && at.y < 1e7) restoreScroll = { x: at.x, y: at.y };
    removeEventListener('message', initialize);
    // HTML is readable before remote images finish. The app's visible window
    // waits two paint frames before uncovering this opaque document.
    if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', parsed, { once: true });
    else parsed();
  }
  function restore() { if (restoreScroll) scrollTo(restoreScroll.x, restoreScroll.y); }
  function parsed() { restore(); observeImages(); if (port) port.postMessage({ type: 'displayed', nonce, revision }); }
  addEventListener('message', initialize);
  addEventListener('scroll', reportScroll, { passive: true });
  addEventListener('load', () => { restore(); restoreScroll = null; }, { once: true });
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
  addEventListener('pagehide', () => { imagesClosed = true; imageObserver?.disconnect(); imageStates.clear(); document.removeEventListener('load', imageSettled, true); document.removeEventListener('error', imageSettled, true); port?.close(); port = null; removeEventListener('message', initialize); document.removeEventListener('DOMContentLoaded', parsed); }, { once: true });
  parent.postMessage({ type: 'mw-reader-address-ready' }, '*');
})();
