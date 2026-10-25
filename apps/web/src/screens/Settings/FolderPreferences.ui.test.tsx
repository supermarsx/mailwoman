import {expect,it,vi} from 'vitest';
import {render,screen,fireEvent} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {FolderPreferences} from './FolderPreferences.tsx';
import type {AccountUiPreferences} from '../../api/accountPreferences.ts';
it('offers fifteen native icons plus automatic and saves only the changed folder preference',async()=>{
 const [prefs,setPrefs]=createSignal<AccountUiPreferences>({});const update=vi.fn((patch:AccountUiPreferences)=>setPrefs(value=>({...value,...patch})));
 render(()=><FolderPreferences folders={()=>[{id:'a',name:'Projects'},{id:'b',name:'Receipts'}]} preferences={prefs} onChange={update}/>);
 const select=await screen.findByRole('combobox',{name:'Icon for Projects'});expect(select.querySelectorAll('option')).toHaveLength(16);
 fireEvent.change(select,{target:{value:'star'}});expect(update).toHaveBeenLastCalledWith({folderIcons:{a:'star'}});
 fireEvent.click(screen.getByRole('checkbox',{name:'Pin Projects'}));expect(update).toHaveBeenLastCalledWith({folderPinned:['a']});
 expect(screen.getByRole('button',{name:'Move Projects down'})).toBeDisabled();
});
it('supports keyboard move controls without any mailbox mutation',()=>{
 const update=vi.fn();render(()=><FolderPreferences folders={()=>[{id:'a',name:'Projects'},{id:'b',name:'Receipts'}]} preferences={()=>({})} onChange={update}/>);
 fireEvent.click(screen.getByRole('button',{name:'Move Projects down'}));expect(update).toHaveBeenCalledWith({folderOrder:['b','a']});
});
