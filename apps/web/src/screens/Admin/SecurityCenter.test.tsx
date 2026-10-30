import { render, screen, within } from '@solidjs/testing-library';
import { beforeAll, beforeEach, expect, it, vi } from 'vitest';
import { loadCatalog } from '../../i18n';
import { AdminSecurityCenter } from './SecurityCenter.tsx';
import { adminRequest } from './shell/adminApi.ts';
import { renderInShell, holding } from './shell/testkit.tsx';
import type { SecurityDashboard } from '../../api/admin/securityDashboard.ts';
vi.mock('./shell/adminApi.ts', () => ({ adminRequest: vi.fn() }));
vi.mock('./TwoFactorPolicy.tsx', () => ({ TwoFactorPolicy: () => <p>require-2fa-panel</p> }));
beforeAll(async () => { await loadCatalog('admin'); await loadCatalog('admin-security'); });
beforeEach(() => { vi.mocked(adminRequest).mockReset(); });

const full: SecurityDashboard = {
  generatedAt: 1_800_000_000,
  factors: { knownAccounts: 10, withFactor: 4, totp: 3, passkey: 2, both: 1, withoutFactorRequired: 2, requiredGlobally: false, requiredDomains: 1, adminsWithoutFactor: 1, staleApiKeys: 3, environmentAdminLoginEnabled: false },
  signins: { failed24h: 7, blocked24h: 2, failed7d: 9, blocked7d: 2, hourly: [{ t: 1_799_996_400, failed: 7, blocked: 2 }], daily: [{ t: 1_799_913_600, failed: 9, blocked: 2 }], topSources: [{ source: '203.0.113.0/24', country: 'PT', count: 6 }], topCountries: [{ country: 'PT', count: 6 }], sourcesMasked: true, geoipReady: true },
  protection: { protectionEnforced24h: 4, protectionObserved24h: 1, activeLocks: 2, activeAdminLocks: 1, activeBans: 3 },
  admins: { activeSessions: 2, adminsSignedIn: 1, recentSignins: [{ username: 'root', at: 1_799_999_000, source: '198.51.100.0/24' }], sourcesMasked: true },
  audit: { recent: [{ id: 'a', at: '2026-10-29T10:00:00Z', actor: 'root', action: 'twofa-policy-set', target: 'domain:example.com' }] },
  posture: { checks: 13, warnings: 1, https: 'pass', hsts: 'info', secureCookies: 'pass', csrf: 'pass', adminFactors: 'warning', geoipReady: true, analyticsTag: false, consoleDefaultPath: true },
};
function returning(data: SecurityDashboard): void {
  vi.mocked(adminRequest).mockResolvedValue({ status: 200, etag: null, data });
}

it('Overview is a dashboard: adoption, failed sign-ins, protection, sessions, audit and posture tiles', async () => {
  returning(full);
  render(() => <AdminSecurityCenter />);
  expect(screen.getByTestId('security-dashboard-skeleton')).toBeInTheDocument();
  const factors = await screen.findByTestId('secdash-factors');
  expect(within(factors).getByText('40%')).toBeInTheDocument();
  expect(within(factors).getByText('Passkey 2 · authenticator app 3 · of 10 accounts')).toBeInTheDocument();
  expect(within(screen.getByTestId('secdash-failed')).getByText('7')).toBeInTheDocument();
  expect(within(screen.getByTestId('secdash-protection')).getByText('4')).toBeInTheDocument();
  expect(within(screen.getByTestId('secdash-admins')).getByText('2')).toBeInTheDocument();
  expect(within(screen.getByTestId('secdash-posture')).getByText('1 warning')).toBeInTheDocument();
  expect(screen.getByText(/2 accounts have no second factor although a requirement covers them/)).toBeInTheDocument();
  expect(screen.getByText('203.0.113.0/24')).toBeInTheDocument();
  expect(screen.getByText(/Addresses are shortened to their network/)).toBeInTheDocument();
  expect(screen.getByText('Two-factor requirement changed')).toBeInTheDocument();
  expect(screen.getByText('Console at the default /admin address')).toBeInTheDocument();
  expect(screen.getByTestId('secdash-spark-24h')).toBeInTheDocument();
  // The old requirements list is no longer on the Overview.
  expect(screen.queryByRole('link', { name: /session\.maxConcurrent/ })).toBeNull();
  expect(screen.queryByText('require-2fa-panel')).toBeNull();
  expect(vi.mocked(adminRequest)).toHaveBeenCalledWith('/security/dashboard');
});

it('a restricted admin sees only the tiles the server sent', async () => {
  returning({ generatedAt: 1_800_000_000, posture: full.posture! });
  renderInShell(() => <AdminSecurityCenter />, { loadMe: holding(['diagnostics.read']), route: () => ({ section: 'security', rest: ['overview'] }) });
  expect(await screen.findByTestId('secdash-posture')).toBeInTheDocument();
  for (const id of ['secdash-factors', 'secdash-failed', 'secdash-protection', 'secdash-admins', 'secdash-audit-panel']) expect(screen.queryByTestId(id)).toBeNull();
  expect(screen.queryByRole('link', { name: 'Two-factor & sign-in requirements' })).toBeNull();
});

it('Requirements tab holds the require-2FA policy and links each passkey and session policy to its editor', async () => {
  renderInShell(() => <AdminSecurityCenter />, { route: () => ({ section: 'security', rest: ['requirements'] }) });
  expect(await screen.findByText('require-2fa-panel')).toBeInTheDocument();
  const panel = screen.getByTestId('security-requirements');
  for (const key of ['twofa.required', 'twofa.passkeyAttachment', 'twofa.passkeyUserVerification', 'twofa.passkeyResidentKey', 'twofa.passkeyLegacy', 'twofa.lastFactorRemoval', 'session.maxConcurrent']) {
    const code = within(panel).getByText(key);
    expect(code.previousElementSibling).toHaveAttribute('href', `#/settings/${key}`);
  }
  expect(screen.getByRole('link', { name: 'Two-factor & sign-in requirements' })).toHaveAttribute('aria-current', 'page');
  expect(vi.mocked(adminRequest)).not.toHaveBeenCalledWith('/security/dashboard');
});
