// Message themes (stationery). A theme is inline styles on a wrapper table, so it
// survives every client; apply and remove are idempotent and touch only the
// wrapper marked `data-mw-theme`. The page colour here IS sent; the reader-side
// "view dark background" (messageBackground.ts) is a separate, view-only setting.

export type ThemeId = 'plain' | 'classic' | 'modern' | 'newsletter' | 'minimal';

export interface MessageTheme {
  id: ThemeId;
  labelKey: string;
  /** Message background; undefined = automatic (nothing painted). */
  pageColor?: string;
  /** Content card colour. */
  contentColor?: string;
  /** Content width px; undefined = full. */
  width?: number;
  align: 'left' | 'center';
  fontFamily: string;
  fontSize: number;
  textColor: string;
  linkColor: string;
  headingFamily?: string;
  accentBar?: string;
  border?: string;
  padding: number;
  paragraphGap: number;
  lineHeight: number;
}

const SANS = 'Arial, Helvetica, sans-serif';
const SERIF = 'Georgia, "Times New Roman", serif';

export const THEMES: readonly MessageTheme[] = [
  { id: 'plain', labelKey: 'layout-theme-plain', align: 'left', fontFamily: SANS, fontSize: 14, textColor: '#000000', linkColor: '#0b57d0', padding: 0, paragraphGap: 8, lineHeight: 1.4 },
  { id: 'classic', labelKey: 'layout-theme-classic', align: 'left', width: 700, contentColor: '#ffffff', fontFamily: SERIF, fontSize: 15, textColor: '#222222', linkColor: '#1a4d8f', headingFamily: SERIF, border: '1px solid #cfcfcf', padding: 16, paragraphGap: 10, lineHeight: 1.5 },
  { id: 'modern', labelKey: 'layout-theme-modern', align: 'left', width: 640, contentColor: '#ffffff', fontFamily: SANS, fontSize: 14, textColor: '#1f2933', linkColor: '#0b6bcb', accentBar: '#0b6bcb', padding: 16, paragraphGap: 10, lineHeight: 1.5 },
  { id: 'newsletter', labelKey: 'layout-theme-newsletter', align: 'center', width: 600, pageColor: '#eef1f5', contentColor: '#ffffff', fontFamily: SANS, fontSize: 15, textColor: '#2b2b2b', linkColor: '#0b6bcb', padding: 24, paragraphGap: 12, lineHeight: 1.55 },
  { id: 'minimal', labelKey: 'layout-theme-minimal', align: 'center', width: 560, fontFamily: SANS, fontSize: 14, textColor: '#4a4a4a', linkColor: '#4a4a4a', padding: 40, paragraphGap: 16, lineHeight: 1.7 },
];

export const THEME_ATTR = 'data-mw-theme';

const HEX = /^#(?:[0-9a-f]{3}|[0-9a-f]{6})$/i;
/** A colour value safe to place in a style attribute, or undefined. */
export function safeHex(v: string | undefined | null): string | undefined {
  const s = (v ?? '').trim();
  return HEX.test(s) ? s.toLowerCase() : undefined;
}

export interface PageOptions { pageColor?: string | undefined; contentColor?: string | undefined; border?: boolean; rounded?: boolean }

/** Wrapper opening/closing markup for a theme plus optional overrides. */
export function themeWrapper(theme: MessageTheme, innerHtml: string, page: PageOptions = {}): string {
  const pageColor = safeHex(page.pageColor) ?? theme.pageColor;
  const card = safeHex(page.contentColor) ?? theme.contentColor;
  const radius = page.rounded ? 'border-radius:8px;' : '';
  const border = page.border ? '1px solid #cfcfcf' : theme.border;
  const text = `font-family:${theme.fontFamily};font-size:${theme.fontSize}px;line-height:${theme.lineHeight};color:${theme.textColor};`;
  const width = theme.width === undefined ? '' : `max-width:${theme.width}px;`;
  const align = theme.align === 'center' ? ' align="center"' : '';
  const margin = theme.align === 'center' ? 'margin:0 auto;' : '';
  const bar = theme.accentBar ? `<tr><td height="4" style="height:4px;line-height:4px;font-size:0;background-color:${theme.accentBar}">&nbsp;</td></tr>` : '';
  const cardStyle = `${card ? `background-color:${card};` : ''}${border ? `border:${border};` : ''}${radius}padding:${theme.padding}px;${text}`;
  const inner = `<table role="presentation"${align} width="100%" cellpadding="0" cellspacing="0" border="0" style="width:100%;${width}${margin}border-collapse:separate">${bar}<tr><td valign="top" data-mw-content="1" style="${cardStyle}">${innerHtml}</td></tr></table>`;
  const outerBg = pageColor ? `bgcolor="${pageColor}" style="background-color:${pageColor};padding:${theme.width === undefined ? 0 : 16}px"` : 'style="padding:0"';
  return `<table role="presentation" ${THEME_ATTR}="${theme.id}" width="100%" cellpadding="0" cellspacing="0" border="0" style="width:100%;border-collapse:collapse"><tr><td ${outerBg}>${inner}</td></tr></table>`;
}

/** The wrapper currently around `root`'s content, if any. */
export function currentTheme(root: ParentNode): HTMLTableElement | null {
  return root.querySelector<HTMLTableElement>(`table[${THEME_ATTR}]`);
}

/** The cell that holds the user's content inside a theme wrapper. */
function contentCell(wrapper: HTMLTableElement): HTMLElement | null {
  return wrapper.querySelector<HTMLElement>("td[data-mw-content]");
}

/** Remove a theme: the user's content is lifted out of the wrapper unchanged. */
export function removeTheme(root: ParentNode): boolean {
  const wrapper = currentTheme(root);
  if (!wrapper) return false;
  const cell = contentCell(wrapper);
  const frag = wrapper.ownerDocument.createDocumentFragment();
  while (cell?.firstChild) frag.appendChild(cell.firstChild);
  wrapper.replaceWith(frag);
  return true;
}

/** Apply a theme to the editable body, replacing any previous one. Idempotent. */
export function applyTheme(body: HTMLElement, theme: MessageTheme, page: PageOptions = {}): void {
  removeTheme(body);
  const doc = body.ownerDocument;
  const html = themeWrapper(theme, '', page);
  const holder = doc.createElement('div');
  holder.innerHTML = html;
  const wrapper = holder.firstElementChild as HTMLTableElement;
  const cell = contentCell(wrapper)!;
  while (body.firstChild) cell.appendChild(body.firstChild);
  if (!cell.firstChild) cell.innerHTML = '<p><br></p>';
  body.appendChild(wrapper);
}

/** Paragraph spacing presets (px of space after a paragraph). */
export const SPACING = { compact: 4, normal: 10, relaxed: 18 } as const;
export type SpacingPreset = keyof typeof SPACING;
