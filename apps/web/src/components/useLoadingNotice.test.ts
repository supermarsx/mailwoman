import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {createRoot,createSignal} from 'solid-js';
import {useLoadingNotice,SLOW_LOADING_DELAY_MS} from './useLoadingNotice.ts';
import {toast,toasts} from './toast/toast.ts';

const preferences=vi.hoisted(()=>({values:{} as Record<string,unknown>}));
vi.mock('../api/notificationControls.ts',()=>({alertValue:(key:string,fallback:unknown)=>preferences.values[key]??fallback}));
const disposers:Array<()=>void>=[];
function loading(){
 return createRoot(dispose=>{
  disposers.push(dispose);
  const [active,setActive]=createSignal(true);
  useLoadingNotice(active);
  return {setActive,dispose};
 });
}
beforeEach(()=>{vi.useFakeTimers();preferences.values={};toast.clear();});
afterEach(()=>{for(const dispose of disposers.splice(0))dispose();toast.clear();vi.useRealTimers();});

it('stays silent for fast loads and dismisses a slow-load notice when finished',()=>{
 const work=loading();
 vi.advanceTimersByTime(SLOW_LOADING_DELAY_MS-1);expect(toasts()).toHaveLength(0);
 vi.advanceTimersByTime(1);expect(toasts()).toHaveLength(1);
 expect(toasts()[0]).toMatchObject({id:'page-still-loading',busy:true,message:'This page is still loading. Please wait…'});
 work.setActive(false);expect(toasts()).toHaveLength(0);
 work.setActive(true);vi.advanceTimersByTime(100);work.setActive(false);
 vi.advanceTimersByTime(SLOW_LOADING_DELAY_MS);expect(toasts()).toHaveLength(0);
});

it('deduplicates concurrent loads and keeps the notice until all slow loads finish',()=>{
 const one=loading(),two=loading();
 vi.advanceTimersByTime(SLOW_LOADING_DELAY_MS);expect(toasts()).toHaveLength(1);
 one.dispose();expect(toasts()).toHaveLength(1);
 two.dispose();expect(toasts()).toHaveLength(0);
});

it('cancels pending timers on navigation and does not repeat a dismissed notice',()=>{
 const quick=loading();quick.dispose();vi.advanceTimersByTime(SLOW_LOADING_DELAY_MS);expect(toasts()).toHaveLength(0);
 const slow=loading();vi.advanceTimersByTime(SLOW_LOADING_DELAY_MS);toast.dismiss('page-still-loading');
 const other=loading();vi.advanceTimersByTime(SLOW_LOADING_DELAY_MS*2);expect(toasts()).toHaveLength(0);
 slow.dispose();other.dispose();
});

it('honors disabled informational toasts',()=>{
 preferences.values['notifications.toasts.info']=false;
 loading();vi.advanceTimersByTime(SLOW_LOADING_DELAY_MS);expect(toasts()).toHaveLength(0);
});
