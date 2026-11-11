import { createSignal } from 'solid-js';
import { describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, screen, waitFor } from '@solidjs/testing-library';
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

  it('replaces the panel when navigating directly between two loaded tabs', async () => {
    const first = COMPLIANCE_TABS.find((tab) => tab.id === 'retention')!;
    const second = COMPLIANCE_TABS.find((tab) => tab.id === 'restrictions')!;
    const loadFirst = vi.spyOn(first, 'load').mockResolvedValue({ default: () => <p>Retention panel</p> });
    const loadSecond = vi.spyOn(second, 'load').mockResolvedValue({ default: () => <p>Restrictions panel</p> });
    const [route, setRoute] = createSignal(parseAdminPath('/admin#/compliance/retention'));
    const view = renderInShell(() => <AdminCompliance />, { route, navigate: (path) => setRoute(parseAdminPath(path)) });
    try {
      await screen.findByText('Retention panel');
      fireEvent.click(screen.getByRole('link', { name: /restrictions/i }));
      await screen.findByText('Restrictions panel');
      expect(screen.queryByText('Retention panel')).toBeNull();
    } finally {
      view.unmount();
      loadFirst.mockRestore();
      loadSecond.mockRestore();
    }
  });

  it('gates tabs on capability', () => {
    const can = { can: (c: string) => c === 'settings.read', grants: () => ({ kind: 'all' }) } as never;
    const ids = COMPLIANCE_TABS.filter((t) => canOpenComplianceTab(can, t)).map((t) => t.id);
    expect(ids).toEqual(expect.arrayContaining(['cookies', 'retention', 'consents']));
    expect(ids).not.toContain('data-requests');
    expect(ids).not.toContain('restrictions');
  });
});
