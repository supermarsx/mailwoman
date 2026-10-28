import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {EmptyTrash} from './EmptyTrash.tsx';
import {toasts,resetToasts} from './toast/toast.ts';
import {CAP_MAIL} from '../api/jmap-types.ts';

// The Trash toolbar button runs the general empty-folder flow (t45) on Trash.
const state=vi.hoisted(()=>({folder:():string=> 'trash',jmap:vi.fn(),refresh:vi.fn(),clear:vi.fn()}));
const box=(id:string,role:string)=>({id,name:id==='trash'?'Trash':'Inbox',parentId:null,role,sortOrder:0,totalEmails:1,unreadEmails:0});
vi.mock('../state/context.ts',()=>({useApp:()=>({accountId:()=> 'a',selectedMailboxId:()=>state.folder(),mailboxes:()=>[box('trash','trash'),box('inbox','inbox')],clearSelection:state.clear,refreshFolders:state.refresh,refreshCurrentMailbox:state.refresh})}));
vi.mock('../api/transport.ts',()=>({createConfiguredClient:()=>({jmap:state.jmap,session:async()=>({accounts:{a:{accountCapabilities:{[CAP_MAIL]:{mailwomanEmptyMailbox:{moveToTrash:true,destroy:true}}}}}})})}));
vi.mock('../api/accountPreferenceLifecycle.ts',()=>({preferenceAccount:()=>({id:'a',generation:1}),preferenceAccountIsCurrent:()=>true,observePreferenceAccount:()=>()=>{}}));
const answer=(over:Record<string,unknown>)=>({methodResponses:[['Mailbox/empty',{accountId:'a',mailboxId:'trash',outcome:'destroy',trashId:null,total:1,processed:0,remaining:1,...over},'empty']],sessionState:'s'});
afterEach(()=>{cleanup();resetToasts();vi.clearAllMocks();state.folder=()=> 'trash';});

it('requires an explicit confirmation and cancellation never deletes',async()=>{
 state.jmap.mockResolvedValue(answer({}));
 render(()=><EmptyTrash/>);fireEvent.click(await screen.findByRole('button',{name:'Empty trash'}));
 await waitFor(()=>expect(screen.getByRole('button',{name:'Delete permanently'})).toBeEnabled());
 expect(screen.getByRole('alertdialog')).toHaveTextContent('The message in Trash will be deleted permanently. This cannot be undone.');
 expect(state.jmap.mock.calls.every(call=>call[0].methodCalls[0][1].limit===0)).toBe(true);
 expect(document.activeElement).toBe(screen.getByRole('button',{name:'Cancel'}));
 fireEvent.click(screen.getByRole('button',{name:'Cancel'}));expect(screen.queryByRole('alertdialog')).toBeNull();
 expect(state.jmap.mock.calls.every(call=>call[0].methodCalls[0][1].limit===0)).toBe(true);
});

it('reports confirmed results and refreshes only after confirmation',async()=>{
 state.jmap.mockResolvedValueOnce(answer({})).mockResolvedValueOnce(answer({processed:1,remaining:0}));
 render(()=><EmptyTrash/>);fireEvent.click(await screen.findByRole('button',{name:'Empty trash'}));
 await waitFor(()=>expect(screen.getByRole('button',{name:'Delete permanently'})).toBeEnabled());expect(state.refresh).not.toHaveBeenCalled();
 fireEvent.click(screen.getByRole('button',{name:'Delete permanently'}));
 await waitFor(()=>expect(state.refresh).toHaveBeenCalledTimes(2));
 expect(toasts().find(item=>item.testId==='empty-folder-result')?.message).toContain('Deleted 1 message from');expect(screen.queryByRole('alertdialog')).toBeNull();
});

it('leaves the toolbar when the folder is no longer Trash',async()=>{
 const [folder,setFolder]=createSignal('trash');state.folder=folder;
 render(()=><EmptyTrash/>);await screen.findByRole('button',{name:'Empty trash'});setFolder('inbox');
 expect(screen.queryByRole('button',{name:'Empty trash'})).toBeNull();
});

it('shows progress with Stop while the batches run',async()=>{
 let finish:((value:unknown)=>void)|undefined;
 state.jmap.mockResolvedValueOnce(answer({})).mockImplementationOnce(()=>new Promise(resolve=>{finish=resolve;}));
 render(()=><EmptyTrash/>);fireEvent.click(await screen.findByRole('button',{name:'Empty trash'}));
 await waitFor(()=>expect(screen.getByRole('button',{name:'Delete permanently'})).toBeEnabled());
 fireEvent.click(screen.getByRole('button',{name:'Delete permanently'}));expect(screen.queryByRole('alertdialog')).toBeNull();
 const progress=toasts().find(item=>item.testId==='empty-folder-progress');
 expect(progress?.persistent).toBe(true);expect(progress?.action?.label).toBe('Stop');
 await progress?.action?.run();
 finish!(answer({processed:1,remaining:0}));
 await waitFor(()=>expect(toasts().find(item=>item.testId==='empty-folder-result')).toBeDefined());
});
