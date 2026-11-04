// The message background while writing (t39 L12): the tone rule, the button's
// flip, the readable-colour rule, and the view-only decorations.
import { describe, expect, it } from 'vitest';
import { EditorState } from 'prosemirror-state';
import { docFromHtml, htmlFromDoc } from './richtext.ts';
import {
  autoTextColor,
  contrastRatio,
  cssColorHex,
  readableColor,
  READABLE_TARGET,
  surfaceTone,
  toggledBackground,
  viewColorDecorations,
  viewColorPlugin,
} from './messageBackground.ts';

const DARK = '#1e1f22';
const LIGHT = '#ffffff';

describe('message background', () => {
  it('follows the theme until the user picks, and the button flips what is on screen', () => {
    expect(surfaceTone('theme', 'dark')).toBe('dark');
    expect(surfaceTone('theme', 'light')).toBe('light');
    expect(surfaceTone('dark', 'light')).toBe('dark');
    expect(surfaceTone('light', 'dark')).toBe('light');
    expect(toggledBackground('theme', 'light')).toBe('dark');
    expect(toggledBackground('theme', 'dark')).toBe('light');
    expect(toggledBackground('dark', 'dark')).toBe('light');
    expect(toggledBackground('light', 'dark')).toBe('dark');
  });

  it('reads painted colours, and treats transparent as unknown', () => {
    expect(cssColorHex('rgb(30, 31, 34)')).toBe(DARK);
    expect(cssColorHex('rgba(255, 255, 255, 1)')).toBe(LIGHT);
    expect(cssColorHex('rgba(0, 0, 0, 0)')).toBeNull();
    expect(cssColorHex('')).toBeNull();
  });

  it('keeps readable colours and lifts unreadable ones to a readable shade', () => {
    expect(readableColor('#ffd700', DARK)).toBeNull();
    expect(readableColor('#c00000', LIGHT)).toBeNull();
    const onDark = readableColor('#000000', DARK)!;
    expect(onDark).not.toBeNull();
    expect(contrastRatio(onDark, DARK)).toBeGreaterThanOrEqual(READABLE_TARGET);
    // Light text the user typed (white, pale yellow) stays readable on the light surface.
    for (const pale of ['#ffffff', '#ffff99', '#e0e0e0']) {
      const shown = readableColor(pale, LIGHT)!;
      expect(shown, pale).not.toBeNull();
      expect(contrastRatio(shown, LIGHT)).toBeGreaterThanOrEqual(READABLE_TARGET);
    }
    // The hue is kept: dark blue on dark becomes a lighter blue, not grey.
    const blue = readableColor('#000080', DARK)!;
    expect(parseInt(blue.slice(5, 7), 16)).toBeGreaterThan(parseInt(blue.slice(1, 3), 16));
  });

  it('decorates only unreadable coloured text and never the HTML', () => {
    const html =
      '<p><span style="color: #000000">black</span> plain <span style="color: #ffd700">gold</span></p>' +
      '<p><span style="background-color: #ffff00"><span style="color: #000000">marked</span></span></p>' +
      '<p style="background-color: #ffffff"><span style="color: #000000">shaded</span></p>';
    const doc = docFromHtml(html);
    const decorations = viewColorDecorations(doc, DARK).find();
    expect(decorations).toHaveLength(1);
    expect(doc.textBetween(decorations[0]!.from, decorations[0]!.to)).toBe('black');
    expect(viewColorDecorations(doc, null).find()).toHaveLength(0);

    let background: string | null = DARK;
    const state = EditorState.create({ doc, plugins: [viewColorPlugin(() => background)] });
    const before = htmlFromDoc(state.doc);
    background = LIGHT;
    const next = state.apply(state.tr.setMeta('mw-surface-changed', true));
    expect(htmlFromDoc(next.doc)).toBe(before);
    // On white, black is readable and gold is not.
    const plugin = next.plugins[0]!;
    const light = (plugin.getState(next) as ReturnType<typeof viewColorDecorations>).find();
    expect(light.map((d) => next.doc.textBetween(d.from, d.to))).toEqual(['gold']);
  });

  it('picks black or white automatically for text that has no colour of its own', () => {
    expect(autoTextColor('#ffffff')).toBe('#000000');
    expect(autoTextColor('#1e1f22')).toBe('#ffffff');
    // Light surface text on a pale highlight or a dark shaded cell is unreadable; both get an automatic colour.
    const doc = docFromHtml(
      '<p>plain</p>' +
        '<p><span style="background-color: #ffff00">marked</span></p>' +
        '<p style="background-color: #202020">shaded</p>',
    );
    const found = viewColorDecorations(doc, DARK, '#e8e8ea').find();
    expect(found.map((d) => [doc.textBetween(d.from, d.to), (d.spec as { 'data-mw-view-color'?: string })['data-mw-view-color'] ?? (d as unknown as { type: { attrs: Record<string, string> } }).type.attrs['data-mw-view-color']])).toEqual([
      ['marked', '#000000'],
    ]);
    // A white-on-dark default survives a dark shade, so nothing is added there.
    expect(doc.textBetween(found[0]!.from, found[0]!.to)).toBe('marked');
  });
});
