import {Show,onMount,type JSX} from 'solid-js';
import {Icon} from './Icon.tsx';
import {loadCatalog,t} from '../i18n';
import {priorityMarkStyle,showsPriorityMark,type MessagePriority} from './messagePriority.ts';
import * as css from './priorityMark.css.ts';

/** High / low priority on a message row, the reader header and a conversation row. Icon plus a name for assistive tech; the words are also visible in the `iconText` style. Draws nothing for normal mail or when `mail.priorityMarks` hides it. */
export function PriorityMark(props:{priority:MessagePriority;/** reader header: always with words */badge?:boolean}):JSX.Element {
 onMount(()=>{void loadCatalog('properties');});
 const label=()=>props.priority==='high'?t('priority-high'):t('priority-low');
 const words=()=>props.badge===true||priorityMarkStyle()==='iconText';
 return <Show when={showsPriorityMark(props.priority)}>
  <span class={props.priority==='high'?css.high:css.low} data-testid={`priority-${props.priority}`} data-priority={props.priority} data-priority-style={priorityMarkStyle()} role="img" aria-label={label()} title={label()}>
   <Show when={priorityMarkStyle()==='bar'&&props.badge!==true} fallback={<>
    <span class={props.priority==='low'?css.lowIcon:undefined} aria-hidden="true">{props.priority==='high'?<Icon name="alert"/>:<Icon name="chevron"/>}</span>
    <Show when={words()}><span class={css.text} aria-hidden="true">{label()}</span></Show>
   </>}><span class={`${css.bar} ${props.priority==='high'?css.barHigh:css.barLow}`} aria-hidden="true"/></Show>
  </span>
 </Show>;
}
