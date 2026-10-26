import {createMemo,createSignal,createUniqueId,For,Show,type JSX} from 'solid-js';
import * as css from './searchableSelect.css.ts';

/** Editable combobox: callers validate free text; options remain keyboard searchable. */
export function SearchableSelect(props:{label:string;value:string;options:readonly {value:string;label:string}[];disabled?:boolean;describedBy?:string;onChange:(value:string)=>void}):JSX.Element {
  const id=createUniqueId(),[open,setOpen]=createSignal(false),[active,setActive]=createSignal(0);
  const matches=createMemo(()=>{const query=props.value.toLocaleLowerCase().replaceAll('_',' ');return props.options.filter(option=>`${option.value} ${option.label}`.toLocaleLowerCase().replaceAll('_',' ').includes(query));});
  const choose=(value:string)=>{props.onChange(value);setOpen(false);setActive(0);};
  return <div class={css.root}>
    <label for={id}>{props.label}</label>
    <input id={id} role="combobox" autocomplete="off" aria-autocomplete="list" aria-expanded={open()} aria-controls={`${id}-options`} aria-activedescendant={open()&&matches()[active()]?`${id}-option-${active()}`:undefined} aria-describedby={props.describedBy} disabled={props.disabled} value={props.value} maxlength={64}
      onFocus={()=>setOpen(true)} onBlur={()=>setOpen(false)}
      onInput={event=>{props.onChange(event.currentTarget.value);setActive(0);setOpen(true);}}
      onKeyDown={event=>{if(event.key==='Escape'){setOpen(false);event.preventDefault();}else if(event.key==='ArrowDown'||event.key==='ArrowUp'){event.preventDefault();setOpen(true);setActive(index=>Math.max(0,Math.min(matches().length-1,index+(event.key==='ArrowDown'?1:-1))));queueMicrotask(()=>document.getElementById(`${id}-option-${active()}`)?.scrollIntoView?.({block:'nearest'}));}else if(event.key==='Enter'&&open()&&matches()[active()]){event.preventDefault();choose(matches()[active()]!.value);}}}/>
    <Show when={open()&&!props.disabled&&matches().length}><div id={`${id}-options`} role="listbox" aria-label={props.label} class={css.options}><For each={matches()}>{(option,index)=><div id={`${id}-option-${index()}`} role="option" aria-selected={index()===active()} class={css.option} onPointerDown={event=>event.preventDefault()} onClick={()=>choose(option.value)}>{option.label}</div>}</For></div></Show>
  </div>;
}
