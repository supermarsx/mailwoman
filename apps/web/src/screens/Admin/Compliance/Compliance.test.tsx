import { describe, expect, it } from 'vitest';
import { cleanup, screen, waitFor } from '@solidjs/testing-library';
import { AdminCompliance } from './index.tsx';
import { COMPLIANCE_TABS, canOpenComplianceTab } from './tabs.ts';
import { complianceRouteAlias } from './complianceRoute.ts';
import { parseAdminPath } from '../shell/router.ts';
import { SECTIONS } from '../shell/sections.ts';
import { renderInShell } from '../shell/testkit.tsx';

describe('Compliance hub', () => {
  it('registers unique tab ids and is a single sidebar section', () => {
    const ids = COMPLIANCE_TABS.map((t) => t.id);
    expect(new Set(ids).size).toBe(ids.length);
    expect(ids).toEqual(expect.arrayContaining(['data-requests', 'cookies', 'retention']));
    expect(SECTIONS.filter((s) => s.id === 'compliance')).toHaveLength(1);
    expect(SECTIONS.some((s) => s.id === 'retention' || s.id === 'data-requests')).toBe(false);
  });

  it('redirects the old routes to their Compliance tab', () => {
    expect(parseAdminPath('/admin#/data-requests')).toMatchObject({ section: 'compliance', rest: ['data-requests'] });
    expect(parseAdminPath('/admin#/retention')).toMatchObject({ section: 'compliance', rest: ['retention'] });
    expect(parseAdminPath('/admin#/privacy/cookies')).toMatchObject({ section: 'compliance', rest: ['cookies'] });
    expect(parseAdminPath('/admin#/privacy/policy/cookies.mode')).toMatchObject({ section: 'compliance', rest: ['cookies', 'cookies.mode'] });
    expect(parseAdminPath('/admin#/privacy/remote')).toMatchObject({ section: 'privacy' });
    expect(complianceRouteAlias({ section: 'users', rest: [] })).toEqual({ section: 'users', rest: [] });
  });

  it('shows one overview card per tab the admin may open', async () => {
    renderInShell(() => <AdminCompliance />, { route: () => ({ section: 'compliance', rest: [] }) });
    await waitFor(() => expect(screen.getByTestId('compliance-overview')).toBeTruthy());
    for (const tab of COMPLIANCE_TABS) expect(screen.queryByTestId(`compliance-card-${tab.id}`)).toBeTruthy();
    cleanup();
  });

  it('gates tabs on capability', () => {
    const can = { can: (c: string) => c === 'settings.read', grants: () => ({ kind: 'all' }) } as never;
    expect(COMPLIANCE_TABS.filter((t) => canOpenComplianceTab(can, t)).map((t) => t.id)).toEqual(['cookies', 'retention']);
  });
});
