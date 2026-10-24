import { describe, it, expect, vi } from 'vitest';
import axe from 'axe-core';
import { fireEvent, render, screen, waitFor, within } from '@solidjs/testing-library';
import { ThirdPartyList, PAGE_SIZE } from './ThirdPartyList.tsx';
import { crateEntries, licenceIdsOf, loadLicenceTexts, safeHref, webEntries, type ThirdPartyEntry } from './thirdParty.ts';
import webDoc from './third-party.web.json';
import cratesDoc from '../../../../crates/mw-server/src/about/third-party.crates.json';

const ENTRIES: ThirdPartyEntry[] = [
  {
    kind: 'npm',
    name: 'solid-js',
    version: '1.9.14',
    licence: 'MIT',
    repository: 'https://github.com/solidjs/solid',
    homepage: 'https://solidjs.com',
    texts: [{ id: 'mit1', file: 'LICENSE' }],
  },
  {
    kind: 'crate',
    name: 'tokio',
    version: '1.53.3',
    licence: 'MIT',
    repository: 'https://github.com/tokio-rs/tokio',
    texts: [{ id: 'mit2', file: 'LICENSE' }],
    artifacts: ['server', 'desktop'],
  },
  {
    kind: 'crate',
    name: 'pulley-interpreter',
    version: '49.0.4',
    licence: 'Apache-2.0 WITH LLVM-exception',
    texts: [{ id: 'apache', standard: 'Apache-2.0' }],
    artifacts: ['server'],
  },
  {
    kind: 'crate',
    name: 'addr2line',
    version: '0.26.1',
    licence: 'Apache-2.0 OR MIT',
    homepage: 'https://github.com/gimli-rs/addr2line',
    texts: [],
    artifacts: ['server'],
  },
  {
    kind: 'font',
    name: 'Jost',
    version: '5.3.0',
    licence: 'OFL-1.1',
    repository: 'https://github.com/indestructible-type/Jost',
    texts: [{ id: 'ofl', file: 'OFL.txt' }],
    note: 'bundled',
  },
];

const TEXTS = { mit1: 'MIT text one', mit2: 'MIT text two', apache: 'Apache License text', ofl: 'SIL OPEN FONT LICENSE' };

const names = (): string[] =>
  screen.getAllByTestId('third-party-entry').map((li) => li.querySelector('h3 span')?.textContent ?? '');

const chip = (label: RegExp): HTMLElement =>
  within(screen.getByRole('group', { name: 'Filter by licence' })).getByRole('button', { name: label });

