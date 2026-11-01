import type {JSX} from 'solid-js';
import {Icon,type IconName} from './Icon.tsx';
import * as css from './iconSelect.css.ts';

/** Keep native mouse, touch and keyboard selection with an icon-only closed control. */
export function IconSelect(props:{label:string;currentLabel:string;icon:IconName;value:string;onChange:(value:string)=>void;children:JSX.Element}) {
 return <label class={css.root} title={`${props.label}: ${props.currentLabel}`}>
  <Icon name={props.icon}/><span class={css.chevron}><Icon name="chevronDown"/></span>
  <select class={css.select} aria-label={props.label} value={props.value} onChange={event=>props.onChange(event.currentTarget.value)}>{props.children}</select>
 </label>;
}
