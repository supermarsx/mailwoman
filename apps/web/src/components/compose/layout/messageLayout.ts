// The message layout: theme, page colour, content width, default font, spacing and
// direction. It is held outside the editors (so the ProseMirror and native HTML
// editors need no schema change) and written into the saved or sent HTML as one
// wrapper table around the part ABOVE a reply quote. A draft that is opened again
// is split back into this state and the plain content, so the layout round-trips
// and a theme can be removed without leaving styles behind.

import { SPACING, THEMES, safeHex, themeWrapper, type MessageTheme, type SpacingPreset, type ThemeId } from './themes.ts';

export interface LayoutFont { family?: string; size?: number; color?: string }

export interface MessageLayout {
  theme?: ThemeId;
  /** Message background (sent). `undefined` = automatic. */
  pageColor?: string;
  contentColor?: string;
  width?: number | 'full';
  align?: 'left' | 'center';
  font?: LayoutFont;
  linkColor?: string;
  spacing?: SpacingPreset;
  rounded?: boolean;
  border?: boolean;
  dir?: 'ltr' | 'rtl';
  padding?: number;
}

export const LAYOUT_ATTR = 'data-mw-layout';
const MARK = 'data-mw-l';

export function isEmptyLayout(l: MessageLayout): boolean {
  return Object.values(l).every((v) => v === undefined || (typeof v === 'object' && v !== null && Object.values(v).every((x) => x === undefined)));
}

const SAFE_FONT = /^[A-Za-z0-9 ,'"-]{1,160}$/;

/** Validate a layout read from a draft; unknown or unsafe members are dropped. */
export function sanitiseLayout(raw: unknown): MessageLayout {
  if (raw === null || typeof raw !== 'object') return {};
  const r = raw as Record<string, unknown>;
  const out: MessageLayout = {};
  if (typeof r['theme'] === 'string' && THEMES.some((t) => t.id === r['theme'])) out.theme = r['theme'] as ThemeId;
  const page = safeHex(r['pageColor'] as string | undefined); if (page) out.pageColor = page;
  const card = safeHex(r['contentColor'] as string | undefined); if (card) out.contentColor = card;
  const link = safeHex(r['linkColor'] as string | undefined); if (link) out.linkColor = link;
  if (r['width'] === 'full') out.width = 'full';
  else if (typeof r['width'] === 'number' && r['width'] >= 240 && r['width'] <= 1200) out.width = Math.round(r['width']);
  if (r['align'] === 'left' || r['align'] === 'center') out.align = r['align'];
  if (typeof r['spacing'] === 'string' && r['spacing'] in SPACING) out.spacing = r['spacing'] as SpacingPreset;
  if (r['rounded'] === true) out.rounded = true;
  if (r['border'] === true) out.border = true;
  if (r['dir'] === 'ltr' || r['dir'] === 'rtl') out.dir = r['dir'];
  if (typeof r['padding'] === 'number' && r['padding'] >= 0 && r['padding'] <= 48) out.padding = Math.round(r['padding']);
  const f = r['font'];
  if (f !== null && typeof f === 'object') {
    const font: LayoutFont = {};
    const fr = f as Record<string, unknown>;
    if (typeof fr['family'] === 'string' && SAFE_FONT.test(fr['family'])) font.family = fr['family'];
    if (typeof fr['size'] === 'number' && fr['size'] >= 8 && fr['size'] <= 48) font.size = Math.round(fr['size']);
    const c = safeHex(fr['color'] as string | undefined); if (c) font.color = c;
    if (Object.keys(font).length) out.font = font;
  }
  return out;
}

/** The theme with the layout's own choices on top. */
export function resolveTheme(l: MessageLayout): MessageTheme {
  const base = THEMES.find((t) => t.id === (l.theme ?? 'plain')) ?? THEMES[0]!;
  const t: MessageTheme = { ...base };
  if (l.width === 'full') delete (t as { width?: number }).width;
  else if (typeof l.width === 'number') t.width = l.width;
  if (l.align) t.align = l.align;
  if (l.font?.family) t.fontFamily = l.font.family;
  if (l.font?.size) t.fontSize = l.font.size;
  if (l.font?.color) t.textColor = l.font.color;
  if (l.linkColor) t.linkColor = l.linkColor;
  if (l.spacing) t.paragraphGap = SPACING[l.spacing];
  if (l.padding !== undefined) t.padding = l.padding;
  return t;
}

const escAttr = (s: string): string => s.replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/</g, '&lt;');

function inertRoot(html: string): HTMLElement {
  const doc = new DOMParser().parseFromString(`<body>${html}</body>`, 'text/html');
  return doc.body;
}

