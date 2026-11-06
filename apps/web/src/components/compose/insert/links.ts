// Link dialog logic: normalising the address, a safety check, the HTML for the
// rich editors and the "caption (url)" form for plain text.

export type LinkWarning = 'http' | 'shortener' | 'idn' | 'lookalike' | 'credentials' | 'script';

export interface LinkSpec {
  caption: string;
  href: string;
  title?: string;
  newTab?: boolean;
}

const SHORTENERS = new Set(['bit.ly', 'tinyurl.com', 't.co', 'goo.gl', 'ow.ly', 'is.gd', 'buff.ly', 'rebrand.ly', 'cutt.ly', 'shorturl.at']);
const BRANDS = ['paypal', 'google', 'microsoft', 'apple', 'amazon', 'facebook', 'netflix', 'github'];

/** Add a scheme where the user left it out: bare emails become mailto:, numbers tel:, hosts https:. */
export function normalizeHref(raw: string): string {
  const v = raw.trim();
  if (v === '') return '';
  if (v.startsWith('#') || /^(https?|mailto|tel):/i.test(v)) return v;
  if (/^[^\s@/]+@[^\s@/]+\.[^\s@/]+$/.test(v)) return `mailto:${v}`;
  if (/^\+?[\d\s().-]{6,}$/.test(v)) return `tel:${v.replace(/[^+\d]/g, '')}`;
  if (/^[a-z][a-z0-9+.-]*:/i.test(v)) return v;
  return `https://${v}`;
}

/** Safe-link check: what to warn about before the link goes out. */
export function checkLink(href: string): LinkWarning[] {
  const out: LinkWarning[] = [];
  if (/^\s*(javascript|data|vbscript):/i.test(href)) return ['script'];
  let url: URL;
  try { url = new URL(href); } catch { return out; }
  if (url.protocol === 'http:') out.push('http');
  if (url.protocol !== 'http:' && url.protocol !== 'https:') return out;
  const host = url.hostname.toLowerCase();
  if (url.username !== '' || url.password !== '') out.push('credentials');
  if (SHORTENERS.has(host)) out.push('shortener');
  // eslint-disable-next-line no-control-regex
  if (host.startsWith('xn--') || host.includes('.xn--') || /[^\x00-\x7f]/.test(host)) out.push('idn');
  const labels = host.split('.');
  const registrable = labels.length >= 2 ? labels[labels.length - 2]! : host;
  const squash = (v: string): string => v.replace(/0/g, 'o').replace(/[1l]/g, 'i').replace(/rn/g, 'm').replace(/vv/g, 'w');
  const squashed = squash(registrable);
  for (const brand of BRANDS) {
    const subdomainBrand = labels.length > 2 && labels.slice(0, -2).join('.').includes(brand);
    const hyphenBrand = registrable.includes(brand) && registrable.includes('-');
    if (registrable !== brand && (squashed === squash(brand) || subdomainBrand || hyphenBrand)) {
      out.push('lookalike');
      break;
    }
  }
  return out;
}

const esc = (s: string): string => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');

/** The anchor. A new tab always carries noopener noreferrer; external links carry rel either way. */
export function linkHtml(spec: LinkSpec): string {
  const caption = spec.caption.trim() === '' ? spec.href : spec.caption;
  const external = /^https?:/i.test(spec.href);
  const attrs = [`href="${esc(spec.href)}"`];
  if (spec.title) attrs.push(`title="${esc(spec.title)}"`);
  if (spec.newTab === true && external) attrs.push('target="_blank"');
  if (external) attrs.push('rel="noopener noreferrer"');
  return `<a ${attrs.join(' ')}>${esc(caption)}</a>`;
}

/** Plain-text mode: `caption (url)`, or the bare address when the caption adds nothing. */
export function linkText(spec: LinkSpec): string {
  const bare = spec.href.replace(/^mailto:|^tel:/i, '');
  const caption = spec.caption.trim();
  if (caption === '' || caption === spec.href || caption === bare) return bare;
  return `${caption} (${bare})`;
}

/** A URL on the clipboard, or null. */
export function urlFromClipboard(text: string): string | null {
  const v = text.trim();
  return /^https?:\/\/\S+$/i.test(v) ? v : null;
}
