import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {cleanup,render,screen,waitFor} from '@solidjs/testing-library';
import {Privacy} from '../Privacy.tsx';
import {AnalyticsConsent} from '../AnalyticsConsent.tsx';
import {LoadTimePreferences} from '../LoadTimePreferences.tsx';
import {ErrorReportPreferences} from '../ErrorReportPreferences.tsx';
import {YourData} from '../YourData.tsx';
import {DataRequests} from '../DataRequests.tsx';
import {clearSettings} from '../../../api/settings.ts';
import * as privacyApi from '../../../api/privacy.ts';
import * as requestsApi from '../../../api/dataRequests.ts';
import {t} from '../../../i18n';
import {SETTINGS_REGISTRY} from './registry.ts';

vi.mock('../../../api/privacy.ts',async importOriginal=>({...await importOriginal<typeof import('../../../api/privacy.ts')>(),decideConsent:vi.fn(),listSessions:vi.fn(),listKeys:vi.fn(),revokeSession:vi.fn(),revokeKey:vi.fn()}));
vi.mock('../../../api/dataRequests.ts',async importOriginal=>({...await importOriginal<typeof import('../../../api/dataRequests.ts')>(),loadOverview:vi.fn(),listRequests:vi.fn(),submitRequest:vi.fn()}));

const overview=()=>({
 account:{address:'ana@example.test',domain:'example.test'},
 categories:[{id:'mail',rows:120,bytes:2097152,retention:null,processors:['mailServer'],purpose:null}],
 consents:{enabled:true,art22:{enabled:true,humanReview:true},purposes:[{id:'assistant',basis:'consent',version:1,text:null,state:'unset',enforcement:'ledger',managedIn:'',decidedAt:null,decidedVersion:null,canToggle:true,defaultOn:false}],assistantFeatures:[],receipts:[]},
 restriction:{active:false,scopes:[],since:null,expiresAt:null},links:{policy:'/privacy',processing:'/api/privacy/processing'},shortcuts:{download:'access',delete:'erasure'},
}) as never;
const policy=()=>({enabled:true,types:{access:true,portability:true,rectification:true,erasure:true,restriction:false,objection:true,other:true},reauthRequired:true,confirmationRequired:true,identityVerificationRequired:false,approval:{export:'auto',erasure:'admin',other:'admin'},graceDays:14,slaDays:30,mailAllowed:true,mailFormat:'mbox',singleUseDownload:true,linkMinutes:60,exportHours:24,contact:{name:'',email:null,url:null},text:{intro:{}}}) as never;

beforeEach(()=>{
 clearSettings();
 vi.mocked(privacyApi.listSessions).mockResolvedValue([]);vi.mocked(privacyApi.listKeys).mockResolvedValue([]);
 vi.mocked(requestsApi.loadOverview).mockResolvedValue({policy:policy()} as never);vi.mocked(requestsApi.listRequests).mockResolvedValue([]);
 vi.stubGlobal('fetch',vi.fn(async()=>new Response(JSON.stringify({settings:{}}))));
});
afterEach(()=>{cleanup();vi.unstubAllGlobals();clearSettings();});

// Rows that exist only for some policy values.
const CONDITIONAL=new Set(['privacy-strip-mailer-headers','dr-include-mail','dr-erasure-sure']);

it('Privacy and data: every card renders once, is registered, and every row label appears once',async()=>{
 render(()=><><Privacy/><AnalyticsConsent loadTag={async()=>null}/><LoadTimePreferences/><ErrorReportPreferences/><YourData load={async()=>overview()}/><DataRequests/></>);
 await screen.findByLabelText(t('privacy-remote-mode'));
 await screen.findByTestId('yd-categories');
 await screen.findByLabelText(t('dr-kind'));
 const cards=SETTINGS_REGISTRY.filter(c=>c.section==='privacy');
 expect(cards.length).toBeGreaterThanOrEqual(12);
 for(const card of cards){
  if(card.id==='privacy-card-cookies')continue; // shown only while the administrator has the cookie control on
  expect(document.querySelectorAll(`section[id="${card.id}"]`),card.id).toHaveLength(1);
  for(const row of card.rows){
   if(CONDITIONAL.has(row.id))continue;
   expect(document.querySelectorAll(`[id="${row.id}"]`),row.id).toHaveLength(1);
   await waitFor(()=>expect(screen.getAllByText(t(row.label),{selector:'label'}),row.label).toHaveLength(1));
  }
 }
 for(const el of document.querySelectorAll('section[id][aria-labelledby]'))expect(SETTINGS_REGISTRY.some(c=>c.id===el.id),el.id).toBe(true);
});
