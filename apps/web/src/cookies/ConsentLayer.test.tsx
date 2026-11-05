import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import type { ConsentConfig } from './policy.ts';

const calls: { url: string; body: unknown }[] = [];
let config: ConsentConfig = { enabled: false };
vi.mock('../api/http.ts', () => ({
  apiFetch: vi.fn(async (url: string, init?: { method?: string; body?: string }) => {
    if (String(url).endsWith('/api/cookies/config')) return new Response(JSON.stringify(config), { status: 200 });
    calls.push({ url: String(url), body: init?.body ? JSON.parse(init.body) : undefined });
    return new Response(null, { status: 204 });
  }),
}));

import { ConsentLayer, CookiePreferencesLink } from './ConsentLayer.tsx';
import { bootConsent, categoryAllowed, consentStored, openCookiePreferences, previewConsent, resetConsentForTests } from './runtime.ts';

const full = (over: Partial<ConsentConfig> = {}): ConsentConfig => ({
  enabled: true, mode: 'optIn', region: 'PT', strictRegion: true, version: 3, lifetimeDays: 180, reprompt: true,
  layout: 'banner', position: 'bottom', rejectProminent: true, policyUrl: 'https://example.test/cookies', privacyUrl: null,
  honourGpc: true, honourDnt: false, scanIntervalHours: 0, unclassified: 'report',
  categories: [
    { id: 'necessary', enabled: true, default: true, label: {}, description: {} },
    { id: 'functional', enabled: true, default: true, label: { en: 'Remember my settings' }, description: {} },
    { id: 'analytics', enabled: true, default: false, label: {}, description: {} },
    { id: 'marketing', enabled: false, default: false, label: {}, description: {} },
  ],
  text: { title: { en: 'Your cookie choices' }, body: {} },
  inventory: [
    { id: 'appearance', kind: 'localStorage', pattern: 'mw.theme.*', category: 'functional', purpose: 'Theme', duration: 'Until cleared', party: 'first', provider: 'This server' },
    { id: 'consent', kind: 'localStorage', pattern: 'mw.consent*', category: 'necessary', purpose: 'Choices', duration: 'Until the consent lifetime ends', party: 'first', provider: 'This server' },
  ],
  rules: [],
  ...over,
});

beforeEach(() => { localStorage.clear(); calls.length = 0; resetConsentForTests(); });
afterEach(() => { cleanup(); resetConsentForTests(); Object.defineProperty(navigator, 'globalPrivacyControl', { value: undefined, configurable: true }); });

async function show(c: ConsentConfig): Promise<void> {
  config = c;
  render(() => <ConsentLayer />);
  await bootConsent();
}

