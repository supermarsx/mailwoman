// The rich editor's message background while writing (t39 L12).
//
// The Format tab's View group has a Dark background button. It changes only
// what the writer sees: the surface's `data-surface` attribute picks a dark or
// a light palette (`richtext.css`), and a ProseMirror decoration plugin gives
// text whose own colour would be hard to read on that surface a readable
// colour ON SCREEN. Decorations never reach the document, so `htmlFromDoc`
// (the HTML that is sent or saved as a signature) is the same with the button
// on or off. There is no option to send the dark background.
//
// The choice is the appearance preference `composeBackground` (synced per user
// with the other appearance settings): `theme` until the user presses the
// button, then `dark` or `light`. Pressing it flips what is on screen, so it
// works the same whether the app theme is light or dark.

import { Plugin, PluginKey, type EditorState } from 'prosemirror-state';
import { Decoration, DecorationSet } from 'prosemirror-view';
import type { Node as PMNode } from 'prosemirror-model';
import { safeColor } from './richFormatting.ts';
import type { ComposeBackground } from '../../theme/appearance.ts';

export type SurfaceTone = 'dark' | 'light';

/** What the surface shows: the explicit pick, or the app's appearance. */
export function surfaceTone(pref: ComposeBackground, appearance: SurfaceTone): SurfaceTone {
  return pref === 'theme' ? appearance : pref;
}

/** The preference after a press of the button: the opposite of what is on screen. */
export function toggledBackground(pref: ComposeBackground, appearance: SurfaceTone): ComposeBackground {
  return surfaceTone(pref, appearance) === 'dark' ? 'light' : 'dark';
}

type Rgb = readonly [number, number, number];

function rgbOf(hex: string): Rgb {
  return [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16)) as unknown as Rgb;
}

function hexOf(rgb: Rgb): string {
  return '#' + rgb.map((v) => Math.round(Math.min(255, Math.max(0, v))).toString(16).padStart(2, '0')).join('');
}

/** A CSS `rgb()`/`rgba()` or hex colour as `#rrggbb`; `null` for anything else or a transparent one. */
export function cssColorHex(value: string): string | null {
  const hex = safeColor(value);
  if (hex) return hex;
  const m = /^rgba?\(\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*(\d{1,3})\s*(?:,\s*([\d.]+)\s*)?\)$/.exec(value.trim().toLowerCase());
  if (!m) return null;
  if (m[4] !== undefined && Number(m[4]) === 0) return null;
  return hexOf([Number(m[1]), Number(m[2]), Number(m[3])]);
}

function luminance(rgb: Rgb): number {
  const [r, g, b] = rgb.map((v) => {
    const c = v / 255;
    return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  }) as unknown as Rgb;
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/** WCAG contrast ratio between two `#rrggbb` colours. */
export function contrastRatio(a: string, b: string): number {
  const la = luminance(rgbOf(a));
  const lb = luminance(rgbOf(b));
  return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05);
}

/** Below this the user's colour is shown adjusted. */
export const UNREADABLE_BELOW = 3;
/** An adjusted colour reaches at least this. */
export const READABLE_TARGET = 4.5;

/**
 * The colour to show for text the user coloured `color` on a `background`
 * surface, or `null` when `color` is readable as it is. The hue is kept: the
 * colour is mixed towards white (dark surface) or black (light surface) until
 * it reaches `READABLE_TARGET`.
 */
export function readableColor(color: string, background: string): string | null {
  const fg = safeColor(color);
  const bg = safeColor(background);
  if (!fg || !bg || contrastRatio(fg, bg) >= UNREADABLE_BELOW) return null;
  const towards: Rgb = luminance(rgbOf(bg)) < 0.18 ? [255, 255, 255] : [0, 0, 0];
  const from = rgbOf(fg);
  for (let step = 1; step <= 20; step++) {
    const k = step / 20;
    const mixed = hexOf(from.map((v, i) => v + (towards[i]! - v) * k) as unknown as Rgb);
    if (contrastRatio(mixed, bg) >= READABLE_TARGET) return mixed;
  }
  return hexOf(towards);
}

export const viewColorKey = new PluginKey<DecorationSet>('mw-view-color');
/** Transaction meta: the surface background changed, recompute. */
export const SURFACE_CHANGED = 'mw-surface-changed';

function shaded(node: PMNode): boolean {
  return safeColor(node.attrs['shade']) !== null;
}

/** The view-only colour decorations for `doc` on a `background` surface. */
export function viewColorDecorations(doc: PMNode, background: string | null): DecorationSet {
  if (!background) return DecorationSet.empty;
  const out: Decoration[] = [];
  const walk = (node: PMNode, pos: number, underShade: boolean): void => {
    node.forEach((child, offset) => {
      const at = pos + offset;
      if (child.isText) {
        if (underShade) return;
        let color: string | null = null;
        for (const mark of child.marks) {
          // Text on its own highlight keeps the pair the user chose.
          if (mark.type.name === 'highlight') return;
          if (mark.type.name === 'textColor') color = safeColor(mark.attrs['value']);
        }
        const shown = color ? readableColor(color, background) : null;
        if (shown) out.push(Decoration.inline(at, at + child.nodeSize, { style: `color:${shown}`, 'data-mw-view-color': shown }));
        return;
      }
      walk(child, at + 1, underShade || shaded(child));
    });
  };
  walk(doc, 0, false);
  return DecorationSet.create(doc, out);
}

/**
 * The plugin. `background()` reads the surface's painted background (a
 * `#rrggbb`, or `null` when unknown); dispatch a transaction with the
 * `SURFACE_CHANGED` meta after the surface changes.
 */
export function viewColorPlugin(background: () => string | null): Plugin<DecorationSet> {
  return new Plugin<DecorationSet>({
    key: viewColorKey,
    state: {
      init: (_config, state: EditorState) => viewColorDecorations(state.doc, background()),
      apply: (tr, old) =>
        tr.docChanged || tr.getMeta(SURFACE_CHANGED) === true ? viewColorDecorations(tr.doc, background()) : old,
    },
    props: {
      decorations: (state) => viewColorKey.getState(state) ?? DecorationSet.empty,
    },
  });
}
