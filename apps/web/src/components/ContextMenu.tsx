import {createEffect,createSignal,For,onCleanup,onMount,Show,type Accessor,type JSX} from 'solid-js';
import {Portal} from 'solid-js/web';
import {Icon,type IconName} from './Icon.tsx';
import * as css from './contextMenu.css.ts';
/** `checked` marks an on/off action (a label); it is then a menuitemcheckbox that says which.
 * `current` draws a trailing check (with `checked`, for one-of-several choices).
 * `separatorBefore` draws a separator above the item, to group related actions; arrow keys skip it. */
export interface ContextAction {id:string;label:string;icon:IconName;disabled?:boolean;checked?:boolean;separatorBefore?:boolean;
 /** A chosen option among several (a radio-like menu, t57): a trailing check is drawn; announced through `checked`. */
 current?:boolean;children?:ContextAction[];run:()=>void|Promise<void>}
export function clampMenu(x:number,y:number,width:number,height:number,viewportWidth:number,viewportHeight:number):{left:number;top:number} {
 return {left:Math.max(8,Math.min(x,viewportWidth-width-8)),top:Math.max(8,Math.min(y,viewportHeight-height-8))};
}
/** The scope is captured at open and rechecked at execution, including identity reuse.
 * `restoreFocus` is given the trigger that is gone: an action that re-renders its row replaces the trigger, and focus would fall to the body. */
