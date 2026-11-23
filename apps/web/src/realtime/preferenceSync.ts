import {preferenceAccount,observePreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import {refreshSettings} from '../api/settings.ts';
import {refreshAccountUiPreferences} from '../api/accountPreferences.ts';
import {appearanceSync} from '../api/prefs.ts';
import {SettingsService} from '../screens/Settings/service.ts';

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
  const stopOwner=observePreferenceAccount(()=>{clearTimeout(timer);timer=undefined;again=false;});
  const visible=()=>{if(document.visibilityState==='visible')changed();};
  window.addEventListener('focus',visible);document.addEventListener('visibilitychange',visible);
  return {changed,dispose(){disposed=true;clearTimeout(timer);stopOwner();window.removeEventListener('focus',visible);document.removeEventListener('visibilitychange',visible);}};
}
