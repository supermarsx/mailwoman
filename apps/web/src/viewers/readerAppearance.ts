export type ReaderColorMode = 'original' | 'adaptive' | 'invert';
export interface ReaderAppearance {
  colorMode: ReaderColorMode;
  fontSize: number;
  lineHeight: number;
  contentWidth: 'comfortable' | 'wide' | 'full';
}

/** Only validated primitives enter CSS; message content cannot supply rules. */
export function readerAppearanceCss(value: ReaderAppearance): string {
  const font = Number.isFinite(value.fontSize) ? Math.max(12, Math.min(24, value.fontSize)) : 14;
  const line = Number.isFinite(value.lineHeight) ? Math.max(1.2, Math.min(2.2, value.lineHeight)) : 1.6;
  const width = value.contentWidth === 'comfortable' ? '76ch' : value.contentWidth === 'wide' ? '110ch' : 'none';
  let css = `html{padding:0}body{box-sizing:border-box;font-size:${font}px;line-height:${line};max-width:${width};margin-inline:auto}pre{font-size:inherit;line-height:inherit}`;
  if (value.colorMode === 'adaptive') {
    css += 'html,body{background:var(--mw-color-bg)!important;color:var(--mw-color-text)!important}' +
      'body *:not(img):not(video):not(svg):not(path){color:inherit!important;background-color:transparent!important;border-color:var(--mw-color-border)!important}' +
      'body a,body a *{color:var(--mw-color-link)!important}';
  } else {
    css += 'html,body{background:#fff;color:#1c1e21;color-scheme:light}';
    if (value.colorMode === 'invert') css += 'html{filter:invert(1) hue-rotate(180deg)}img,video,svg{filter:invert(1) hue-rotate(180deg)}';
  }
  return css + '@media print{html{filter:none!important}html,body,body *{color:#000!important;background:transparent!important;filter:none!important}body{max-width:none}}';
}

const PAINT_PROPERTIES = new Set(['color', 'background', 'background-color', 'background-image', 'text-shadow']);

/** Split an inline style into declarations at top-level `;` (not inside quotes or `url(...)`). */
function declarations(style: string): string[] {
  const parts: string[] = [];
  let depth = 0, quote = '', start = 0;
  for (let i = 0; i < style.length; i++) {
    const c = style[i]!;
    if (quote) { if (c === '\\') i++; else if (c === quote) quote = ''; }
    else if (c === '"' || c === "'") quote = c;
    else if (c === '(') depth++;
    else if (c === ')') depth = Math.max(0, depth - 1);
    else if (c === ';' && depth === 0) { parts.push(style.slice(start, i)); start = i + 1; }
  }
  parts.push(style.slice(start));
  return parts.filter(part => part.trim() !== '');
}

/** Drop sender paint overrides only in adaptive mode, retaining layout/media.
 *
 *  Works on the attribute text, not on `element.style`: a `DOMParser` document
 *  inherits the app's CSP, and Chromium does not parse inline style attributes
 *  there under `style-src` without 'unsafe-inline' (its `element.style` is empty),
 *  so CSSOM removal silently kept every sender colour in Chromium only. */
export function adaptReaderColors(html: string): string {
  if (typeof DOMParser === 'undefined') return html;
  const doc = new DOMParser().parseFromString(html, 'text/html');
  for (const element of doc.body.querySelectorAll<HTMLElement>('*')) {
    const style = element.getAttribute('style');
    if (style !== null) {
      const kept = declarations(style).filter(part => !PAINT_PROPERTIES.has(part.slice(0, part.indexOf(':')).trim().toLowerCase()));
      if (kept.length) element.setAttribute('style', kept.map(part => part.trim()).join(';'));
      else element.removeAttribute('style');
    }
    for (const attribute of ['bgcolor', 'color', 'text']) element.removeAttribute(attribute);
  }
  return doc.body.innerHTML;
}
