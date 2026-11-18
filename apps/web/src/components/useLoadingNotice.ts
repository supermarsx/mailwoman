import {createEffect,onCleanup,type Accessor} from 'solid-js';
import {t} from '../i18n';
import {toast} from './toast/toast.ts';
import {alertValue} from '../api/notificationControls.ts';

export const SLOW_LOADING_DELAY_MS=5_000;
const noticeId='page-still-loading';
const pending=new Set<symbol>();

/** One dismissible notice for overlapping slow loads; never interrupts fast work. */
export function useLoadingNotice(active:Accessor<boolean>):void {
 createEffect(()=>{
  if(!active())return;
  const token=Symbol();
  const timer=setTimeout(()=>{
   const first=pending.size===0;
   pending.add(token);
   if(first&&alertValue('notifications.toasts.enabled',true)&&alertValue('notifications.toasts.info',true))toast.info(t('common-loading-still'),{id:noticeId,duration:Infinity,busy:true,respectPreferences:true,testId:'slow-loading-toast'});
  },SLOW_LOADING_DELAY_MS);
  onCleanup(()=>{
   clearTimeout(timer);
   if(pending.delete(token)&&pending.size===0)toast.dismiss(noticeId);
  });
 });
}
