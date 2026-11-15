import {sanitizeStrip} from '../src/sanitize/index.ts';
import {startSharedToasts} from '../src/components/toast/shared.ts';
import {toast,toasts} from '../src/components/toast/toast.ts';
import {publishSharedView,observeSharedViews} from '../src/worker/networkProxy.ts';
import {apiFetch} from '../src/api/http.ts';
import {installTrustedTypesPolicy} from '../src/security/trustedTypes.ts';
// Browser integration entry, substituted for main.tsx only by the loopback
// test build. All client, vault, worker and replay implementations are real.
import {createConfiguredClient} from '../src/api/transport.ts';
import {activateAccountPreferences} from '../src/api/accountPreferenceLifecycle.ts';
import {setPrimaryMailboxAuthority} from '../src/api/linkedMailboxSession.ts';
import {configureOfflinePolicy,isOffline} from '../src/offline/runtime.ts';
import {createSharedPushClient,networkCoordinationMode} from '../src/worker/networkProxy.ts';
import {registerServiceWorker,requestBackgroundReplay} from '../src/sw/register.ts';
import {encryptedOutboxStore} from '../src/offline/encryptedOutbox.ts';
import {idbOutboxStore} from '../src/offline/idb.ts';
import {accountOutbox} from '../src/offline/accountOutbox.ts';
import {enqueueOutbound,drainOutbox,type QueuedItem} from '../src/offline/outbox.ts';
import {uploadBlob} from '../src/api/jmap.ts';
import type {JmapRequest} from '../src/api/jmap-types.ts';
installTrustedTypesPolicy();
startSharedToasts();let selected:unknown,actions=0;observeSharedViews(view=>{if(view.topic==='selection:fixture')selected=view.value;});
const productionClient=createConfiguredClient(),failures:string[]=[];
const client={...productionClient,jmap:async(...args:Parameters<typeof productionClient.jmap>)=>{try{return await productionClient.jmap(...args);}catch(error){failures.push(error instanceof Error?`${error.name}: ${error.message}`:String(error));throw error;}}};
let owner:{accountId:string;storageAuthority:string}|undefined;
let changes=0;
const query:JmapRequest={using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[['Email/query',{accountId:'a',filter:{inMailbox:'inbox'},limit:50},'q'],['Email/get',{accountId:'a','#ids':{resultOf:'q',name:'Email/query',path:'/ids'},properties:['id','subject','receivedAt','keywords','mailboxIds']},'g']]};
const push=createSharedPushClient({});push.onStateChange(()=>changes++);
export const harness={
 async start(){const me=await client.me();if(!me.storageAuthority)throw new Error('Missing authority');owner={accountId:me.accountId,storageAuthority:me.storageAuthority};activateAccountPreferences(me.accountId,me.storageAuthority);setPrimaryMailboxAuthority(me.storageAuthority);configureOfflinePolicy({enabled:true,budgetMb:10,retentionDays:90,eviction:'lru'},me);await client.me();await client.session();await registerServiceWorker();push.connect();return client.jmap(query);},
 query:()=>client.jmap(query),
 async pdfDocument(){const response=await apiFetch('/pdfjs/web/mailwoman.html');return {html:await response.text(),policy:response.headers.get('content-security-policy')};},
 sanitize:()=>sanitizeStrip('<p>Offline readable body</p><script>evil()</script>'),
 select:(ids:string[])=>publishSharedView('selection:fixture',ids),selection:()=>selected,
 notify:()=>toast.info('Shared browser notification',{id:'cross-window',action:{label:'Apply',run:()=>{actions++;}}}),notifications:()=>toasts().map(row=>({id:row.id,message:row.message,action:row.action?.label})),actions:()=>actions,
 async invokeNotification(){const row=toasts().find(row=>row.id==='cross-window');if(row?.action){const run=row.action.run;toast.dismiss(row.id,'action');await run();}},
 async attachment(range?:string){const response=await apiFetch('/jmap/download/a/pdf/doc',{...(range?{headers:{range}}:{})});return {status:response.status,range:response.headers.get('content-range'),text:await response.text()};},
 async calendarData(){return client.jmap({using:query.using,methodCalls:[['CalendarEvent/get',{accountId:'a',ids:null},'events']]});},
 async calendar(){return client.jmap({using:query.using,methodCalls:[['CalendarEvent/expand',{accountId:'a',start:'2030-03-24T00:00:00Z',end:'2030-04-07T00:00:00Z'},'expand']]});},
 async body(){return client.jmap({using:query.using,methodCalls:[['Email/get',{accountId:'a',ids:['m'],properties:['id','subject','bodyValues'],fetchAllBodyValues:true},'body']]});},
 async createNote(){return client.jmap({using:query.using,methodCalls:[['Note/set',{accountId:'a',create:{new:{title:'Created offline',bodyHtml:'<p>Queued note</p>'}}},'note-set']]});},
 async editNote(id:string){return client.jmap({using:query.using,methodCalls:[['Note/set',{accountId:'a',update:{[id]:{title:'Edited offline'}}},'note-set']]});},
 async note(){return client.jmap({using:query.using,methodCalls:[['Note/get',{accountId:'a',ids:null},'notes']]});},
 async queueMove(){if(!owner)throw new Error('Start first');const store=accountOutbox(encryptedOutboxStore(idbOutboxStore()),owner.accountId,owner.storageAuthority);const item=await enqueueOutbound(store,{type:'move',payload:{accountId:'a',emailId:'m',mailboxIds:{archive:true},previousMailboxIds:{inbox:true}}});await store.put({...item,storageAuthority:owner.storageAuthority,linkId:null,serverBase:''} as QueuedItem);return item.id;},
 async queueSend(){const attachment=await this.file();if(!owner)throw new Error('Start first');const store=accountOutbox(encryptedOutboxStore(idbOutboxStore()),owner.accountId,owner.storageAuthority);const item=await enqueueOutbound(store,{type:'send',payload:{accountId:'a',draft:{from:{email:'a@example.test'},to:'b@example.test',subject:'Queued browser send',htmlBody:'Offline body',draftMailboxId:'drafts',attachments:[{...attachment,name:'offline.txt'}]}}});await store.put({...item,storageAuthority:owner.storageAuthority,linkId:null,serverBase:''} as QueuedItem);return requestBackgroundReplay();},
 async replay(){if(!owner)throw new Error('Start first');const live=await client.me();if(isOffline()||live.storageAuthority!==owner.storageAuthority)throw new Error('Offline authority not verified');return drainOutbox(accountOutbox(encryptedOutboxStore(idbOutboxStore()),owner.accountId,owner.storageAuthority),client);},
 async file(){return uploadBlob('/jmap/upload/{accountId}','a',Object.assign(new Blob(['Encrypted local attachment'],{type:'text/plain'}),{name:'offline.txt'}));},
 async debug(){return {failures,queued:await encryptedOutboxStore(idbOutboxStore()).all()};},
 mode:networkCoordinationMode,offline:()=>isOffline(),changes:()=>changes,
};
(window as unknown as {syncHarness:typeof harness}).syncHarness=harness;
