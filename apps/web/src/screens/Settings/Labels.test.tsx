import {beforeEach,expect,it,vi} from 'vitest';
import {createSignal} from 'solid-js';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {Labels} from './Labels.tsx';
import {createSettingsNavigation} from './navigation.tsx';
import {AppContext} from '../../state/context.ts';
import type {AppState} from '../../state/store.ts';
const owner={id:'account-a',generation:1};
let status:()=>string;let setStatus:(value:string)=>void;let current=true;
vi.mock('../../api/accountPreferences.ts',async(importOriginal)=>({...await importOriginal<typeof import('../../api/accountPreferences.ts')>(),accountUiPreferenceStatus:()=>status(),retryAccountUiPreferences:vi.fn()}));
vi.mock('../../api/accountPreferenceLifecycle.ts',()=>({preferenceAccount:()=>owner,preferenceAccountIsCurrent:()=>current,observePreferenceAccount:()=>()=>{}}));
beforeEach(()=>{[status,setStatus]=createSignal('synced');current=true;});
function fixture(){
  const app={tags:()=>[{id:'work',name:'Work',color:'#112233',icon:'tag'}],tagByKeyword:(id:string)=>id==='work'?{}:undefined,addTag:vi.fn(()=>{setStatus('saving');return 'travel';}),updateTag:vi.fn(),deleteTag:vi.fn()};
  const navigation=createSettingsNavigation({section:'labels'});render(()=><AppContext.Provider value={app as unknown as AppState}><Labels navigation={navigation}/></AppContext.Provider>);return {app,navigation};
}
it('opens a dedicated label editor and waits for server synchronization before returning',async()=>{
  const {app,navigation}=fixture();await screen.findByRole('list',{name:'Labels'});fireEvent.click(screen.getByRole('button',{name:'New label'}));await screen.findByLabelText('Label name');expect(screen.queryByRole('list',{name:'Labels'})).not.toBeInTheDocument();
  fireEvent.input(screen.getByLabelText('Label name'),{target:{value:'Travel'}});fireEvent.click(screen.getByRole('button',{name:'Save label'}));expect(app.addTag).toHaveBeenCalledWith('Travel','#2563eb','tag');expect(navigation.route().editor?.kind).toBe('new');
  setStatus('error');await screen.findByRole('alert');expect(screen.getByRole('button',{name:'Retry synchronization'})).toBeVisible();expect(navigation.route().editor).toBeDefined();
  setStatus('synced');await waitFor(()=>expect(navigation.route().editor).toBeUndefined());
});
it('rejects duplicate definitions and cannot write a draft after account switching',async()=>{
  const {app}=fixture();fireEvent.click(await screen.findByRole('button',{name:'New label'}));await screen.findByLabelText('Label name');fireEvent.input(screen.getByLabelText('Label name'),{target:{value:'Work'}});fireEvent.click(screen.getByRole('button',{name:'Save label'}));await screen.findByRole('alert');expect(app.addTag).not.toHaveBeenCalled();
  fireEvent.input(screen.getByLabelText('Label name'),{target:{value:'Travel'}});current=false;fireEvent.click(screen.getByRole('button',{name:'Save label'}));expect(app.addTag).not.toHaveBeenCalled();
});

it('keeps label search and row actions in the shared linked-mailbox manager with a separate editor',async()=>{
 const {navigation}=fixture();const list=await screen.findByRole('list',{name:'Labels'});expect(list.closest('section')?.id).toBe('labels-card');expect(screen.getByRole('heading',{name:'Labels'})).toBeVisible();
 expect(screen.getByRole('button',{name:'Edit label Work'})).toHaveAttribute('title','Edit label Work');
 fireEvent.input(screen.getByRole('searchbox',{name:'Search'}),{target:{value:'missing'}});expect(list.children).toHaveLength(0);
 fireEvent.input(screen.getByRole('searchbox',{name:'Search'}),{target:{value:'work'}});expect(list.children).toHaveLength(1);
 fireEvent.click(screen.getByRole('button',{name:'Edit label Work'}));await screen.findByLabelText('Label name');expect(navigation.route().editor).toEqual({kind:'edit',id:'work'});expect(document.querySelector('#labels-card #labels-card-editor')).toBeNull();
});
