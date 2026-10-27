import {createSignal,onCleanup} from 'solid-js';
import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import type {Client} from '../api/client.ts';
import {listMailbox,responseFor} from '../api/jmap.ts';
import type {Email,EmailGetResponse,EmailQueryResponse} from '../api/jmap-types.ts';

/** One retained folder owns its page, selection and viewport; no main-list state. */
export function createFolderView(client:Client,mailboxId:string,allowed:()=>boolean){
 const owner=preferenceAccount();
 const [messages,setMessages]=createSignal<Email[]>([]),[position,setPosition]=createSignal(0),[total,setTotal]=createSignal<number|null>(0);
 const [loading,setLoading]=createSignal(false),[error,setError]=createSignal(false),[selection,setSelection]=createSignal<ReadonlySet<string>>(new Set<string>());
 const [scroll,setScroll]=createSignal(0);
 let request=0,disposed=false,abort:AbortController|undefined;
 const current=()=>!disposed&&owner!==null&&preferenceAccountIsCurrent(owner)&&allowed();
 function clear(){request++;abort?.abort();setMessages([]);setSelection(new Set<string>());setTotal(0);setLoading(false);setScroll(0);}
 const stop=observePreferenceAccount(next=>{if(next!==owner)clear();});
 onCleanup(()=>{disposed=true;clear();stop();});
 async function load(next=position()):Promise<void>{
  if(!current()){clear();return;}
  const generation=++request;abort?.abort();abort=new AbortController();setLoading(true);setError(false);
  try{
   const response=await client.jmap(listMailbox(owner!.id,mailboxId,100,{position:Math.max(0,next)}),{signal:abort.signal});
   if(!current()||generation!==request)return;
   const query=responseFor<EmailQueryResponse>(response,'q'),result=responseFor<EmailGetResponse>(response,'g');
   if(query.accountId!==owner!.id||result.accountId!==owner!.id||!Array.isArray(query.ids)||!Array.isArray(result.list)||query.ids.length>100)throw new Error('Invalid folder page');
   if(query.ids.length===0&&next>0&&query.total!==undefined){const last=Math.max(0,Math.floor((query.total-1)/100)*100);if(last<next){await load(last);return;}}
   const byId=new Map(result.list.map(email=>[email.id,email]));
   setMessages(query.ids.flatMap(id=>{const email=byId.get(id);return email?[email]:[];}));
   if(next!==position()){setSelection(new Set<string>());setScroll(0);}
   else setSelection(old=>new Set([...old].filter(id=>query.ids.includes(id))));
   setPosition(query.position);setTotal(query.total??null);
  }catch{if(current()&&generation===request)setError(true);}
  finally{if(current()&&generation===request)setLoading(false);}
 }
 function select(id:string,extend=false,toggle=false){
  if(!current()||!messages().some(email=>email.id===id))return;
  const previous=[...selection()].at(-1),rows=messages();
  if(extend&&previous){const a=rows.findIndex(row=>row.id===previous),b=rows.findIndex(row=>row.id===id);if(a>=0){setSelection(new Set(rows.slice(Math.min(a,b),Math.max(a,b)+1).map(row=>row.id)));return;}}
  setSelection(old=>{if(!toggle)return new Set<string>([id]);const next=new Set<string>(old);if(next.has(id))next.delete(id);else next.add(id);return next;});
 }
 function retainSelection(ids:readonly string[]){setSelection(old=>new Set([...old].filter(id=>ids.includes(id))));}
 function selectAll(){if(current())setSelection(new Set(messages().map(email=>email.id)));}
 return {messages,position,total,loading,error,selection,scroll,setScroll,load,select,current,retainSelection,selectAll};
}
