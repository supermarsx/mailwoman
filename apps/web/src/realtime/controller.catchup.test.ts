import {afterEach,expect,it,vi} from 'vitest';
import type {PushClientOptions,PushStatus,PushStatusDetail} from './pushClient.ts';
import type {PushTransport} from '../contracts/push.ts';
const fake=vi.hoisted(()=>({options:undefined as PushClientOptions|undefined}));
vi.mock('../worker/networkProxy.ts',()=>({createSharedPushClient:(options:PushClientOptions)=>{fake.options=options;return {connect:vi.fn(),close:vi.fn(),reconnect:vi.fn(),onStateChange:()=>()=>{},onMailboxChange:()=>()=>{}};}}));
import {createRealtimeController} from './controller.ts';
afterEach(()=>vi.useRealTimers());
const report=(status:PushStatus,transport:PushTransport,detail?:PushStatusDetail)=>fake.options!.onStatus!(status,transport,detail);
it('poll fallback catches up the active account even without any initial StateChange',()=>{
 vi.useFakeTimers();const catchUp=vi.fn();const controller=createRealtimeController({onCatchUp:catchUp,random:()=>0.5});controller.start();
 report('degraded','poll');expect(catchUp).toHaveBeenCalledTimes(1);
 vi.advanceTimersByTime(30000);expect(catchUp).toHaveBeenCalledTimes(2);
 report('degraded','poll');vi.advanceTimersByTime(30000);expect(catchUp).toHaveBeenCalledTimes(3);
 report('open','ws');expect(catchUp).toHaveBeenCalledTimes(4);vi.advanceTimersByTime(60000);expect(catchUp).toHaveBeenCalledTimes(4);controller.stop();
});
it('stops catch-up on logout or an expired session and catches a reconnect without known state',()=>{
 vi.useFakeTimers();const catchUp=vi.fn();const controller=createRealtimeController({onCatchUp:catchUp,random:()=>0.5});controller.start();
 report('degraded','poll');report('unauthorized','offline');vi.advanceTimersByTime(60000);expect(catchUp).toHaveBeenCalledTimes(1);
 report('reconnecting','ws');report('open','ws');expect(catchUp).toHaveBeenCalledTimes(2);
 report('degraded','poll');controller.stop();const count=catchUp.mock.calls.length;vi.advanceTimersByTime(60000);expect(catchUp).toHaveBeenCalledTimes(count);
});
it('names the first open apart from a reconnect (t52w)',()=>{
 const catchUp=vi.fn();const controller=createRealtimeController({onCatchUp:catchUp,random:()=>0.5});controller.start();
 report('open','ws');expect(catchUp).toHaveBeenLastCalledWith('open');
 report('reconnecting','ws');report('open','ws');expect(catchUp).toHaveBeenLastCalledWith('resync');
 controller.stop();
});

it('an immediate catch-up when the reconnect had no restart and fewer than two failures',()=>{
 vi.useFakeTimers();const catchUp=vi.fn();const controller=createRealtimeController({onCatchUp:catchUp,random:()=>0.5});controller.start();
 report('open','ws',{spreadCatchUp:false});report('reconnecting','ws');report('open','ws',{spreadCatchUp:false});
 expect(catchUp).toHaveBeenLastCalledWith('resync');controller.stop();
});
it('staggers the catch-up inside the window after a restart and runs it once',()=>{
 vi.useFakeTimers();const catchUp=vi.fn();const controller=createRealtimeController({onCatchUp:catchUp,random:()=>0.5});controller.start();
 report('open','ws',{spreadCatchUp:false});catchUp.mockClear();
 report('reconnecting','ws');report('open','ws',{spreadCatchUp:true});
 expect(catchUp).not.toHaveBeenCalled();
 vi.advanceTimersByTime(999);expect(catchUp).not.toHaveBeenCalled();
 vi.advanceTimersByTime(1);expect(catchUp).toHaveBeenCalledTimes(1);expect(catchUp).toHaveBeenLastCalledWith('resync');
 vi.advanceTimersByTime(60000);expect(catchUp).toHaveBeenCalledTimes(1);controller.stop();
});
it('the stagger never exceeds the window, and one catch-up serves repeated open reports',()=>{
 vi.useFakeTimers();const catchUp=vi.fn();const controller=createRealtimeController({onCatchUp:catchUp,random:()=>0.999999});controller.start();
 report('reconnecting','ws');report('open','ws',{spreadCatchUp:true});report('open','ws',{spreadCatchUp:true});
 const resyncs=()=>catchUp.mock.calls.filter(c=>c[0]==='resync').length;
 vi.advanceTimersByTime(1998);expect(resyncs()).toBe(0);vi.advanceTimersByTime(2);expect(resyncs()).toBe(1);controller.stop();
});
it('drops a waiting catch-up when the stream goes down again, and catches up after the next open',()=>{
 vi.useFakeTimers();const catchUp=vi.fn();const controller=createRealtimeController({onCatchUp:catchUp,random:()=>0.5});controller.start();
 report('reconnecting','ws');report('open','ws',{spreadCatchUp:true});
 report('reconnecting','ws');vi.advanceTimersByTime(5000);expect(catchUp).not.toHaveBeenCalled();
 report('open','ws',{spreadCatchUp:true});vi.advanceTimersByTime(1000);expect(catchUp).toHaveBeenCalledTimes(1);controller.stop();
});
it('does not run a waiting catch-up after stop or sign-out',()=>{
 vi.useFakeTimers();const catchUp=vi.fn();const controller=createRealtimeController({onCatchUp:catchUp,random:()=>0.5});controller.start();
 report('reconnecting','ws');report('open','ws',{spreadCatchUp:true});controller.stop();vi.advanceTimersByTime(5000);expect(catchUp).not.toHaveBeenCalled();
 report('reconnecting','ws');report('open','ws',{spreadCatchUp:true});report('unauthorized','offline');vi.advanceTimersByTime(5000);expect(catchUp).not.toHaveBeenCalled();
});
it('a poll catch-up replaces a waiting staggered one',()=>{
 vi.useFakeTimers();const catchUp=vi.fn();const controller=createRealtimeController({onCatchUp:catchUp,random:()=>0.5});controller.start();
 report('reconnecting','ws');report('open','ws',{spreadCatchUp:true});report('reconnecting','ws');report('degraded','poll');
 expect(catchUp).toHaveBeenCalledTimes(1);vi.advanceTimersByTime(1000);expect(catchUp).toHaveBeenCalledTimes(1);controller.stop();
});
it('the poll interval is 30 s plus or minus 20 %',()=>{
 for(const [draw,wait] of [[0,24000],[0.999999,36000]] as const){
  vi.useFakeTimers();const catchUp=vi.fn();const controller=createRealtimeController({onCatchUp:catchUp,random:()=>draw});controller.start();
  report('degraded','poll');expect(catchUp).toHaveBeenCalledTimes(1);
  vi.advanceTimersByTime(wait-2);expect(catchUp).toHaveBeenCalledTimes(1);vi.advanceTimersByTime(3);expect(catchUp).toHaveBeenCalledTimes(2);
  controller.stop();vi.useRealTimers();
 }
});
