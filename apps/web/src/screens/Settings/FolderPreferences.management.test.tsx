import {afterEach, expect, it, vi} from 'vitest';
import {cleanup, createEvent, fireEvent, render, screen, waitFor, within} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {ApiError} from '../../api/client.ts';
import {toast} from '../../components/toast/toast.ts';
import {FolderPreferences} from './FolderPreferences.tsx';
import type {PreferenceFolder} from './folderPreferenceModel.ts';
import type {AccountUiPreferences} from '../../api/accountPreferences.ts';
import {activateAccountPreferences, suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import {AppContext} from '../../state/context.ts';
import type {AppState} from '../../state/store.ts';
import type {JmapRequest} from '../../api/jmap-types.ts';
import {clearSettings,loadSettings} from '../../api/settings.ts';
import * as hostModule from '../../components/properties/host.tsx';

const service=vi.hoisted(()=>({manages:true,jmap:vi.fn()}));
vi.mock('../../api/transport.ts',()=>({createConfiguredClient:()=>({jmap:service.jmap,session:async()=>({accounts:{}})})}));
vi.mock('../../components/RenameFolder.tsx',()=>({useRenameContext:()=>()=>({manages:service.manages,delimiter:'/',maxLength:255})}));
vi.mock('../../components/QuotaMeter.tsx',()=>({QuotaDetails:()=>null}));
vi.mock('./FolderRoles.tsx',()=>({FolderRoles:()=>null}));
vi.mock('./FocusedInboxPreference.tsx',()=>({FocusedInboxPreference:()=>null}));
vi.mock('./FocusedSenders.tsx',()=>({FocusedSenders:()=>null}));
vi.mock('./FolderCountPreferences.tsx',()=>({FolderCountPreferences:()=>null}));
afterEach(()=>{cleanup();clearSettings();suspendAccountPreferences();service.manages=true;service.jmap.mockReset();});

function mount(initialFolders:PreferenceFolder[],initialPrefs:AccountUiPreferences={},withApp=false){
 activateAccountPreferences('acct1');
 const [folders,setFolders]=createSignal(initialFolders),[prefs,setPrefs]=createSignal(initialPrefs);
 const update=vi.fn((patch:AccountUiPreferences)=>setPrefs(previous=>({...previous,...patch})));
 const refresh=vi.fn(async()=>{}),toast=vi.fn();
 const app={accountId:()=>'acct1',mailboxes:folders,refreshFolders:refresh,showToast:toast} as unknown as AppState;
 const content=()=> <FolderPreferences folders={folders} preferences={prefs} {...(withApp?{}:{onChange:update})}/>;
 render(()=>withApp?<AppContext.Provider value={app}>{content()}</AppContext.Provider>:content());
 return {folders,setFolders,prefs,update,refresh,toast};
}
const selected=(name:string)=>screen.getByRole('checkbox',{name:`Select folder: ${name}`});
const click=(name:string)=>fireEvent.click(screen.getByRole('button',{name}));
const search=(value:string)=>fireEvent.input(screen.getByLabelText('Search folders'),{target:{value}});
const drag=(name:string)=>screen.getByRole('button',{name:`Drag ${name} to reorder`});
const transfer=()=>({setData:vi.fn(),types:['application/x-mailwoman-folder']});
/** jsdom drops clientX/clientY from drag events: set them on the created event. */
function fire(kind:'dragOver',row:Element,clientY:number,clientX:number):void{
 const event=createEvent[kind](row,{dataTransfer:transfer()});
 Object.defineProperty(event,'clientY',{value:clientY});Object.defineProperty(event,'clientX',{value:clientX});
 fireEvent(row,event);
}
/** Drag `source` and release over `target`: zone 'before' (top quarter), 'inside' (middle) or 'after' (bottom quarter). */
function drop(source:string,target:string,zone:'before'|'inside'|'after'='before',x=0):void{
 const row=drag(target).closest('tr')!;row.getBoundingClientRect=()=>({top:0,left:0,height:40,width:400,bottom:40,right:400,x:0,y:0,toJSON:()=>({})});
 fireEvent.dragStart(drag(source),{dataTransfer:transfer()});
 fire('dragOver',row,zone==='before'?2:zone==='inside'?20:38,x);
 fireEvent.drop(row,{dataTransfer:transfer()});
}

it('selects visible search matches separately from all folders and keeps hidden selections manageable',async()=>{
 mount([{id:'i',name:'Inbox',role:'inbox'},{id:'a',name:'Project Alpha'},{id:'b',name:'Archive'},{id:'c',name:'Project Beta'}]);
 search('Project');
 expect(screen.queryByRole('checkbox',{name:'Select folder: Inbox'})).toBeNull();
 click('Select visible');
 expect(selected('Project Alpha')).toBeChecked();expect(selected('Project Beta')).toBeChecked();
 search('');
 expect(selected('Inbox')).not.toBeChecked();expect(selected('Archive')).not.toBeChecked();
 search('Project');click('Select all folders');search('');
 expect(selected('Inbox')).toBeChecked();expect(selected('Archive')).toBeChecked();
 click('Clear selection');
 expect(selected('Project Alpha')).not.toBeChecked();expect(selected('Inbox')).not.toBeChecked();
});

it('bulk hides selected folders while protecting Inbox and preserving unrelated hidden preferences',()=>{
 const {update,prefs}=mount([{id:'i',name:'Inbox',role:'inbox'},{id:'a',name:'Projects'}],{folderHidden:['other-account-folder']});
 click('Select all folders');click('Hide selected');
 expect(update).toHaveBeenLastCalledWith({folderHidden:['other-account-folder','a']});
 expect(prefs().folderHidden).not.toContain('i');
});

it('bulk showing a descendant restores every hidden ancestor and leaves unrelated folders hidden',()=>{
 const {update}=mount([{id:'a',name:'Projects'},{id:'b',name:'Year',parentId:'a'},{id:'c',name:'Reports',parentId:'b'}],{folderHidden:['a','b','c','unrelated']});
 fireEvent.click(selected('Reports'));click('Show selected');
 expect(update).toHaveBeenLastCalledWith({folderHidden:['unrelated']});
});

it('bulk pins and unpins without discarding unrelated or currently unknown folder IDs',()=>{
 const {update,prefs}=mount([{id:'a',name:'Projects'},{id:'b',name:'Receipts'}],{folderPinned:['unknown','b']});
 fireEvent.click(selected('Projects'));click('Pin selected');
 expect(prefs().folderPinned).toEqual(['unknown','b','a']);
 click('Unpin selected');expect(update).toHaveBeenLastCalledWith({folderPinned:['unknown','b']});
});

it('reorders through the full folder order and turns dragging off while a search filters the list',()=>{
 const {update}=mount([{id:'a',name:'Match A'},{id:'middle',name:'Other'},{id:'b',name:'Match B'},{id:'p',name:'Pinned'}],{folderOrder:['a','middle','b','p']});
 search('Match');drop('Match A','Match B');
 expect(update).not.toHaveBeenCalled();
 search('');drop('Match A','Match B');
 expect(update).toHaveBeenLastCalledWith({folderOrder:['middle','a','b','p']});
});

it('nests a folder by dropping on the middle of a row and keeps it a view-only arrangement without a server',()=>{
 const {update}=mount([{id:'a',name:'Projects'},{id:'b',name:'Receipts'}],{folderNestMode:'local'});
 drop('Receipts','Projects','inside');
 expect(update).toHaveBeenLastCalledWith({folderOrder:['a','b'],folderParent:{b:'a'}});
});

it('refuses nesting a folder into its own subfolder and into a locked place',()=>{
 const {update}=mount([{id:'a',name:'Projects'},{id:'b',name:'Year',parentId:'a'}]);
 drop('Projects','Year','inside');
 expect(update).not.toHaveBeenCalled();
});

it('shows the arranged-only badge for a folder whose visual parent differs from the server',()=>{
 mount([{id:'a',name:'Projects'},{id:'b',name:'Receipts'}],{folderParent:{b:'a'}});
 expect(screen.getAllByTestId('arranged-only')).toHaveLength(1);
});

it('moves selected folders together, keeping their order, with the keyboard Move to dialog',async()=>{
 const {update}=mount([{id:'a',name:'Projects'},{id:'b',name:'Receipts'},{id:'c',name:'Archive'}],{folderNestMode:'local'});
 fireEvent.click(selected('Receipts'));fireEvent.click(selected('Archive'));
 click('Move selected to…');
 fireEvent.change(await screen.findByRole('listbox'),{target:{value:'a'}});
 click('Move');
 expect(update).toHaveBeenLastCalledWith({folderOrder:['a','b','c'],folderParent:{b:'a',c:'a'}});
});

it('cancels stale drags and prunes selected folders removed by a refresh',()=>{
 const {setFolders,update}=mount([{id:'a',name:'Projects'},{id:'b',name:'Receipts'}]);
 fireEvent.click(selected('Projects'));
 fireEvent.dragStart(drag('Projects'),{dataTransfer:{setData:vi.fn()}});
 setFolders([{id:'b',name:'Receipts'}]);
 fireEvent.drop(drag('Receipts').closest('tr')!);
 expect(update).not.toHaveBeenCalled();
 setFolders([{id:'a',name:'Projects'},{id:'b',name:'Receipts'}]);
 expect(selected('Projects')).not.toBeChecked();
});

it('clears selected rows and drag authority when the authenticated account changes',()=>{
 const {update}=mount([{id:'a',name:'Projects'},{id:'b',name:'Receipts'}]);
 fireEvent.click(selected('Projects'));fireEvent.dragStart(drag('Projects'),{dataTransfer:{setData:vi.fn()}});
 suspendAccountPreferences();activateAccountPreferences('acct1');
 expect(selected('Projects')).not.toBeChecked();
 fireEvent.drop(drag('Receipts').closest('tr')!);expect(update).not.toHaveBeenCalled();
});

it('creates folders from settings through the existing account-bound dialog and refreshes the list',async()=>{
 const {refresh,toast}=mount([{id:'i',name:'Inbox',role:'inbox'}],{},true);
 expect(screen.getByRole('button',{name:'Create folder'}).closest('header')?.closest('section')?.id).toBe('folders-card-list');
 expect(screen.getByLabelText('Hide empty folders in the sidebar')).not.toBeVisible();
 service.jmap.mockImplementation(async(request:JmapRequest)=>({methodResponses:[['Mailbox/set',{accountId:'acct1',created:{folder:{id:'new'}}},request.methodCalls[0]![2]]],sessionState:'s'}));
 click('Create folder');const dialog=await screen.findByRole('alertdialog',{name:'Create folder'});
 fireEvent.input(screen.getByLabelText('Folder name'),{target:{value:'Reports'}});
 fireEvent.click(within(dialog).getByRole('button',{name:'Create folder'}));
 await waitFor(()=>expect(refresh).toHaveBeenCalledOnce());
 expect(service.jmap).toHaveBeenCalledWith(expect.objectContaining({methodCalls:[['Mailbox/set',{accountId:'acct1',create:{folder:{name:'Reports',parentId:null,isSubscribed:true}}},'create']]}),expect.objectContaining({signal:expect.any(AbortSignal)}));
 await waitFor(()=>expect(screen.queryByRole('alertdialog')).toBeNull());expect(toast).toHaveBeenCalledWith('success',expect.any(String));
});

it('does not expose a usable create control when the account cannot manage folders',()=>{
 service.manages=false;mount([{id:'i',name:'Inbox',role:'inbox'}],{},true);
 const button=screen.queryByRole('button',{name:'Create folder'});
 if(button)expect(button).toBeDisabled();
 expect(service.jmap).not.toHaveBeenCalled();
});

const applySettings=(values:Record<string,unknown>)=>loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:Object.fromEntries(Object.entries(values).map(([key,value])=>[key,{value,source:'global',locked:true}]))}),{headers:{'content-type':'application/json'}})});

