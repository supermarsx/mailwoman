import {afterEach,expect,it,vi} from 'vitest';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {beginMessageDrag,clearMessageDrag,MESSAGE_DRAG_TYPE,peekMessageDrag,takeMessageDrag} from './messageDrag.ts';
const policy=vi.hoisted(()=>({enabled:true}));
vi.mock('./mailInteractions.ts',()=>({dragToFolderEnabled:()=>policy.enabled}));
function event(){const values=new Map<string,string>();return {preventDefault:vi.fn(),dataTransfer:{effectAllowed:'none',get types(){return [...values.keys()];},setData:(key:string,value:string)=>values.set(key,value),getData:(key:string)=>values.get(key)??''}} as unknown as DragEvent;}
afterEach(()=>{clearMessageDrag();suspendAccountPreferences();policy.enabled=true;vi.useRealTimers();});
it('transfers only a local token and consumes the current owned selection once',()=>{
 activateAccountPreferences('a');const drag=event();expect(beginMessageDrag(drag,'a',['one','two','one'])).toBe(true);
 expect(drag.dataTransfer?.types).toEqual([MESSAGE_DRAG_TYPE]);expect(drag.dataTransfer?.getData(MESSAGE_DRAG_TYPE)).not.toContain('one');
 expect(peekMessageDrag(drag,'a')).toEqual(['one','two']);expect(takeMessageDrag(drag,'a')).toEqual(['one','two']);expect(takeMessageDrag(drag,'a')).toBeNull();
});
it('rejects forged external data, changed account generation, expired gestures and locked policy',()=>{
 vi.useFakeTimers();activateAccountPreferences('a');const drag=event();beginMessageDrag(drag,'a',['one']);
 const forged=event();forged.dataTransfer?.setData(MESSAGE_DRAG_TYPE,'one');expect(takeMessageDrag(forged,'a')).toBeNull();expect(peekMessageDrag(drag,'b')).toBeNull();
 policy.enabled=false;expect(peekMessageDrag(drag,'a')).toBeNull();policy.enabled=true;vi.advanceTimersByTime(30001);expect(takeMessageDrag(drag,'a')).toBeNull();
 beginMessageDrag(drag,'a',['one']);suspendAccountPreferences();activateAccountPreferences('a');expect(takeMessageDrag(drag,'a')).toBeNull();
});
