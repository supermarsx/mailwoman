/** Shell-injected launch mode only; this is never account/session authority. */
interface NativeBackgroundConfig {native?:boolean;backgroundReplay?:boolean}
const config=()=>(globalThis as {__MW_CONFIG__?:NativeBackgroundConfig}).__MW_CONFIG__;
export const nativeBackgroundStartup=():boolean=>config()?.native===true&&config()?.backgroundReplay===true;

/** Browser and ordinary native foreground boots never load the replay runner. */
export async function startNativeBackgroundOnStartup():Promise<void>{
 if(!nativeBackgroundStartup())return;
 let runner:typeof import('./nativeBackground.ts');
 try{runner=await import('./nativeBackground.ts');}
 catch(error){
  // Loading failed BEFORE replay/finally could run. Keep the job incomplete and
  // preserve the existing cross-WebView scheduling lock. No queue or identity
  // is read here. Completion is best-effort; the original load error survives.
  try{
   const [{invokeBackground},{nativeReplayScheduler}]=await Promise.all([
    import('../platform/tauri.ts'),import('./nativeReplaySchedule.ts'),
   ]);
   await nativeReplayScheduler({locks:globalThis.navigator?.locks,pending:async()=>true,invoke:invokeBackground}).finish(false);
  }catch{/* An unavailable shell/chunk must never be reported as completed. */}
  throw error;
 }
 // Execution failures already run the runner's own finally. Never finish twice.
 await runner.startNativeBackgroundReplay();
}
