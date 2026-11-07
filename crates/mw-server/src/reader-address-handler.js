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
  // Firefox logs 'Layout was forced before the page was fully loaded' for a layout read made
  // before the document finished loading, so box measurements wait for `load`; until then an
  // unsized loading image gets its temporary minimum from attributes alone (CSS only).
  let settled = document.readyState === 'complete';
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
      tipRefresh(image);
      return;
    }
    const state = pending || !image.complete ? 'loading' : image.naturalWidth > 0 ? 'loaded' : 'error';
    image.setAttribute('data-mw-image-state', state);
    countImage(image, state);
    if (state === 'loading' && !image.hasAttribute('width') && !image.hasAttribute('height') && !image.style.width && !image.style.height && !image.style.aspectRatio && (!settled || (!image.getBoundingClientRect().width && !image.getBoundingClientRect().height))) image.setAttribute('data-mw-image-unsized', '');
    else if (state !== 'loading') image.removeAttribute('data-mw-image-unsized');
    tipRefresh(image);
  }
  // A signed image the proxy could not serve right now (busy, rate-limited, a
  // transient upstream failure) is asked for again after a growing pause, so the
  // reader sees the picture arrive instead of a broken image. Bounded, per image.
  const retries = new WeakMap();
  const retryDelays = [800, 2000, 4500, 9000];
  function retryImage(image) {
    if (imagesClosed || image?.tagName !== 'IMG') return;
    const source = image.getAttribute('src') || '';
    if (!prefixes.some(prefix => prefix.endsWith('/i/') && source.startsWith(prefix))) return;
    const attempt = retries.get(image) || 0;
    if (attempt >= retryDelays.length) return;
    retries.set(image, attempt + 1);
    setTimeout(() => {
      if (imagesClosed || !document.body.contains(image) || image.getAttribute('src') !== source || (image.complete && image.naturalWidth > 0)) return;
      image.removeAttribute('src');
      image.setAttribute('src', source);
    }, retryDelays[attempt] + Math.floor(Math.random() * 300));
  }
  function imageSettled(event) { imageState(event.target); if (event.type === 'error') retryImage(event.target); }
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
      if (op.a === 'src') retries.delete(node);
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
  function reportScroll() { tipHide();
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
  function parsed() { if (settled) restore(); observeImages(); if (port) port.postMessage({ type: 'displayed', nonce, revision }); }
  // Themed image tooltip. The frame is opaque, so a native `title` tooltip would
  // be OS-styled; the app strips those into `data-mw-tip` and this draws one
  // bubble (`.mw-frame-tip`, styled by the document's own --mw-tip-* tokens) after
  // a short hover delay or on keyboard focus. Text comes from the sender's own
  // title, else the alt, else a string the app put on <body data-mw-tip-*>; it is
  // only ever set as textContent. The bubble is a child of <html>, after <body>,
  // so body element indices used by in-place patches never shift.
  let tipEl = null, tipImage = null, tipHost = null, tipTimer = 0;
  function tipImageOf(node) {
    const element = node?.nodeType === 1 ? node : node?.parentElement;
    if (!element) return null;
    if (element.tagName === 'IMG') return element;
    const link = element.closest('a,area');
    const images = link ? link.querySelectorAll('img') : [];
    return images.length === 1 ? images[0] : null;
  }
  function tipText(image) {
    if (!image || !document.body.contains(image)) return null;
    const placeholder = image.getAttribute('data-mw-ph');
    if (placeholder === 'hidden' || placeholder === 'deco') return null;
    const source = image.getAttribute('src') || '', state = image.getAttribute('data-mw-image-state');
    const kind = placeholder !== null && (!source || source === emptyImage || /^mw-(?:rc|original):/i.test(source)) ? 'blocked'
      : state === 'loading' ? 'loading' : state === 'error' ? 'error' : null;
    const text = (image.getAttribute('data-mw-tip') || '').trim()
      || (kind ? (image.getAttribute('alt') || '').trim() || (document.body.getAttribute('data-mw-tip-' + kind) || '').trim() : '');
    return text ? text.slice(0, 300) : null;
  }
  function tipHide() {
    clearTimeout(tipTimer); tipTimer = 0;
    if (tipHost) {
      const rest = (tipHost.getAttribute('aria-describedby') || '').split(/\s+/).filter(part => part && part !== 'mw-frame-tip');
      if (rest.length) tipHost.setAttribute('aria-describedby', rest.join(' ')); else tipHost.removeAttribute('aria-describedby');
    }
    if (tipEl) tipEl.remove();
    tipEl = null; tipImage = null; tipHost = null;
  }
  function tipPosition() {
    if (!tipEl || !tipImage || !tipImage.isConnected) { tipHide(); return; }
    const anchor = tipImage.getBoundingClientRect(), box = tipEl.getBoundingClientRect();
    const width = document.documentElement.clientWidth, height = document.documentElement.clientHeight, edge = 4, gap = 6;
    if (anchor.bottom < 0 || anchor.top > height || anchor.right < 0 || anchor.left > width) { tipHide(); return; }
    const top = Math.max(anchor.top, 0), bottom = Math.min(anchor.bottom, height);
    const x = Math.max(edge, Math.min(anchor.left + (anchor.width - box.width) / 2, width - box.width - edge));
    let y = top - box.height - gap;
    if (y < edge) y = bottom + gap;
    y = Math.max(edge, Math.min(y, height - box.height - edge));
    tipEl.style.left = Math.round(x) + 'px'; tipEl.style.top = Math.round(y) + 'px';
  }
  function tipShow(image, host) {
    const text = tipText(image);
    if (!text) { tipHide(); return; }
    if (!tipEl) {
      tipEl = document.createElement('div');
      tipEl.className = 'mw-frame-tip'; tipEl.id = 'mw-frame-tip'; tipEl.setAttribute('role', 'tooltip');
      document.documentElement.append(tipEl);
    }
    tipEl.textContent = text; tipImage = image;
    if (tipHost !== host) {
      tipHost = host;
      host.setAttribute('aria-describedby', [host.getAttribute('aria-describedby'), 'mw-frame-tip'].filter(Boolean).join(' '));
    }
    tipPosition();
  }
  function tipRefresh(image) {
    if (!tipEl || image !== tipImage) return;
    if (!tipText(image)) tipHide(); else tipShow(image, tipHost || image);
  }
  const tipListeners = [
    ['pointerover', event => {
      if (!event.isTrusted || event.pointerType === 'touch') return;
      const image = tipImageOf(event.target);
      if (!image || (image === tipImage && (tipEl || tipTimer))) return;
      tipHide();
      if (!tipText(image)) return;
      tipImage = image;
      tipTimer = setTimeout(() => { tipTimer = 0; tipShow(image, image); }, 400);
    }, true],
    ['pointerout', event => {
      if (!tipImage || tipImageOf(event.target) !== tipImage) return;
      if (event.relatedTarget && tipImageOf(event.relatedTarget) === tipImage) return;
      tipHide();
    }, true],
    ['focusin', event => {
      const image = tipImageOf(event.target);
      tipHide();
      if (!image || !event.isTrusted) return;
      try { if (!event.target.matches(':focus-visible')) return; } catch { /* engine without :focus-visible */ }
      tipShow(image, event.target);
    }, false],
    ['focusout', tipHide, false],
    ['keydown', event => { if (event.key === 'Escape') tipHide(); }, false],
    ['pointerdown', tipHide, true],
  ];
  for (const [type, listener, capture] of tipListeners) document.addEventListener(type, listener, capture);
  addEventListener('resize', tipHide);
  addEventListener('message', initialize);
  addEventListener('scroll', reportScroll, { passive: true });
  addEventListener('load', () => { settled = true; restore(); restoreScroll = null; document.querySelectorAll('img[data-mw-image-unsized]').forEach(image => { if (image.getBoundingClientRect().width || image.getBoundingClientRect().height) image.removeAttribute('data-mw-image-unsized'); }); }, { once: true });
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
  addEventListener('pagehide', () => { tipHide(); for (const [type, listener, capture] of tipListeners) document.removeEventListener(type, listener, capture); removeEventListener('resize', tipHide); imagesClosed = true; imageObserver?.disconnect(); imageStates.clear(); document.removeEventListener('load', imageSettled, true); document.removeEventListener('error', imageSettled, true); port?.close(); port = null; removeEventListener('message', initialize); document.removeEventListener('DOMContentLoaded', parsed); }, { once: true });
  parent.postMessage({ type: 'mw-reader-address-ready' }, '*');
})();
