/** The same-origin Web Lock spans the queue check and OS scheduler command.
 * A foreground enqueue may race a hidden runner's exit: its registration must
 * happen after a completed runner removes the old job, never before it. */
export interface NativeScheduleLocks {
  request<T>(name:string,callback:()=>Promise<T>):Promise<T>;
}
interface Scheduler {
  locks:NativeScheduleLocks|undefined;
  pending:()=>Promise<boolean>;
  invoke:(command:'mw_background_replay_schedule'|'mw_background_replay_complete',args:Record<string,unknown>)=>Promise<{supported:boolean}>;
}
const LOCK='mw-native-background-schedule';
export function nativeReplayScheduler(deps:Scheduler){
  return {
    async cancel():Promise<boolean>{
      const cancel=async()=>(await deps.invoke('mw_background_replay_schedule',{enabled:false})).supported;
      return deps.locks?deps.locks.request(LOCK,cancel):cancel();
    },
    async schedule():Promise<boolean>{
      // A per-process mutex cannot protect another WebView process. Older
      // runtimes keep foreground replay instead of promising reliable OS jobs.
      if(!deps.locks)return false;
      return deps.locks.request(LOCK,async()=>
        (await deps.invoke('mw_background_replay_schedule',{enabled:await deps.pending()})).supported,
      );
    },
    async finish(complete:boolean):Promise<void>{
      if(!deps.locks){await deps.invoke('mw_background_replay_complete',{complete:false});return;}
      await deps.locks.request(LOCK,async()=>{
        const pending=await deps.pending();
        await deps.invoke('mw_background_replay_complete',{complete:complete&&!pending});
      });
    },
  };
}