it('shows a lock and refuses drags and moves when the administrator fixes the order',async()=>{
 await applySettings({'folders.order.locked':true});
 const {update}=mount([{id:'a',name:'Projects'},{id:'b',name:'Receipts'}]);
 expect(screen.getAllByTestId('folder-lock')).toHaveLength(2);
 expect(drag('Receipts')).toBeDisabled();
 drop('Receipts','Projects');
 expect(update).not.toHaveBeenCalled();
 expect(screen.getByRole('button',{name:'Move Receipts up'})).toBeDisabled();
});

it('force-pins the administrator list: those rows are locked, pinned and cannot be unpinned',async()=>{
 await applySettings({'folders.pinned.admin':['role:inbox','Receipts']});
 const {update}=mount([{id:'i',name:'Inbox',role:'inbox'},{id:'a',name:'Projects'},{id:'b',name:'Receipts'}]);
 expect(screen.getAllByTestId('folder-lock')).toHaveLength(2);
 expect(screen.getByRole('button',{name:'Pin Receipts'})).toBeDisabled();
 drop('Receipts','Projects');
 expect(update).not.toHaveBeenCalled();
 expect(screen.getByRole('button',{name:'Pin Projects'})).not.toBeDisabled();
});

it('marks starred rows with a decorative star, separately from pins',()=>{
 mount([{id:'a',name:'Projects'},{id:'b',name:'Receipts'}],{folderStarred:['b'],folderPinned:['a']});
 const rows=screen.getAllByRole('row').slice(1);
 expect(rows[0]!.querySelector('.sidebar__box-star')).toBeNull();
 expect(rows[1]!.querySelector('.sidebar__box-star')).not.toBeNull();
 expect(within(rows[1]!).getByRole('button',{name:'Star Receipts'})).toHaveAttribute('aria-pressed','true');
 expect(within(rows[0]!).getByRole('button',{name:'Pin Projects'})).toHaveAttribute('aria-pressed','true');
});

