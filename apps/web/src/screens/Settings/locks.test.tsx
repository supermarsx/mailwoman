import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
// Account settings an administrator holds (worker f01): the notifications
// switch and the offline budget/retention are disabled, noted, and use the held
// value; a refused save shows localised text.

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { clearSettings, loadSettings, type Fetcher } from '../../api/settings.ts';
import { Notifications } from './Notifications.tsx';
import { Preferences } from './Preferences.tsx';
import { SettingsService } from './service.ts';
import { DEFAULT_PREFS, type SettingsPrefs } from './prefs.ts';
import { t } from '../../test/i18n.ts';

function json(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });
}

const held = (value: unknown) => ({ value, source: 'domain', locked: true, lockedBy: 'domain' });

describe('held account settings', () => {
  afterEach(()=>suspendAccountPreferences());
  beforeEach(() => {
    suspendAccountPreferences();activateAccountPreferences('account','msa1_verified_settings_authority');
    clearSettings();
    localStorage.clear();
  });

  it('a held notifications switch is disabled, shows the held value and saves it', async () => {
    await loadSettings({
      fetcher: vi.fn<Fetcher>(async () => json(200, { settings: { 'notifications.enabled': held(false) } })),
    });
    const saved: unknown[] = [];
    const service = new SettingsService(
      vi.fn<Fetcher>(async (_u, init) => {
        if (init?.method === 'PUT') {
          saved.push(JSON.parse(String(init.body)));
          return json(200, { ok: true });
        }
        return json(200, { enabled: true, rules: [], quietHours: { enabled: false, start: '22:00', end: '07:00' } });
      }),
    );
    render(() => <Notifications service={service} />);
    const box = await screen.findByLabelText(t('settings-notif-enabled-label'));
    await waitFor(() => expect(box).not.toBeChecked());
    expect(box).toBeDisabled();
    expect(screen.getByTestId('setting-lock-note')).toHaveAttribute('id', 'settings-notif-enabled-lock');
    fireEvent.click(screen.getByRole('button', { name: t('settings-save') }));
    await waitFor(() => expect(saved).toHaveLength(1));
    expect((saved[0] as { enabled: boolean }).enabled).toBe(false);
  });

  it('a refused save shows localised text, not the server string', async () => {
    const service = new SettingsService(
      vi.fn<Fetcher>(async (_u, init) =>
        init?.method === 'PUT'
          ? json(403, { code: 'setting_locked', error: 'notifications.enabled is set by your administrator' })
          : json(200, { enabled: true, rules: [], quietHours: { enabled: false, start: '22:00', end: '07:00' } }),
      ),
    );
    render(() => <Notifications service={service} />);
    await screen.findByLabelText(t('settings-notif-enabled-label'));
    fireEvent.click(screen.getByRole('button', { name: t('settings-save') }));
    const alert = await screen.findByText(t('settings-policy-error-locked'));
    expect(alert.textContent).not.toContain('notifications.enabled');
  });

  it('a held offline budget replaces the device value and cannot be edited', async () => {
    await loadSettings({
      fetcher: vi.fn<Fetcher>(async () =>
        json(200, { settings: { 'offline.budgetMb': held(64), 'offline.retentionDays': held(7) } }),
      ),
    });
    const [prefs, setPrefs] = createSignal<SettingsPrefs>({ ...DEFAULT_PREFS });
    render(() => <Preferences prefs={prefs} onChange={setPrefs} />);
    await waitFor(() => expect(prefs().offlineBudgetMb).toBe(64));
    expect(prefs().offlineRetentionDays).toBe(7);
    expect(screen.getByLabelText(t('settings-offline-budget-label'))).toBeDisabled();
    expect(screen.getByLabelText(t('settings-offline-retention-label'))).toBeDisabled();
    expect(screen.getAllByTestId('setting-lock-note')).toHaveLength(2);
  });
});
