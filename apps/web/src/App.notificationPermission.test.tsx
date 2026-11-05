import {afterEach, expect, it, vi} from 'vitest';
import {cleanup, render, screen, waitFor} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import type {AppState} from './state/store.ts';
import type {Platform} from './platform/index.ts';
import type {Email} from './api/jmap-types.ts';
import type {NotificationConfig} from './screens/Settings/types.ts';
import {activateAccountPreferences,suspendAccountPreferences} from './api/accountPreferenceLifecycle.ts';
import {createBrowserPlatform} from './platform/browser.ts';

const hooks=vi.hoisted(()=>({app:null as AppState|null,platform:null as Platform|null,native:false,config:null as NotificationConfig|null,notifications:vi.fn()}));
vi.mock('./state/store.ts',()=>({createAppState:()=>hooks.app!}));
vi.mock('./api/transport.ts',()=>({createConfiguredClient:()=>({})}));
vi.mock('./platform/index.ts',()=>({getPlatform:()=>hooks.platform!,initPlatform:async()=>hooks.platform!,isTauri:()=>false}));
vi.mock('./platform/capabilities.ts',()=>({capabilityEnabled:(name:string)=>name==='notifications'&&hooks.native}));
vi.mock('./api/settings.ts',()=>({clearSettings:()=>{},loadSettings:async()=>{}}));
vi.mock('./api/policy.ts',()=>({resetClientPolicy:()=>{}}));
vi.mock('./api/startAnalytics.ts',()=>({startAnalytics:()=>{}}));
vi.mock('./theme/interfacePreferences.ts',()=>({useInterfacePreferences:()=>{}}));
vi.mock('./screens/Settings/service.ts',()=>({SettingsService:class{notifications(){hooks.notifications();return Promise.resolve(hooks.config!);}}}));
vi.mock('./screens/Mailbox.tsx',()=>({MailboxScreen:()=> <p>Notification test mailbox</p>}));
vi.mock('./plugins-ui/Tier.tsx',()=>({default:()=>null}));
vi.mock('./components/NotificationPermissionPrompt.tsx',()=>({NotificationPermissionPrompt:()=>null}));
vi.mock('./realtime/ConnectionToast.tsx',()=>({ConnectionToast:()=>null}));
vi.mock('./state/NewVersionToast.tsx',()=>({NewVersionToast:()=>null}));
vi.mock('./components/toast/index.ts',()=>({ToastRegion:()=>null}));

const {App}=await import('./App.tsx');
afterEach(()=>{cleanup();suspendAccountPreferences();vi.restoreAllMocks();vi.unstubAllGlobals();hooks.notifications.mockClear();});
const email=(id:string,subject=id):Email=>({id,subject,keywords:{},mailboxIds:{inbox:true},from:[{email:'sender@example.test',name:'Sender'}],to:[],receivedAt:'2026-11-02T10:00:00Z',preview:'',hasAttachment:false});

it.each(['browser','native'] as const)('reseeds %s delivery after permission/config changes and notifies the first new hidden unread without a storm',async kind=>{
 let permission:NotificationPermission='default';
 const systemNotification=vi.fn(function(_title:string,_options?:NotificationOptions){});
 Object.defineProperty(systemNotification,'permission',{get:()=>permission});
 vi.stubGlobal('Notification',systemNotification);
 const nativeNotify=vi.fn(async()=>{}),badge=vi.fn(async()=>{});
 hooks.native=kind==='native';
 hooks.platform=kind==='browser'?createBrowserPlatform():{notify:nativeNotify,setBadgeCount:badge} as unknown as Platform;
 const [messages,setMessages]=createSignal([email('existing')]),[checked,setChecked]=createSignal(false);
 hooks.config={enabled:false,rules:[],quietHours:{enabled:false,start:'22:00',end:'07:00'}};
 hooks.app={authChecked:checked,me:()=>({accountId:'account',username:'me@example.test'}),init:async()=>{setChecked(true);},messages,mailboxes:()=>[{id:'inbox',name:'Inbox'}],assist:{loadConfig:async()=>{}},startRealtime:()=>{},stopRealtime:()=>{},cacheHeaders:()=>{}} as unknown as AppState;
 activateAccountPreferences('account','notification_test_realm');
 let hidden=false;vi.spyOn(document,'hidden','get').mockImplementation(()=>hidden);
 render(()=><App/>);await screen.findByText('Notification test mailbox');
 expect(systemNotification).not.toHaveBeenCalled();expect(nativeNotify).not.toHaveBeenCalled();

 // This is the event emitted after the permission prompt has saved delivery.
 // Seed while visible; the existing unread message must never be announced.
 permission='granted';hooks.config={...hooks.config,enabled:true};
 window.dispatchEvent(new Event('mw:notifications-changed'));
 await Promise.resolve();expect(systemNotification).not.toHaveBeenCalled();expect(nativeNotify).not.toHaveBeenCalled();
 hidden=true;setMessages([email('new','First new message'),email('existing')]);
 if(kind==='browser')await waitFor(()=>expect(systemNotification).toHaveBeenCalledExactlyOnceWith('New message',{body:'First new message',tag:'new'}));
 else await waitFor(()=>expect(nativeNotify).toHaveBeenCalledExactlyOnceWith({id:'new',title:'New message',body:'First new message'}));
 expect(hooks.notifications).toHaveBeenCalledOnce();

 // Delivery still obeys the freshly loaded server rule configuration.
 hooks.config={...hooks.config,rules:[{id:'mute',label:'Quiet',match:'Muted',action:'mute'}]};
 setMessages([email('muted','Muted arrival'),email('new','First new message'),email('existing')]);
 await waitFor(()=>expect(hooks.notifications).toHaveBeenCalledTimes(2));await Promise.resolve();
 expect(kind==='browser'?systemNotification:nativeNotify).toHaveBeenCalledOnce();
});
