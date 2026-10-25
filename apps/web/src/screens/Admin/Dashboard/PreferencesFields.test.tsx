import { it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { PersonalPreferencesProvider } from '../shell/personalPreferences.tsx';
import { DashboardPreferencesFields } from './PreferencesFields.tsx';

it('persists visibility, ordering and refresh with other personal preferences intact', async () => {
  const save = vi.fn(async (data: Record<string, unknown>) => ({ data, etag: '"2"' }));
  render(() => <PersonalPreferencesProvider username="root" api={{ load: async () => ({ data: { unrelated: 'preserved' }, etag: '"1"' }), save }}><DashboardPreferencesFields /></PersonalPreferencesProvider>);
  await waitFor(() => expect(screen.getByRole('checkbox', { name: 'HTTP traffic' })).toBeEnabled());
  fireEvent.click(screen.getByRole('checkbox', { name: 'HTTP traffic' }));
  await waitFor(() => expect(save).toHaveBeenLastCalledWith(expect.objectContaining({ unrelated: 'preserved', dashboard: expect.objectContaining({ hidden: ['traffic'] }) }), expect.anything()));
  fireEvent.click(screen.getByRole('button', { name: 'Move HTTP traffic up' }));
  await waitFor(() => expect(save).toHaveBeenLastCalledWith(expect.objectContaining({ dashboard: expect.objectContaining({ order: ['checklist', 'overview', 'traffic', 'security', 'health', 'storage', 'audit', 'server'] }) }), expect.anything()));
  fireEvent.change(screen.getByLabelText('Automatic refresh'), { target: { value: '60' } });
  await waitFor(() => expect(save).toHaveBeenLastCalledWith(expect.objectContaining({ dashboard: expect.objectContaining({ refreshSecs: 60 }) }), expect.anything()));
});
