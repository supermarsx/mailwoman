import {preferenceAccount,observePreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import {refreshSettings} from '../api/settings.ts';
import {refreshAccountUiPreferences} from '../api/accountPreferences.ts';
import {appearanceSync} from '../api/prefs.ts';
import {SettingsService} from '../screens/Settings/service.ts';

/** How soon after an account became active its stream's first open skips the refresh (t52w). */
export const FIRST_OPEN_SKIP_MS=10_000;
/** Coalesce bursts and keep one trailing refresh if a write lands during a read. */
export function createPreferenceRefresh(work:()=>Promise<unknown>=async()=>{
  await Promise.allSettled([refreshSettings(),refreshAccountUiPreferences(),appearanceSync()?.refresh(),new SettingsService().notifications()]);
}) {
  let timer:ReturnType<typeof setTimeout>|undefined,busy=false,again=false,disposed=false;
  const changed=():void=>{
    if(disposed||!preferenceAccount())return;
    if(busy){again=true;return;}
    if(timer!==undefined)return;
    timer=setTimeout(()=>{timer=undefined;void run();},80);
  };
  async function run():Promise<void>{
    if(disposed||!preferenceAccount())return;
    busy=true;
    try{await work();}catch{/* Reconnect/focus and the next invalidation retry. */}
    finally{busy=false;if(again){again=false;changed();}}
  }
  // t52w: the stream's first open after an account became active used to repeat
  // the settings, preferences, appearance and notification reads the load had
  // just made. That one is skipped within FIRST_OPEN_SKIP_MS of the activation;
  // a reconnect, the poll fallback, a wake and any later open always refresh.
  let activeSince=0,caughtUpOnce=false;
  const stopOwner=observePreferenceAccount(account=>{clearTimeout(timer);timer=undefined;again=false;activeSince=account?Date.now():0;caughtUpOnce=false;});
  const caughtUp=(firstOpen:boolean):void=>{
    const first=!caughtUpOnce;caughtUpOnce=true;
    if(firstOpen&&first&&activeSince>0&&Date.now()-activeSince<FIRST_OPEN_SKIP_MS)return;
    changed();
  };
  const visible=()=>{if(document.visibilityState==='visible')changed();};
  window.addEventListener('focus',visible);document.addEventListener('visibilitychange',visible);
  return {changed,caughtUp,dispose(){disposed=true;clearTimeout(timer);stopOwner();window.removeEventListener('focus',visible);document.removeEventListener('visibilitychange',visible);}};
}
