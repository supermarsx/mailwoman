import type { AdminRoute } from '../shell/router.ts';
import { LEGACY_COMPLIANCE_ROUTES } from './tabs.ts';

/** Old routes keep working: `#/data-requests`, `#/retention` and `#/privacy/cookies` open the Compliance tab. */
export function complianceRouteAlias(route: AdminRoute): AdminRoute {
  const tab = LEGACY_COMPLIANCE_ROUTES[route.section];
  if (tab !== undefined) return { ...route, section: 'compliance', rest: [tab, ...route.rest] };
  if (route.section === 'privacy' && route.rest[0] === 'cookies') return { ...route, section: 'compliance', rest: route.rest };
  if (route.section === 'privacy' && route.rest[0] === 'policy' && (route.rest[1] ?? '').startsWith('cookies.')) return { ...route, section: 'compliance', rest: ['cookies', route.rest[1]!] };
  return route;
}