describe('ThirdPartyList', () => {
  it('shows a count per licence and filters by one', () => {
    render(() => <ThirdPartyList entries={ENTRIES} loadTexts={async () => TEXTS} />);
    expect(chip(/^All \(5\)$/)).toHaveAttribute('aria-pressed', 'true');
    // MIT: solid-js, tokio, addr2line (an OR counts for each of its licences)
    expect(chip(/^MIT \(3\)$/)).toBeInTheDocument();
    expect(chip(/^Apache-2\.0 \(2\)$/)).toBeInTheDocument();
    expect(chip(/^OFL-1\.1 \(1\)$/)).toBeInTheDocument();
    // the exception is not offered as a licence
    expect(screen.queryByRole('button', { name: /LLVM-exception/ })).toBeNull();

    fireEvent.click(chip(/^Apache-2\.0/));
    expect(chip(/^Apache-2\.0/)).toHaveAttribute('aria-pressed', 'true');
    expect(chip(/^All/)).toHaveAttribute('aria-pressed', 'false');
    expect(names()).toEqual(['pulley-interpreter', 'addr2line']);
    expect(screen.getByTestId('third-party-summary')).toHaveTextContent('Showing 2 of 2 entries.');

    // pressing the active chip again clears it
    fireEvent.click(chip(/^Apache-2\.0/));
    expect(names()).toHaveLength(5);
  });

  it('searches name, version and licence, and offers to clear when nothing matches', () => {
    render(() => <ThirdPartyList entries={ENTRIES} loadTexts={async () => TEXTS} />);
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
    expect(names()).toHaveLength(5);
    expect(box).toHaveValue('');
  });

  it('every link opens in a new tab with rel="noopener noreferrer" and says so to screen readers', () => {
    const { container } = render(() => <ThirdPartyList entries={ENTRIES} loadTexts={async () => TEXTS} />);
    const links = [...container.querySelectorAll('a')];
    expect(links).toHaveLength(5);
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
    expect(screen.getByRole('link', { name: 'Home page for addr2line (opens in a new tab)' })).toBeInTheDocument();
  });

  it('loads the licence texts on first open, once, and marks a standard text', async () => {
    const load = vi.fn(async (_ids: readonly string[]) => TEXTS);
    render(() => <ThirdPartyList entries={ENTRIES} loadTexts={load} />);
    expect(load).not.toHaveBeenCalled();
    const entry = screen.getAllByTestId('third-party-entry')[2]!;
    const details = entry.querySelector('details')!;
    expect(within(entry).getByText('Licence text')).toBeInTheDocument();
    details.open = true;
    fireEvent(details, new Event('toggle'));
    await waitFor(() => expect(within(entry).getByText('Apache License text')).toBeInTheDocument());
    expect(within(entry).getByRole('heading', { level: 4 })).toHaveTextContent(
      'Apache-2.0 licence (standard text, no file in package)',
    );
    // the scroll box is focusable and named
    expect(within(entry).getByLabelText('Licence text of pulley-interpreter: Apache-2.0')).toHaveAttribute('tabindex', '0');
    details.open = false;
    fireEvent(details, new Event('toggle'));
    details.open = true;
    fireEvent(details, new Event('toggle'));
    expect(load).toHaveBeenCalledTimes(1);
    expect(load).toHaveBeenCalledWith(['apache']);
    // an entry without any text says so instead of an empty disclosure
    expect(within(screen.getAllByTestId('third-party-entry')[3]!).getByText('No licence text is available for this entry.')).toBeInTheDocument();
  });

  it('a failed text load shows an error with a retry that works', async () => {
    let fail = true;
    const load = vi.fn(async () => {
      if (fail) throw new Error('offline');
      return TEXTS;
    });
    render(() => <ThirdPartyList entries={ENTRIES} loadTexts={load} />);
    const entry = screen.getAllByTestId('third-party-entry')[0]!;
    const details = entry.querySelector('details')!;
    details.open = true;
    fireEvent(details, new Event('toggle'));
    await waitFor(() => expect(within(entry).getByRole('alert')).toHaveTextContent('The licence texts could not be loaded.'));
    fail = false;
    fireEvent.click(within(entry).getByRole('button', { name: 'Try again' }));
    await waitFor(() => expect(within(entry).getByText('MIT text one')).toBeInTheDocument());
  });

  it('renders a page at a time with a Show more button', () => {
    const many: ThirdPartyEntry[] = Array.from({ length: PAGE_SIZE + 7 }, (_, i) => ({
      kind: 'crate',
      name: `crate-${String(i).padStart(3, '0')}`,
      version: '1.0.0',
      licence: 'MIT',
      texts: [],
    }));
    render(() => <ThirdPartyList entries={many} loadTexts={async () => ({})} />);
    expect(screen.getAllByTestId('third-party-entry')).toHaveLength(PAGE_SIZE);
    fireEvent.click(screen.getByRole('button', { name: 'Show more (7 left)' }));
    expect(screen.getAllByTestId('third-party-entry')).toHaveLength(PAGE_SIZE + 7);
    expect(screen.queryByRole('button', { name: /Show more/ })).toBeNull();
  });

  it('has no axe violations with a licence text open (colour checks need a browser: e76)', async () => {
    const { container } = render(() => <ThirdPartyList entries={ENTRIES} loadTexts={async () => TEXTS} />);
    const entry = screen.getAllByTestId('third-party-entry')[0]!;
    const details = entry.querySelector('details')!;
    details.open = true;
    fireEvent(details, new Event('toggle'));
    await waitFor(() => expect(within(entry).getByText('MIT text one')).toBeInTheDocument());
    const result = await axe.run(container, { rules: { 'color-contrast': { enabled: false }, region: { enabled: false } } });
    expect(result.violations.map((v) => `${v.id}: ${v.nodes.map((n) => n.target.join(' ')).join(', ')}`)).toEqual([]);
  });

  it('shows where a crate is used and what an asset is', () => {
    render(() => <ThirdPartyList entries={ENTRIES} loadTexts={async () => TEXTS} />);
    expect(screen.getByText('Used in: server, desktop app')).toBeInTheDocument();
    expect(screen.getByText('Bundled with the web app.')).toBeInTheDocument();
  });
});

describe('third-party data', () => {
  it('drops a non-https link from data that came over the network', () => {
    const [e] = crateEntries({
      packages: [
        { name: 'x', version: '1', licence: 'MIT', repository: 'http://example.com/x', homepage: 'javascript:alert(1)', texts: [] },
      ],
    });
    expect(e!.repository).toBeUndefined();
    expect(e!.homepage).toBeUndefined();
    expect(safeHref('https://example.com/a')).toBe('https://example.com/a');
    expect(safeHref('https://u:p@example.com/')).toBeUndefined();
    expect(licenceIdsOf('(MIT OR Apache-2.0 WITH LLVM-exception) AND GPL-2.0+')).toEqual(['MIT', 'Apache-2.0', 'GPL-2.0']);
  });

  it('the committed lists are complete: every entry has a licence and every text id resolves', async () => {
    const web = webEntries(webDoc);
    const crates = crateEntries(cratesDoc);
    expect(web.length).toBeGreaterThan(20);
    expect(crates.length).toBeGreaterThan(500);
    expect(web.some((e) => e.kind === 'font' && e.name === 'Jost')).toBe(true);
    const all = [...web, ...crates];
    const texts = await loadLicenceTexts(all.flatMap((e) => e.texts.map((r) => r.id)));
    for (const e of all) {
      expect(e.licence, e.name).not.toBe('');
      expect(e.texts.length, `${e.name} has a licence text`).toBeGreaterThan(0);
      for (const ref of e.texts) expect(texts[ref.id], `${e.name} ${ref.id}`).toBeTypeOf('string');
      for (const href of [e.repository, e.homepage]) if (href) expect(href).toMatch(/^https:\/\//);
    }
  });
});
