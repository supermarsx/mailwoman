import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {makeClient} from './appHarness.tsx';
import {createAppState} from '../state/store.ts';
import {AppContext} from '../state/context.ts';
import {CAP_MAIL,type JmapRequest,type JmapResponse,type Mailbox} from '../api/jmap-types.ts';
import {FolderContextMenu} from './FolderContextMenu.tsx';
import {suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {canDeleteFolder,deleteFolder} from '../api/deleteFolder.ts';


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
  const target=(call['destroy'] as string[])[0]!;
  if(server.refuse)return {methodResponses:[['Mailbox/set',{accountId:'acct1',destroyed:[],notDestroyed:{[target]:{type:server.refuse}}},id]],sessionState:'s'};
  server.boxes=server.boxes.filter(box=>box.id!==target);
  return {methodResponses:[['Mailbox/set',{accountId:'acct1',destroyed:[target],notDestroyed:null},id]],sessionState:'s'};
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
async function openDelete(label='Home'){
 await waitFor(()=>{fireEvent.contextMenu(screen.getByText(`${label} target`));expect(screen.getByRole('menuitem',{name:'Delete folder'})).toBeInTheDocument();});
 fireEvent.click(screen.getByRole('menuitem',{name:'Delete folder'}));return screen.findByRole('dialog');
}
describe('delete folder confirmation',()=>{
 it('opens from the real context menu, names the folder and focuses Cancel without mutating',async()=>{
  await sidebar();await openDelete();expect(screen.getByRole('heading').textContent).toContain('Home');await waitFor(()=>expect(screen.getByRole('button',{name:'Cancel'})).toHaveFocus());fireEvent.click(screen.getByRole('button',{name:'Cancel'}));expect(screen.queryByRole('dialog')).toBeNull();expect(server.sets).toEqual([]);
 });
 it('deletes an empty folder with explicit false and refreshes the list',async()=>{
  const app=await sidebar();await openDelete();fireEvent.click(screen.getByRole('button',{name:'Delete folder'}));await waitFor(()=>expect(screen.queryByRole('dialog')).toBeNull());expect(server.sets).toEqual([{accountId:'acct1',destroy:['home'],onDestroyRemoveEmails:false}]);expect(app.mailboxes().some(box=>box.id==='home')).toBe(false);
 });
 it('reports confirmed deletion before refreshing unmounts the folder row',async()=>{
  const app=await sidebar();await openDelete();const toast=vi.spyOn(app,'showToast');
  vi.spyOn(app,'refreshFolders').mockImplementationOnce(async()=>{expect(toast).toHaveBeenCalledWith('success',expect.stringContaining('Home'));cleanup();});
  fireEvent.click(screen.getByRole('button',{name:'Delete folder'}));await waitFor(()=>expect(toast).toHaveBeenCalledWith('success',expect.stringContaining('Home')));expect(server.sets).toHaveLength(1);
 });
 it('requires opt-in for messages and never automatically consents after a stale empty count',async()=>{
  await sidebar();server.refuse='mailboxHasEmail';await openDelete();fireEvent.click(screen.getByRole('button',{name:'Delete folder'}));await screen.findByRole('alert');expect(server.sets[0]?.['onDestroyRemoveEmails']).toBe(false);expect(screen.getByRole('button',{name:'Delete folder'})).toBeDisabled();server.refuse=null;fireEvent.click(screen.getByRole('checkbox'));fireEvent.click(screen.getByRole('button',{name:'Delete folder'}));await waitFor(()=>expect(screen.queryByRole('dialog')).toBeNull());expect(server.sets[1]?.['onDestroyRemoveEmails']).toBe(true);
 });
 it('requires consent before deleting a known nonempty folder',async()=>{
  await sidebar();await openDelete('Q3');expect(screen.getByRole('button',{name:'Delete folder'})).toBeDisabled();expect(server.sets).toEqual([]);fireEvent.click(screen.getByRole('checkbox'));fireEvent.click(screen.getByRole('button',{name:'Delete folder'}));await waitFor(()=>expect(server.sets).toHaveLength(1));expect(server.sets[0]?.['onDestroyRemoveEmails']).toBe(true);
 });
 it('does not offer special folders, parents, saved searches or folders without rights',async()=>{
  await sidebar();await openDelete();fireEvent.click(screen.getByRole('button',{name:'Cancel'}));
  for(const name of ['Inbox','Trash','Work','Team','Saved']){fireEvent.contextMenu(screen.getByText(`${name} target`));expect(screen.queryByRole('menuitem',{name:'Delete folder'})).toBeNull();fireEvent.keyDown(document.activeElement??document.body,{key:'Escape'});}
  expect(canDeleteFolder(server.boxes.find(box=>box.id==='home')!,server.boxes,{manages:false,maxLength:255})).toBe(false);
 });
 it('closes confirmation when account authority changes, without a deletion',async()=>{
  await sidebar();await openDelete();suspendAccountPreferences();expect(screen.queryByRole('dialog')).toBeNull();expect(server.sets).toEqual([]);
 });
 it('refuses API mutation and stale results across account authority',async()=>{
  const jmap=vi.fn(async()=>({methodResponses:[['Mailbox/set',{accountId:'acct1',destroyed:['home']},'delete-folder']],sessionState:'s'} as JmapResponse));
  expect((await deleteFolder({jmap},'acct1','home',false,{current:()=>false})).ok).toBe(false);expect(jmap).not.toHaveBeenCalled();let current=true;jmap.mockImplementationOnce(async()=>{current=false;return {methodResponses:[['Mailbox/set',{accountId:'acct1',destroyed:['home']},'delete-folder']],sessionState:'s'};});expect((await deleteFolder({jmap},'acct1','home',false,{current:()=>current})).ok).toBe(false);
 });
});
