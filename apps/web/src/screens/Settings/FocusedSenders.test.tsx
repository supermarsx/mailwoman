import {afterEach,expect,it,vi} from 'vitest';
import {createSignal} from 'solid-js';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {FocusedSenders} from './FocusedSenders.tsx';
import type {FocusedSender} from '../../api/focusedSenders.ts';
// t94: Settings > Folders and sidebar > Manage Focused senders.
afterEach(()=>cleanup());
it('lists each override with its tab and removes one',async()=>{
 const [rows,setRows]=createSignal<FocusedSender[]>([{email:'boss@work.example',tab:'focused',updatedAt:1},{email:'news@shop.example',tab:'other',updatedAt:2}]);
 const onRemove=vi.fn(async(email:string)=>{setRows(rows().filter(row=>row.email!==email));});
 render(()=><FocusedSenders senders={rows} onRemove={onRemove}/>);
 expect(screen.getByRole('group',{name:'Manage Focused senders'})).toBeVisible();
 const items=screen.getAllByTestId('focused-sender');
 expect(items.map(item=>item.textContent)).toEqual(['boss@work.exampleAlways FocusedRemove','news@shop.exampleAlways OtherRemove']);
 fireEvent.click(screen.getByRole('button',{name:'Remove news@shop.example'}));
 await waitFor(()=>expect(screen.getAllByTestId('focused-sender')).toHaveLength(1));
 expect(onRemove).toHaveBeenCalledWith('news@shop.example');
});
it('says so when there is nothing, and when a removal fails',async()=>{
 const [rows]=createSignal<FocusedSender[]>([{email:'a@x.example',tab:'other',updatedAt:1}]);
 render(()=><FocusedSenders senders={rows} onRemove={async()=>{throw new Error('503');}}/>);
 fireEvent.click(screen.getByRole('button',{name:'Remove a@x.example'}));
 expect(await screen.findByRole('alert')).toHaveTextContent('could not be removed');
 cleanup();
 render(()=><FocusedSenders senders={()=>[]} onRemove={async()=>undefined}/>);
 expect(screen.getByTestId('focused-senders-empty')).toHaveTextContent('No senders yet');
});