/** Index in `body.childNodes` where the quoted original starts, or -1. */
export function quoteStart(body: HTMLElement): number {
  const nodes = Array.from(body.childNodes);
  const q = nodes.findIndex((n) => n.nodeType === 1 && ((n as Element).tagName === 'BLOCKQUOTE' || (n as Element).hasAttribute('data-mw-quote')));
  if (q < 0) return -1;
  const prev = nodes[q - 1] as Element | undefined;
  // The attribution line ("On ... wrote:") belongs to the quote.
  if (prev && prev.nodeType === 1 && prev.tagName === 'P' && (prev.textContent ?? '').trim() !== '') return q - 1;
  return q;
}

/** Style the user's content for the theme, marking what was touched so `splitLayout` can undo it. */
function styleContent(root: HTMLElement, theme: MessageTheme): void {
  const touch = (el: HTMLElement, prop: string, value: string): void => {
    if (el.style.getPropertyValue(prop) !== '') return;
    el.style.setProperty(prop, value);
    el.setAttribute(MARK, `${el.getAttribute(MARK) ?? ''} ${prop}`.trim());
  };
  for (const a of Array.from(root.querySelectorAll<HTMLElement>('a'))) touch(a, 'color', theme.linkColor);
  for (const p of Array.from(root.querySelectorAll<HTMLElement>('p'))) if (!p.closest('blockquote')) touch(p, 'margin', `0 0 ${theme.paragraphGap}px 0`);
  if (theme.headingFamily) for (const h of Array.from(root.querySelectorAll<HTMLElement>('h1,h2,h3,h4,h5,h6'))) touch(h, 'font-family', theme.headingFamily);
}

function unstyleContent(root: HTMLElement): void {
  for (const el of Array.from(root.querySelectorAll<HTMLElement>(`[${MARK}]`))) {
    for (const prop of (el.getAttribute(MARK) ?? '').split(' ')) if (prop) el.style.removeProperty(prop);
    el.removeAttribute(MARK);
    if (el.getAttribute('style') === '') el.removeAttribute('style');
  }
}

/** The HTML to save or send: the part above any reply quote wrapped in the layout. */
export function joinLayout(l: MessageLayout, html: string): string {
  if (isEmptyLayout(l)) return html;
  const body = inertRoot(html);
  const at = quoteStart(body);
  const nodes = Array.from(body.childNodes);
  const above = at < 0 ? nodes : nodes.slice(0, at);
  const holder = body.ownerDocument.createElement('div');
  for (const n of above) holder.appendChild(n);
  const theme = resolveTheme(l);
  styleContent(holder, theme);
  let wrapped = themeWrapper(theme, holder.innerHTML, {
    pageColor: l.pageColor, contentColor: l.contentColor, border: l.border === true, rounded: l.rounded === true,
  });
  wrapped = wrapped.replace(`data-mw-theme="${theme.id}"`, `data-mw-theme="${theme.id}" ${LAYOUT_ATTR}="${escAttr(JSON.stringify(l))}"${l.dir ? ` dir="${l.dir}"` : ''}`);
  const rest = at < 0 ? '' : Array.from(body.childNodes).map((n) => (n.nodeType === 1 ? (n as Element).outerHTML : n.textContent ?? '')).join('');
  return wrapped + rest;
}

/** The inverse of `joinLayout`: the layout and the plain content. HTML without a layout comes back unchanged. */
export function splitLayout(html: string): { layout: MessageLayout; html: string } {
  if (!html.includes(LAYOUT_ATTR)) return { layout: {}, html };
  const body = inertRoot(html);
  const wrapper = Array.from(body.children).find((el) => el.tagName === 'TABLE' && el.hasAttribute(LAYOUT_ATTR)) as HTMLElement | undefined;
  if (!wrapper) return { layout: {}, html };
  let layout: MessageLayout = {};
  try { layout = sanitiseLayout(JSON.parse(wrapper.getAttribute(LAYOUT_ATTR) ?? '{}')); } catch { /* damaged marker: keep the content */ }
  const cell = wrapper.querySelector<HTMLElement>('td[data-mw-content]');
  const frag = body.ownerDocument.createDocumentFragment();
  if (cell) { unstyleContent(cell); while (cell.firstChild) frag.appendChild(cell.firstChild); }
  wrapper.replaceWith(frag);
  return { layout, html: body.innerHTML };
}

/** A short description for the ribbon ("Newsletter, 600 px"). */
export function describeLayout(l: MessageLayout): string {
  const parts: string[] = [];
  if (l.theme) parts.push(l.theme);
  if (l.width !== undefined) parts.push(l.width === 'full' ? 'full' : `${l.width}px`);
  return parts.join(', ');
}
