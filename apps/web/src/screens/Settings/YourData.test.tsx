import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor, within } from '@solidjs/testing-library';
import { YourData } from './YourData.tsx';
import * as api from '../../api/privacy.ts';
import { requestedKind } from '../../api/privacyShortcut.ts';
import type { ConsentView, PrivacyOverview } from '../../api/privacy.ts';

vi.mock('../../api/privacy.ts', async (importOriginal) => ({
  ...await importOriginal<typeof import('../../api/privacy.ts')>(),
  decideConsent: vi.fn(), listSessions: vi.fn(), listKeys: vi.fn(), revokeSession: vi.fn(), revokeKey: vi.fn(),
}));

const consents = (over: Partial<ConsentView> = {}): ConsentView => ({
  enabled: true,
  art22: { enabled: true, humanReview: true },
  purposes: [
    { id: 'assistant', basis: 'consent', version: 1, text: null, state: 'unset', enforcement: 'ledger', managedIn: '', decidedAt: null, decidedVersion: null, canToggle: true, defaultOn: false },
    { id: 'analytics', basis: 'consent', version: 1, text: null, state: 'granted', enforcement: 'preference', managedIn: 'privacy', decidedAt: 1_700_000_000, decidedVersion: 1, canToggle: false, defaultOn: false },
  ],
  assistantFeatures: [{ id: 'assistant.summarize', feature: 'summarize', enabled: true, decidedAt: null }],
  receipts: [],
  ...over,
});

const overview = (over: Partial<PrivacyOverview> = {}): PrivacyOverview => ({
  account: { address: 'ana@example.test', domain: 'example.test' },
  categories: [
    { id: 'mail', rows: 120, bytes: 2_097_152, retention: null, processors: ['mailServer'], purpose: null },
    { id: 'signInHistory', rows: 4, bytes: 0, retention: { days: 90, source: 'setting' }, processors: [], purpose: null },
  ],
  consents: consents(),
  restriction: { active: false, scopes: [], since: null, expiresAt: null },
  links: { policy: '/privacy', processing: '/api/privacy/processing' },
  shortcuts: { download: 'access', delete: 'erasure' },
  ...over,
});

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(api.listSessions).mockResolvedValue([{ handle: 'h1', current: true }, { handle: 'h2', current: false, deviceLabel: 'Phone' }]);
  vi.mocked(api.listKeys).mockResolvedValue([{ prefix: 'mwk_abc' }]);
});

describe('Your data (Settings)', () => {
  it('lists stored data with counts, size, retention and processors', async () => {
    render(() => <YourData load={async () => overview()} />);
    const mail = await screen.findByTestId('yd-cat-mail');
    expect(mail.textContent).toContain('120');
    expect(mail.textContent).toContain('your mail provider');
    expect(screen.getByTestId('yd-cat-signInHistory').textContent).toContain('90 days');
  });

  it('shows the restriction notice only while a restriction is active', async () => {
    const { unmount } = render(() => <YourData load={async () => overview()} />);
    await screen.findByTestId('yd-categories');
    expect(screen.queryByTestId('yd-restriction')).toBeNull();
    unmount();
    render(() => <YourData load={async () => overview({ restriction: { active: true, scopes: ['assistant', 'push'], since: 1_700_000_000, expiresAt: null } })} />);
    const note = await screen.findByTestId('yd-restriction');
    expect(note.textContent).toContain('assistant');
    expect(note.textContent).toContain('push notifications');
  });

  it('grants a purpose through the ledger and shows the new state', async () => {
    vi.mocked(api.decideConsent).mockResolvedValue(consents({ purposes: [{ ...consents().purposes[0]!, state: 'granted', decidedAt: 1_700_000_100 }, consents().purposes[1]!] }));
    render(() => <YourData load={async () => overview()} />);
    const row = await screen.findByTestId('yd-purpose-assistant');
    const box = within(row).getByRole('checkbox') as HTMLInputElement;
    expect(box.checked).toBe(false);
    fireEvent.click(box);
    await waitFor(() => expect(api.decideConsent).toHaveBeenCalledWith('assistant', 'granted'));
    await waitFor(() => expect((within(screen.getByTestId('yd-purpose-assistant')).getByRole('checkbox') as HTMLInputElement).checked).toBe(true));
  });

  it('does not offer a toggle for a purpose switched elsewhere', async () => {
    render(() => <YourData load={async () => overview()} />);
    const row = await screen.findByTestId('yd-purpose-analytics');
    expect(within(row).queryByRole('checkbox')).toBeNull();
  });

  it('shows the automated processing notice and a per-feature opt-out', async () => {
    vi.mocked(api.decideConsent).mockResolvedValue(consents({ assistantFeatures: [{ id: 'assistant.summarize', feature: 'summarize', enabled: false, decidedAt: 1 }] }));
    render(() => <YourData load={async () => overview()} />);
    const notice = await screen.findByTestId('yd-art22');
    expect(notice.textContent).toContain('never send, delete or file mail without your confirmation');
    fireEvent.click(within(screen.getByTestId('yd-feature-summarize')).getByRole('checkbox'));
    await waitFor(() => expect(api.decideConsent).toHaveBeenCalledWith('assistant.summarize', 'withdrawn'));
  });

  it('hides the consent section when consent is switched off', async () => {
    render(() => <YourData load={async () => overview({ consents: consents({ enabled: false }) })} />);
    await screen.findByTestId('yd-categories');
    expect(screen.queryByTestId('yd-consents')).toBeNull();
  });

  it('revokes another session and an app key, and keeps the current session', async () => {
    vi.mocked(api.revokeSession).mockResolvedValue();
    vi.mocked(api.revokeKey).mockResolvedValue();
    render(() => <YourData load={async () => overview()} />);
    const sessions = await screen.findByTestId('yd-sessions');
    expect(within(sessions).getAllByRole('button')).toHaveLength(1);
    fireEvent.click(within(sessions).getByRole('button'));
    await waitFor(() => expect(api.revokeSession).toHaveBeenCalledWith('h2'));
    fireEvent.click(within(await screen.findByTestId('yd-keys')).getByRole('button'));
    await waitFor(() => expect(api.revokeKey).toHaveBeenCalledWith('mwk_abc'));
  });

  it('the download and delete shortcuts pre-select the request type', async () => {
    render(() => <YourData load={async () => overview()} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Download my data' }));
    expect(requestedKind()).toBe('portability');
    fireEvent.click(screen.getByRole('button', { name: 'Delete my account' }));
    expect(requestedKind()).toBe('erasure');
  });

  it('links to the policy page', async () => {
    render(() => <YourData load={async () => overview()} />);
    expect((await screen.findByRole('link', { name: 'Privacy policy' })).getAttribute('href')).toContain('/privacy');
  });
});
