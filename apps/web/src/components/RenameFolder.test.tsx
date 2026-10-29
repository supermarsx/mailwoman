// t65: "Rename…" in the folder menu — offered only for user folders, inline
// edit (Enter saves, Escape cancels, blur saves), validation said under the
// field, the server's refusal said the same way, and F2 on a focused folder.
import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {makeClient} from './appHarness.tsx';
import {createAppState} from '../state/store.ts';
import {AppContext} from '../state/context.ts';
import {CAP_MAIL,type JmapRequest,type JmapResponse,type Mailbox} from '../api/jmap-types.ts';
import {FolderContextMenu} from './FolderContextMenu.tsx';
import {getKeymap} from '../keymap/index.ts';
import {canRename,nameProblem,renameAbility,renameFolder} from '../api/renameFolder.ts';

const rights=(mayRename:boolean)=>({mayReadItems:true,mayAddItems:true,mayRemoveItems:true,maySetSeen:true,maySetKeywords:true,mayCreateChild:true,mayRename,mayDelete:mayRename,maySubmit:true});
const initial=():Mailbox[]=>[
 {id:'inbox',name:'Inbox',parentId:null,role:'inbox',sortOrder:0,totalEmails:4,unreadEmails:2,myRights:rights(false)},
 {id:'trash',name:'Trash',parentId:null,role:'trash',sortOrder:2,totalEmails:0,unreadEmails:0,myRights:rights(false)},
 {id:'work',name:'Work',parentId:null,role:null,sortOrder:10,totalEmails:5,unreadEmails:1,myRights:rights(true)},
 {id:'q3',name:'Q3',parentId:'work',role:null,sortOrder:10,totalEmails:2,unreadEmails:0,myRights:rights(true)},
 {id:'home',name:'Home',parentId:null,role:null,sortOrder:10,totalEmails:0,unreadEmails:0,myRights:rights(true)},
 {id:'shared',name:'Team',parentId:null,role:null,sortOrder:10,totalEmails:0,unreadEmails:0,myRights:rights(false)},
 {id:'search',name:'Saved',parentId:null,role:null,sortOrder:20,totalEmails:0,unreadEmails:0,mailwomanSearchQuery:'{}',myRights:rights(false)},
];

const server=vi.hoisted(()=>({boxes:[] as Mailbox[],manages:true as boolean,sets:[] as Record<string,unknown>[],refuse:null as string|null}));
function answer(request:JmapRequest):JmapResponse|null{
 const [name,args,id]=request.methodCalls[0]!;const call=args as Record<string,unknown>;
 if(name==='Mailbox/get')return {methodResponses:[['Mailbox/get',{accountId:'acct1',state:'s',list:Array.isArray(call['ids'])&&(call['ids'] as unknown[]).length===0?[]:server.boxes,notFound:[],mailwomanHierarchyDelimiter:'/'},id]],sessionState:'s'};
 if(name==='Mailbox/set'){
  server.sets.push(call);
  const [target,patch]=Object.entries(call['update'] as Record<string,{name:string}>)[0]!;
  if(server.refuse)return {methodResponses:[['Mailbox/set',{accountId:'acct1',updated:null,notUpdated:{[target]:{type:server.refuse}}},id]],sessionState:'s'};
  server.boxes=server.boxes.map(box=>box.id===target?{...box,name:patch.name}:box);
  return {methodResponses:[['Mailbox/set',{accountId:'acct1',updated:{[target]:null},notUpdated:null},id]],sessionState:'s'};
 }
 return null;
}
vi.mock('../api/transport.ts',()=>({createConfiguredClient:()=>({
 session:async()=>({capabilities:{},accounts:{acct1:{name:'T',isPersonal:true,isReadOnly:false,accountCapabilities:{[CAP_MAIL]:{mailwomanMailboxSet:server.manages,maxSizeMailboxName:255}}}},primaryAccounts:{},username:'me',apiUrl:'',downloadUrl:'',uploadUrl:'',eventSourceUrl:'',state:'s'}),
 jmap:vi.fn(async(request:JmapRequest)=>{const reply=answer(request);if(!reply)throw new Error(`unexpected ${request.methodCalls[0]![0]}`);return reply;}),
})}));

beforeEach(()=>{server.boxes=initial();server.manages=true;server.sets=[];server.refuse=null;});
afterEach(()=>{cleanup();localStorage.clear();});

