import {settings} from '../api/settings.ts';
import {preferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import {toast} from '../components/toast/toast.ts';
import {loadCatalog,t} from '../i18n';
import {SHORTCUT_ACTIONS,type ShortcutAction} from './custom.ts';
export const SHORTCUT_TOAST_KEY='shortcuts.activationToast';
/** Acknowledges the accepted key gesture, not completion of its possibly asynchronous action. */
export function notifyShortcutActivation(action:string,event:KeyboardEvent):void{
 if(event.repeat||settings()[SHORTCUT_TOAST_KEY]?.value!==true||!SHORTCUT_ACTIONS.includes(action as ShortcutAction)&&action!=='undo'&&action!=='redo')return;
 const owner=preferenceAccount();
 const keys=[event.ctrlKey?'Ctrl':'',event.metaKey?'⌘':'',event.altKey?'Alt':'',event.shiftKey?'Shift':'',({ArrowDown:'↓',ArrowUp:'↑',ArrowLeft:'←',ArrowRight:'→',Escape:'Esc',' ':'Space'} as Record<string,string>)[event.key]??(event.key.length===1?event.key.toUpperCase():event.key)].filter(Boolean).join('+');
 void Promise.all([loadCatalog('settings'),...(action==='undo'||action==='redo'?[loadCatalog('undo')]:[])]).then(()=>{
  if(owner!==preferenceAccount()||settings()[SHORTCUT_TOAST_KEY]?.value!==true)return;
  toast.info(t('settings-shortcut-activated',{action:action==='undo'?t('undo-button'):action==='redo'?t('redo-button'):t(`settings-kbd-action-${action}`),keys}),{id:'keyboard-shortcut-activation',duration:2000,testId:'shortcut-activation-toast'});
 }).catch(()=>undefined);
}
