// Client-side allowlist sanitizer for the notes rich-text editor (plan §3 e6).
//
// Notes bodies are held as plaintext HTML on the client and sealed only at rest
// server-side (plan §1.6); the client is therefore responsible for sanitizing
// the editor's `contentEditable` output before it is sent, reusing the app's
// allowlist discipline (the mail Reader renders server-sanitized HTML in a
// sandboxed iframe — here we own the source, so we sanitize at authoring time).
//
// The rule is an ALLOWLIST, not a blocklist: only known-safe elements survive,
// everything else is unwrapped (children kept) or dropped (script/style content
// discarded). No `<script>`, event handler, `style`, or `javascript:`/`data:`
// URL can survive a round-trip through `sanitizeNoteHtml`.

/** Inline + block elements a note body may contain. */
const ALLOWED_TAGS = new Set<string>([
  'p', 'br', 'div', 'span',
  'b', 'strong', 'i', 'em', 'u', 's', 'strike', 'del', 'mark', 'sub', 'sup',
  'a',
  'ul', 'ol', 'li',
  'h1', 'h2', 'h3', 'h4',
  'blockquote', 'code', 'pre', 'hr',
]);

/** Elements whose entire subtree is discarded (never merely unwrapped). */
const DROP_WITH_CONTENT = new Set<string>([
  'script', 'style', 'iframe', 'object', 'embed', 'template', 'noscript', 'link', 'meta', 'title', 'head',
]);

/** Per-tag attribute allowlist. Anything not listed here is stripped. */
const ALLOWED_ATTRS: Record<string, Set<string>> = {
  a: new Set(['href', 'title']),
};

/** URL schemes a note link may point at (incl. the `mailwoman:` cross-link). */
const SAFE_URL = /^(https?:|mailto:|mailwoman:|#|\/)/i;

/** Is `href` a scheme we allow (rejecting `javascript:`, `data:`, etc.)? */
export function isSafeHref(href: string): boolean {
  // Strip whitespace (tab/newline/space) first so an obfuscated scheme like
  // `java\tscript:` or `  javascript:` cannot slip past the prefix test.
  const normalized = href.replace(/\s+/g, '');
  return SAFE_URL.test(normalized);
}

/** Recursively copy `src`'s allowed descendants into `dst` (an owned document). */
function copyChildren(src: Node, dst: Node, doc: Document): void {
  for (const child of Array.from(src.childNodes)) {
    if (child.nodeType === 3 /* text */) {
      dst.appendChild(doc.createTextNode(child.textContent ?? ''));
      continue;
    }
    if (child.nodeType !== 1 /* element */) continue; // comments, PIs → dropped
    const el = child as Element;
    const tag = el.tagName.toLowerCase();

    if (DROP_WITH_CONTENT.has(tag)) continue; // subtree discarded entirely

    if (!ALLOWED_TAGS.has(tag)) {
      // Unknown but not dangerous: unwrap — keep the (sanitized) children.
      copyChildren(el, dst, doc);
      continue;
    }

    const clean = doc.createElement(tag);
    const allowed = ALLOWED_ATTRS[tag];
    if (allowed !== undefined) {
      for (const attr of Array.from(el.attributes)) {
        const name = attr.name.toLowerCase();
        if (!allowed.has(name)) continue;
        if (name === 'href' && !isSafeHref(attr.value)) continue;
        clean.setAttribute(name, attr.value);
      }
    }
    // Links always open without leaking the opener / referrer.
    if (tag === 'a' && clean.hasAttribute('href')) {
      clean.setAttribute('rel', 'noopener noreferrer nofollow');
    }
    copyChildren(el, clean, doc);
    dst.appendChild(clean);
  }
}

/**
 * Return an allowlist-sanitized copy of `html`: only safe elements/attributes
 * survive — no scripts, no event handlers, no unsafe URLs. Idempotent: running
 * it twice yields the same output.
 */
export function sanitizeNoteHtml(html: string): string {
  if (html.length === 0) return '';
  const doc = new DOMParser().parseFromString(`<body>${html}</body>`, 'text/html');
  const out = doc.createElement('div');
  copyChildren(doc.body, out, doc);
  return out.innerHTML;
}

/** Plain-text projection of a note body (for previews + the body search scan). */
export function htmlToText(html: string): string {
  const doc = new DOMParser().parseFromString(`<body>${html}</body>`, 'text/html');
  return (doc.body.textContent ?? '').replace(/\s+/g, ' ').trim();
}
