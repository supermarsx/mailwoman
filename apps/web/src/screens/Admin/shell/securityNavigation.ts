import type { Can, Capability } from './can.ts';
import type { AdminRoute } from './router.ts';

export const SECURITY_TABS: readonly { id: string; label: string; capability: Capability; unscoped?: boolean }[] = [
  {id:'overview',label:'admin-security-tab-overview',capability:'security.read',unscoped:true},
  {id:'bans',label:'admin-obs-bans',capability:'bans.read',unscoped:true},
  {id:'locks',label:'admin-obs-locks-title',capability:'bans.read',unscoped:true},
  {id:'geography',label:'admin-obs-geo-title',capability:'bans.read',unscoped:true},
  {id:'protection',label:'admin-protection-title',capability:'bans.read',unscoped:true},
  {id:'policy',label:'admin-nav-security-policy',capability:'security.read',unscoped:true},
  {id:'inbound',label:'admin-inbound-title',capability:'security.read',unscoped:true},
  {id:'deployment',label:'admin-security-tab-deployment',capability:'diagnostics.read',unscoped:true},
  {id:'sessions',label:'admin-nav-sessions',capability:'sessions.read'},
];
export function canOpenSecurityTab(can: Can, tab: (typeof SECURITY_TABS)[number]): boolean {
  const grants=can.grants();
  return can.can(tab.capability) && (!tab.unscoped || grants.kind==='all' || grants.me.domainScope===null);
}
/** Canonicalize old bookmarks before section permission checks. */
export function securityRouteAlias(route: AdminRoute): AdminRoute {
  const standalone:Record<string,string>={'sessions':'sessions','security-policy':'policy','inbound-allowlist':'inbound'};
  const tab=standalone[route.section];
  if(tab) return {...route,section:'security',rest:[tab,...route.rest]};
  if(route.section==='two-factor') return {...route,section:'security',rest:route.rest};
  if(route.section==='observability' && ['bans','locks','geography','protection'].includes(route.rest[0]??'')) return {...route,section:'security'};
  return route;
}
