import {afterEach,beforeEach,describe,it,expect,vi} from 'vitest';
import {render,screen,fireEvent,waitFor,within,cleanup} from '@solidjs/testing-library';
import {createAppState} from '../state/store.ts';
import {AppContext} from '../state/context.ts';
import {makeClient,mkEmail} from './appHarness.tsx';
import {MessageList} from './MessageList.tsx';
import {clearSettings,loadSettings,type Fetcher,type SettingEntry} from '../api/settings.ts';
import {MESSAGE_LIST_ORDER_KEY,messageListOrder} from './messageOrder.ts';
import {MailboxScreen} from '../screens/Mailbox.tsx';
import {RealtimeContext} from '../realtime/context.ts';
import {createRealtimeController} from '../realtime/controller.ts';
import type {Email,JmapRequest} from '../api/jmap-types.ts';

const json=(body:unknown,status=200)=>new Response(JSON.stringify(body),{status,headers:{'content-type':'application/json'}});
const seed=()=>[
 mkEmail('a',{subject:'Zulu',from:[{name:'Ben',email:'ben@example.org'}],receivedAt:'2026-01-03T12:00:00Z'}),
 mkEmail('b',{subject:'Alpha',from:[{name:'Zoe',email:'zoe@example.org'}],receivedAt:'2026-01-01T12:00:00Z'}),
 mkEmail('c',{subject:'Mike',from:[{name:'Ann',email:'ann@example.org'}],receivedAt:'2026-01-02T12:00:00Z'}),
];
function policy(style='cards',locked=false,fail=false){
 const entries:Record<string,SettingEntry>={
  [MESSAGE_LIST_ORDER_KEY]:{value:'newest',source:locked?'global':'user',locked},
  'mail.messageListStyle':{value:style,source:'user',locked:false},
 };
 const writes:string[]=[];
 const fetcher:Fetcher=async(input,init)=>{
  if(init?.method==='PUT'){
   writes.push(String(input));if(fail)return json({code:'failure'},500);
   const {value}=JSON.parse(String(init.body)) as {value:unknown};entries[MESSAGE_LIST_ORDER_KEY]={value,source:'user',locked:false};return json(entries[MESSAGE_LIST_ORDER_KEY]);
  }
  return json({settings:entries});
 };
 return {fetcher,writes};
}
async function mount(options:{style?:string;empty?:boolean;locked?:boolean;fail?:boolean;mobile?:boolean}={}){
 if(options.mobile)vi.stubGlobal('innerWidth',390);
 const data=policy(options.style,options.locked,options.fail);
 let app!:ReturnType<typeof createAppState>;
 const result=render(()=>{app=createAppState(makeClient({emails:options.empty?[]:seed()}));return <AppContext.Provider value={app}><MessageList hideReadingPane hideSelectionToggle/></AppContext.Provider>;});
 await app.login({username:'me@example.org',password:'p'});await loadSettings({refresh:true,fetcher:data.fetcher});
 await waitFor(()=>expect(app.listLoading()).toBe(false));
 return {app,...data,...result};
}
const openOrder=()=>{fireEvent.click(screen.getByRole('button',{name:'Order by'}));return screen.getByRole('menu',{name:'Order by'});};
beforeEach(()=>{localStorage.clear();clearSettings();vi.stubGlobal('fetch',vi.fn(async(_input:RequestInfo|URL,init?:RequestInit)=>{
 const body=JSON.parse(typeof init?.body==='string'?init.body:'{}') as Partial<JmapRequest>;
 if(body.methodCalls)return json({sessionState:'s',methodResponses:body.methodCalls.map(([name,args,id])=>[name,name==='Mailbox/get'?{accountId:args['accountId'],mailwomanInitialSync:'ready',list:(args['ids'] as string[]??[]).map(value=>({id:value,mailwomanSynced:true}))}:{},id])});
 return json({});
}));});
afterEach(()=>{cleanup();clearSettings();vi.unstubAllGlobals();vi.restoreAllMocks();localStorage.clear();});