it('resets the arrangement to the server order',()=>{
 const {update}=mount([{id:'a',name:'Projects'},{id:'b',name:'Receipts'}],{folderOrder:['b','a'],folderParent:{a:'b'}});
 fireEvent.click(screen.getByRole('tab',{name:'Arrange folders'}));
 click('Reset order to default');
 expect(update).toHaveBeenLastCalledWith({folderOrder:[],folderParent:{}});
});

const deletable=(id:string,name:string,extra:Record<string,unknown>={}):PreferenceFolder=>({id,name,role:null,parentId:null,totalEmails:0,unreadEmails:0,sortOrder:0,myRights:{mayDelete:true,mayRename:true,mayCreateChild:true},...extra} as unknown as PreferenceFolder);

it('bulk delete names the folders and messages affected, skips system folders, and deletes each root with its subfolders on confirm',async()=>{
 mount([{id:'i',name:'Inbox',role:'inbox'} as PreferenceFolder,deletable('a','Alpha',{totalEmails:7}),deletable('a1','Alpha child',{parentId:'a',totalEmails:2}),deletable('b','Beta',{totalEmails:1})],{},true);
 service.jmap.mockImplementation(async(request:JmapRequest)=>{
  const call=request.methodCalls[0]!,args=call[1] as {destroy?:string[]};
  return {methodResponses:[['Mailbox/set',{accountId:'acct1',destroyed:args.destroy??[]},call[2]]]};
 });
 click('Select all folders');
 fireEvent.click(screen.getByTestId('bulk-delete'));
 const dialog=await screen.findByRole('alertdialog');
 expect(dialog).toHaveTextContent('3 folders (with their subfolders) and the 10 messages in them will be deleted');
 expect(within(dialog).getByTestId('bulk-delete-skipped')).toHaveTextContent('Inbox');
 expect(service.jmap).not.toHaveBeenCalled();
 fireEvent.click(within(dialog).getByRole('button',{name:'Delete 2 folders'}));
 await waitFor(()=>expect(service.jmap.mock.calls.map(([request])=>(request as JmapRequest).methodCalls[0]![1])).toEqual([
  expect.objectContaining({destroy:['a1']}),expect.objectContaining({destroy:['a']}),expect.objectContaining({destroy:['b']}),
 ]));
});

