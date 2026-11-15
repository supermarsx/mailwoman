import {createEffect,createSignal,splitProps,Show,type JSX} from 'solid-js';
import {Icon} from './Icon.tsx';
import {loadCatalog,t} from '../i18n';
import * as css from './searchInput.css.ts';
void loadCatalog('mail');
/** A native search input with a browser-independent clear action. */
export function SearchInput(props:JSX.InputHTMLAttributes<HTMLInputElement>){
 const [local,rest]=splitProps(props,['ref']);const [text,setText]=createSignal(String(props.value??''));let input:HTMLInputElement|undefined;
 createEffect(()=>{setText(String(props.value??''));});
 const clear=()=>{if(!input||input.matches(':disabled')||input.readOnly)return;input.value='';setText('');const EventClass=input.ownerDocument.defaultView?.Event??Event;input.dispatchEvent(new EventClass('input',{bubbles:true}));input.dispatchEvent(new EventClass('change',{bubbles:true}));input.focus();};
 return <span class={css.root} onInput={event=>setText((event.target as HTMLInputElement).value)}><input {...rest} type="search" ref={element=>{input=element;if(typeof local.ref==='function')local.ref(element);}}/><Show when={text()&&!props.disabled&&!props.readOnly}><button class={css.clear} type="button" aria-label={`${t('mail-search-clear')}: ${props['aria-label']??props.placeholder??''}`} title={t('mail-search-clear')} onPointerDown={event=>event.preventDefault()} onClick={clear}><Icon name="close"/></button></Show></span>;
}
