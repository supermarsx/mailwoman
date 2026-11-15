import {createSignal} from 'solid-js';
import type {NotificationConfig} from '../screens/Settings/types.ts';
import {observePreferenceAccount} from './accountPreferenceLifecycle.ts';
import {shouldNotify} from './notificationPolicy.ts';
const [config,setConfig]=createSignal<NotificationConfig>();
observePreferenceAccount(()=>setConfig(undefined));
export function configureNotificationRuntime(value:NotificationConfig):void{setConfig(value);}
/** Connection success toasts obey quiet hours without treating mail-matching
 * notification rules as connection rules. Persistent status remains visible. */
export function connectionToastAllowed():boolean{const value=config();return !value||shouldNotify({...value,enabled:true,rules:[]},{});}
