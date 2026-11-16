import {expect,it,vi} from 'vitest';
import {nativeReplayScheduler,type NativeScheduleLocks} from './nativeReplaySchedule.ts';

function serialLocks():NativeScheduleLocks {
 let tail=Promise.resolve();
 return {request<T>(_name:string,work:()=>Promise<T>):Promise<T>{const result=tail.then(work);tail=result.then(()=>undefined,()=>undefined);return result;}};
}
async function racingEnqueue(locks:NativeScheduleLocks):Promise<boolean>{
 let pending=false,job=true,entered!:()=>void,release!:()=>void;
 const atDelete=new Promise<void>(resolve=>{entered=resolve;}),deleting=new Promise<void>(resolve=>{release=resolve;});
 const scheduler=nativeReplayScheduler({locks,pending:async()=>pending,invoke:async(command,args)=>{
  if(command==='mw_background_replay_complete'&&args['complete']===true){entered();await deleting;job=false;}
  if(command==='mw_background_replay_schedule')job=args['enabled']===true;
  return {supported:true};
 }});
 const completion=scheduler.finish(true);await atDelete;
 pending=true;const registration=scheduler.schedule();
 // Give an unprotected foreground registration time to run before deletion.
 await Promise.resolve();await Promise.resolve();await Promise.resolve();
 release();await Promise.all([completion,registration]);return job;
}
it('demonstrates the lost wakeup when scheduler operations are not serialized',async()=>{
 const unlocked:NativeScheduleLocks={request:(_name,work)=>work()};
 expect(await racingEnqueue(unlocked)).toBe(false);
});
it('registers a racing foreground enqueue after the hidden runner removes its old job',async()=>{
 expect(await racingEnqueue(serialLocks())).toBe(true);
});
it('rechecks the queue before completing instead of trusting the earlier drain result',async()=>{
 const invoke=vi.fn(async()=>({supported:true}));
 await nativeReplayScheduler({locks:serialLocks(),pending:async()=>true,invoke}).finish(true);
 expect(invoke).toHaveBeenCalledExactlyOnceWith('mw_background_replay_complete',{complete:false});
});
it('keeps an incomplete runner scheduled even when its last queue scan is empty',async()=>{
 const invoke=vi.fn(async()=>({supported:true}));
 await nativeReplayScheduler({locks:serialLocks(),pending:async()=>false,invoke}).finish(false);
 expect(invoke).toHaveBeenCalledExactlyOnceWith('mw_background_replay_complete',{complete:false});
});
it('reports unsupported coordination and avoids claiming completion without cross-process locks',async()=>{
 const invoke=vi.fn(async()=>({supported:true})),pending=vi.fn(async()=>true),scheduler=nativeReplayScheduler({locks:undefined,pending,invoke});
 expect(await scheduler.schedule()).toBe(false);expect(invoke).not.toHaveBeenCalled();
 await scheduler.finish(true);expect(invoke).toHaveBeenCalledExactlyOnceWith('mw_background_replay_complete',{complete:false});expect(pending).not.toHaveBeenCalled();
});
it('serializes logout cancellation after an already-running registration',async()=>{
 let job=false,entered!:()=>void,release!:()=>void;
 const registered=new Promise<void>(resolve=>{entered=resolve;}),registering=new Promise<void>(resolve=>{release=resolve;});
 const scheduler=nativeReplayScheduler({locks:serialLocks(),pending:async()=>true,invoke:async(_command,args)=>{
  if(args['enabled']===true){entered();await registering;job=true;}else job=false;return {supported:true};
 }});
 const registeringJob=scheduler.schedule();await registered;const logout=scheduler.cancel();release();await Promise.all([registeringJob,logout]);expect(job).toBe(false);
});
it('can still cancel an old scheduler job when the runtime has no locks',async()=>{
 const invoke=vi.fn(async()=>({supported:true}));
 expect(await nativeReplayScheduler({locks:undefined,pending:async()=>true,invoke}).cancel()).toBe(true);
 expect(invoke).toHaveBeenCalledExactlyOnceWith('mw_background_replay_schedule',{enabled:false});
});
