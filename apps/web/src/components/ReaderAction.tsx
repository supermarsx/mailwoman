import type {JSX} from 'solid-js';
import {settings} from '../api/settings.ts';
import {Icon,type IconName} from './Icon.tsx';
import * as a11y from './mailA11y.css.ts';
import './reader-action.css';
export function ReaderAction(props:{icon:IconName;label:string;help:string;testId?:string;disabled?:boolean;pressed?:boolean;onClick:()=>void}):JSX.Element {
 return <button type="button" class={`btn btn--ghost reader-action ${a11y.focusable}`} classList={{'reader-action--compact':settings()['reader.actionStyle']?.value!=='labels'}} aria-label={props.label} title={props.help} aria-pressed={props.pressed} disabled={props.disabled} data-testid={props.testId} onClick={props.onClick}><Icon name={props.icon}/><span class={settings()['reader.actionStyle']?.value==='labels'?'':a11y.srOnly}>{props.label}</span></button>;
}
