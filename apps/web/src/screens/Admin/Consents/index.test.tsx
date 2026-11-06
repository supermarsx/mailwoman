import { describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import Consents, { type ConsentOverview, type ConsentsApi } from './index.tsx';
import { COMPLIANCE_TABS } from '../Compliance/tabs.ts';
import { renderInShell } from '../shell/testkit.tsx';

const overview = (over: Partial<ConsentOverview> = {}): ConsentOverview => ({
  from: 1_700_000_000, to: 1_700_086_400, total: 5,
  purposes: [
    { purpose: 'assistant', granted: 3, withdrawn: 1, withdrawalRate: 0.25, grantEvents: 4, withdrawEvents: 1 },
    { purpose: 'assistant.summarize', granted: 0, withdrawn: 2, withdrawalRate: 1, grantEvents: 0, withdrawEvents: 2 },
  ],
  series: [{ t: 1_700_000_000, granted: 4, withdrawn: 1 }],
  ...over,
});
const api = (over: Partial<ConsentsApi> = {}): ConsentsApi => ({
  stats: vi.fn(async () => overview()),
  exportCsv: vi.fn(async () => 'time,domain\r\n'),
  settings: vi.fn(async () => []),
  ...over,
});

describe('Admin consents', () => {
  it('is registered in the Compliance hub with its own tab id', () => {
    expect(COMPLIANCE_TABS.some((t) => t.id === 'consents')).toBe(true);
  });

  it('shows counts and the withdrawal rate per purpose, including a feature opt-out', async () => {
    renderInShell(() => <Consents api={api()} />);
    const table = await screen.findByTestId('consent-table');
    expect(within(table).getByTestId('consent-row-assistant')).toHaveTextContent('25%');
    expect(within(table).getByTestId('consent-row-assistant.summarize')).toHaveTextContent('100%');
    cleanup();
  });

  it('draws the withdrawal-rate and daily graphs from the stats', async () => {
    renderInShell(() => <Consents api={api()} />);
    expect(await screen.findByTestId('consent-chart-rate')).toBeInTheDocument();
    expect(await screen.findByTestId('consent-chart-daily')).toBeInTheDocument();
    cleanup();
  });

  it('exports the ledger for the chosen range', async () => {
    const a = api();
    (globalThis.URL as unknown as { createObjectURL: () => string }).createObjectURL = () => 'blob:x';
    (globalThis.URL as unknown as { revokeObjectURL: () => void }).revokeObjectURL = () => {};
    renderInShell(() => <Consents api={a} />);
    fireEvent.click(await screen.findByTestId('consent-export'));
    await waitFor(() => expect(a.exportCsv).toHaveBeenCalledTimes(1));
    cleanup();
  });

  it('says so when nobody has decided yet', async () => {
    renderInShell(() => <Consents api={api({ stats: vi.fn(async () => overview({ purposes: [], series: [], total: 0 })) })} />);
    expect(await screen.findByText('No consent decisions yet.', { selector: 'td' })).toBeInTheDocument();
    cleanup();
  });
});
