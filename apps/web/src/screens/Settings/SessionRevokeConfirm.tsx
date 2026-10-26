import {createSignal,createUniqueId,type JSX} from 'solid-js';
import {Portal} from 'solid-js/web';
import {createFocusTrap} from '../../components/a11y';
import {t} from '../../i18n';
import * as css from './styles.css.ts';
import * as layout from './editor.css.ts';
export function SessionRevokeConfirm(props:{count:number;busy:boolean;onCancel:()=>void;onConfirm:()=>Promise<void>}):JSX.Element{
 const id=createUniqueId();const [element,setElement]=createSignal<HTMLDivElement>();let cancel:HTMLButtonElement|undefined;
 const close=():void=>{if(!props.busy)props.onCancel();};
 createFocusTrap(element,{onEscape:close,initialFocus:()=>cancel});
 return <Portal><div class={layout.scrim}/><div ref={setElement} class={layout.dialog} role="alertdialog" aria-modal="true" aria-labelledby={`${id}-title`} aria-describedby={`${id}-body`} tabindex="-1">
  <h3 id={`${id}-title`}>{t('settings-sessions-selected-title',{count:props.count})}</h3>
  <p id={`${id}-body`}>{t('settings-sessions-selected-confirm')}</p>
  <div class={css.actions}><button ref={cancel} class={css.button} disabled={props.busy} onClick={close}>{t('settings-sessions-cancel')}</button><button class={css.danger} disabled={props.busy} onClick={()=>{if(!props.busy)void props.onConfirm();}}>{t('settings-sessions-selected-action')}</button></div>
 </div></Portal>;
}
