import {createSignal,onMount,onCleanup} from 'solid-js';
import {createContextMenu} from './ContextMenu.tsx';
import {observePreferenceAccount} from '../api/accountPreferenceLifecycle.ts';

/** Keep native selects as accessible form controls, while every dropdown opens the app menu. */
export function ThemedSelects(){
 const [control,setControl]=createSignal<HTMLSelectElement>();
 const label=()=>{const select=control();if(!select)return '';const ids=select.getAttribute('aria-labelledby');return select.getAttribute('aria-label')||(ids?ids.split(/\s+/).map(id=>select.ownerDocument.getElementById(id)?.textContent??'').join(' '):'')||Array.from(select.labels??[]).map(item=>item.textContent??'').join(' ')||select.title;};
 const menu=createContextMenu({label,scope:()=>control()?.id??label(),focusCurrent:true,ownerDocument:()=>control()?.ownerDocument??document,actions:()=>{
   const select=control();return select?Array.from(select.options).filter(option=>!option.hidden).map((option,index)=>({id:String(index),label:option.textContent??option.label,icon:'check' as const,omitIcon:true,radio:true,current:option.selected,checked:option.selected,disabled:option.disabled||option.parentElement?.matches('optgroup:disabled')===true,run:()=>{
     if(!select.isConnected||select.matches(':disabled')||option.disabled||option.parentElement?.matches('optgroup:disabled'))return;
     if(select.value!==option.value){select.value=option.value;const EventClass=select.ownerDocument.defaultView?.Event??Event;select.dispatchEvent(new EventClass('input',{bubbles:true}));select.dispatchEvent(new EventClass('change',{bubbles:true}));}
   }})):[];
 }});
 onMount(()=>{
   let pressed:HTMLSelectElement|undefined;
   const find=(event:Event)=>{const target=event.target;if(!(target instanceof Element))return;const select=target.closest('select');return select&&!select.multiple&&select.size<=1&&!select.matches(':disabled')?select:undefined;};
   const open=(select:HTMLSelectElement)=>{if(menu.isOpen()&&control()===select){menu.close();return;}menu.close(false);setControl(select);select.focus();menu.openBelow(select);};
   const pointer=(event:MouseEvent|PointerEvent)=>{const select=find(event);if(!select||event.button!==0)return;event.preventDefault();event.stopPropagation();pressed=select;open(select);};
   // A physical activation produces pointerdown, mousedown and click. Toggle
   // only once; the compatibility mouse events must not reopen a closed menu.
   const mouse=(event:MouseEvent)=>{const select=find(event);if(!select||event.button!==0)return;event.preventDefault();event.stopPropagation();if(event.type==='click'){const handled=pressed===select;pressed=undefined;if(!handled)open(select);}else if(pressed!==select){pressed=select;open(select);}};
   const keyboard=(event:KeyboardEvent)=>{const select=find(event);if(!select||!['ArrowDown','ArrowUp',' ','Enter','F4'].includes(event.key))return;event.preventDefault();event.stopPropagation();if(!menu.isOpen())open(select);};
   document.addEventListener('pointerdown',pointer,true);document.addEventListener('mousedown',mouse,true);document.addEventListener('click',mouse,true);document.addEventListener('keydown',keyboard,true);
   const stop=observePreferenceAccount(()=>menu.close(false));
   const observer=new MutationObserver(()=>{if(control()&&(!control()!.isConnected||control()!.matches(':disabled')))menu.close(false);});observer.observe(document.body,{childList:true,subtree:true,attributes:true,attributeFilter:['disabled']});
   onCleanup(()=>{stop();observer.disconnect();document.removeEventListener('pointerdown',pointer,true);document.removeEventListener('mousedown',mouse,true);document.removeEventListener('click',mouse,true);document.removeEventListener('keydown',keyboard,true);});
 });
 return menu.menu();
}
