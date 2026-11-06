import {chooseComposeFormat,structuredComposeEditor} from './compose/nativeHtmlTestHarness.ts';
// The composer ribbon on the t38 primitives: every tab is a RibbonPanel of
// 3-row RibbonGroups (the editor's groups joining their tab through a slot),
// the bar pushes the body instead of covering it, and in auto-hide mode a tab
// opens it, Esc closes it, and pointing at it while typing does not.
// Real geometry (88 px groups, no vertical overflow, the fold order) needs a
// layout engine; it is measured in Chromium and Firefox (logs/t38-e5.md).

import { beforeAll, beforeEach, describe, expect, it } from 'vitest';
import { fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { Compose } from './Compose.tsx';
import { renderWithApp } from './appHarness.tsx';
import { clearSettings, loadSettings, type Fetcher } from '../api/settings.ts';
import { HOVER_OPEN_MS } from './bars/index.ts';

beforeAll(() => {
  Object.defineProperty(Range.prototype, 'getClientRects', { configurable: true, value: () => [] });
  Object.defineProperty(Range.prototype, 'getBoundingClientRect', {
    configurable: true,
    value: () => ({ left: 0, right: 0, top: 0, bottom: 0, width: 0, height: 0 }),
  });
});

beforeEach(() => clearSettings());

function json(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });
}

async function ribbonMode(value: 'pinned' | 'auto'): Promise<void> {
  await loadSettings({
    fetcher: (async () =>
      json(200, { settings: { 'ui.bar.composerRibbon': { value, source: 'user', locked: false } } })) as Fetcher,
  });
}

function bar(): HTMLElement {
  const el = document.querySelector<HTMLElement>('[data-bar="composerRibbon"]');
  if (!el) throw new Error('no composer ribbon');
  return el;
}

const tabPanel = (tab: string): HTMLElement => bar().querySelector<HTMLElement>(`[data-ribbon-tab="${tab}"]`)!;
const groupIds = (tab: string): string[] =>
  Array.from(tabPanel(tab).querySelectorAll('[data-ribbon-group]')).map((g) => g.getAttribute('data-testid') ?? '');

