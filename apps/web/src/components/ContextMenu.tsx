import {createEffect,createSignal,For,onCleanup,onMount,Show,type Accessor,type JSX} from 'solid-js';
import {Portal} from 'solid-js/web';
import {Icon,type IconName} from './Icon.tsx';
import * as css from './contextMenu.css.ts';
export interface ContextAction {id:string;label:string;icon:IconName;disabled?:boolean;run:()=>void|Promise<void>}
export function clampMenu(x:number,y:number,width:number,height:number,viewportWidth:number,viewportHeight:number):{left:number;top:number} {
 return {left:Math.max(8,Math.min(x,viewportWidth-width-8)),top:Math.max(8,Math.min(y,viewportHeight-height-8))};
}
/** The scope is captured at open and rechecked at execution, including identity reuse. */
export function createContextMenu(options:{label:Accessor<string>;scope:Accessor<string>;actions:Accessor<ContextAction[]>;onError?:()=>void;restoreFocus?:()=>void}) {
 const [position,setPosition]=createSignal<{x:number;y:number;scope:string;trigger:HTMLElement}>();
 function close(restore=true):void {const old=position();setPosition(undefined);if(restore){if(old?.trigger.isConnected)old.trigger.focus();else options.restoreFocus?.();}}
 function open(event:MouseEvent|KeyboardEvent):void {
  const target=event.target;if(target instanceof Element&&target.closest('input,textarea,[contenteditable="true"]'))return;
  if(window.getSelection()?.toString()||options.actions().length===0)return;
  const trigger=(target instanceof Element?target.closest<HTMLElement>('button,[tabindex]'):null)??event.currentTarget as HTMLElement;const rect=trigger.getBoundingClientRect();
  event.preventDefault();event.stopPropagation();document.dispatchEvent(new Event('mailwoman-context-open'));setPosition({x:event instanceof MouseEvent&&event.type==='contextmenu'?event.clientX:rect.left,y:event instanceof MouseEvent&&event.type==='contextmenu'?event.clientY:rect.bottom,scope:options.scope(),trigger});
 }
 function keydown(event:KeyboardEvent):void {if(event.key==='ContextMenu'||(event.shiftKey&&event.key==='F10'))open(event);}
 createEffect(()=>{const value=position();if(value&&value.scope!==options.scope())close(false);});
 function MenuBody(props:{point:NonNullable<ReturnType<typeof position>>}):JSX.Element {
  let element!:HTMLDivElement;const [coords,setCoords]=createSignal({left:8,top:8});
  const buttons=()=>Array.from(element.querySelectorAll<HTMLButtonElement>('button:not([aria-disabled="true"])'));
  onMount(()=>{const rect=element.getBoundingClientRect();setCoords(clampMenu(props.point.x,props.point.y,rect.width,rect.height,window.innerWidth,window.innerHeight));buttons()[0]?.focus({preventScroll:true});
   const outside=(e:PointerEvent):void=>{if(e.target instanceof Node&&!element.contains(e.target))close(false);};const dismiss=(e:Event):void=>{if(e.target instanceof Node&&element.contains(e.target))return;close(false);};
   document.addEventListener('mailwoman-context-open',dismiss);document.addEventListener('pointerdown',outside,true);window.addEventListener('resize',dismiss);window.addEventListener('scroll',dismiss,true);
   onCleanup(()=>{document.removeEventListener('mailwoman-context-open',dismiss);document.removeEventListener('pointerdown',outside,true);window.removeEventListener('resize',dismiss);window.removeEventListener('scroll',dismiss,true);});
  });
  function navigate(e:KeyboardEvent):void {if(e.key==='Escape'){e.preventDefault();e.stopPropagation();close();return;}if(e.key==='Tab'){close();return;}const keys=['ArrowDown','ArrowUp','Home','End'];if(!keys.includes(e.key))return;e.preventDefault();e.stopPropagation();const rows=buttons(),at=rows.indexOf(document.activeElement as HTMLButtonElement);const next=e.key==='Home'?0:e.key==='End'?rows.length-1:(at+(e.key==='ArrowDown'?1:-1)+rows.length)%rows.length;const target=rows[next];if(target){target.focus({preventScroll:true});const item=target.getBoundingClientRect(),panel=element.getBoundingClientRect();if(item.bottom>panel.bottom)element.scrollTop+=item.bottom-panel.bottom;else if(item.top<panel.top)element.scrollTop-=panel.top-item.top;}}
  return <div ref={element} role="menu" aria-label={options.label()} class={css.menu} style={{left:`${coords().left}px`,top:`${coords().top}px`}} onKeyDown={navigate} onContextMenu={e=>{e.preventDefault();e.stopPropagation();}} onClick={e=>e.stopPropagation()}><For each={options.actions().map(action=>action.id)}>{id=>{const action=()=>options.actions().find(item=>item.id===id)!;return <button type="button" role="menuitem" class={css.item} aria-disabled={action().disabled===true} onClick={()=>{const current=options.actions().find(item=>item.id===id);if(props.point.scope!==options.scope()||!current||current.disabled){close(false);return;}close();try{void Promise.resolve(current.run()).catch(()=>options.onError?.());}catch{options.onError?.();}}}><Icon name={action().icon}/>{action().label}</button>;}}</For></div>;
 }
 const menu=():JSX.Element=><Show when={position()}>{point=><Portal><MenuBody point={point()}/></Portal>}</Show>;
 return {open,keydown,close,menu,isOpen:()=>position()!==undefined};
}
