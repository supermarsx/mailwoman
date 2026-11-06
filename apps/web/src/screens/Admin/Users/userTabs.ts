// The account detail page's tabs (`#/users/<accountId>/<tab>`). Every control the
// single stacked page had lives in exactly one tab: identity and quota in Overview,
// the feature profile in Profile, the two account flags in Security, session
// revocation in Sessions, block/unblock/delete in Access, and (new) the account's
// data requests in Compliance.

import type { Can, Capability } from '../shell/can.ts';

export type UserTabId = 'overview' | 'profile' | 'security' | 'sessions' | 'compliance' | 'access';
export interface UserTab { id: UserTabId; label: string; capability?: Capability }

export const USER_TABS: readonly UserTab[] = [
  { id: 'overview', label: 'admin-user-tab-overview' },
  { id: 'profile', label: 'admin-user-tab-profile', capability: 'settings.user_values' },
  { id: 'security', label: 'admin-user-tab-security' },
  { id: 'sessions', label: 'admin-user-tab-sessions' },
  { id: 'compliance', label: 'admin-user-tab-compliance', capability: 'data_export.run' },
  { id: 'access', label: 'admin-user-tab-access' },
];

export const visibleUserTabs = (can: Can | undefined): UserTab[] => USER_TABS.filter((tab) => tab.capability === undefined || (can?.can(tab.capability) ?? true));

/** The tab a route segment names; anything unknown or not permitted falls back to Overview. */
export function resolveUserTab(segment: string | undefined, tabs: readonly UserTab[]): UserTabId {
  return tabs.find((tab) => tab.id === segment)?.id ?? 'overview';
}
