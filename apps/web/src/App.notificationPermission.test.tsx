import type {IncomingMail} from './realtime/incomingMail.ts';
import {toasts,resetToasts} from './components/toast/toast.ts';
import {messageOpenRequest,takeMessageOpenRequest} from './state/messageOpenRequest.ts';
import {afterEach, expect, it, vi} from 'vitest';
import {cleanup, render, screen, waitFor} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import type {AppState} from './state/store.ts';
import type {Platform} from './platform/index.ts';
import type {Email} from './api/jmap-types.ts';
import type {NotificationConfig} from './screens/Settings/types.ts';
import {activateAccountPreferences,suspendAccountPreferences,preferenceAccount} from './api/accountPreferenceLifecycle.ts';
import {createBrowserPlatform} from './platform/browser.ts';

const hooks=vi.hoisted(()=>({app:null as AppState|null,platform:null as Platform|null,native:false,config:null as NotificationConfig|null,notifications:vi.fn(),incoming:null as ((batch:IncomingMail)=>void)|null}));
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
afterEach(()=>{cleanup();suspendAccountPreferences();vi.restoreAllMocks();vi.unstubAllGlobals();hooks.notifications.mockClear();hooks.incoming=null;resetToasts();takeMessageOpenRequest();if('serviceWorker'in navigator)delete(navigator as unknown as {serviceWorker?:unknown}).serviceWorker;});
const email=(id:string,subject=id):Email=>({id,subject,keywords:{},mailboxIds:{inbox:true},from:[{email:'sender@example.test',name:'Sender'}],to:[],receivedAt:'2026-11-02T10:00:00Z',preview:'',hasAttachment:false});

it.each(['browser','native'] as const)('delivers %s incoming events after permission/config changes without announcing the initial list',async kind=>{
 let permission:NotificationPermission='default';
 const systemNotification=vi.fn(function(_title:string,_options?:NotificationOptions){});
 Object.defineProperty(systemNotification,'permission',{get:()=>permission});
 vi.stubGlobal('Notification',systemNotification);
 const nativeNotify=vi.fn(async()=>{}),badge=vi.fn(async()=>{});
 hooks.native=kind==='native';
 hooks.platform=kind==='browser'?createBrowserPlatform():{notify:nativeNotify,setBadgeCount:badge} as unknown as Platform;
 const [messages,setMessages]=createSignal([email('existing')]),[checked,setChecked]=createSignal(false);
 hooks.config={enabled:false,rules:[],quietHours:{enabled:false,start:'22:00',end:'07:00'}};
 hooks.app={authChecked:checked,me:()=>({accountId:'account',username:'me@example.test'}),init:async()=>{setChecked(true);},messages,mailboxes:()=>[{id:'inbox',name:'Inbox'}],assist:{loadConfig:async()=>{}},startRealtime:()=>{},stopRealtime:()=>{},cacheHeaders:()=>{},onIncomingMail:(handler:(batch:IncomingMail)=>void)=>{hooks.incoming=handler;return()=>{hooks.incoming=null;};}} as unknown as AppState;
 activateAccountPreferences('account','notification_test_realm');
 let hidden=false;vi.spyOn(document,'hidden','get').mockImplementation(()=>hidden);
 render(()=><App/>);await screen.findByText('Notification test mailbox');
 expect(systemNotification).not.toHaveBeenCalled();expect(nativeNotify).not.toHaveBeenCalled();

 // This is the event emitted after the permission prompt has saved delivery.
 // Seed while visible; the existing unread message must never be announced.
 permission='granted';hooks.config={...hooks.config,enabled:true};
 window.dispatchEvent(new Event('mw:notifications-changed'));
 await Promise.resolve();expect(systemNotification).not.toHaveBeenCalled();expect(nativeNotify).not.toHaveBeenCalled();
 hidden=true;setMessages([email('new','First new message'),email('existing')]);hooks.incoming?.({owner:preferenceAccount()!,accountId:'account',messages:[email('new','First new message')]});
 if(kind==='browser')await waitFor(()=>expect(systemNotification).toHaveBeenCalledExactlyOnceWith('New message',expect.objectContaining({body:expect.stringContaining('First new message'),tag:expect.stringContaining(':new')})));
 else await waitFor(()=>expect(nativeNotify).toHaveBeenCalledExactlyOnceWith(expect.objectContaining({id:expect.stringContaining(':new'),title:'New message',body:expect.stringContaining('First new message')})));
 expect(hooks.notifications).toHaveBeenCalledOnce();

 // Delivery still obeys the freshly loaded server rule configuration.
 hooks.config={...hooks.config,rules:[{id:'mute',label:'Quiet',match:'Muted',action:'mute'}]};
 setMessages([email('muted','Muted arrival'),email('new','First new message'),email('existing')]);hooks.incoming?.({owner:preferenceAccount()!,accountId:'account',messages:[email('muted','Muted arrival')]});
 await waitFor(()=>expect(hooks.notifications).toHaveBeenCalledTimes(2));await Promise.resolve();
 expect(kind==='browser'?systemNotification:nativeNotify).toHaveBeenCalledOnce();
});

