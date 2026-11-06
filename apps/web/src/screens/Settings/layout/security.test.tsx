import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {cleanup,render,screen} from '@solidjs/testing-library';
import {SecurityCenter} from '../SecurityCenter.tsx';
import {SettingsService} from '../service.ts';
import {Identities} from '../Identities.tsx';
import {Signatures} from '../Signatures.tsx';
import {clearSettings} from '../../../api/settings.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../../../api/accountPreferenceLifecycle.ts';
import {t} from '../../../i18n';
import {SETTINGS_REGISTRY} from './registry.ts';

vi.mock('../../../api/http.ts',()=>({apiFetch:vi.fn(async(path:string)=>new Response(JSON.stringify(path.includes('remembered')?{devices:[]}:path.includes('signin-geography')?{}:{events:[]}),{status:200}))}));
beforeEach(()=>{
 clearSettings();activateAccountPreferences('layout-sec');
 vi.spyOn(SettingsService.prototype,'twofaStatus').mockResolvedValue({totp:false,passkeys:[],recoveryRemaining:3,passkeysEnabled:true} as never);
 vi.spyOn(SettingsService.prototype,'sessions').mockResolvedValue([]);
 vi.spyOn(SettingsService.prototype,'listIdentities').mockResolvedValue([]);
 vi.spyOn(SettingsService.prototype,'listSignatures').mockResolvedValue([]);
 vi.stubGlobal('fetch',vi.fn(async()=>new Response(JSON.stringify({settings:{}}))));
});
afterEach(()=>{cleanup();vi.restoreAllMocks();vi.unstubAllGlobals();clearSettings();suspendAccountPreferences();});

// Rows that exist only after the owning policy/settings load.
const CONDITIONAL=new Set(['signin-geo-mode','signin-geo-countries','signin-geo-ips','session-alerts-country','session-alerts-channels','session-alerts-sensitivity']);

it('Security: every card is rendered once, registered, and every always-present row appears once',async()=>{
 render(()=><SecurityCenter/>);
 await screen.findByText(t('settings-2fa-totp-title'));
 await screen.findByText(t('settings-sessions-title',{} as never),{selector:'h4'});
 const specs=SETTINGS_REGISTRY.filter(c=>c.section==='security');
 expect(specs.length).toBeGreaterThanOrEqual(10);
 for(const card of specs){
  expect(document.querySelectorAll(`section[id="${card.id}"]`),card.id).toHaveLength(1);
  for(const row of card.rows){
   if(CONDITIONAL.has(row.id))continue;
   expect(document.querySelectorAll(`[id="${row.id}"]`),row.id).toHaveLength(1);
   expect(screen.getAllByText(t(row.label),{selector:'label'}),row.label).toHaveLength(1);
  }
 }
 // previous controls of the sessions card
 expect(screen.getByRole('table',{name:t('settings-sessions-title'),hidden:true})).toBeInTheDocument();
 expect(screen.getByRole('searchbox',{hidden:true})).toBeInTheDocument();
 expect(screen.getByText(t('settings-2fa-recovery-regenerate'))).toBeInTheDocument();
 for(const el of document.querySelectorAll('section[id][aria-labelledby]'))expect(SETTINGS_REGISTRY.some(c=>c.id===el.id),el.id).toBe(true);
});

it('Identities and Signatures render as one registered wide card with their primary actions',async()=>{
 render(()=><><Identities service={new SettingsService()}/><Signatures service={new SettingsService()}/></>);
 await screen.findByText(t('settings-ident-new'));
 for(const id of ['identities-card','signatures-card']){
  expect(document.querySelectorAll(`section[id="${id}"]`)).toHaveLength(1);
  expect(SETTINGS_REGISTRY.some(c=>c.id===id)).toBe(true);
 }
 expect(screen.getByText(t('settings-sig-new'))).toBeInTheDocument();
});
