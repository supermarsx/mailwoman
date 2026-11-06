// A correction request opens Settings > Identities with its reference in view.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@solidjs/testing-library';
import { activateAccountPreferences, suspendAccountPreferences } from '../../api/accountPreferenceLifecycle.ts';
import { requestReference } from '../../api/dataRequests.ts';
import { Identities } from './Identities.tsx';
import { createSettingsNavigation, SettingsNavigationContext } from './navigation.tsx';
import type { SettingsService } from './service.ts';

beforeEach(() => activateAccountPreferences('rectification'));
afterEach(() => { cleanup(); suspendAccountPreferences(); });

const service = { listIdentities: vi.fn(async () => []), listSignatures: vi.fn(async () => []) } as unknown as SettingsService;

describe('correction request banner on Identities', () => {
  it('derives a short R- reference from the request id', () => {
    expect(requestReference('9f3a1c27-0000-4000-8000-000000000000')).toBe('R-9F3A');
  });

  it('shows the reference passed through route state and dismisses it with Mark as done', async () => {
    const navigation = createSettingsNavigation({ section: 'identities', rectification: { requestId: 'r-1', reference: 'R-9F3A' } });
    render(() => <SettingsNavigationContext.Provider value={navigation}><Identities service={service} /></SettingsNavigationContext.Provider>);
    const banner = await screen.findByTestId('rectification-banner');
    expect(banner).toHaveTextContent('Updating details for request R-9F3A');
    fireEvent.click(screen.getByRole('button', { name: 'Mark as done' }));
    expect(screen.queryByTestId('rectification-banner')).toBeNull();
  });

  it('keeps the banner while an identity editor is opened and closed', async () => {
    const navigation = createSettingsNavigation({ section: 'identities', rectification: { requestId: 'r-1', reference: 'R-9F3A' } });
    render(() => <SettingsNavigationContext.Provider value={navigation}><Identities service={service} /></SettingsNavigationContext.Provider>);
    await screen.findByTestId('rectification-banner');
    await navigation.navigate({ section: 'identities', editor: { kind: 'new' } });
    expect(screen.getByTestId('rectification-banner')).toBeInTheDocument();
  });

  it('shows no banner when Identities is opened normally', async () => {
    render(() => <Identities service={service} />);
    await screen.findByTestId('identity-list');
    expect(screen.queryByTestId('rectification-banner')).toBeNull();
  });
});
