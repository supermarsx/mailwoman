import {afterEach,expect,it,vi} from 'vitest';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {beginWorkspaceTabDrag,endWorkspaceTabDrag,workspaceTabDragId} from './workspaceTabDrag.ts';
afterEach(()=>{endWorkspaceTabDrag();suspendAccountPreferences();});
function transfer(){const values=new Map<string,string>();const data={types:[] as string[],effectAllowed:'',setData:(key:string,value:string)=>{values.set(key,value);data.types=[key];},getData:(key:string)=>values.get(key)??''};return {data,event:{dataTransfer:data,preventDefault:vi.fn()} as unknown as DragEvent};}
it('accepts only an internal current-generation token and rejects external data',()=>{
 activateAccountPreferences('same-id','msa1_abcdefghijklmnop');const {event,data}=transfer();beginWorkspaceTabDrag(event,'owned-tab');expect(workspaceTabDragId(event,true)).toBe('owned-tab');
 data.setData(data.types[0]!,'forged');expect(workspaceTabDragId(event,true)).toBeNull();
});
it('invalidates a drag when the same wire account enters a different verified realm',()=>{
 activateAccountPreferences('same-id','msa1_abcdefghijklmnop');const {event}=transfer();beginWorkspaceTabDrag(event,'owned-tab');
 activateAccountPreferences('same-id','msa1_otherabcdefghijk');expect(workspaceTabDragId(event)).toBeNull();expect(workspaceTabDragId(event,true)).toBeNull();
});
it('clears the token at drag end and refuses anonymous drags',()=>{
 activateAccountPreferences('account');const {event}=transfer();beginWorkspaceTabDrag(event,'tab');endWorkspaceTabDrag();expect(workspaceTabDragId(event,true)).toBeNull();
 suspendAccountPreferences();beginWorkspaceTabDrag(event,'tab');expect(event.preventDefault).toHaveBeenCalled();
});
