import type { Can, Capability } from './can.ts';
import type { AdminRoute } from './router.ts';

/** The Privacy screen's tabs (`#/privacy/<id>`). `anyOf`: the tab opens with any one of these. */
export const PRIVACY_TABS: readonly { id: string; label: string; capability: Capability; anyOf?: readonly Capability[]; unscoped?: boolean }[] = [
  { id: 'overview', label: 'privacy-tab-overview', capability: 'egress.read', unscoped: true },
  { id: 'remote', label: 'privacy-tab-remote', capability: 'settings.read' },
  { id: 'image-proxy', label: 'privacy-tab-image-proxy', capability: 'egress.read', anyOf: ['egress.read', 'settings.read'], unscoped: true },
  { id: 'fetch-limits', label: 'privacy-tab-fetch-limits', capability: 'settings.read' },
  { id: 'traffic', label: 'privacy-tab-traffic', capability: 'egress.read', unscoped: true },
];

export function canOpenPrivacyTab(can: Can, tab: (typeof PRIVACY_TABS)[number]): boolean {
  const grants = can.grants();
  const held = (tab.anyOf ?? [tab.capability]).some((cap) => can.can(cap));
  return held && (!tab.unscoped || grants.kind === 'all' || grants.me.domainScope === null);
}

/** The tab id a route names: a tab segment, or the old `policy/<key>` deep link (resolved by the screen from the key). */
export function privacyRouteTab(route: AdminRoute): { tab: string | undefined; focusKey: string | undefined } {
  const first = route.rest[0];
  if (first === 'policy') return { tab: undefined, focusKey: route.rest[1] };
  return { tab: first, focusKey: route.rest[1] };
}
