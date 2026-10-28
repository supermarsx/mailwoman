import { For, type JSX } from 'solid-js';
import { hrefFor } from './router.ts';
import { horizontalWheel } from '../../../components/horizontalWheel.ts';
import * as css from './sectionTabs.css.ts';
export function SectionTabs(props:{label:string;items:readonly {id:string;label:string;path:string}[];selected:string;navigate:(path:string)=>void}):JSX.Element {
 return <nav class={css.tabs} aria-label={props.label} ref={horizontalWheel}><div class={css.strip}><For each={props.items}>{item=><a href={hrefFor(item.path)} aria-current={props.selected===item.id?'page':undefined} onClick={event=>{if(event.button!==0||event.ctrlKey||event.metaKey||event.shiftKey||event.altKey)return;event.preventDefault();props.navigate(item.path);}}>{item.label}</a>}</For></div></nav>;
}
