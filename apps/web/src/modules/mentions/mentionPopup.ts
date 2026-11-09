import type {ContactSuggestion} from '../contacts/autocomplete.ts';
import {t} from '../../i18n/index.ts';
import * as css from './mentionPopup.css.ts';
let nextId=0;
export function mentionPopup(anchor:HTMLElement,pick:(value:ContactSuggestion)=>void,dismiss?:()=>void){
 const doc=anchor.ownerDocument,list=doc.createElement('ul');list.id=`mw-mentions-${++nextId}`;list.className=css.popup;list.setAttribute('role','listbox');list.setAttribute('aria-label',t('mentions-suggestions'));list.dataset['testid']='compose-mentions';
 let values:ContactSuggestion[]=[],index=0;
 function close():void{values=[];list.remove();for(const name of ['aria-autocomplete','aria-controls','aria-expanded','aria-activedescendant'])anchor.removeAttribute(name);}
 function select(i:number):void{const value=values[i];if(value){close();pick(value);}}
 function paint():void{list.replaceChildren(...values.map((value,i)=>{const item=doc.createElement('li');item.id=`${list.id}-${i}`;item.className=css.option;item.setAttribute('role','option');item.setAttribute('aria-selected',String(index===i));item.textContent=value.name?`${value.name} <${value.email}>`:value.email;item.addEventListener('pointerdown',event=>event.preventDefault());item.addEventListener('click',()=>select(i));return item;}));anchor.setAttribute('aria-activedescendant',`${list.id}-${index}`);list.children[index]?.scrollIntoView?.({block:'nearest'});}
 function show(suggestions:ContactSuggestion[],coords:{left:number;bottom:number}):void{values=suggestions;index=0;if(!values.length){close();return;}const win=doc.defaultView!;list.style.left=`${Math.max(8,Math.min(coords.left,win.innerWidth-Math.min(352,win.innerWidth-32)-8))}px`;list.style.top=`${Math.max(8,Math.min(coords.bottom+4,win.innerHeight-260))}px`;anchor.setAttribute('aria-autocomplete','list');anchor.setAttribute('aria-controls',list.id);anchor.setAttribute('aria-expanded','true');paint();doc.body.append(list);}
 function key(event:KeyboardEvent):boolean{if(!values.length||event.isComposing)return false;if(['ArrowDown','ArrowUp'].includes(event.key)){index=(index+(event.key==='ArrowDown'?1:-1)+values.length)%values.length;paint();event.preventDefault();return true;}if(event.key==='Enter'||event.key==='Tab'){event.preventDefault();select(index);return true;}if(event.key==='Escape'){event.preventDefault();close();dismiss?.();return true;}return false;}
 const outside=(event:PointerEvent):void=>{const target=event.target as Node|null;if(target&&!anchor.contains(target)&&!list.contains(target)){close();dismiss?.();}};doc.addEventListener('pointerdown',outside,true);
 return {show,close,key,destroy:()=>{close();doc.removeEventListener('pointerdown',outside,true);}};
}
