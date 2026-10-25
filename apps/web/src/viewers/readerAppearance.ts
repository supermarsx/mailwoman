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

/** Drop sender paint overrides only in adaptive mode, retaining layout/media. */
export function adaptReaderColors(html: string): string {
  if (typeof DOMParser === 'undefined') return html;
  const doc = new DOMParser().parseFromString(html, 'text/html');
  for (const element of doc.body.querySelectorAll<HTMLElement>('*')) {
    for (const property of ['color', 'background', 'background-color', 'background-image', 'text-shadow']) element.style?.removeProperty(property);
    for (const attribute of ['bgcolor', 'color', 'text']) element.removeAttribute(attribute);
  }
  return doc.body.innerHTML;
}
