import {afterEach,describe,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {FolderContextMenu} from './FolderContextMenu.tsx';
import {makeClient} from './appHarness.tsx';
import {createAppState} from '../state/store.ts';
import {AppContext} from '../state/context.ts';
import {accountUiPreferences} from '../api/accountPreferences.ts';
import {type JmapRequest,type JmapResponse,type Mailbox} from '../api/jmap-types.ts';

const rights={mayReadItems:true,mayAddItems:true,mayRemoveItems:true,maySetSeen:true,maySetKeywords:true,mayCreateChild:true,mayRename:true,mayDelete:true,maySubmit:true};
const BOXES:Mailbox[]=[
 {id:'inbox',name:'Inbox',parentId:null,role:'inbox',sortOrder:0,totalEmails:4,unreadEmails:3,myRights:{...rights,mayRename:false,mayDelete:false}},
 {id:'trash',name:'Trash',parentId:null,role:'trash',sortOrder:2,totalEmails:0,unreadEmails:0,myRights:{...rights,mayRename:false,mayDelete:false}},
 {id:'work',name:'Work',parentId:null,role:null,sortOrder:10,totalEmails:5,unreadEmails:1,myRights:rights},
 {id:'q3',name:'Q3',parentId:'work',role:null,sortOrder:10,totalEmails:2,unreadEmails:2,myRights:rights},
 {id:'q4',name:'Q4',parentId:'work',role:null,sortOrder:11,totalEmails:0,unreadEmails:0,myRights:rights},
 {id:'home',name:'Home',parentId:null,role:null,sortOrder:12,totalEmails:0,unreadEmails:0,myRights:rights},
];
const jmap=vi.hoisted(()=>({calls:[] as {name:string;args:Record<string,unknown>}[],boxes:[] as Mailbox[]}));
function answer(request:JmapRequest):JmapResponse|null{
 const [name,args,callId]=request.methodCalls[0]!;jmap.calls.push({name,args:args as Record<string,unknown>});
 if(name==='Mailbox/get')return {methodResponses:[['Mailbox/get',{accountId:'acct1',state:'s',list:jmap.boxes,notFound:[],mailwomanHierarchyDelimiter:'/'},callId]],sessionState:'s'};
 if(name==='Email/query'&&callId==='page')return {methodResponses:[['Email/query',{accountId:'acct1',ids:['u1','u2'],position:0},callId]],sessionState:'s'};
 if(name==='Email/set'&&callId==='set'){const update=(args as {update:Record<string,unknown>}).update;return {methodResponses:[['Email/set',{accountId:'acct1',updated:Object.fromEntries(Object.keys(update).map(key=>[key,null]))},callId]],sessionState:'s'};}
 return null;
}
vi.mock('../api/transport.ts',()=>({createConfiguredClient:()=>({
 session:async()=>({capabilities:{},accounts:{acct1:{name:'T',isPersonal:true,isReadOnly:false,accountCapabilities:{['urn:ietf:params:jmap:mail']:{mailwomanMailboxSet:true,maxSizeMailboxName:255}}}},primaryAccounts:{},username:'me',apiUrl:'',downloadUrl:'',uploadUrl:'',eventSourceUrl:'',state:'s'}),
 jmap:vi.fn(async(request:JmapRequest)=>{const reply=answer(request);if(!reply)throw new Error(`unexpected ${request.methodCalls[0]![0]}`);return reply;}),
})}));
afterEach(()=>{cleanup();localStorage.clear();jmap.calls=[];});
async function open(id:string,selection?:string[]){
 jmap.boxes=BOXES;const base=makeClient({mailboxes:BOXES});
 const client={...base,jmap:vi.fn(async(request:JmapRequest)=>{const calls=jmap.calls.length;const reply=answer(request);if(reply)return reply;jmap.calls.length=calls;return base.jmap(request);})} as typeof base;
 const app=createAppState(client);
 const box=BOXES.find(value=>value.id===id)!;
 render(()=><AppContext.Provider value={app}><FolderContextMenu box={box} {...(selection?{selection:()=>new Set(selection)}:{})} onOpen={()=>{}} onSettings={()=>{}} onShare={()=>{}} canShare={()=>false} onProbe={()=>{}}><button class="sidebar__box">{`${box.name} target`}</button></FolderContextMenu></AppContext.Provider>);
 await app.login({username:'me@example.org',password:'p'});await waitFor(()=>expect(app.mailboxes()).toHaveLength(BOXES.length));
 await waitFor(()=>{fireEvent.contextMenu(screen.getByText(`${box.name} target`));expect(screen.queryByRole('menuitem',{name:'New subfolder'})).not.toBeNull();});
 return app;
}
const names=()=>screen.getAllByRole('menuitem').map(item=>item.textContent);

describe('folder context menu by folder kind',()=>{
 it('system folder: mark all read, no rename or subfolder actions it cannot take, second-level actions disabled with a tooltip',async()=>{
  await open('inbox');
  expect(names()).toEqual(expect.arrayContaining(['Open folder','Mark all as read','Pin','Refresh folder','New subfolder','Folder settings','More']));
  expect(screen.queryByRole('menuitem',{name:'Rename'})).toBeNull();
  fireEvent.click(screen.getByRole('menuitem',{name:'More'}));
  const remove=await screen.findByRole('menuitem',{name:'Delete folder'});
  expect(remove).toHaveAttribute('aria-disabled','true');expect(remove).toHaveAttribute('title','A system folder cannot be changed this way.');
  for(const label of ['Move up','Move down','Move folder to…'])expect(screen.getByRole('menuitem',{name:label})).toHaveAttribute('aria-disabled','true');
  for(const label of ['Copy all items to…','Move all items to…'])expect(screen.getByRole('menuitem',{name:label})).not.toHaveAttribute('aria-disabled','true');
 });
 it('custom folder with subfolders: subfolder, rename, sort submenu and a mark-read choice',async()=>{
  await open('work');
  expect(names()).toEqual(expect.arrayContaining(['Mark all as read','New subfolder','Rename…','Sort subfolders by','More']));
  fireEvent.click(screen.getByRole('menuitem',{name:'Mark all as read'}));
  expect(await screen.findByRole('menuitem',{name:'This folder'})).toBeInTheDocument();expect(screen.getByRole('menuitem',{name:'Include subfolders'})).toBeInTheDocument();
 });
 it('a folder without subfolders marks read directly and offers no sort',async()=>{
  await open('home');
  expect(screen.queryByRole('menuitem',{name:'Sort subfolders by'})).toBeNull();
  expect(screen.getByRole('menuitem',{name:'Mark all as read'})).toHaveAttribute('aria-disabled','true');
 });
 it('second level: delete (with subfolders for a parent), copy and move all, move to, move up and down by position',async()=>{
  await open('work');fireEvent.click(screen.getByRole('menuitem',{name:'More'}));
  expect(await screen.findByRole('menuitem',{name:'Delete folder and subfolders'})).not.toHaveAttribute('aria-disabled','true');
  expect(screen.getByRole('menuitem',{name:'Copy all items to…'})).toBeInTheDocument();expect(screen.getByRole('menuitem',{name:'Move all items to…'})).toBeInTheDocument();
  expect(screen.getByRole('menuitem',{name:'Move folder to…'})).not.toHaveAttribute('aria-disabled','true');
  expect(screen.getByRole('menuitem',{name:'Move up'})).toHaveAttribute('aria-disabled','true');
  expect(screen.getByRole('menuitem',{name:'Move down'})).not.toHaveAttribute('aria-disabled','true');
 });
 it('Move down saves a custom order for the siblings and switches that parent to custom sorting',async()=>{
  await open('work');fireEvent.click(screen.getByRole('menuitem',{name:'More'}));
  fireEvent.click(await screen.findByRole('menuitem',{name:'Move down'}));
  await waitFor(()=>expect(accountUiPreferences().folderOrder).toBeDefined());
  const order=accountUiPreferences().folderOrder!;expect(order.indexOf('home')).toBeLessThan(order.indexOf('work'));
  expect(accountUiPreferences().folderSort?.['_top']).toEqual({by:'custom',dir:'asc'});
 });
 it('Sort subfolders by saves the choice per parent and marks it current',async()=>{
  await open('work');fireEvent.click(screen.getByRole('menuitem',{name:'Sort subfolders by'}));
  fireEvent.click(await screen.findByRole('menuitemradio',{name:'Unread count'}));
  await waitFor(()=>expect(accountUiPreferences().folderSort?.['work']).toEqual({by:'unread',dir:'asc'}));
 });
 it('Mark all as read with subfolders marks every unread message of the folder and its subfolders',async()=>{
  const app=await open('work');const clear=vi.spyOn(app,'clearFolderUnread');
  fireEvent.click(screen.getByRole('menuitem',{name:'Mark all as read'}));fireEvent.click(await screen.findByRole('menuitem',{name:'Include subfolders'}));
  await waitFor(()=>expect(jmap.calls.some(call=>call.name==='Email/set')).toBe(true));
  expect(clear).toHaveBeenCalledWith(['work','q3','q4']);
  const queried=jmap.calls.filter(call=>call.name==='Email/query').map(call=>(call.args['filter'] as {inMailbox:string}).inMailbox);
  expect(queried).toEqual(['work','q3','q4']);
  const set=jmap.calls.find(call=>call.name==='Email/set')!.args['update'] as Record<string,unknown>;expect(set['u1']).toEqual({'keywords/$seen':true});
 });
 it('Delete with subfolders asks for confirmation naming the folder and message counts before deleting anything',async()=>{
  await open('work');fireEvent.click(screen.getByRole('menuitem',{name:'More'}));fireEvent.click(await screen.findByRole('menuitem',{name:'Delete folder and subfolders'}));
  const dialog=await screen.findByRole('alertdialog');expect(dialog.textContent).toContain('3 folders');expect(dialog.textContent).toContain('7 messages');
  expect(jmap.calls.some(call=>call.name==='Mailbox/set')).toBe(false);
 });
 it('marquee selection: a menu opened on one of several selected folders marks exactly those read',async()=>{
  const app=await open('work',['work','home','q3']);const clear=vi.spyOn(app,'clearFolderUnread');
  const row=await screen.findByRole('menuitem',{name:'Mark 3 selected folders as read'});
  expect(row).not.toHaveAttribute('aria-disabled','true');
  fireEvent.click(row);
  await waitFor(()=>expect(clear).toHaveBeenCalled());
  expect([...clear.mock.calls[0]![0]].sort()).toEqual(['home','q3','work']);
 });
 it('marquee selection: no bulk entry for a single selected folder or for a folder outside the selection',async()=>{
  await open('work',['work']);
  expect(screen.queryByRole('menuitem',{name:/selected folders as read/})).toBeNull();
  cleanup();
  await open('work',['home','q3']);
  expect(screen.queryByRole('menuitem',{name:/selected folders as read/})).toBeNull();
 });
});