describe('consent control', () => {
  it('shows nothing while the administrator has it off', async () => {
    await show({ enabled: false });
    expect(screen.queryByRole('dialog')).toBeNull();
    expect(categoryAllowed('functional')).toBe(true);
  });

  it('asks in opt-in mode with Accept all and Reject all equally prominent, and uses the administrator text', async () => {
    await show(full());
    const dialog = await screen.findByRole('dialog', { name: 'Your cookie choices' });
    expect(dialog).toHaveAttribute('aria-modal', 'false');
    const accept = screen.getByRole('button', { name: 'Accept all' });
    const reject = screen.getByRole('button', { name: 'Reject all' });
    expect(accept.className).toBe(reject.className);
    expect(screen.getByRole('link', { name: 'Cookie policy' })).toHaveAttribute('href', 'https://example.test/cookies');
    expect(categoryAllowed('functional')).toBe(false);
  });

  it('makes Reject all a text button when the policy does not ask for equal prominence', async () => {
    await show(full({ rejectProminent: false }));
    const accept = await screen.findByRole('button', { name: 'Accept all' });
    expect(screen.getByRole('button', { name: 'Reject all' }).className).not.toBe(accept.className);
  });

  it('records Accept all locally and on the server, applies it and hides the control', async () => {
    await show(full());
    fireEvent.click(await screen.findByRole('button', { name: 'Accept all' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(categoryAllowed('functional')).toBe(true);
    expect(categoryAllowed('analytics')).toBe(true);
    expect(categoryAllowed('marketing')).toBe(false);
    const stored = JSON.parse(localStorage.getItem('mw.consent')!);
    expect(stored).toMatchObject({ version: 3, decision: 'acceptAll' });
    await waitFor(() => expect(calls.find((c) => c.url.endsWith('/api/cookies/consent'))).toBeTruthy());
    expect(calls.find((c) => c.url.endsWith('/api/cookies/consent'))!.body).toMatchObject({ version: 3, decision: 'acceptAll', signal: 'none' });
  });

  it('Reject all removes optional storage that is already there', async () => {
    localStorage.setItem('mw.theme.prefs', 'dark');
    await show(full({ mode: 'optOut' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Reject all' }));
    await waitFor(() => expect(localStorage.getItem('mw.theme.prefs')).toBeNull());
    expect(consentStored()?.decision).toBe('rejectAll');
  });

  it('customizes by category: necessary is always on and cannot be changed', async () => {
    await show(full());
    fireEvent.click(await screen.findByRole('button', { name: 'Choose categories' }));
    const necessary = screen.getByRole('checkbox', { name: /Strictly necessary/ });
    expect(necessary).toBeChecked();
    expect(necessary).toBeDisabled();
    expect(screen.queryByRole('checkbox', { name: /Marketing/ })).toBeNull();
    fireEvent.click(screen.getByRole('checkbox', { name: /Remember my settings/ }));
    fireEvent.click(screen.getByRole('button', { name: 'Save choices' }));
    await waitFor(() => expect(consentStored()?.decision).toBe('custom'));
    expect(consentStored()?.categories).toEqual({ necessary: true, functional: true, analytics: false, marketing: false });
  });

  it('opt-out applies the defaults before a decision', async () => {
    await show(full({ mode: 'optOut' }));
    await screen.findByRole('dialog');
    expect(categoryAllowed('functional')).toBe(true);
    expect(categoryAllowed('analytics')).toBe(false);
  });

  it('notice-only shows one button and no toggles', async () => {
    await show(full({ mode: 'noticeOnly' }));
    await screen.findByRole('dialog');
    expect(screen.queryByRole('button', { name: 'Reject all' })).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'OK' }));
    await waitFor(() => expect(consentStored()?.decision).toBe('notice'));
  });

  it('keeps analytics off and says why when the browser sends Global Privacy Control', async () => {
    Object.defineProperty(navigator, 'globalPrivacyControl', { value: true, configurable: true });
    await show(full());
    fireEvent.click(await screen.findByRole('button', { name: 'Choose categories' }));
    expect(screen.getByText(/Global Privacy Control/)).toBeInTheDocument();
    expect(screen.getByRole('checkbox', { name: /Analytics/ })).toBeDisabled();
    fireEvent.click(screen.getByRole('button', { name: 'Accept all' }));
    await waitFor(() => expect(consentStored()).not.toBeNull());
    expect(categoryAllowed('analytics')).toBe(false);
    expect(consentStored()?.signal).toBe('gpc');
  });

  it('asks again when the policy version changed, with the reason', async () => {
    localStorage.setItem('mw.consent', JSON.stringify({ version: 2, decision: 'acceptAll', categories: { necessary: true, functional: true, analytics: true, marketing: false }, signal: 'none', decidedAt: Date.now() }));
    await show(full());
    expect(await screen.findByText(/cookie policy changed/)).toBeInTheDocument();
    expect(categoryAllowed('analytics')).toBe(false);
  });

  it('does not ask while a current decision is stored, and reopens as a modal dialog from a link', async () => {
    localStorage.setItem('mw.consent', JSON.stringify({ version: 3, decision: 'rejectAll', categories: { necessary: true, functional: false, analytics: false, marketing: false }, signal: 'none', decidedAt: Date.now() }));
    config = full();
    render(() => <><ConsentLayer /><CookiePreferencesLink /></>);
    await bootConsent();
    expect(screen.queryByRole('dialog')).toBeNull();
    fireEvent.click(await screen.findByTestId('cookie-preferences-link'));
    const dialog = await screen.findByRole('dialog', { name: 'Cookie preferences' });
    expect(dialog).toHaveAttribute('aria-modal', 'true');
    expect(dialog).toContainElement(document.activeElement as HTMLElement);
    fireEvent.keyDown(document.activeElement ?? document.body, { key: 'Escape' });
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
  });

  it('lists what is stored per category with its purpose', async () => {
    await show(full());
    fireEvent.click(await screen.findByRole('button', { name: 'Choose categories' }));
    fireEvent.click(screen.getByRole('button', { name: 'Show what is stored' }));
    expect(screen.getByText('mw.theme.*')).toBeInTheDocument();
  });

  it('previews a draft configuration without storing or recording anything', async () => {
    await show({ enabled: false });
    previewConsent(full({ layout: 'modal' }));
    const dialog = await screen.findByRole('dialog');
    expect(dialog).toHaveAttribute('aria-modal', 'true');
    fireEvent.click(screen.getByRole('button', { name: 'Accept all' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(localStorage.getItem('mw.consent')).toBeNull();
    expect(calls).toEqual([]);
    openCookiePreferences();
    expect(screen.queryByRole('dialog')).toBeNull();
  });
});
