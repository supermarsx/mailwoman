// Test helpers for the admin shell (not a suite: vitest collects only *.test.*).
// Mock loaders for `/admin/me`, `/admin/dashboard` and `/admin/search`, and a
// render that provides the shell context around one component.

import { render } from '@solidjs/testing-library';
import { vi } from 'vitest';
import type { JSX } from 'solid-js';
import type { AdminGrants, AdminMeLoader, Capability } from './can.ts';
import { createLoadedCan } from './can.ts';
import type { AdminSearch } from './CommandPalette.tsx';
import type { DashboardLoader, DashboardResult } from '../../../api/admin/dashboard.ts';
import { ShellContext, type ShellContextValue } from './context.ts';
import type { AdminRouter } from './router.ts';

/** `/admin/me` → 404 (no RBAC yet): every capability. */
export const allCapable: AdminMeLoader = async () => ({ kind: 'all', reason: 'no-rbac' });

/** `/admin/me` → an admin holding exactly `caps`. */
export function holding(caps: Capability[]): AdminMeLoader {
  return async (): Promise<AdminGrants> => ({
    kind: 'listed',
    me: { username: 'helpdesk', displayName: null, roles: ['custom'], capabilities: caps, domainScope: null, totpEnrolled: false },
  });
}

/** `/admin/dashboard` → 404 (the endpoint is not mounted yet). */
export const dashboardUnavailable: DashboardLoader = async () => ({ kind: 'unavailable' });

export function dashboardReturning(result: DashboardResult): DashboardLoader {
  return vi.fn(async () => result);
}

/** `/admin/search` → 404. */
export const searchUnavailable: AdminSearch = async () => 'unavailable';

/** The props `AdminScreen` needs so no test reaches the network. */
export function offlineShell(): {
  loadMe: AdminMeLoader;
  loadDashboard: DashboardLoader;
  search: AdminSearch;
} {
  return { loadMe: allCapable, loadDashboard: dashboardUnavailable, search: searchUnavailable };
}

/** Render `ui` inside a shell context (for screens that call `useShell`). */
export function renderInShell(
  ui: () => JSX.Element,
  opts: { loadMe?: AdminMeLoader; loadDashboard?: DashboardLoader; navigate?: (p: string) => void; username?: string } = {},
): ReturnType<typeof render> & { navigate: ReturnType<typeof vi.fn> } {
  const can = createLoadedCan(opts.loadMe ?? allCapable);
  void can.reload();
  const navigate = vi.fn(async (p: string) => {
    opts.navigate?.(p);
    return true;
  });
  const router: AdminRouter = {
    route: () => ({ section: 'dashboard', rest: [] }),
    navigate,
    dispose: () => undefined,
  };
  const value: ShellContextValue = {
    can,
    router,
    services: { loadDashboard: opts.loadDashboard ?? dashboardUnavailable, search: searchUnavailable },
    username: () => opts.username ?? 'root',
  };
  const utils = render(() => <ShellContext.Provider value={value}>{ui()}</ShellContext.Provider>);
  return { ...utils, navigate };
}