it('bulk delete is unavailable when nothing selected can be deleted, and never offered to an account that cannot manage folders',async()=>{
 mount([{id:'i',name:'Inbox',role:'inbox'} as PreferenceFolder],{},true);
 fireEvent.click(selected('Inbox'));
 expect(screen.getByTestId('bulk-delete')).toBeDisabled();
 cleanup();service.manages=false;
 mount([deletable('a','Alpha')],{},true);
 fireEvent.click(selected('Alpha'));
 expect(screen.queryByTestId('bulk-delete')).toBeNull();expect(screen.queryByTestId('bulk-subscribe')).toBeNull();
});

it('bulk subscribe and unsubscribe send one Mailbox/set with isSubscribed for every selected folder and report partial failures',async()=>{
 const {refresh}=mount([deletable('a','Alpha'),deletable('b','Beta')],{},true);
 service.jmap.mockImplementation(async(request:JmapRequest)=>{
  const call=request.methodCalls[0]!,update=(call[1] as {update:Record<string,unknown>}).update;
  return {methodResponses:[['Mailbox/set',{accountId:'acct1',updated:{a:null},notUpdated:'b' in update?{b:{type:'forbidden'}}:{}},call[2]]]};
 });
 click('Select all folders');
 fireEvent.click(screen.getByTestId('bulk-unsubscribe'));
 await waitFor(()=>expect(service.jmap).toHaveBeenCalledTimes(1));
 expect((service.jmap.mock.calls[0]![0] as JmapRequest).methodCalls[0]![1]).toMatchObject({update:{a:{isSubscribed:false},b:{isSubscribed:false}}});
 await waitFor(()=>expect(refresh).toHaveBeenCalled());
});

