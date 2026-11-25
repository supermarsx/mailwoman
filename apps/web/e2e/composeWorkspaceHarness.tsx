import {createSignal,onMount,Show} from 'solid-js';
import {render} from 'solid-js/web';
import {AppContext} from '../src/state/context.ts';
import {createAppState} from '../src/state/store.ts';
import {Compose} from '../src/components/Compose.tsx';
import {Settings} from '../src/screens/Settings.tsx';
import {createSettingsNavigation} from '../src/screens/Settings/navigation.tsx';
import {makeContactsClient,defaultSeed} from '../src/modules/contacts/mockClient.ts';
import {ToastRegion} from '../src/components/toast/ToastRegion.tsx';
import {CAP_MAIL} from '../src/api/jmap-types.ts';
import '../src/styles/app.css';
import '../src/styles/workspace.css.ts';
import '../src/components/detachedWorkspace.css.ts';
import '../src/theme/themes.css.ts';

const client=makeContactsClient(defaultSeed()),jmap=client.jmap,session=client.session;
client.login=async()=>({username:'me@example.org',accountId:'acct1',storageAuthority:'compose-fixture'});
client.session=async()=>({...await session(),primaryAccounts:{[CAP_MAIL]:'acct1'}});
client.jmap=async(request,options)=>{
 const response=await jmap(request,options);
 for(const call of response.methodResponses){
  if(call[0]==='Identity/get')call[1]={accountId:'acct1',state:'s',notFound:[],list:[{id:'identity',name:'Sender',email:'me@example.org',replyTo:null,sentMailboxId:null,signatureText:'Mailwoman Team\nteam@example.org',signatureHtml:'<p><strong>Mailwoman Team</strong><br>team@example.org</p>',signatureFormat:'html'},{id:'work',name:'Work',email:'work@example.org',replyTo:null,sentMailboxId:null,signatureText:'Work Team',signatureHtml:null,signatureFormat:'plain'}]};
  if(call[0]==='Mailbox/get')call[1]={accountId:'acct1',state:'s',notFound:[],list:[{id:'inbox',name:'Inbox',role:'inbox',parentId:null,sortOrder:0,totalEmails:0,unreadEmails:0}]};
  if(call[0]==='Email/query')call[1]={accountId:'acct1',queryState:'q',position:0,total:0,ids:[]};
  if(call[0]==='Email/get')call[1]={accountId:'acct1',state:'s',notFound:[],list:[]};
 }
 return response;
};
render(()=>{
 const app=createAppState(client),[ready,setReady]=createSignal(false);
 onMount(()=>{void app.login({username:'me@example.org',password:'fixture'}).then(async()=>{await app.loadIdentities();setReady(true);});});
 return <AppContext.Provider value={app}><Show when={ready()}><Show when={new URLSearchParams(location.search).has('settings-workspace')} fallback={<Compose onClose={()=>{}}/>}>
  <section class="workspace-pane" data-testid="settings-workspace-pane"><div class="detached-workspace__home"><div class="detached-workspace__portal"><Settings embedded navigation={createSettingsNavigation({section:'appearance'})} onClose={()=>{}}/></div></div></section>
 </Show></Show><ToastRegion/></AppContext.Provider>;
},document.getElementById('root')!);
