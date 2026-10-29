// t39-e7 (L11): the vacation reply is offered only when something will send it,
// the panel says who sends it, and "saved" is never shown for a reply that is
// off or that failed to save.
import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import { t } from '../../test/i18n.ts';
import type { JmapSession } from '../../api/jmap-types.ts';
import type { RulesSlice } from '../../state/slices/rules.ts';
import type { SieveApi } from './sieve-api.ts';
import type { Vacation, VacationApi } from './vacation-api.ts';

const client = vi.hoisted(() => ({ session: vi.fn() }));
vi.mock('../../api/client.ts', () => ({ createClient: () => ({ session: client.session, jmap: vi.fn() }) }));

import { vacationSupport } from './vacation-api.ts';
import { VacationPanel } from './VacationPanel.tsx';
import { RulesSettings } from './RulesSettings.tsx';

const VAC = 'urn:ietf:params:jmap:vacationresponse';
const SIEVE = 'urn:ietf:params:jmap:sieve';
function session(caps: Record<string, unknown>): JmapSession {
  return { capabilities: Object.fromEntries(Object.keys(caps).map((k) => [k, {}])), accounts: { a: { accountCapabilities: caps } } } as unknown as JmapSession;
}
const sieve = (extensions: string[], runsAt?: 'engine' | 'server-sieve') => ({
  sieveExtensions: extensions,
  ...(runsAt ? { 'urn:mailwoman:policy': { rawEdit: false, forwardExternal: false, ownDomains: [], runsAt } } : {}),
});

describe('vacationSupport', () => {
  it('is unavailable without the vacationresponse capability', () => {
    expect(vacationSupport(session({}), 'a')).toEqual({ available: false });
    expect(vacationSupport(session({ [SIEVE]: sieve(['vacation'], 'engine') }), 'a')).toEqual({ available: false });
  });
  it('is unavailable when Sieve does not offer the vacation extension (server lacks it or policy is off)', () => {
    expect(vacationSupport(session({ [VAC]: {}, [SIEVE]: sieve(['fileinto'], 'server-sieve') }), 'a')).toEqual({ available: false });
  });
  it('says who sends the reply', () => {
    expect(vacationSupport(session({ [VAC]: {}, [SIEVE]: sieve(['vacation'], 'server-sieve') }), 'a')).toEqual({ available: true, runsAt: 'server-sieve' });
    expect(vacationSupport(session({ [VAC]: {}, [SIEVE]: sieve(['vacation'], 'engine') }), 'a')).toEqual({ available: true, runsAt: 'engine' });
    // Proxy mode: the upstream JMAP server answers VacationResponse itself.
    expect(vacationSupport(session({ [VAC]: {} }), 'a')).toEqual({ available: true, runsAt: 'server' });
  });
});

function vacationService(saved: Partial<Vacation>): VacationApi {
  let value: Vacation = { id: 'singleton', isEnabled: false, fromDate: null, toDate: null, subject: null, textBody: 'Away', htmlBody: null };
  return {
    get: vi.fn(async () => ({ state: 's', value })),
    save: vi.fn(async () => { value = { ...value, ...saved }; }),
  };
}

describe('VacationPanel', () => {
  it('names the engine as the sender and confirms an enabled reply', async () => {
    const api = vacationService({ isEnabled: true });
    render(() => <VacationPanel api={api} runsAt="engine" />);
    expect(screen.getByTestId('vacation-runs-at')).toHaveTextContent(t('rules-vacation-runs-engine'));
    const enable = await screen.findByLabelText(t('rules-vacation-enabled'));
    fireEvent.change(enable, { target: { checked: true } });
    await waitFor(() => expect(screen.getByRole('button', { name: t('common-save') })).not.toBeDisabled());
    fireEvent.click(screen.getByRole('button', { name: t('common-save') }));
    expect(await screen.findByRole('status')).toHaveTextContent(t('rules-vacation-saved'));
  });
  it('says the reply is off when the saved reply is disabled', async () => {
    const api = vacationService({ isEnabled: false });
    render(() => <VacationPanel api={api} runsAt="server-sieve" />);
    expect(screen.getByTestId('vacation-runs-at')).toHaveTextContent(t('rules-vacation-runs-server'));
    await waitFor(() => expect(screen.getByRole('button', { name: t('common-save') })).not.toBeDisabled());
    fireEvent.click(screen.getByRole('button', { name: t('common-save') }));
    expect(await screen.findByRole('status')).toHaveTextContent(t('rules-vacation-saved-off'));
    expect(screen.queryByText(t('rules-vacation-saved'))).not.toBeInTheDocument();
  });
});

describe('RulesSettings vacation tab', () => {
  const slice = { rules: () => [], loadRules: vi.fn(async () => undefined) } as unknown as RulesSlice;
  const scripts = { connect: vi.fn(async () => null) } as unknown as SieveApi;

  it('hides the tab and explains why when nothing would send the reply', async () => {
    client.session.mockResolvedValue(session({ [VAC]: {}, [SIEVE]: sieve(['fileinto'], 'engine') }));
    render(() => <RulesSettings accountId="a" slice={slice} scriptApi={scripts} />);
    expect(await screen.findByTestId('rules-vacation-unavailable')).toHaveTextContent(t('rules-vacation-unavailable'));
    expect(screen.queryByRole('tab', { name: t('rules-vacation-title') })).toBeNull();
    expect(screen.getByRole('tab', { name: t('rules-tab-raw') })).toBeInTheDocument();
  });
  it('offers the tab when the account can send it', async () => {
    client.session.mockResolvedValue(session({ [VAC]: {}, [SIEVE]: sieve(['vacation'], 'server-sieve') }));
    render(() => <RulesSettings accountId="a" slice={slice} scriptApi={scripts} />);
    const tab = await screen.findByRole('tab', { name: t('rules-vacation-title') });
    expect(screen.queryByTestId('rules-vacation-unavailable')).toBeNull();
    fireEvent.click(tab);
    expect(await screen.findByTestId('vacation-runs-at')).toHaveTextContent(t('rules-vacation-runs-server'));
  });
});