it('shift+click on row checkboxes selects the range in the order shown, ctrl-free click toggles, and Select all user folders skips system folders',async()=>{
 mount([{id:'i',name:'Inbox',role:'inbox'} as PreferenceFolder,deletable('a','Alpha'),deletable('b','Beta'),deletable('c','Gamma'),deletable('d','Delta')],{},true);
 fireEvent.click(selected('Alpha'));
 fireEvent.click(selected('Gamma'),{shiftKey:true});
 expect(selected('Alpha')).toBeChecked();expect(selected('Beta')).toBeChecked();expect(selected('Gamma')).toBeChecked();expect(selected('Delta')).not.toBeChecked();
 fireEvent.click(selected('Beta'),{shiftKey:true});
 expect(selected('Gamma')).not.toBeChecked();expect(selected('Beta')).toBeChecked();
 fireEvent.click(screen.getByTestId('select-user-folders'));
 expect(selected('Inbox')).not.toBeChecked();expect(selected('Delta')).toBeChecked();
});

it('bulk empty counts every folder first, says what goes to Trash, empties them in batches and lists a folder that failed',async()=>{
 const trash={id:'t',name:'Trash',role:'trash',parentId:null,totalEmails:0,unreadEmails:0,sortOrder:0,myRights:{mayDelete:false,mayRemoveItems:true}} as unknown as PreferenceFolder;
 mount([trash,deletable('a','Alpha',{totalEmails:2}),deletable('b','Beta',{totalEmails:1}),deletable('e','Empty')],{},true);
 const left:Record<string,string[]>={a:['a1','a2'],b:['b1']};
 service.jmap.mockImplementation(async(request:JmapRequest)=>{
  const calls=request.methodCalls.map(([method,args,id])=>{
   const a=args as {filter?:{inMailbox?:string};calculateTotal?:boolean;limit?:number;update?:Record<string,unknown>};
   const box=a.filter?.inMailbox??'';
   if(method==='Email/query')return [method,{accountId:'acct1',ids:a.calculateTotal?[]:(left[box]??[]).slice(0,a.limit),total:(left[box]??[]).length},id];
   const ids=Object.keys(a.update??{});
   // Beta's move is refused by the server.
   if(ids.includes('b1'))return [method,{accountId:'acct1',updated:null,notUpdated:{b1:{type:'forbidden'}}},id];
   for(const m of Object.keys(left))left[m]=left[m]!.filter(x=>!ids.includes(x));
   return [method,{accountId:'acct1',updated:Object.fromEntries(ids.map(x=>[x,null]))},id];
  });
  // Email/set precedes its Email/query count in the request: evaluate in order, so the count sees the move.
  return {methodResponses:calls};
 });
 click('Select all user folders');
 await waitFor(()=>expect(screen.getByTestId('bulk-empty')).not.toBeDisabled());
 fireEvent.click(screen.getByTestId('bulk-empty'));
 const dialog=await screen.findByRole('alertdialog');
 await waitFor(()=>expect(within(dialog).getByTestId('bulk-empty-a')).toHaveTextContent('2 messages move to Trash'));
 expect(within(dialog).getByTestId('bulk-empty-e')).toHaveTextContent('already empty, skipped');
 expect(dialog).toHaveTextContent('3 messages in 2 folders will be removed');
 fireEvent.click(within(dialog).getByRole('button',{name:'Empty the folders'}));
 const failed=await within(dialog).findByTestId('bulk-empty-failed');
 expect(failed).toHaveTextContent('Beta');
 expect(left['a']).toEqual([]);
},20000);