describe('composer ribbon (t38)', () => {
  it('builds every tab from 3-row groups, with the editor groups inside their tab', async () => {
    renderWithApp(() => <Compose onClose={() => undefined} />);
    await structuredComposeEditor();
    await screen.findByTestId('rt-bold');
    expect(bar()).toHaveAttribute('data-reveal', 'overlay');
    expect(bar()).toHaveAttribute('data-mode', 'pinned');

    expect(groupIds('format')).toEqual([
      'rt-group-clipboard',
      'rt-group-painter',
      'rt-group-font',
      'rt-group-paragraph',
      'rt-group-styles',
      'rt-group-editing',
      'rt-group-view',
      'compose-group-recipients',
      'compose-group-message-type',
    ]);
    expect(groupIds('insert')).toEqual(['compose-response-templates', 'compose-attach', 'insert-group-illustrations', 'insert-group-links', 'insert-group-cards', 'insert-group-symbols', 'rt-group-table-insert']);
    expect(groupIds('table')).toEqual(['rt-group-table-rows', 'rt-group-table-columns', 'rt-group-table-cells', 'rt-group-table-shading', 'rt-group-table']);
    expect(groupIds('picture')).toEqual(['rt-group-picture-size', 'rt-group-picture-arrange', 'rt-group-picture-text', 'rt-group-picture-style', 'rt-group-picture-edit']);
    expect(groupIds('layout')).toEqual(['layout-group-columns', 'layout-group-width', 'layout-group-spacing', 'layout-group-page', 'layout-group-themes', 'layout-group-font', 'layout-group-direction']);
    expect(groupIds('security')).toEqual(['compose-crypto']);
    expect(groupIds('delivery')).toEqual(['compose-send-options', 'compose-delivery-priority', 'compose-delivery-receipts', 'compose-delivery-keys', 'compose-delivery-message-options', 'compose-delivery-saving']);

    // Only ribbon items sit in a group's grid: nothing outside the 3-row contract.
    for (const controls of Array.from(bar().querySelectorAll('[data-ribbon-controls]'))) {
      for (const child of Array.from(controls.children)) expect(child).toHaveAttribute('data-ribbon-item');
    }
    // One editor; its controls are the ribbon's, not a toolbar of its own.
    expect(screen.getAllByTestId('compose-richtext')).toHaveLength(1);
    expect(document.querySelector('.compose-rt__chrome')).toBeNull();
    // The always-live security evaluation is mounted while its tab is hidden.
    expect(tabPanel('security')).toHaveAttribute('hidden');
    expect(within(tabPanel('security')).getByTestId('encrypt-toggle')).toBeInTheDocument();
    // The ribbon sits in the flow above the body (push), never over it.
    const body = document.querySelector('.compose > .field--grow')!;
    expect(bar().compareDocumentPosition(body) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(bar().contains(body)).toBe(false);
  });

  it('shows the Table layout tab only while the caret is in a table, and Delivery groups carry titles', async () => {
    renderWithApp(() => <Compose onClose={() => undefined} />);
    await structuredComposeEditor();
    await screen.findByTestId('rt-bold');
    expect(screen.queryByRole('tab', { name: 'Table layout' })).toBeNull();
    fireEvent.click(screen.getByRole('tab', { name: 'Insert' }));
    fireEvent.click(screen.getByRole('button', { name: 'Insert table' }));
    await waitFor(() => expect(screen.getByRole('tab', { name: 'Table layout' })).toBeInTheDocument());
    fireEvent.click(screen.getByRole('tab', { name: 'Table layout' }));
    expect(within(tabPanel('table')).getByRole('button', { name: 'Row below' })).toBeEnabled();
    const titles = Array.from(tabPanel('delivery').querySelectorAll('[data-ribbon-group]')).map((g) => g.getAttribute('data-testid'));
    expect(titles).toContain('compose-delivery-receipts');
    for (const label of ['Send timing', 'Priority', 'Receipts and tracking', 'Keys and files', 'Sent copy']) expect(within(tabPanel('delivery')).getByText(label)).toBeInTheDocument();
  });

  it('drops the Table tab and its groups in plain text, and restores them in rich text', async () => {
    renderWithApp(() => <Compose onClose={() => undefined} />);
    await structuredComposeEditor();
    await screen.findByTestId('rt-bold');
    await chooseComposeFormat('plain');
    await waitFor(() => expect(screen.queryByTestId('rt-bold')).toBeNull());
    expect(screen.queryByRole('tab', { name: 'Table' })).toBeNull();
    expect(groupIds('format')).toEqual(['compose-group-recipients', 'compose-group-message-type']);
    await chooseComposeFormat('rich');
    await structuredComposeEditor();
    await screen.findByTestId('rt-bold');
    expect(groupIds('format')[0]).toBe('rt-group-clipboard');
  });

  it('auto-hide: a tab opens the ribbon in the flow and Esc closes it back to the tab', async () => {
    await ribbonMode('auto');
    renderWithApp(() => <Compose onClose={() => undefined} />);
    await structuredComposeEditor();
    await screen.findByTestId('rt-bold');
    expect(bar()).toHaveAttribute('data-mode', 'auto');
    // Selecting rich text is now a real menu gesture and focuses the ribbon.
    fireEvent.keyDown(screen.getByTestId('format-toggle'),{key:'Escape'});
    expect(bar()).toHaveAttribute('data-expanded', 'false');
    expect(document.getElementById('mw-bar-composerRibbon')).toHaveAttribute('inert');

    const insert = screen.getByRole('tab', { name: 'Insert' });
    insert.focus();
    fireEvent.click(insert);
    expect(bar()).toHaveAttribute('data-expanded', 'true');
    expect(insert).toHaveAttribute('aria-selected', 'true');
    expect(insert).toHaveAttribute('aria-expanded', 'true');
    expect(insert).toHaveAttribute('aria-controls', 'mw-bar-composerRibbon');
    expect(document.getElementById('mw-bar-composerRibbon')).not.toHaveAttribute('inert');

    fireEvent.keyDown(within(tabPanel('insert')).getByLabelText('Attach a file'), { key: 'Escape' });
    expect(bar()).toHaveAttribute('data-expanded', 'false');
    expect(document.activeElement).toBe(insert);

    // The open tab, clicked again, hides the ribbon (Office).
    fireEvent.click(insert);
    expect(bar()).toHaveAttribute('data-expanded', 'true');
    fireEvent.click(insert);
    expect(bar()).toHaveAttribute('data-expanded', 'false');
  });

  it('auto-hide: pointing at the ribbon while typing in the composer does not open it', async () => {
    await ribbonMode('auto');
    renderWithApp(() => <Compose onClose={() => undefined} />);
    await structuredComposeEditor();
    await screen.findByTestId('rt-bold');
    fireEvent.keyDown(screen.getByTestId('format-toggle'),{key:'Escape'});
    const subject = screen.getByLabelText('Subject');
    subject.focus();
    fireEvent.keyDown(subject, { key: 'a' });
    fireEvent.pointerEnter(bar());
    await new Promise((resolve) => setTimeout(resolve, HOVER_OPEN_MS + 100));
    expect(bar()).toHaveAttribute('data-expanded', 'false');

    // Not typing: the same hover opens it after the intent delay.
    fireEvent.pointerLeave(bar());
    subject.blur();
    fireEvent.pointerEnter(bar());
    await waitFor(() => expect(bar()).toHaveAttribute('data-expanded', 'true'));
  });

  it('auto-hide: asking for the signing key reveals the Security tab', async () => {
    await ribbonMode('auto');
    renderWithApp(() => <Compose onClose={() => undefined} />);
    await structuredComposeEditor();
    await screen.findByTestId('rt-bold');
    fireEvent.click(within(tabPanel('security')).getByTestId('sign-toggle'));
    await waitFor(() => expect(bar()).toHaveAttribute('data-expanded', 'true'));
    expect(screen.getByRole('tab', { name: 'Security' })).toHaveAttribute('aria-selected', 'true');
    expect(screen.getByTestId('compose-sign-unlock')).toBeInTheDocument();
  });
});