describe('message order surfaces',()=>{
 it.each([{style:'cards'},{style:'table'},{style:'cards',empty:true},{style:'table',empty:true},{style:'cards',mobile:true}])('keeps the sort icon on the visible list bar (%j)',async options=>{
  const result=await mount(options);const button=screen.getByRole('button',{name:'Order by'});expect(button).toBeVisible();expect(button).toHaveAttribute('aria-haspopup','menu');
  expect(button.querySelector('[data-icon="sort"]')).not.toBeNull();expect(screen.getByRole('group',{name:'View options'})).toContainElement(button);
  expect(screen.queryByRole('button',{name:'Select messages'})).toBeNull();expect(screen.queryByRole('combobox',{name:'Reading pane'})).toBeNull();
  const menu=openOrder();expect(within(menu).getAllByRole('menuitemradio')).toHaveLength(12);expect(within(menu).getByRole('menuitemradio',{name:'Newest first'})).toHaveAttribute('aria-checked','true');
  if(options.empty){expect(await screen.findByText('No messages')).toBeInTheDocument();expect(result.container.querySelector('.list__empty')).not.toBeNull();}
 });
 it('offers Order by on both a message and the empty list background',async()=>{
  const result=await mount();const row=result.container.querySelector('.list__row')!;fireEvent.contextMenu(row);
  const parent=screen.getByRole('menuitem',{name:'Order by'});expect(parent).toHaveAttribute('aria-haspopup','menu');fireEvent.click(parent);
  expect(screen.getByRole('menu',{name:'Order by'})).toBeVisible();fireEvent.keyDown(screen.getByRole('menu',{name:'Order by'}),{key:'Escape'});fireEvent.keyDown(screen.getByRole('menu',{name:'More actions'}),{key:'Escape'});
  fireEvent.contextMenu(result.container.querySelector('section.list')!);expect(screen.getByRole('menu',{name:'View options'})).toBeVisible();fireEvent.click(screen.getByRole('menuitem',{name:'Order by'}));
  expect(screen.getAllByRole('menuitemradio')).toHaveLength(12);
 });
 it('does not offer a mutation while the administrator locks message order',async()=>{
  const result=await mount({style:'table',locked:true});expect(screen.getByRole('button',{name:'Order by'})).toBeEnabled();const lockedMenu=openOrder();expect(within(lockedMenu).getByRole('menuitemradio',{name:'Oldest first'})).toHaveAttribute('aria-disabled','true');expect(within(lockedMenu).getByRole('menuitem',{name:'Group by'})).toHaveAttribute('aria-haspopup','menu');fireEvent.keyDown(lockedMenu,{key:'Escape'});expect(screen.getByRole('combobox',{name:'Message order'})).toBeDisabled();
  fireEvent.contextMenu(result.container.querySelector('.list__row')!);fireEvent.click(screen.getByRole('menuitem',{name:'Order by'}));expect(screen.getByRole('menuitemradio',{name:'Oldest first'})).toHaveAttribute('aria-disabled','true');expect(result.writes).toEqual([]);
 });
 it('reports a failed menu save and keeps the confirmed checked order',async()=>{
  const result=await mount({fail:true}),toast=vi.spyOn(result.app,'showToast');fireEvent.click(within(openOrder()).getByRole('menuitemradio',{name:'Oldest first'}));
  await waitFor(()=>expect(toast).toHaveBeenCalledWith('error','The message order could not be saved.'));
  expect(messageListOrder()).toBe('newest');expect(within(openOrder()).getByRole('menuitemradio',{name:'Newest first'})).toHaveAttribute('aria-checked','true');expect(result.writes).toHaveLength(1);
 });
 it('restores the table select after a failed save instead of implying a saved order',async()=>{
  const result=await mount({style:'table',fail:true}),toast=vi.spyOn(result.app,'showToast');const select=screen.getByRole('combobox',{name:'Message order'});fireEvent.change(select,{target:{value:'oldest'}});
  await waitFor(()=>expect(toast).toHaveBeenCalledOnce());expect(select).toHaveValue('newest');expect(messageListOrder()).toBe('newest');
 });
});

