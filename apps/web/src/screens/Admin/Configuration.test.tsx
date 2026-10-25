import { describe, expect, it } from 'vitest';
import { fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { Configuration } from './Configuration.tsx';
import type { EffectiveEntry } from './shell/adminApi.ts';
import { renderInShell } from './shell/testkit.tsx';
import { expectNoAxeViolations } from './shell/uxTestkit.ts';

const rows: EffectiveEntry[] = [
  { id: 'MW_SERVER_KEY', group: 'server', class: 'readOnly', value: { set: true }, source: 'boot', valueStatus: 'resolved', restartNeeded: true },
  { id: 'unreadable', group: 'mail', class: 'readOnly', value: null, configuredInput: 'declared-only', source: 'boot', valueStatus: 'unavailable' },
  { id: 'cap.import', group: 'features', class: 'ui', value: false, source: 'stored', valueStatus: 'resolvedPolicy', href: '#/settings?field=cap.import', locked: 'global' },
];
const load = async () => ({ kind: 'ok' as const, data: rows, etag: 'one' });

describe('effective configuration', () => {
  it('distinguishes runtime-unknown inputs, hidden secrets and policy edits', async () => {
    history.replaceState(null, '', '/admin#/config');
    const { container } = renderInShell(() => <Configuration load={load} />);
    expect(await screen.findByText('Secret configured (hidden)')).toBeInTheDocument();
    expect(screen.getByText('Runtime value unavailable')).toBeInTheDocument();
    expect(screen.getByText('declared-only')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Open settings editor' })).toHaveAttribute('href', '/admin#/settings/cap.import');
    expect(screen.getByText('Policy locked')).toBeInTheDocument();
    await expectNoAxeViolations(container);
  });
  it('filters by field links, search and area without changing values', async () => {
    history.replaceState(null, '', '/admin#/config?field=cap.import');
    renderInShell(() => <Configuration load={load} />);
    await screen.findByText('cap.import');
    expect(screen.queryByText('MW_SERVER_KEY')).toBeNull();
    fireEvent.input(screen.getByRole('searchbox'), { target: { value: '' } });
    fireEvent.change(screen.getByRole('combobox', { name: 'Area' }), { target: { value: 'server' } });
    expect(screen.getByText('MW_SERVER_KEY')).toBeInTheDocument();
    expect(screen.queryByText('cap.import')).toBeNull();
  });
  it('shows unavailable and clears old values after a reload error', async () => {
    history.replaceState(null, '', '/admin#/config');
    let first = true;
    renderInShell(() => <Configuration load={async () => {
      if (first) { first = false; return load(); }
      throw new Error('offline');
    }} />);
    await screen.findByText('MW_SERVER_KEY');
    fireEvent.click(screen.getByRole('button', { name: 'Refresh configuration' }));
    await screen.findByRole('alert');
    await waitFor(() => expect(screen.queryByText('MW_SERVER_KEY')).toBeNull());
  });
});
