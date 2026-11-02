import {afterEach, expect, it, vi} from 'vitest';
import {cleanup, fireEvent, render, screen, waitFor, within} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {FolderPreferences} from './FolderPreferences.tsx';
import type {PreferenceFolder} from './folderPreferenceModel.ts';
import type {AccountUiPreferences} from '../../api/accountPreferences.ts';
import {activateAccountPreferences, suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import {AppContext} from '../../state/context.ts';
import type {AppState} from '../../state/store.ts';
import type {JmapRequest} from '../../api/jmap-types.ts';

const service=vi.hoisted(()=>({manages:true,jmap:vi.fn()}));
vi.mock('../../api/transport.ts',()=>({createConfiguredClient:()=>({jmap:service.jmap})}));
vi.mock('../../components/RenameFolder.tsx',()=>({useRenameContext:()=>()=>({manages:service.manages,delimiter:'/',maxLength:255})}));
vi.mock('../../components/QuotaMeter.tsx',()=>({QuotaDetails:()=>null}));
vi.mock('./FolderRoles.tsx',()=>({FolderRoles:()=>null}));
vi.mock('./FocusedInboxPreference.tsx',()=>({FocusedInboxPreference:()=>null}));
vi.mock('./FocusedSenders.tsx',()=>({FocusedSenders:()=>null}));
vi.mock('./FolderCountPreferences.tsx',()=>({FolderCountPreferences:()=>null}));
afterEach(()=>{cleanup();suspendAccountPreferences();service.manages=true;service.jmap.mockReset();});

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
function drop(source:string,target:string):void{
 fireEvent.dragStart(drag(source),{dataTransfer:{setData:vi.fn()}});
 fireEvent.drop(drag(target).closest('tr')!);
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

it('reorders filtered rows using the full folder order and refuses cross-pin drops',()=>{
 const {update}=mount([{id:'a',name:'Match A'},{id:'middle',name:'Other'},{id:'b',name:'Match B'},{id:'p',name:'Pinned'}],{folderPinned:['p'],folderOrder:['a','middle','b','p']});
 search('Match');drop('Match A','Match B');
 expect(update).toHaveBeenLastCalledWith({folderOrder:['p','middle','b','a']});
 search('');update.mockClear();drop('Match B','Pinned');
 expect(update).not.toHaveBeenCalled();
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
