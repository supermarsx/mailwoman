import {afterEach,expect,it,vi} from 'vitest';
import {fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {createEntityContextMenu} from './entityContextMenu.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {handleModuleActions,requestModuleAction} from '../state/moduleActions.ts';

afterEach(()=>suspendAccountPreferences());
function harness(){
 const run=vi.fn();const [items,setItems]=createSignal([{id:'one',name:'First'}]);
 render(()=>{const menu=createEntityContextMenu({items,id:item=>item.id,name:item=>item.name,actions:()=>[{id:'edit',label:'Edit',icon:'edit',run}]});return <><button {...menu.bind('one')}>First</button>{menu.menu()}</>;});
 return {run,setItems};
}
it('opens from Shift+F10 and executes the selected live action',async()=>{
 const {run}=harness();const row=screen.getByRole('button',{name:'First'});row.focus();fireEvent.keyDown(row,{key:'F10',shiftKey:true});
 fireEvent.click(await screen.findByRole('menuitem',{name:'Edit'}));expect(run).toHaveBeenCalledOnce();expect(row).toHaveFocus();
});
it('closes when the target disappears rather than executing a stale action',async()=>{
 const {run,setItems}=harness();fireEvent.contextMenu(screen.getByRole('button'));await screen.findByRole('menu');setItems([]);
 await waitFor(()=>expect(screen.queryByRole('menu')).toBeNull());expect(run).not.toHaveBeenCalled();
});
it('closes across an account switch even when the row ID is reused',async()=>{
 activateAccountPreferences('first');const {run}=harness();fireEvent.contextMenu(screen.getByRole('button'));await screen.findByRole('menu');activateAccountPreferences('second');
 await waitFor(()=>expect(screen.queryByRole('menu')).toBeNull());expect(run).not.toHaveBeenCalled();
});
it('delivers a sidebar action after its lazy module mounts exactly once',async()=>{
 activateAccountPreferences('first');const handle=vi.fn();requestModuleAction('notes','new');
 render(()=>{handleModuleActions('notes',handle);return <div/>;});await waitFor(()=>expect(handle).toHaveBeenCalledWith('new'));expect(handle).toHaveBeenCalledOnce();
});
it('drops a queued sidebar action when its account has changed',async()=>{
 activateAccountPreferences('first');const handle=vi.fn();requestModuleAction('notes','new');activateAccountPreferences('second');
 render(()=>{handleModuleActions('notes',handle);return <div/>;});await Promise.resolve();expect(handle).not.toHaveBeenCalled();
});
