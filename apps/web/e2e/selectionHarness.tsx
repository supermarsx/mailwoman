import {onMount} from 'solid-js';
import {render} from 'solid-js/web';
import {AppContext} from '../src/state/context.ts';
import {createAppState} from '../src/state/store.ts';
import {MessageList,messageRowHeight} from '../src/components/MessageList.tsx';
import type {Client} from '../src/api/client.ts';
import {CAP_MAIL,type Email,type JmapRequest,type JmapResponse} from '../src/api/jmap-types.ts';
import '../src/styles/app.css';
import '../src/theme/themes.css.ts';

const emails:Email[]=Array.from({length:600},(_,index)=>({id:`message-${index}`,subject:`Message ${index}`,mailboxIds:{inbox:true},from:[{name:'Sender',email:'sender@example.test'}],to:[],receivedAt:new Date(Date.UTC(2026,10,25)-index*60000).toISOString(),preview:'Selection regression fixture',keywords:{$seen:true}}));
const identity={username:'me@example.test',accountId:'acct1',storageAuthority:'selection-fixture'};
const client:Client={
 login:async()=>identity,logout:async()=>{},me:async()=>identity,
 session:async()=>({capabilities:{},accounts:{acct1:{name:'Fixture',isPersonal:true,isReadOnly:false,accountCapabilities:{}}},primaryAccounts:{[CAP_MAIL]:'acct1'},username:identity.username,apiUrl:'/jmap/api',downloadUrl:'/d',uploadUrl:'/u',eventSourceUrl:'/e',state:'s'}),
 sanitize:async html=>html,onNetwork:()=>()=>{},
 jmap:async(request:JmapRequest):Promise<JmapResponse>=>{
  const query=request.methodCalls.find(call=>call[0]==='Email/query');
  if(query){const args=query[1],position=typeof args['anchor']==='string'?emails.findIndex(message=>message.id===args['anchor']):Number(args['position']??0),page=emails.slice(position,position+Number(args['limit']??50));return {sessionState:'s',methodResponses:[['Email/query',{accountId:'acct1',queryState:'stable',position,total:emails.length,ids:page.map(message=>message.id)},'q'],['Email/get',{accountId:'acct1',state:'s',list:page,notFound:[]},'g']]};}
  return {sessionState:'s',methodResponses:request.methodCalls.map(([method,,tag])=>[method,method==='Mailbox/get'?{accountId:'acct1',state:'s',list:[{id:'inbox',name:'Inbox',role:'inbox',parentId:null,sortOrder:0,totalEmails:600,unreadEmails:0}],notFound:[]}:{},tag])};
 },
};
render(()=>{
 const app=createAppState(client);
 Object.assign(globalThis,{selectionHarness:{selected:()=>[...app.selectedIds()],range:app.loadedRange,rowHeight:()=>messageRowHeight(app.density(),false),targets:app.actionTargets}});
 onMount(()=>{void app.login({username:identity.username,password:'fixture'});});
 return <AppContext.Provider value={app}><MessageList/></AppContext.Provider>;
},document.getElementById('root')!);