it('bulk delete can leave subfolders alone: a folder that still has an unselected subfolder is kept',async()=>{
 mount([deletable('a','Alpha'),deletable('a1','Alpha child',{parentId:'a'}),deletable('b','Beta')],{},true);
 service.jmap.mockImplementation(async(request:JmapRequest)=>{
  const call=request.methodCalls[0]!,args=call[1] as {destroy?:string[]};
  return {methodResponses:[['Mailbox/set',{accountId:'acct1',destroyed:args.destroy??[]},call[2]]]};
 });
 fireEvent.click(selected('Alpha'));fireEvent.click(selected('Beta'));
 fireEvent.click(screen.getByTestId('bulk-delete'));
 const dialog=await screen.findByRole('alertdialog');
 fireEvent.click(within(dialog).getByTestId('bulk-delete-subfolders'));
 expect(within(dialog).getByTestId('bulk-delete-skipped')).toHaveTextContent('Alpha');
 fireEvent.click(within(dialog).getByRole('button',{name:'Delete 1 folder'}));
 await waitFor(()=>expect(service.jmap.mock.calls.map(([r])=>(r as JmapRequest).methodCalls[0]![1])).toEqual([expect.objectContaining({destroy:['b']})]));
});

it('Alt+Enter on a folder row opens its Properties dialog',()=>{
 const open=vi.spyOn(hostModule,'openFolderProperties').mockReturnValue(true);
 mount([{id:'i',name:'Inbox',role:'inbox'},{id:'a',name:'Projects'}],{},true);
 const row=document.querySelector('tr[data-folder-id="a"]')!;
 fireEvent.keyDown(row,{key:'Enter',altKey:true});
 expect(open).toHaveBeenCalledOnce();
 expect(open.mock.calls[0]![0]).toMatchObject({id:'a'});
 fireEvent.keyDown(row,{key:'Enter'});
 expect(open).toHaveBeenCalledOnce();
 open.mockRestore();
});

function emptyMock(left:Record<string,string[]>,sets:Array<Record<string,unknown>>){
 service.jmap.mockImplementation(async(request:JmapRequest)=>({methodResponses:request.methodCalls.map(([method,args,id])=>{
  const a=args as {filter?:{inMailbox?:string};calculateTotal?:boolean;limit?:number;position?:number;update?:Record<string,Record<string,unknown>>};
  const box=a.filter?.inMailbox??'';
  if(method==='Email/query')return [method,{accountId:'acct1',ids:a.calculateTotal?[]:(left[box]??[]).slice(a.position??0,(a.position??0)+(a.limit??100)),total:(left[box]??[]).length},id];
  const ids=Object.keys(a.update??{});sets.push(a.update??{});
  for(const key of Object.keys(left))left[key]=left[key]!.filter(x=>!ids.includes(x));
  return [method,{accountId:'acct1',updated:Object.fromEntries(ids.map(x=>[x,null]))},id];
 })}));
}
const trashBox=()=>({id:'t',name:'Trash',role:'trash',parentId:null,totalEmails:0,unreadEmails:0,sortOrder:0,myRights:{mayDelete:false,mayRemoveItems:true}}) as unknown as PreferenceFolder;

it('bulk empty to Trash is undoable through the global undo history and says how many go to Trash',async()=>{
 const {clearUndoHistory,undoEntries,undoLast}=await import('../../state/undoHistory.ts');
 clearUndoHistory();
 mount([trashBox(),deletable('a','Alpha',{totalEmails:2}),deletable('b','Beta',{totalEmails:1})],{},true);
 const left:Record<string,string[]>={a:['a1','a2'],b:['b1']},sets:Array<Record<string,unknown>>=[];
 emptyMock(left,sets);
 click('Select all user folders');
 await waitFor(()=>expect(screen.getByTestId('bulk-empty')).not.toBeDisabled());
 fireEvent.click(screen.getByTestId('bulk-empty'));
 const dialog=await screen.findByRole('alertdialog');
 await waitFor(()=>expect(dialog).toHaveTextContent('3 messages in 2 folders will be removed: 3 move to Trash and can be undone, 0 are deleted for good'));
 fireEvent.click(within(dialog).getByRole('button',{name:'Empty the folders'}));
 await waitFor(()=>expect(undoEntries().length).toBe(1));
 expect(undoEntries()[0]!.label).toBe('Emptied 2 folders (3 messages)');
 sets.length=0;
 await undoLast();
 const restored=Object.assign({},...sets) as Record<string,Record<string,unknown>>;
 expect(Object.keys(restored).sort()).toEqual(['a1','a2','b1']);
 expect(restored['a1']).toEqual({'mailboxIds/t':null,'mailboxIds/a':true});
 expect(restored['b1']).toEqual({'mailboxIds/t':null,'mailboxIds/b':true});
 clearUndoHistory();
},20000);

