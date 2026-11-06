import { expect, it, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { AdminScreen } from './index.tsx';
import { PersonalPreferences } from './PersonalPreferences.tsx';
import { mockAdminApi } from './testkit.tsx';
import { holding, offlineShell } from './shell/testkit.tsx';
import type { AdminPreferencesApi } from './shell/personalPreferences.tsx';
import type { SessionBarApi } from './shell/sessionBar.ts';

const sections = [{ id: 'preferences', group: 'personal', icon: 'preferences', label: () => 'My preferences', component: PersonalPreferences }] as const;

function timing() {
  const now = Math.floor(Date.now() / 1000);
  return { idleTimeoutSeconds: 1800, absoluteTimeoutSeconds: 43200, signedInAt: now, idleExpiresAt: now + 1700, absoluteExpiresAt: now + 40000, expiresAt: now + 1700, serverNow: now };
}

it('the session bar preference is saved with the other preferences, and the bar follows it', async () => {
  history.replaceState(null, '', '/admin#/preferences/layout');
  const saved = vi.fn<AdminPreferencesApi['save']>(async (data) => ({ data, etag: 'v2' }));
  const sessionBarApi: SessionBarApi = { status: async () => timing(), extend: async () => timing() };
  render(() => (
    <AdminScreen api={mockAdminApi()} {...offlineShell()} loadMe={holding([])} sessionBarApi={sessionBarApi}
      preferencesApi={{ load: async () => ({ data: { future: 'keep' }, etag: 'v1' }), save: saved }}
      sections={sections as never} />
  ));
  const select = await screen.findByTestId('admin-session-bar-mode');
  // Default: always shown.
  expect(await screen.findByTestId('admin-session-bar')).toBeInTheDocument();
  expect((select as HTMLSelectElement).value).toBe('always');
  await waitFor(() => expect(select).not.toBeDisabled());
  fireEvent.change(select, { target: { value: 'hidden' } });
  await waitFor(() => expect(saved).toHaveBeenCalledWith({ future: 'keep', sessionBar: 'hidden' }, 'v1'));
  await waitFor(() => expect(screen.queryByTestId('admin-session-bar')).toBeNull());
});
