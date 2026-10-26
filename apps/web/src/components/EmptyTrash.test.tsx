import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {EmptyTrash} from './EmptyTrash.tsx';
const state=vi.hoisted(()=>({folder:():string=> 'trash',prepare:vi.fn(),empty:vi.fn(),refresh:vi.fn(),clear:vi.fn()}));
vi.mock('../state/context.ts',()=>({useApp:()=>({accountId:()=> 'a',selectedMailboxId:()=>state.folder(),mailboxes:()=>[{id:'trash',role:'trash'},{id:'inbox',role:'inbox'}],clearSelection:state.clear,refreshFolders:state.refresh,refreshCurrentMailbox:state.refresh})}));
vi.mock('../api/transport.ts',()=>({createConfiguredClient:()=>({})}));
vi.mock('../api/accountPreferenceLifecycle.ts',()=>({preferenceAccount:()=>({id:'a',generation:1}),preferenceAccountIsCurrent:()=>true}));
vi.mock('../api/emptyTrash.ts',()=>({TRASH_LIMIT:5000,prepareTrash:state.prepare,emptyTrash:state.empty}));
afterEach(()=>{cleanup();vi.clearAllMocks();state.folder=()=> 'trash';});
it('requires an explicit confirmation and cancellation never deletes',async()=>{
 state.prepare.mockResolvedValue({accountId:'a',mailboxId:'trash',ids:['one'],more:false});
 render(()=><EmptyTrash/>);fireEvent.click(screen.getByRole('button',{name:'Empty trash'}));
 await waitFor(()=>expect(screen.getByRole('button',{name:'Delete permanently'})).not.toBeDisabled());
 expect(state.empty).not.toHaveBeenCalled();expect(document.activeElement).toBe(screen.getByRole('button',{name:'Close'}));
 fireEvent.click(screen.getByRole('button',{name:'Close'}));expect(screen.queryByRole('alertdialog')).toBeNull();expect(state.empty).not.toHaveBeenCalled();
});
it('reports confirmed results and refreshes only after confirmation',async()=>{
 state.prepare.mockResolvedValue({accountId:'a',mailboxId:'trash',ids:['one','two'],more:false});state.empty.mockResolvedValue({deleted:1,skipped:1,remaining:0,stopped:false});
 render(()=><EmptyTrash/>);fireEvent.click(screen.getByRole('button',{name:'Empty trash'}));await waitFor(()=>expect(screen.getByRole('button',{name:'Delete permanently'})).not.toBeDisabled());fireEvent.click(screen.getByRole('button',{name:'Delete permanently'}));
 await waitFor(()=>expect(state.refresh).toHaveBeenCalledTimes(2));expect(state.empty).toHaveBeenCalledTimes(1);expect(screen.getByRole('status').textContent).toContain('Deleted 1');expect(screen.getByRole('button',{name:'Delete permanently'})).toBeDisabled();
});
it('closes and aborts the preview when changing folders',async()=>{
 const [folder,setFolder]=createSignal('trash');state.folder=folder;state.prepare.mockReturnValue(new Promise(()=>{}));
 render(()=><EmptyTrash/>);fireEvent.click(screen.getByRole('button',{name:'Empty trash'}));const authority=state.prepare.mock.calls[0]?.[3] as {signal:AbortSignal};setFolder('inbox');
 expect(screen.queryByRole('alertdialog')).toBeNull();expect(authority.signal.aborted).toBe(true);expect(state.empty).not.toHaveBeenCalled();expect(screen.queryByRole('button',{name:'Empty trash'})).toBeNull();
});
