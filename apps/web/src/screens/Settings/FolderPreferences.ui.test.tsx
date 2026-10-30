import {expect,it,vi} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {FolderPreferences} from './FolderPreferences.tsx';
import type {AccountUiPreferences} from '../../api/accountPreferences.ts';
it('keeps hidden folders manageable and restores their hidden ancestors without modifying mail',async()=>{
 const [prefs,setPrefs]=createSignal<AccountUiPreferences>({folderHidden:['a'],userFoldersCollapsed:true});
 const update=vi.fn((patch:AccountUiPreferences)=>setPrefs(value=>({...value,...patch})));
 render(()=><FolderPreferences folders={()=>[{id:'inbox',name:'Inbox',role:'inbox'},{id:'a',name:'Projects'},{id:'b',name:'Child',parentId:'a'}]} preferences={prefs} onChange={update}/>);
 expect(await screen.findByRole('button',{name:'Hide Inbox from sidebar'})).toBeDisabled();
 expect(screen.getByRole('checkbox',{name:'Collapse user folders in the sidebar'})).toBeChecked();
 fireEvent.click(screen.getByRole('button',{name:'Show Child in sidebar'}));
 expect(update).toHaveBeenLastCalledWith({folderHidden:[]});
 fireEvent.click(screen.getByRole('button',{name:'Hide Projects from sidebar'}));
 expect(screen.getByRole('button',{name:'Show Child in sidebar'})).toBeVisible();
 expect(screen.getByRole('button',{name:'Show Projects in sidebar'})).toBeVisible();
 fireEvent.click(screen.getByRole('checkbox',{name:'Collapse user folders in the sidebar'}));
 expect(update).toHaveBeenLastCalledWith({userFoldersCollapsed:false});
});
it('offers fifty native icons plus automatic and saves only the changed folder preference',async()=>{
 const [prefs,setPrefs]=createSignal<AccountUiPreferences>({});const update=vi.fn((patch:AccountUiPreferences)=>setPrefs(value=>({...value,...patch})));
 render(()=><FolderPreferences folders={()=>[{id:'a',name:'Projects'},{id:'b',name:'Receipts'}]} preferences={prefs} onChange={update}/>);
 expect(screen.queryByRole('combobox')).toBeNull();
 fireEvent.click(await screen.findByRole('button',{name:'Edit Projects appearance'}));
 const select=await screen.findByRole('combobox',{name:'Icon for Projects'});expect(select.querySelectorAll('option')).toHaveLength(51);
 fireEvent.change(select,{target:{value:'star'}});expect(update).not.toHaveBeenCalled();
 fireEvent.click(screen.getByRole('button',{name:'Save folder appearance'}));expect(update).toHaveBeenLastCalledWith({folderIcons:{a:'star'}});
 await waitFor(()=>expect(screen.getByRole('table')).toBeVisible());
 fireEvent.click(screen.getByRole('button',{name:'Pin Projects'}));expect(update).toHaveBeenLastCalledWith({folderPinned:['a']});
 expect(screen.getByRole('button',{name:'Move Projects down'})).toBeDisabled();
});
it('supports keyboard move controls without any mailbox mutation',()=>{
 const update=vi.fn();render(()=><FolderPreferences folders={()=>[{id:'a',name:'Projects'},{id:'b',name:'Receipts'}]} preferences={()=>({})} onChange={update}/>);
 fireEvent.click(screen.getByRole('button',{name:'Move Projects down'}));expect(update).toHaveBeenCalledWith({folderOrder:['b','a']});
});
it('edits folder labels in the dedicated page without modifying message keywords',async()=>{
 const [prefs,setPrefs]=createSignal<AccountUiPreferences>({folderLabels:{a:['old']}});const update=vi.fn((patch:AccountUiPreferences)=>setPrefs(value=>({...value,...patch})));
 render(()=><FolderPreferences folders={()=>[{id:'a',name:'Projects'}]} preferences={prefs} onChange={update} tags={()=>[{id:'work',name:'Work',color:'#112233',icon:'tag'}]}/>);
 fireEvent.click(await screen.findByRole('button',{name:'Edit Projects appearance'}));await screen.findByRole('checkbox',{name:'Work'});fireEvent.click(screen.getByRole('checkbox',{name:'Work'}));fireEvent.click(screen.getByRole('checkbox',{name:'old'}));expect(update).not.toHaveBeenCalled();
 fireEvent.click(screen.getByRole('button',{name:'Save folder appearance'}));expect(update).toHaveBeenCalledWith({folderIcons:{},folderLabels:{a:['work']}});await screen.findByRole('table');
});