it('bulk empty skips a folder that would be deleted for good when the administrator turned permanent delete off',async()=>{
 await loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{'properties.message.permanentDelete':{value:'deny',source:'global',locked:true}}}),{headers:{'Content-Type':'application/json'}})});
 mount([trashBox(),deletable('a','Alpha',{totalEmails:1}),{id:'j',name:'Junk',role:'junk',parentId:null,totalEmails:4,unreadEmails:0,sortOrder:1,myRights:{mayDelete:false,mayRemoveItems:true}} as unknown as PreferenceFolder],{},true);
 emptyMock({a:['a1'],j:['j1','j2','j3','j4']},[]);
 fireEvent.click(selected('Alpha'));fireEvent.click(selected('Junk'));
 await waitFor(()=>expect(screen.getByTestId('bulk-empty')).not.toBeDisabled());
 fireEvent.click(screen.getByTestId('bulk-empty'));
 const dialog=await screen.findByRole('alertdialog');
 await waitFor(()=>expect(within(dialog).getByTestId('bulk-empty-j')).toHaveTextContent('administrator has turned off'));
 expect(within(dialog).getByTestId('bulk-empty-a')).toHaveTextContent('1 messages move to Trash');
 expect(dialog).toHaveTextContent('1 messages in 1 folders will be removed');
},20000);

it('bulk delete names the folders the server refused because the account is under a legal hold, and deletes nothing',async()=>{
 mount([deletable('a','Alpha'),deletable('b','Beta')],{},true);
 service.jmap.mockRejectedValue(new ApiError(403,'legal hold',{code:'legalHold'}));
 fireEvent.click(selected('Alpha'));fireEvent.click(selected('Beta'));
 fireEvent.click(screen.getByTestId('bulk-delete'));
 const dialog=await screen.findByRole('alertdialog');
 fireEvent.click(within(dialog).getByRole('button',{name:'Delete 2 folders'}));
 await waitFor(()=>expect(dialog).toHaveTextContent(/Not deleted because the account is under a legal hold \(2 folders\)/));
 expect(dialog).toHaveTextContent('Alpha');expect(dialog).toHaveTextContent('Beta');
 expect(service.jmap).toHaveBeenCalledTimes(2);
},20000);

it('bulk delete reports a partial refusal: what went and what the restriction held back',async()=>{
 const failure=vi.spyOn(toast,'error');
 mount([deletable('a','Alpha'),deletable('b','Beta')],{},true);
 service.jmap.mockImplementation(async(request:JmapRequest)=>{
  const call=request.methodCalls[0]!,args=call[1] as {destroy?:string[]};
  if(args.destroy?.includes('b'))throw new ApiError(403,'restricted',{code:'processingRestricted'});
  return {methodResponses:[['Mailbox/set',{accountId:'acct1',destroyed:args.destroy??[]},call[2]]]};
 });
 fireEvent.click(selected('Alpha'));fireEvent.click(selected('Beta'));
 fireEvent.click(screen.getByTestId('bulk-delete'));
 const dialog=await screen.findByRole('alertdialog');
 fireEvent.click(within(dialog).getByRole('button',{name:'Delete 2 folders'}));
 await waitFor(()=>expect(service.jmap).toHaveBeenCalledTimes(2));
 await waitFor(()=>expect(failure).toHaveBeenCalledWith(expect.stringMatching(/Deleted 1 folders\. Not deleted because processing of the account is restricted: .*Beta/)));
 failure.mockRestore();
},20000);
