import { afterEach, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { CollapsibleBar, RibbonPanel } from '../../bars/index.ts';

const map = vi.hoisted(() => ({ off: new Set<string>() }));
vi.mock('../../../api/settings.ts', () => ({
  settings: () => Object.fromEntries([...map.off].map((id) => [`compose.feature.${id}`, { value: false, locked: true, source: 'domain' }])),
  loadSettings: vi.fn(async () => undefined),
}));

import { LayoutGroups, decodeFont, encodeFont } from './LayoutGroups.tsx';
import type { MessageLayout } from './messageLayout.ts';

afterEach(() => { cleanup(); map.off.clear(); });

function harness(over: { plain?: boolean } = {}) {
  const [layout, setLayout] = createSignal<MessageLayout>({});
  const inserted: string[] = [];
  const replaced: string[] = [];
  const saved: string[] = [];
  let html = '';
  render(() => (
    <CollapsibleBar id="composerRibbon" revealPolicy="overlay" label="Ribbon" header={<span />}>
      <RibbonPanel label="Layout">
        <LayoutGroups layout={layout} update={(c) => setLayout(c)} insertHtml={(h) => inserted.push(h)} currentHtml={() => html} replaceHtml={(h) => replaced.push(h)} plain={() => over.plain === true} saveFont={(v) => { saved.push(v); }} />
      </RibbonPanel>
    </CollapsibleBar>
  ));
  return { layout, inserted, replaced, saved, setHtml: (h: string) => { html = h; } };
}

it('inserts mail-safe column tables from the presets', () => {
  const h = harness();
  fireEvent.click(screen.getByTestId('layout-cols-2'));
  fireEvent.click(screen.getByTestId('layout-cols-sidebarLeft'));
  expect(h.inserted[0]).toContain('data-mw-columns="2"');
  expect(h.inserted[1]).toContain('width="30%"');
});

it('sets width, alignment, spacing, themes and direction on the layout', async () => {
  const h = harness();
  fireEvent.change(screen.getByTestId('layout-width'), { target: { value: '640' } });
  fireEvent.change(screen.getByTestId('layout-align'), { target: { value: 'left' } });
  fireEvent.change(screen.getByTestId('layout-spacing'), { target: { value: 'compact' } });
  fireEvent.click(screen.getByTestId('layout-dir-rtl'));
  expect(h.layout()).toMatchObject({ width: 640, align: 'left', spacing: 'compact', dir: 'rtl' });
  fireEvent.change(screen.getByTestId('layout-width'), { target: { value: 'theme' } });
  expect(h.layout().width).toBeUndefined();
});

it('applies a theme from the gallery and removes it cleanly', async () => {
  const h = harness();
  fireEvent.click(screen.getByTestId('layout-themes'));
  fireEvent.click(await screen.findByTestId('layout-theme-newsletter'));
  expect(h.layout().theme).toBe('newsletter');
  fireEvent.change(screen.getByTestId('layout-width'), { target: { value: '800' } });
  fireEvent.click(screen.getByTestId('layout-theme-remove'));
  expect(h.layout()).toEqual({});
});

it('merges columns through the editor HTML', () => {
  const h = harness();
  fireEvent.click(screen.getByTestId('layout-cols-merge'));
  expect(h.replaced).toEqual([]);
  h.setHtml('<table data-mw-columns="2"><tr><td valign="top" width="50%" style="width:50%"><p>a</p></td><td valign="top" width="50%" style="width:50%"><p>b</p></td></tr></table>');
  fireEvent.click(screen.getByTestId('layout-cols-merge'));
  expect(h.replaced[0]).toContain('<p>a</p><p>b</p>');
});

it('saves the default font', () => {
  const h = harness();
  fireEvent.click(screen.getByTestId('layout-font-default'));
  expect(h.saved[0]).toMatch(/\|\d+\|/);
  expect(decodeFont(encodeFont({ family: 'Georgia, serif', size: 14, color: '#112233' }))).toEqual({ family: 'Georgia, serif', size: 14, color: '#112233' });
  expect(decodeFont('||junk')).toEqual({});
});

it('plain-text mode disables the controls with a tooltip', () => {
  harness({ plain: true });
  const select = screen.getByTestId('layout-width') as HTMLSelectElement;
  expect(select.disabled).toBe(true);
  expect(select.title).toMatch(/formatted message/);
  expect((screen.getByTestId('layout-cols-2') as HTMLButtonElement).disabled).toBe(true);
});

it('hides a group whose feature flag is off', () => {
  map.off.add('layout.columns');
  map.off.add('layout.themes');
  harness();
  expect(screen.queryByTestId('layout-group-columns')).toBeNull();
  expect(screen.queryByTestId('layout-group-themes')).toBeNull();
  expect(screen.getByTestId('layout-group-width')).toBeTruthy();
});
