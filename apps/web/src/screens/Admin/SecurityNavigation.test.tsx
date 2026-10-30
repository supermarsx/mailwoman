import {beforeAll,expect,it,vi} from 'vitest';
import {render,screen} from '@solidjs/testing-library';
import {Suspense} from 'solid-js';
import {loadCatalog} from '../../i18n';
import {AdminSecurityCenter} from './SecurityCenter.tsx';
import {AdminContext} from './context.ts';
import {createAdminSlice} from '../../state/slices/admin.ts';
import {mockAdminApi} from './testkit.tsx';
import {ShellContext} from './shell/context.ts';
import {createCan,type Capability} from './shell/can.ts';
import {parseAdminPath} from './shell/router.ts';
import {canOpenSection,SECTIONS} from './shell/sections.ts';
import {SECURITY_TABS,canOpenSecurityTab} from './shell/securityNavigation.ts';
function grants(caps:Capability[],scope:string[]|null=null){return createCan(()=>({kind:'listed',me:{username:'operator',displayName:null,roles:[],capabilities:caps,domainScope:scope,totpEnrolled:false}}));}
beforeAll(async()=>{await loadCatalog('admin');await loadCatalog('admin-security');});
it('keeps old security bookmarks and nested ban editor query parameters functional',()=>{
 expect(parseAdminPath('/admin#/observability/bans/new?field=ip')).toEqual({section:'security',rest:['bans','new'],query:{field:'ip'},field:'ip'});
 // t97: the require-2FA policy moved to the Requirements tab; old anchors land there.
 expect(parseAdminPath('/admin#/two-factor')).toEqual({section:'security',rest:['requirements']});
 expect(parseAdminPath('/admin#/security/two-factor')).toEqual({section:'security',rest:['requirements']});
 expect(parseAdminPath('/admin#/security/overview')).toEqual({section:'security',rest:['overview']});
 for(const [legacy,tab] of [['sessions','sessions'],['security-policy','policy'],['inbound-allowlist','inbound']])expect(parseAdminPath('/admin#/'+legacy)).toEqual({section:'security',rest:[tab]});
 expect(parseAdminPath('/admin#/observability/statistics').section).toBe('observability');
});
it('shows the centre to each independent authorized role without granting other tabs',()=>{
 const section=SECTIONS.find(row=>row.id==='security')!;
 for(const cap of ['security.read','bans.read','diagnostics.read','sessions.read'] as const)expect(canOpenSection(section,grants([cap]))).toBe(true);
 expect(canOpenSection(section,grants(['users.read']))).toBe(false);
 // The Overview opens for any tile's capability (the server sends only those tiles).
 expect(SECURITY_TABS.filter(tab=>canOpenSecurityTab(grants(['diagnostics.read']),tab)).map(tab=>tab.id)).toEqual(['overview','deployment']);
 expect(SECURITY_TABS.filter(tab=>canOpenSecurityTab(grants(['security.read']),tab)).map(tab=>tab.id)).toContain('requirements');
 expect(SECURITY_TABS.filter(tab=>canOpenSecurityTab(grants(['bans.read']),tab)).map(tab=>tab.id)).not.toContain('requirements');
 expect(SECURITY_TABS.filter(tab=>canOpenSecurityTab(grants(['bans.read'],['example.com']),tab)).map(tab=>tab.id)).toEqual([]);
});
function mount(caps:Capability[],rest:string[]){
 const api=mockAdminApi({listBans:vi.fn(async()=>[{ip:'192.0.2.20',reason:'test',bannedAt:'2026-01-01',expiresAt:null}])});
 const admin=createAdminSlice(api);const navigate=vi.fn(async()=>true);
 render(()=><AdminContext.Provider value={admin}><ShellContext.Provider value={{can:grants(caps),router:{route:()=>({section:'security',rest}),navigate,dispose:()=>{}},username:()=> 'operator',services:{loadDashboard:async()=>({kind:'unavailable'}),search:async()=> 'unavailable'}}}><Suspense fallback={<p>pending</p>}><AdminSecurityCenter/></Suspense></ShellContext.Provider></AdminContext.Provider>);
 return api;
}
it('loads only bans for a bans reader and disables mutations without write permission',async()=>{
 const api=mount(['bans.read'],['bans']);await screen.findByText('192.0.2.20');
 expect(api.listAudit).not.toHaveBeenCalled();
 expect(screen.getByRole('button',{name:'Unban 192.0.2.20'})).toBeDisabled();
 expect(screen.queryByRole('link',{name:'Deployment checks'})).not.toBeInTheDocument();
 expect(screen.getByRole('link',{name:'IP ban list'})).toHaveAttribute('aria-current','page');
});
it('does not mount or fetch a forbidden child even when directly addressed',async()=>{
 const api=mount(['diagnostics.read'],['bans']);
 expect(await screen.findByRole('alert')).toHaveTextContent('not available');
 expect(api.listBans).not.toHaveBeenCalled();expect(api.listAudit).not.toHaveBeenCalled();
});
