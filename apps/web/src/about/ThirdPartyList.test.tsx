import { describe, it, expect, vi } from 'vitest';
import axe from 'axe-core';
import { fireEvent, render, screen, waitFor, within } from '@solidjs/testing-library';
import { ThirdPartyList, PAGE_SIZE } from './ThirdPartyList.tsx';
import { aboutEntries, licenceIdsOf, loadLicenceTexts, safeHref, type ThirdPartyEntry } from './thirdParty.ts';
import doc from './third-party.json';

const ENTRIES: ThirdPartyEntry[] = [
  {
    group: 'web',
    kind: 'npm',
    name: 'solid-js',
    version: '1.9.14',
    licence: 'MIT',
    repository: 'https://github.com/solidjs/solid',
    homepage: 'https://solidjs.com',
    texts: [{ id: 'mit1', file: 'LICENSE' }],
  },
  {
    group: 'rust',
    kind: 'crate',
    name: 'tokio',
    version: '1.53.3',
    licence: 'MIT',
    repository: 'https://github.com/tokio-rs/tokio',
    texts: [{ id: 'mit2', file: 'LICENSE' }],
    artifacts: ['server', 'desktop'],
  },
  {
    group: 'rust',
    kind: 'crate',
    name: 'pulley-interpreter',
    version: '49.0.4',
    licence: 'Apache-2.0 WITH LLVM-exception',
    texts: [{ id: 'apache', standard: 'Apache-2.0' }],
    artifacts: ['server'],
  },
  {
    group: 'buildTools',
    kind: 'crate',
    name: 'syn',
    version: '2.0.0',
    licence: 'Apache-2.0 OR MIT',
    homepage: 'https://github.com/dtolnay/syn',
    texts: [],
    artifacts: ['server'],
  },
  {
    group: 'assets',
    kind: 'font',
    name: 'Jost',
    version: '5.3.0',
    licence: 'OFL-1.1',
    repository: 'https://github.com/indestructible-type/Jost',
    texts: [{ id: 'ofl', file: 'OFL.txt' }],
    note: 'bundled',
  },
  {
    group: 'runtimeImage',
    kind: 'os',
    name: 'libc6',
    version: '2.36-9+deb12u14',
    licence: '',
    homepage: 'https://www.gnu.org/software/libc/libc.html',
    texts: [{ id: 'glibc', file: '/usr/share/doc/libc6/copyright' }],
    images: ['runtime'],
  },
];

const TEXTS = {
  mit1: 'MIT text one',
  mit2: 'MIT text two',
  apache: 'Apache License text',
  ofl: 'SIL OPEN FONT LICENSE',
  glibc: 'GNU C Library copyright',
};

const names = (): string[] =>
  screen.getAllByTestId('third-party-entry').map((li) => li.querySelector('h4 span')?.textContent ?? '');

const chip = (label: RegExp): HTMLElement =>
  within(screen.getByRole('group', { name: 'Filter by licence' })).getByRole('button', { name: label });

const renderList = (
  entries: ThirdPartyEntry[] = ENTRIES,
  load: (ids: readonly string[]) => Promise<Record<string, string>> = async () => TEXTS,
) =>
  render(() => <ThirdPartyList entries={entries} loadTexts={load} />);

