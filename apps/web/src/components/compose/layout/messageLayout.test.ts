import { expect, it } from 'vitest';
import { LAYOUT_ATTR, joinLayout, quoteStart, resolveTheme, sanitiseLayout, splitLayout, type MessageLayout } from './messageLayout.ts';
import { buildColumnsHtml } from './columns.ts';

const root = (html: string): HTMLElement => { const d = document.createElement('div'); d.innerHTML = html; return d; };

it('an empty layout leaves the HTML untouched', () => {
  expect(joinLayout({}, '<p>x</p>')).toBe('<p>x</p>');
  expect(splitLayout('<p>x</p>')).toEqual({ layout: {}, html: '<p>x</p>' });
});

it('join then split round-trips content and layout', () => {
  const l: MessageLayout = { theme: 'newsletter', pageColor: '#102030', width: 640, font: { family: 'Georgia, serif', size: 16, color: '#222222' }, spacing: 'relaxed', dir: 'rtl' };
  const html = `<p>Hello <a href="https://example.com">link</a></p><h2>Head</h2>${buildColumnsHtml({ widths: [50, 50] }, ['a', 'b'])}`;
  const joined = joinLayout(l, html);
  expect(joined).toContain(LAYOUT_ATTR);
  expect(joined).toContain('bgcolor="#102030"');
  expect(joined).toContain('max-width:640px');
  expect(joined).toContain('font-size:16px');
  expect(joined).toContain('dir="rtl"');
  const back = splitLayout(joined);
  expect(back.layout).toEqual(l);
  // Theme removal is clean: nothing the theme added stays on the content.
  expect(back.html).not.toContain('data-mw-l');
  expect(back.html).toContain('<h2>Head</h2>');
  expect(root(back.html).querySelector('p')!.getAttribute('style')).toBeNull();
  expect(root(back.html).querySelector('a')!.getAttribute('style')).toBeNull();
  expect(root(back.html).querySelectorAll('table[data-mw-columns]').length).toBe(1);
});

it('wraps only the part above a reply quote', () => {
  const html = '<p>my answer</p><p>On Monday, Bob wrote:</p><blockquote><p>original</p></blockquote>';
  expect(quoteStart(root(html))).toBe(1);
  const joined = joinLayout({ theme: 'classic' }, html);
  const wrapper = root(joined).querySelector('table[data-mw-theme]')!;
  expect(wrapper.textContent).toContain('my answer');
  expect(wrapper.textContent).not.toContain('original');
  expect(wrapper.textContent).not.toContain('wrote');
  expect(joined.endsWith('</blockquote>')).toBe(true);
  expect(splitLayout(joined).html).toBe(html);
});

it('a forward-style body without blockquote is wrapped whole', () => {
  expect(quoteStart(root('<p>a</p><p>b</p>'))).toBe(-1);
});

it('sanitises layouts read from a draft', () => {
  const bad = sanitiseLayout({ theme: 'evil', pageColor: 'red;background:url(x)', width: 99999, font: { family: 'a;}</style>', size: 4, color: '#GGG' }, spacing: 'huge', dir: 'up' });
  expect(bad).toEqual({});
  expect(sanitiseLayout({ pageColor: '#ABCDEF', width: 'full' })).toEqual({ pageColor: '#abcdef', width: 'full' });
  expect(splitLayout(`<table ${LAYOUT_ATTR}='not json'><tr><td data-mw-content="1"><p>kept</p></td></tr></table>`).html).toBe('<p>kept</p>');
});

it('overrides win over the theme and "full" removes the width', () => {
  expect(resolveTheme({ theme: 'newsletter', width: 'full' }).width).toBeUndefined();
  expect(resolveTheme({ theme: 'newsletter', width: 700 }).width).toBe(700);
  expect(resolveTheme({ theme: 'modern', font: { size: 18 } }).fontSize).toBe(18);
});

it('the sent page colour is separate from the view-only dark background', () => {
  const joined = joinLayout({ pageColor: '#ffffff' }, '<p>x</p>');
  expect(joined).not.toMatch(/data-mw-view|viewColor|dark/i);
});
