import {beforeEach,expect,it,vi} from 'vitest';
import {fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {FolderRoles} from './FolderRoles.tsx';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
const state=vi.hoisted(()=>({locked:false,save:vi.fn(),reset:vi.fn()}));
vi.mock('./useSetting.ts',()=>({useSetting:()=>({value:()=>null,entry:()=>({}),locked:()=>state.locked,saving:()=>false,error:()=>'',set:state.save,reset:state.reset})}));
const folders=()=>[{id:'trash',name:'Trash',role:'trash',mailwomanUpstreamName:'Trash'}];
beforeEach(()=>{state.locked=false;state.save.mockReset().mockResolvedValue(true);state.reset.mockReset().mockResolvedValue(true);suspendAccountPreferences();activateAccountPreferences('account');});
it('saves an owned folder and refreshes effective roles, with explicit automatic and inherited choices',async()=>{
 const refresh=vi.fn().mockResolvedValue(undefined);render(()=><FolderRoles folders={folders} onSaved={refresh}/>);
 const select=screen.getByRole('combobox',{name:'Trash'});fireEvent.change(select,{target:{value:'id:trash'}});
 await waitFor(()=>expect(refresh).toHaveBeenCalledOnce());expect(state.save).toHaveBeenLastCalledWith({id:'trash'});
 fireEvent.change(select,{target:{value:'auto'}});await waitFor(()=>expect(state.save).toHaveBeenLastCalledWith(null));
 await waitFor(()=>expect(select).not.toBeDisabled());fireEvent.click(screen.getAllByRole('button',{name:'Use administrator default'})[0]!);await waitFor(()=>expect(state.reset).toHaveBeenCalledOnce());
});
it('does not offer writes for policy-held roles or proxy mailboxes',()=>{
 state.locked=true;const view=render(()=><FolderRoles folders={folders}/>);expect(screen.getAllByRole('combobox').every(node=>(node as HTMLSelectElement).disabled)).toBe(true);view.unmount();
 render(()=><FolderRoles folders={()=>[{id:'proxy',name:'Trash',role:'trash'}]}/>);expect(screen.queryByRole('combobox')).toBeNull();
});
it('keeps controls disabled during a save and ignores refresh after account replacement',async()=>{
 let finish!:(value:boolean)=>void;state.save.mockImplementation(()=>new Promise<boolean>(resolve=>{finish=resolve;}));const refresh=vi.fn();render(()=><FolderRoles folders={folders} onSaved={refresh}/>);
 const select=screen.getByRole('combobox',{name:'Trash'});fireEvent.change(select,{target:{value:'id:trash'}});expect(select).toBeDisabled();suspendAccountPreferences();activateAccountPreferences('replacement');finish(true);
 await waitFor(()=>expect(select).not.toBeDisabled());expect(refresh).not.toHaveBeenCalled();
});
