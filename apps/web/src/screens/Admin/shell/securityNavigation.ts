import type { Can, Capability } from './can.ts';
import type { AdminRoute } from './router.ts';

/** `anyOf`: the tab opens with any one of these (the Overview shows each admin
 *  only the tiles their capabilities allow, t97). */
export const SECURITY_TABS: readonly { id: string; label: string; capability: Capability; anyOf?: readonly Capability[]; unscoped?: boolean }[] = [
  {id:'overview',label:'admin-security-tab-overview',capability:'security.read',anyOf:['security.read','bans.read','diagnostics.read','logs.read','admins.read','audit.read'],unscoped:true},
  {id:'requirements',label:'admin-security-tab-requirements',capability:'security.read',unscoped:true},
  {id:'bans',label:'admin-obs-bans',capability:'bans.read',unscoped:true},
  {id:'locks',label:'admin-obs-locks-title',capability:'bans.read',unscoped:true},
  {id:'geography',label:'admin-obs-geo-title',capability:'bans.read',unscoped:true},
  {id:'geoip',label:'admin-security-tab-geoip',capability:'security.read',unscoped:true},
  {id:'protection',label:'admin-protection-title',capability:'bans.read',unscoped:true},
  {id:'policy',label:'admin-nav-security-policy',capability:'security.read',unscoped:true},
  {id:'inbound',label:'admin-inbound-title',capability:'security.read',unscoped:true},
  {id:'deployment',label:'admin-security-tab-deployment',capability:'diagnostics.read',unscoped:true},
  {id:'console',label:'admin-security-tab-console',capability:'security.read',unscoped:true},
  {id:'client-access',label:'admin-client-access-title',capability:'settings.read',unscoped:true},
  {id:'sessions',label:'admin-nav-sessions',capability:'sessions.read'},
];
export function canOpenSecurityTab(can: Can, tab: (typeof SECURITY_TABS)[number]): boolean {
  const grants=can.grants();
  const held=(tab.anyOf??[tab.capability]).some(cap=>can.can(cap));
  return held && (!tab.unscoped || grants.kind==='all' || grants.me.domainScope===null);
}
/** Canonicalize old bookmarks before section permission checks. */
export function securityRouteAlias(route: AdminRoute): AdminRoute {
  const standalone:Record<string,string>={'sessions':'sessions','security-policy':'policy','inbound-allowlist':'inbound'};
  const tab=standalone[route.section];
  if(tab) return {...route,section:'security',rest:[tab,...route.rest]};
  // The require-2FA policy moved from the Overview to its own tab (t97).
  if(route.section==='two-factor') return {...route,section:'security',rest:['requirements',...route.rest]};
  if(route.section==='security' && (route.rest[0]==='two-factor' || route.rest[0]==='2fa' || route.rest[0]==='passkeys')) return {...route,rest:['requirements',...route.rest.slice(1)]};
  if(route.section==='observability' && ['bans','locks','geography','protection'].includes(route.rest[0]??'')) return {...route,section:'security'};
  return route;
}