export function createContextMenu(options:{label:Accessor<string>;scope:Accessor<string>;actions:Accessor<ContextAction[]>;onError?:()=>void;restoreFocus?:(trigger?:HTMLElement)=>void}) {
 const [position,setPosition]=createSignal<{x:number;y:number;scope:string;trigger:HTMLElement;above?:boolean}>();
 function close(restore=true):void {const old=position();setPosition(undefined);if(restore){if(old?.trigger.isConnected)old.trigger.focus();else options.restoreFocus?.(old?.trigger);}}
 function open(event:MouseEvent|KeyboardEvent):void {
  const target=event.target;if(target instanceof Element&&target.closest('input,textarea,[contenteditable="true"]'))return;
  if(window.getSelection()?.toString()||options.actions().length===0)return;
  const trigger=(target instanceof Element?target.closest<HTMLElement>('button,[tabindex]'):null)??event.currentTarget as HTMLElement;const rect=trigger.getBoundingClientRect();
  event.preventDefault();event.stopPropagation();document.dispatchEvent(new Event('mailwoman-context-open'));setPosition({x:event instanceof MouseEvent&&event.type==='contextmenu'?event.clientX:rect.left,y:event instanceof MouseEvent&&event.type==='contextmenu'?event.clientY:rect.bottom,scope:options.scope(),trigger});
 }
 /** Open from a control near the bottom of its pane (the selection bar, t58): the menu opens upward, its bottom edge just above the trigger. */
 function openAbove(trigger:HTMLElement):void {
  if(options.actions().length===0)return;const rect=trigger.getBoundingClientRect();
  document.dispatchEvent(new Event('mailwoman-context-open'));setPosition({x:rect.left,y:rect.top-4,scope:options.scope(),trigger,above:true});
 }
 function keydown(event:KeyboardEvent):void {if(event.key==='ContextMenu'||(event.shiftKey&&event.key==='F10'))open(event);}
 createEffect(()=>{const value=position();if(value&&value.scope!==options.scope())close(false);});
 function MenuBody(props:{point:NonNullable<ReturnType<typeof position>>}):JSX.Element {
  let container!:HTMLDivElement;
  onMount(()=>{
   const outside=(e:PointerEvent):void=>{if(e.target instanceof Node&&!container.contains(e.target))close(false);};
   const dismiss=(e:Event):void=>{if(e.target instanceof Node&&container.contains(e.target))return;close(false);};
   document.addEventListener('mailwoman-context-open',dismiss);document.addEventListener('pointerdown',outside,true);window.addEventListener('resize',dismiss);window.addEventListener('scroll',dismiss,true);
   onCleanup(()=>{document.removeEventListener('mailwoman-context-open',dismiss);document.removeEventListener('pointerdown',outside,true);window.removeEventListener('resize',dismiss);window.removeEventListener('scroll',dismiss,true);});
  });
  function Panel(panel:{actions:Accessor<ContextAction[]>;label:string;anchor?:HTMLButtonElement;back?:()=>void;valid:()=>boolean}):JSX.Element {
   let element!:HTMLDivElement;
   const [coords,setCoords]=createSignal({left:8,top:8});
   const [child,setChild]=createSignal<{id:string;anchor:HTMLButtonElement}>();
   const buttons=()=>Array.from(element.querySelectorAll<HTMLButtonElement>(':scope > button:not([aria-disabled="true"])'));
   const dismissChild=()=>{const old=child();setChild(undefined);old?.anchor.focus({preventScroll:true});};
   const childAction=()=>panel.actions().find(action=>action.id===child()?.id);
   createEffect(()=>{if(child()&&(!childAction()?.children?.length||childAction()?.disabled))dismissChild();});
   onMount(()=>{
    const rect=element.getBoundingClientRect(),anchor=panel.anchor?.getBoundingClientRect();
    const rtl=getComputedStyle(element).direction==='rtl';
    let x=anchor?(rtl?anchor.left-rect.width:anchor.right):props.point.x;
    if(anchor&&(x+rect.width>window.innerWidth-8||x<8))x=rtl?anchor.right:anchor.left-rect.width;
    setCoords(clampMenu(x,anchor?anchor.top:props.point.above?props.point.y-rect.height:props.point.y,rect.width,rect.height,window.innerWidth,window.innerHeight));
    buttons()[0]?.focus({preventScroll:true});
   });
   function activate(id:string,button:HTMLButtonElement):void {
    const current=panel.actions().find(item=>item.id===id);
    if(props.point.scope!==options.scope()||!panel.valid()||!current||current.disabled){close(false);return;}
    if(current.children?.length){setChild({id,anchor:button});return;}
    const trigger=props.point.trigger;close();
    try{void Promise.resolve(current.run()).catch(()=>options.onError?.());}catch{options.onError?.();}
    queueMicrotask(()=>{if(!trigger.isConnected&&(document.activeElement===null||document.activeElement===document.body))options.restoreFocus?.(trigger);});
   }
   function navigate(e:KeyboardEvent):void {
    const rtl=getComputedStyle(element).direction==='rtl';
    if(e.key==='Escape'||(panel.back&&e.key===(rtl?'ArrowRight':'ArrowLeft'))){e.preventDefault();e.stopPropagation();if(panel.back)panel.back();else close();return;}
    if(e.key==='Tab'){close();return;}
    if(e.key===(rtl?'ArrowLeft':'ArrowRight')){const button=document.activeElement as HTMLButtonElement;const action=panel.actions().find(item=>item.id===button.dataset['actionId']);if(action?.children?.length){e.preventDefault();e.stopPropagation();activate(action.id,button);}return;}
    const keys=['ArrowDown','ArrowUp','Home','End'];if(!keys.includes(e.key))return;
    e.preventDefault();e.stopPropagation();setChild(undefined);const rows=buttons(),at=rows.indexOf(document.activeElement as HTMLButtonElement);const next=e.key==='Home'?0:e.key==='End'?rows.length-1:(at+(e.key==='ArrowDown'?1:-1)+rows.length)%rows.length;
    const target=rows[next];if(target){target.focus({preventScroll:true});const item=target.getBoundingClientRect(),rect=element.getBoundingClientRect();if(item.bottom>rect.bottom)element.scrollTop+=item.bottom-rect.bottom;else if(item.top<rect.top)element.scrollTop-=rect.top-item.top;}
   }
   return <><div ref={element} role="menu" aria-label={panel.label} class={css.menu} style={{left:`${coords().left}px`,top:`${coords().top}px`}} onKeyDown={navigate} onContextMenu={e=>{e.preventDefault();e.stopPropagation();}} onClick={e=>e.stopPropagation()}>
    <For each={panel.actions().map(action=>action.id)}>{id=>{const action=()=>panel.actions().find(item=>item.id===id)!;return <><Show when={action().separatorBefore}><div role="separator" class={css.separator}/></Show><button type="button" data-action-id={id} role={action().checked===undefined?'menuitem':'menuitemcheckbox'} aria-checked={action().checked} aria-haspopup={action().children?.length?'menu':undefined} aria-expanded={action().children?.length?child()?.id===id:undefined} class={css.item} aria-disabled={action().disabled===true} onClick={e=>activate(id,e.currentTarget)} onPointerEnter={()=>{if(child()?.id!==id)setChild(undefined);}}><Icon name={action().icon}/>{action().label}<Show when={action().current||action().children?.length}><span class={css.current} aria-hidden="true"><Icon name={action().children?.length?'chevron':'check'}/></span></Show></button></>;}}</For>
   </div><Show when={child()}>{entry=><Panel actions={()=>panel.actions().find(action=>action.id===entry().id)?.children??[]} label={childAction()?.label??''} anchor={entry().anchor} back={dismissChild} valid={()=>panel.valid()&&!panel.actions().find(action=>action.id===entry().id)?.disabled}/>}</Show></>;
  }
  return <div ref={container}><Panel actions={options.actions} label={options.label()} valid={()=>true}/></div>;
 }
 const menu=():JSX.Element=><Show when={position()}>{point=><Portal><MenuBody point={point()}/></Portal>}</Show>;
 return {open,openAbove,keydown,close,menu,isOpen:()=>position()!==undefined};
}