/** The folder list's rows, each wrapped in the menu as Mailbox.tsx does. */
async function sidebar(){
 const base=makeClient();
 const client={...base,jmap:vi.fn(async(request:JmapRequest)=>answer(request)??base.jmap(request))} as typeof base;
 const app=createAppState(client);
 render(()=><AppContext.Provider value={app}><nav>{server.boxes.map(box=><FolderContextMenu box={box} onOpen={()=>{}} onSettings={()=>{}} onShare={()=>{}} canShare={()=>false} onProbe={()=>{}}><button type="button" class="sidebar__box">{`${box.name} target`}</button></FolderContextMenu>)}</nav></AppContext.Provider>);
 await app.login({username:'me@example.org',password:'p'});await waitFor(()=>expect(app.mailboxes()).toHaveLength(server.boxes.length));
 return app;
}
const openMenu=(label:string)=>fireEvent.keyDown(screen.getByText(`${label} target`),{key:'F10',shiftKey:true});
async function startRename(label:string){
 await waitFor(()=>{openMenu(label);expect(screen.getByRole('menuitem',{name:'Rename…'})).toBeInTheDocument();});
 fireEvent.click(screen.getByRole('menuitem',{name:'Rename…'}));
 return await screen.findByRole('textbox',{name:new RegExp(`New name for .*${label}`)}) as HTMLInputElement;
}

describe('the folder menu',()=>{
 it('offers Rename… for custom folders and subfolders only',async()=>{
  await sidebar();
  await waitFor(()=>{openMenu('Work');expect(screen.getByRole('menuitem',{name:'Rename…'})).toBeInTheDocument();});
  fireEvent.keyDown(document.activeElement??document.body,{key:'Escape'});
  openMenu('Q3');expect(screen.getByRole('menuitem',{name:'Rename…'})).toBeInTheDocument();fireEvent.keyDown(document.activeElement??document.body,{key:'Escape'});
  for(const label of ['Inbox','Trash','Team','Saved']){
   openMenu(label);
   expect(screen.getByRole('menuitem',{name:'Open folder'})).toBeInTheDocument();
   expect(screen.queryByRole('menuitem',{name:'Rename…'}),label).toBeNull();
   fireEvent.keyDown(document.activeElement??document.body,{key:'Escape'});
  }
 });
 it('hides Rename… on an account that cannot manage folders (POP3, bridges)',async()=>{
  server.manages=false;await sidebar();
  await new Promise(resolve=>setTimeout(resolve,20));
  openMenu('Work');expect(screen.getByRole('menuitem',{name:'Open folder'})).toBeInTheDocument();
  expect(screen.queryByRole('menuitem',{name:'Rename…'})).toBeNull();
 });
});

describe('inline rename',()=>{
 it('selects the name, saves on Enter, and the folder keeps its id and counts',async()=>{
  const app=await sidebar();
  const input=await startRename('Work');
  await waitFor(()=>expect(document.activeElement).toBe(input));
  expect(input.value).toBe('Work');expect(input.selectionStart).toBe(0);expect(input.selectionEnd).toBe(4);
  expect(screen.getByText('Work target').closest('[data-folder-rename]')).toBeNull();
  fireEvent.input(input,{target:{value:'Relatórios'}});fireEvent.keyDown(input,{key:'Enter'});
  await waitFor(()=>expect(screen.queryByRole('textbox')).toBeNull());
  expect(server.sets).toEqual([{accountId:'acct1',update:{work:{name:'Relatórios'}}}]);
  const work=app.mailboxes().find(box=>box.id==='work')!;
  expect(work.name).toBe('Relatórios');expect(work.totalEmails).toBe(5);expect(work.unreadEmails).toBe(1);
  expect(app.mailboxes().find(box=>box.id==='q3')!.parentId).toBe('work');
  await waitFor(()=>expect(document.activeElement).toBe(screen.getByText('Work target')));
 });
 it('cancels on Escape without a request',async()=>{
  await sidebar();
  const input=await startRename('Home');
  fireEvent.input(input,{target:{value:'Elsewhere'}});fireEvent.keyDown(input,{key:'Escape'});
  await waitFor(()=>expect(screen.queryByRole('textbox')).toBeNull());
  expect(server.sets).toEqual([]);
 });
 it('saves on blur',async()=>{
  const app=await sidebar();
  const input=await startRename('Home');
  fireEvent.input(input,{target:{value:'House'}});fireEvent.blur(input);
  await waitFor(()=>expect(screen.queryByRole('textbox')).toBeNull());
  expect(app.mailboxes().find(box=>box.id==='home')!.name).toBe('House');
 });
 it('leaving an unchanged name sends nothing',async()=>{
  await sidebar();
  const input=await startRename('Home');fireEvent.keyDown(input,{key:'Enter'});
  await waitFor(()=>expect(screen.queryByRole('textbox')).toBeNull());expect(server.sets).toEqual([]);
 });
 it('says what is wrong under the field and keeps it open',async()=>{
  await sidebar();
  const input=await startRename('Home');
  const cases:[string,RegExp][]=[
   ['   ',/Enter a name/],
   ['a/b',/cannot contain “\/”/],
   ['Work',/already a folder called .*Work/],
   ['x'.repeat(256),/too long.*255 bytes/],
   ['a*b',/cannot contain \*, %/],
  ];
  for(const [value,message] of cases){
   fireEvent.input(input,{target:{value}});fireEvent.keyDown(input,{key:'Enter'});
   const alert=await screen.findByRole('alert');expect(alert).toHaveTextContent(message);
   expect(input).toHaveAttribute('aria-invalid','true');expect(input.getAttribute('aria-describedby')).toBe(alert.id);
  }
  expect(server.sets).toEqual([]);
  // A sibling of a subfolder is a different level: Q3 under Work does not clash with a top-level Q3.
  fireEvent.input(input,{target:{value:'Q3'}});fireEvent.keyDown(input,{key:'Enter'});
  await waitFor(()=>expect(screen.queryByRole('textbox')).toBeNull());
  expect(server.sets).toHaveLength(1);
 });
 it('shows the server refusal inline',async()=>{
  server.refuse='alreadyExists';await sidebar();
  const input=await startRename('Home');
  fireEvent.input(input,{target:{value:'Elsewhere'}});fireEvent.keyDown(input,{key:'Enter'});
  expect(await screen.findByRole('alert')).toHaveTextContent(/already a folder called .*Elsewhere/);
  expect(screen.getByRole('textbox')).toBeInTheDocument();
 });
 it('F2 on a focused user folder starts the rename; on Inbox it does nothing',async()=>{
  await sidebar();
  const inbox=screen.getByText('Inbox target');inbox.focus();
  await new Promise(resolve=>setTimeout(resolve,20));
  expect(getKeymap().dispatch(new KeyboardEvent('keydown',{key:'F2',bubbles:true,cancelable:true}))).toBe(false);
  expect(screen.queryByRole('textbox')).toBeNull();
  const home=screen.getByText('Home target');home.focus();
  await waitFor(()=>{fireEvent.keyDown(home,{key:'F2'});expect(screen.getByRole('textbox',{name:/New name for .*Home/})).toBeInTheDocument();});
 });
});