describe('ThirdPartyList', () => {
  it('groups entries by origin, in a fixed order, with a count each', () => {
    renderList();
    const headings = screen.getAllByRole('heading', { level: 3 }).map((h) => h.textContent);
    expect(headings).toEqual([
      'Rust crates in the server, the web app and the plugins (2)',
      'npm packages in the web app (1)',
      'Fonts, icons and images (1)',
      'Server image files (1)',
      'Build tools: run while building, not shipped (1)',
    ]);
    const rust = screen.getByRole('region', { name: /^Rust crates in the server/ });
    expect(within(rust).getAllByTestId('third-party-entry')).toHaveLength(2);
  });

  it('shows a count per licence and filters by one; empty groups disappear', () => {
    renderList();
    expect(chip(/^All \(6\)$/)).toHaveAttribute('aria-pressed', 'true');
    expect(chip(/^MIT \(3\)$/)).toBeInTheDocument();
    expect(chip(/^Apache-2\.0 \(2\)$/)).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /LLVM-exception/ })).toBeNull();

    fireEvent.click(chip(/^Apache-2\.0/));
    expect(names()).toEqual(['pulley-interpreter', 'syn']);
    expect(screen.getAllByRole('heading', { level: 3 }).map((h) => h.textContent)).toEqual([
      'Rust crates in the server, the web app and the plugins (1)',
      'Build tools: run while building, not shipped (1)',
    ]);
    expect(screen.getByTestId('third-party-summary')).toHaveTextContent('Showing 2 of 2 entries.');
    fireEvent.click(chip(/^Apache-2\.0/));
    expect(names()).toHaveLength(6);
  });

  it('searches name, version and licence, and offers to clear when nothing matches', () => {
    renderList();
    const box = screen.getByRole('searchbox', { name: 'Search by name, version or licence' });
    fireEvent.input(box, { target: { value: 'TOK' } });
    expect(names()).toEqual(['tokio']);
    fireEvent.input(box, { target: { value: '49.0' } });
    expect(names()).toEqual(['pulley-interpreter']);
    fireEvent.input(box, { target: { value: 'ofl' } });
    expect(names()).toEqual(['Jost']);
    fireEvent.input(box, { target: { value: 'no-such-package' } });
    expect(screen.queryAllByTestId('third-party-entry')).toHaveLength(0);
    expect(screen.getByText('Nothing matches this search and licence.')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Clear search and filter' }));
    expect(names()).toHaveLength(6);
    expect(box).toHaveValue('');
  });

  it('every link opens in a new tab with rel="noopener noreferrer" and says so to screen readers', () => {
    const { container } = renderList();
    const links = [...container.querySelectorAll('a')];
    expect(links).toHaveLength(6);
    for (const a of links) {
      expect(a.getAttribute('href')).toMatch(/^https:\/\//);
      expect(a).toHaveAttribute('target', '_blank');
      expect(a.getAttribute('rel')?.split(/\s+/).sort()).toEqual(['noopener', 'noreferrer']);
      expect(a).toHaveAccessibleName(/\(opens in a new tab\)$/);
    }
    expect(screen.getByRole('link', { name: 'Source code for solid-js (opens in a new tab)' })).toHaveAttribute(
      'href',
      'https://github.com/solidjs/solid',
    );
  });

  it('opens the licence texts in a modal dialog, loads them once, and returns focus on Escape', async () => {
    const load = vi.fn(async (_ids: readonly string[]) => TEXTS);
    renderList(ENTRIES, load);
    expect(load).not.toHaveBeenCalled();
    const entry = screen.getAllByTestId('third-party-entry').find((li) => li.textContent?.includes('pulley-interpreter'))!;
    const open = within(entry).getByRole('button', { name: 'Licence text for pulley-interpreter' });
    open.focus();
    fireEvent.click(open);
    const dialog = await screen.findByRole('dialog', { name: 'Licence text: pulley-interpreter 49.0.4' });
    expect(dialog).toHaveAttribute('aria-modal', 'true');
    await waitFor(() => expect(within(dialog).getByText('Apache License text')).toBeInTheDocument());
    expect(within(dialog).getByRole('heading', { level: 3 })).toHaveTextContent(
      'Apache-2.0 licence (standard text, no file in package)',
    );
    expect(within(dialog).getByLabelText('Licence text of pulley-interpreter: Apache-2.0')).toHaveAttribute('tabindex', '0');
    await waitFor(() => expect(dialog.contains(document.activeElement)).toBe(true));
    fireEvent.keyDown(dialog, { key: 'Escape' });
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(document.activeElement).toBe(open);
    fireEvent.click(open);
    await screen.findByRole('dialog');
    fireEvent.click(screen.getByRole('button', { name: 'Close' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(load).toHaveBeenCalledTimes(1);
    expect(load).toHaveBeenCalledWith(['apache']);
    // an entry without any text says so instead of offering an empty dialog
    const syn = screen.getAllByTestId('third-party-entry').find((li) => li.textContent?.includes('syn'))!;
    expect(within(syn).getByText('No licence text is available for this entry.')).toBeInTheDocument();
  });

  it('a failed text load shows an error with a retry that works', async () => {
    let fail = true;
    const load = vi.fn(async (_ids: readonly string[]) => {
      if (fail) throw new Error('offline');
      return TEXTS;
    });
    renderList(ENTRIES, load);
    fireEvent.click(screen.getByRole('button', { name: 'Licence text for solid-js' }));
    const dialog = await screen.findByRole('dialog');
    await waitFor(() => expect(within(dialog).getByRole('alert')).toHaveTextContent('The licence texts could not be loaded.'));
    fail = false;
    fireEvent.click(within(dialog).getByRole('button', { name: 'Try again' }));
    await waitFor(() => expect(within(dialog).getByText('MIT text one')).toBeInTheDocument());
  });

  it('renders a page at a time per group with a Show more button', () => {
    const many: ThirdPartyEntry[] = Array.from({ length: PAGE_SIZE + 7 }, (_, i) => ({
      group: 'rust',
      kind: 'crate',
      name: `crate-${String(i).padStart(3, '0')}`,
      version: '1.0.0',
      licence: 'MIT',
      texts: [],
    }));
    renderList(many, async () => ({}));
    expect(screen.getAllByTestId('third-party-entry')).toHaveLength(PAGE_SIZE);
    fireEvent.click(screen.getByRole('button', { name: 'Show more (7 left)' }));
    expect(screen.getAllByTestId('third-party-entry')).toHaveLength(PAGE_SIZE + 7);
    expect(screen.queryByRole('button', { name: /Show more/ })).toBeNull();
  });

  it('says where a crate is used, which image a package is in, and what an asset is', () => {
    renderList();
    expect(screen.getByText('Used in: server, desktop app')).toBeInTheDocument();
    expect(screen.getByText('Bundled with the web app.')).toBeInTheDocument();
    expect(screen.getByText('In: the distroless runtime image')).toBeInTheDocument();
    const libc = screen.getAllByTestId('third-party-entry').find((li) => li.textContent?.includes('libc6'))!;
    expect(within(libc).getByText('see the copyright file')).toBeInTheDocument();
  });

  it('has no axe violations with the licence dialog open (colour checks need a browser: e76)', async () => {
    const { container } = renderList();
    fireEvent.click(screen.getByRole('button', { name: 'Licence text for solid-js' }));
    const dialog = await screen.findByRole('dialog');
    await waitFor(() => expect(within(dialog).getByText('MIT text one')).toBeInTheDocument());
    const opts = { rules: { 'color-contrast': { enabled: false }, region: { enabled: false } } };
    for (const node of [container, dialog]) {
      const result = await axe.run(node, opts);
      expect(result.violations.map((v) => `${v.id}: ${v.nodes.map((n) => n.target.join(' ')).join(', ')}`)).toEqual([]);
    }
  });

  it('defaults to the committed list when no entries are passed', () => {
    render(() => <ThirdPartyList loadTexts={async () => TEXTS} />);
    expect(screen.getByRole('region', { name: /^npm packages in the web app/ })).toBeInTheDocument();
    expect(screen.getByRole('region', { name: /^Build tools/ })).toBeInTheDocument();
  });
});

describe('third-party data', () => {
  it('drops a non-https link and a malformed entry', () => {
    const entries = aboutEntries({
      rust: [
        { kind: 'crate', name: 'x', version: '1', licence: 'MIT', repository: 'http://example.com/x', homepage: 'javascript:alert(1)', texts: [] },
        { kind: 'crate', licence: 'MIT' },
      ],
    });
    expect(entries).toHaveLength(1);
    expect(entries[0]!.group).toBe('rust');
    expect(entries[0]!.repository).toBeUndefined();
    expect(entries[0]!.homepage).toBeUndefined();
    expect(safeHref('https://example.com/a')).toBe('https://example.com/a');
    expect(safeHref('https://u:p@example.com/')).toBeUndefined();
    expect(licenceIdsOf('(MIT OR Apache-2.0 WITH LLVM-exception) AND GPL-2.0+')).toEqual(['MIT', 'Apache-2.0', 'GPL-2.0']);
  });

  it('the committed list is complete: every group present, every entry has a text, every text id resolves', async () => {
    const all = aboutEntries(doc);
    for (const g of ['rust', 'shells', 'web', 'assets', 'runtimeImage', 'buildTools'] as const) {
      expect(all.some((e) => e.group === g), g).toBe(true);
    }
    expect(all.filter((e) => e.group === 'rust').length).toBeGreaterThan(300);
    expect(all.some((e) => e.kind === 'font' && e.name === 'Jost')).toBe(true);
    const texts = await loadLicenceTexts(all.flatMap((e) => e.texts.map((r) => r.id)));
    for (const e of all) {
      expect(e.texts.length, `${e.name} has a licence text`).toBeGreaterThan(0);
      for (const ref of e.texts) expect(texts[ref.id], `${e.name} ${ref.id}`).toBeTypeOf('string');
      for (const href of [e.repository, e.homepage]) if (href) expect(href).toMatch(/^https:\/\//);
    }
  });
});