async function visibleApp(enabled=true):Promise<void>{
 vi.stubGlobal('Notification',{permission:'denied'});vi.spyOn(document,'hidden','get').mockReturnValue(false);
 hooks.native=false;hooks.platform={notify:vi.fn(async()=>{}),setBadgeCount:vi.fn(async()=>{}),onNotificationAction:()=>()=>{}} as unknown as Platform;
 hooks.config={enabled,rules:[],quietHours:{enabled:false,start:'22:00',end:'07:00'}};
 hooks.app={authChecked:()=>true,me:()=>({accountId:'account',username:'me@example.test'}),init:async()=>{},messages:()=>[email('existing')],mailboxes:()=>[{id:'inbox',name:'Inbox'}],assist:{loadConfig:async()=>{}},startRealtime:()=>{},stopRealtime:()=>{},cacheHeaders:()=>{},onIncomingMail:(handler:(batch:IncomingMail)=>void)=>{hooks.incoming=handler;return()=>{hooks.incoming=null;};}} as unknown as AppState;
 activateAccountPreferences('account','notification_test_realm');render(()=><App/>);await screen.findByText('Notification test mailbox');
}
it('shows incoming toasts without browser permission and opens the correct message from the action',async()=>{
 await visibleApp();hooks.incoming?.({owner:preferenceAccount()!,accountId:'account',messages:[email('new','Visible incoming mail')]});await waitFor(()=>expect(toasts()).toHaveLength(1));expect(toasts()[0]!.message).toContain('Visible incoming mail');expect(hooks.platform!.notify).not.toHaveBeenCalled();await toasts()[0]!.action!.run();expect(messageOpenRequest()?.messageId).toBe('new');
});
it('does not bypass a disabled notification preference for in-app toasts',async()=>{
 await visibleApp(false);hooks.incoming?.({owner:preferenceAccount()!,accountId:'account',messages:[email('new')]});await waitFor(()=>expect(hooks.notifications).toHaveBeenCalledOnce());await Promise.resolve();expect(toasts()).toHaveLength(0);expect(hooks.platform!.notify).not.toHaveBeenCalled();
});
it('clears incoming toasts and rejects their old actions after switching accounts',async()=>{
 await visibleApp();hooks.incoming?.({owner:preferenceAccount()!,accountId:'account',messages:[email('new')]});await waitFor(()=>expect(toasts()).toHaveLength(1));const action=toasts()[0]!.action!;activateAccountPreferences('another-account','another_notification_realm');expect(toasts()).toHaveLength(0);await action.run();expect(messageOpenRequest()).toBeUndefined();
});
it('groups an arrival burst and continues to respect mute rules',async()=>{
 await visibleApp();hooks.config={...hooks.config!,rules:[{id:'mute',label:'Muted',match:'Muted',action:'mute'}]};hooks.incoming?.({owner:preferenceAccount()!,accountId:'account',messages:[email('one','First'),email('two','Second'),email('muted','Muted arrival')]});await waitFor(()=>expect(toasts()).toHaveLength(1));expect(toasts()[0]!.message).toContain('New messages: 2');
});
it('suppresses a policy response that arrives after the account changes',async()=>{
 await visibleApp();let release!:(value:NotificationConfig)=>void;hooks.notifications.mockImplementationOnce(()=>{});const previous=hooks.config!;
 // The service promise is deferred independently of the message arrival.
 const method=await import('./screens/Settings/service.ts');const spy=vi.spyOn(method.SettingsService.prototype,'notifications').mockImplementationOnce(()=>new Promise(resolve=>{release=resolve;}));
 hooks.incoming?.({owner:preferenceAccount()!,accountId:'account',messages:[email('new')]});activateAccountPreferences('another-account','another_realm');release(previous);await Promise.resolve();await Promise.resolve();expect(toasts()).toHaveLength(0);spy.mockRestore();
});

it('does not duplicate an OS alert already displayed by a background push wake',async()=>{
 const worker=new EventTarget();Object.defineProperty(navigator,'serviceWorker',{configurable:true,value:worker});await visibleApp();vi.stubGlobal('Notification',{permission:'granted'});vi.spyOn(document,'hidden','get').mockReturnValue(true);worker.dispatchEvent(new MessageEvent('message',{data:{type:'mw-push-wake',notificationShown:true}}));hooks.incoming?.({owner:preferenceAccount()!,accountId:'account',messages:[email('new')]});await waitFor(()=>expect(hooks.notifications).toHaveBeenCalledOnce());await Promise.resolve();expect(hooks.platform!.notify).not.toHaveBeenCalled();expect(toasts()).toHaveLength(0);
});
