// Themed tooltips for images inside the message frame.
//
// The message body is an opaque-origin srcdoc, so a `title` on an <img> (or on a
// link that wraps only an image) would draw the browser's NATIVE tooltip: OS
// styled, unthemed, not stylable. This module
//   1. moves such titles into `data-mw-tip` (no native tooltip, text kept), and
//   2. supplies the tooltip's CSS (`--mw-tip-*` tokens from the app theme, always,
//      independent of the reader colour mode) and its strings, which the
//      hash-authorized frame handler (crates/mw-server/src/reader-address-handler.js)
//      draws on hover/focus. Ordinary text links keep their native `title`.

import { THEMES } from '../theme/tokens.ts';
import type { ThemeName } from '../theme/contract.css.ts';

/** Localized strings for the three placeholder states, resolved by the parent. */
export interface FrameTipLabels { blocked: string; loading: string; error: string }
export interface FrameTips { theme: ThemeName; labels: FrameTipLabels; invert?: boolean }

/** `data-mw-tip` is read by the frame handler; sender text, never markup. */
export const TIP_ATTR = 'data-mw-tip';

/** Remove every native tooltip source from body images. Run on already sanitized HTML. */
export function withoutImageTitles(html: string): string {
  if (!/<img\b/i.test(html) || !/\btitle\s*=/i.test(html) || typeof DOMParser === 'undefined') return html;
  let doc: Document;
  try { doc = new DOMParser().parseFromString(html, 'text/html'); } catch { return html; }
  let changed = false;
  const move = (from: Element, img: Element): void => {
    const title = (from.getAttribute('title') ?? '').trim();
    from.removeAttribute('title');
    changed = true;
    if (title && !img.hasAttribute(TIP_ATTR)) img.setAttribute(TIP_ATTR, title.slice(0, 300));
  };
  for (const img of Array.from(doc.body.querySelectorAll('img[title]'))) move(img, img);
  for (const a of Array.from(doc.body.querySelectorAll('a[title],area[title]'))) {
    const imgs = a.querySelectorAll('img');
    // Only a link whose sole content is one image: a text link keeps its title.
    if (imgs.length === 1 && (a.textContent ?? '').trim() === '') move(a, imgs[0]!);
  }
  return changed ? doc.body.innerHTML : html;
}

/** `:root` tokens for the tooltip, from the same palette the app tooltip uses. */
export function frameTipVars(theme: ThemeName): string {
  const t = THEMES[theme] ?? THEMES.light;
  return `:root{--mw-tip-bg:${t.color.surface};--mw-tip-text:${t.color.text};--mw-tip-border:${t.color.borderStrong};` +
    `--mw-tip-radius:${t.radius.sm};--mw-tip-shadow:${t.elevation[2]};--mw-tip-font:${t.font.ui};--mw-tip-size:13px}`;
}

/** Tooltip styling. Padding and width are em of the bubble's own 13px (the app bubble is 0.8125rem), so the
 *  reader's font-size/density setting, which resizes the message root, never resizes this chrome. */
export const FRAME_TIP_STYLE =
  '.mw-frame-tip{position:fixed;z-index:2147483647;box-sizing:border-box;margin:0;max-width:min(27em,calc(100vw - 1em));' +
  'padding:.615em .923em;overflow-wrap:anywhere;white-space:normal;pointer-events:none;-webkit-user-select:none;user-select:none;' +
  'color:var(--mw-tip-text,#1c1e21);background:var(--mw-tip-bg,#fff);border:1px solid var(--mw-tip-border,#bdb8ad);' +
  'border-radius:var(--mw-tip-radius,0);box-shadow:var(--mw-tip-shadow,.25em .25em 0 0 #111);' +
  'font:normal var(--mw-tip-size,13px)/1.45 var(--mw-tip-font,system-ui,sans-serif);letter-spacing:normal;text-align:start;unicode-bidi:plaintext;' +
  'animation:mw-frame-tip-in .12s ease-out}' +
  '@keyframes mw-frame-tip-in{from{opacity:0}to{opacity:1}}' +
  '@media(prefers-reduced-motion:reduce){.mw-frame-tip{animation:none}}' +
  '@media(forced-colors:active){.mw-frame-tip{color:CanvasText;background:Canvas;border-color:CanvasText;box-shadow:none}}';

/** Everything `bodyFrameDoc` adds for the tooltip: style text and `<body>` attributes. */
export function frameTipParts(tips: FrameTips | undefined): { css: string; bodyAttrs: string } {
  if (!tips) return { css: '', bodyAttrs: '' };
  const attr = (name: string, value: string): string => ` data-mw-tip-${name}="${value.replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/</g, '&lt;').slice(0, 200)}"`;
  return {
    css: frameTipVars(tips.theme) + FRAME_TIP_STYLE + (tips.invert ? '.mw-frame-tip{filter:invert(1) hue-rotate(180deg)}' : ''),
    bodyAttrs: attr('blocked', tips.labels.blocked) + attr('loading', tips.labels.loading) + attr('error', tips.labels.error),
  };
}