// The adapter implements the existing JMAP sort contract, independently of UI
// choices. The real Mailbox screen must react to a saved preference, query again
// and render the server's reordered messages (a successful PUT alone is not proof).
it('requeries and actually reorders the mailbox from the icon, row menu and background menu',async()=>{
 const emails=seed(),client=makeClient({emails}),original=client.jmap,queries:JmapRequest[]=[];
 client.jmap=vi.fn(async(request:JmapRequest)=>{
  const response=await original(request);
  const query=request.methodCalls.find(call=>call[0]==='Email/query');
  if(query){
   queries.push(request);const sort=query[1]['sort'] as {property:string;isAscending:boolean}[];
   const field=(mail:Email,key:string)=>key==='from'?mail.from?.[0]?.email??'':key==='subject'?mail.subject??'':mail.receivedAt;
   const ordered=[...emails].sort((a,b)=>{for(const item of sort){const result=field(a,item.property).localeCompare(field(b,item.property));if(result)return item.isAscending?result:-result;}return 0;});
   for(const [name,args]of response.methodResponses){if(name==='Email/get')args['list']=ordered;if(name==='Email/query')args['ids']=ordered.map(mail=>mail.id);}
  }
  for(const [name,args,id]of request.methodCalls)if(/^(AddressBook|ContactCard|ContactGroup)\//.test(name)){
   const index=response.methodResponses.findIndex(call=>call[2]===id);if(index>=0)response.methodResponses[index]=[name,{accountId:args['accountId'],state:'s',list:[],ids:[],notFound:[]},id];
  }
  return response;
 });
 let app!:ReturnType<typeof createAppState>;
 const result=render(()=>{app=createAppState(client);const realtime=createRealtimeController();return <AppContext.Provider value={app}><RealtimeContext.Provider value={realtime}><MailboxScreen/></RealtimeContext.Provider></AppContext.Provider>;});
 await app.login({username:'me@example.org',password:'p'});const data=policy();await loadSettings({refresh:true,fetcher:data.fetcher});
 const ids=()=>Array.from(result.container.querySelectorAll<HTMLElement>('.list__row')).map(row=>row.dataset['emailId']);
 await waitFor(()=>expect(ids()).toEqual(['a','c','b']));
 fireEvent.click(within(openOrder()).getByRole('menuitemradio',{name:'Oldest first'}));await waitFor(()=>expect(ids()).toEqual(['b','c','a']));
 fireEvent.contextMenu(result.container.querySelector('.list__row')!);fireEvent.click(screen.getByRole('menuitem',{name:'Order by'}));fireEvent.click(screen.getByRole('menuitemradio',{name:'Sender'}));await waitFor(()=>expect(ids()).toEqual(['c','a','b']));
 fireEvent.contextMenu(result.container.querySelector('section.list')!);fireEvent.click(screen.getByRole('menuitem',{name:'Order by'}));fireEvent.click(screen.getByRole('menuitemradio',{name:'Subject'}));await waitFor(()=>expect(ids()).toEqual(['b','c','a']));
 const sorts=queries.map(request=>request.methodCalls.find(call=>call[0]==='Email/query')![1]['sort']);
 expect(sorts).toContainEqual([{property:'receivedAt',isAscending:true}]);expect(sorts).toContainEqual([{property:'from',isAscending:true},{property:'receivedAt',isAscending:false}]);expect(sorts).toContainEqual([{property:'subject',isAscending:true},{property:'receivedAt',isAscending:false}]);expect(data.writes).toHaveLength(3);
 await app.searchMessages('subject:Topic',{semantic:true});expect(app.searchActive()).toBe(true);
 const prior=queries.at(-1)!.methodCalls.find(call=>call[0]==='Email/query')![1]['filter'];
 const before=queries.length;fireEvent.click(within(openOrder()).getByRole('menuitemradio',{name:'Sender'}));await waitFor(()=>expect(ids()).toEqual(['c','a','b']));
 const searchSort=queries.slice(before).find(request=>request.methodCalls.some(call=>call[0]==='Email/query'&&JSON.stringify(call[1]['sort']).includes('"from"')))!;
 expect(searchSort).toBeDefined();expect(searchSort.methodCalls.find(call=>call[0]==='Email/query')![1]['filter']).toEqual(prior);
 expect(prior).toMatchObject({semantic:true});expect(app.search()).toBe('subject:Topic');expect(app.searchActive()).toBe(true);expect(data.writes).toHaveLength(4);

});
