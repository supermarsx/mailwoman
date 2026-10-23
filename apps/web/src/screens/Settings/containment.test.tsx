// t32-h6: a failing account-settings section stays inside its own section.
//
// Each section reads its resource straight in JSX, and Solid rethrows a rejected
// resource on read, so one failed `/api/account/*` load used to escape to the
// app's error boundary and replace the whole mailbox.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { ErrorBoundary } from 'solid-js';
import { AccountSettings } from './AccountSettings.tsx';
import type { SettingsService } from './service.ts';

function service(sessions: () => Promise<unknown>): SettingsService {
  return {
    twofaStatus: vi.fn(async () => ({ totp: false, passkeys: [], recoveryRemaining: 0, policyRequired: false })),
    sessions: vi.fn(sessions),
    listSignatures: vi.fn(async () => []),
    listIdentities: vi.fn(async () => []),
    notifications: vi.fn(async () => ({ enabled: true, rules: [], quietHours: { enabled: false, start: '22:00', end: '07:00' } })),
    listSavedSearches: vi.fn(async () => []),
  } as unknown as SettingsService;
}

describe('AccountSettings — a failing section is contained (t32-h6)', () => {
  beforeEach(() => localStorage.clear());

  it('shows the failure and a retry in place of that section; the rest and the mailbox stay', async () => {
    let n = 0;
    const svc = service(async () => {
      n += 1;
      if (n === 1) throw new Error('request failed with 500');
      return [];
    });
    render(() => (
      <ErrorBoundary fallback={<p data-testid="app-boundary">Something went wrong</p>}>
        <main data-testid="mailbox">Inbox</main>
        <AccountSettings service={svc} />
      </ErrorBoundary>
    ));

    const retry = await screen.findByTestId('async-retry');
    expect(screen.queryByTestId('app-boundary')).not.toBeInTheDocument();
    expect(screen.getByTestId('mailbox')).toBeInTheDocument();
    expect(screen.getByLabelText('Two-factor authentication')).toBeInTheDocument();
    expect(screen.queryByLabelText('Active sessions')).not.toBeInTheDocument();

    fireEvent.click(retry);
    await waitFor(() => expect(screen.getByLabelText('Active sessions')).toBeInTheDocument());
    expect(screen.queryByTestId('async-retry')).not.toBeInTheDocument();
  });
});