describe('rename rules',()=>{
 const boxes=initial();const home=boxes.find(box=>box.id==='home')!,q3=boxes.find(box=>box.id==='q3')!;
 it('offers rename only for real folders the server lets this user rename',()=>{
  const ability={manages:true,maxLength:255};
  expect(boxes.filter(box=>canRename(box,ability)).map(box=>box.id)).toEqual(['work','q3','home']);
  expect(canRename(home,{manages:false,maxLength:255})).toBe(false);
  expect(canRename(home,null)).toBe(false);
  const {myRights:_,...noRights}=home;expect(canRename(noRights,ability)).toBe(false);
 });
 it('validates against the siblings, the delimiter and the byte length',()=>{
  expect(nameProblem('Work',home,boxes,'/',255)).toBe('duplicate');
  expect(nameProblem('work',home,boxes,'/',255)).toBeNull();
  expect(nameProblem('INBOX',home,boxes,'.',255)).toBe('duplicate');
  expect(nameProblem('Work',q3,boxes,'/',255)).toBeNull();
  expect(nameProblem('a.b',home,boxes,'.',255)).toBe('separator');
  expect(nameProblem('a.b',home,boxes,null,255)).toBeNull();
  expect(nameProblem('é'.repeat(128),home,boxes,'/',255)).toBe('long');
  expect(nameProblem('é'.repeat(127),home,boxes,'/',255)).toBeNull();
  expect(nameProblem('\u0007',home,boxes,'/',255)).toBe('reserved');
  expect(nameProblem('',home,boxes,'/',255)).toBe('empty');
 });
 it('reads the ability from the session',()=>{
  expect(renameAbility({accounts:{a:{name:'',isPersonal:true,isReadOnly:false,accountCapabilities:{[CAP_MAIL]:{mailwomanMailboxSet:false}}}}},'a')).toEqual({manages:false,maxLength:255});
  expect(renameAbility({accounts:{a:{name:'',isPersonal:true,isReadOnly:false,accountCapabilities:{[CAP_MAIL]:{maxSizeMailboxName:100}}}}},'a')).toEqual({manages:true,maxLength:100});
 });
 it('maps the server answer and refuses a reply for another account',async()=>{
  const reply=(body:unknown)=>({jmap:async()=>({methodResponses:[['Mailbox/set',body,'rename']],sessionState:'s'})}) as unknown as Parameters<typeof renameFolder>[0];
  const current={current:()=>true};
  expect(await renameFolder(reply({accountId:'a',updated:{x:null}}),'a','x','N',current)).toEqual({ok:true});
  expect(await renameFolder(reply({accountId:'b',updated:{x:null}}),'a','x','N',current)).toEqual({ok:false,problem:'failed'});
  expect(await renameFolder(reply({accountId:'a',notUpdated:{x:{type:'forbidden'}}}),'a','x','N',current)).toEqual({ok:false,problem:'forbidden'});
  expect(await renameFolder(reply({accountId:'a',notUpdated:{x:{type:'notFound'}}}),'a','x','N',current)).toEqual({ok:false,problem:'notFound'});
  expect(await renameFolder(reply({accountId:'a',updated:{x:null}}),'a','x','N',{current:()=>false})).toEqual({ok:false,problem:'failed'});
 });
});
