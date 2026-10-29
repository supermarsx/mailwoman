import {Show,type JSX} from 'solid-js';
import {settings} from '../api/settings.ts';
import {Icon,type IconName} from './Icon.tsx';
import * as a11y from './mailA11y.css.ts';
import './reader-action.css';
/** One reader action-row button. `badge` is a small count drawn over the icon
 *  (decorative: the count is part of `label`); `popup` marks a button that opens
 *  a dialog or menu (t57). */
export function ReaderAction(props:{icon:IconName;label:string;help:string;testId?:string;disabled?:boolean;pressed?:boolean|undefined;badge?:number|undefined;popup?:'dialog'|'menu';expanded?:boolean;onClick:(event:MouseEvent)=>void}):JSX.Element {
 return <button type="button" class={`btn btn--ghost reader-action ${a11y.focusable}`} classList={{'reader-action--compact':settings()['reader.actionStyle']?.value!=='labels'}} aria-label={props.label} title={props.help} aria-pressed={props.pressed} aria-haspopup={props.popup} aria-expanded={props.popup?props.expanded===true:undefined} disabled={props.disabled} data-testid={props.testId} onClick={event=>props.onClick(event)}><Icon name={props.icon}/><Show when={props.badge!==undefined&&props.badge>0}><span class="reader-action__badge" aria-hidden="true">{props.badge!>99?'99+':props.badge}</span></Show><span class={settings()['reader.actionStyle']?.value==='labels'?'':a11y.srOnly}>{props.label}</span></button>;
}
