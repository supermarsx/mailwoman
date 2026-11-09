import {createEffect,createSignal,For,onCleanup,onMount,Show,type Accessor,type JSX} from 'solid-js';
import {Portal} from 'solid-js/web';
import {Icon,type IconName} from './Icon.tsx';
import * as css from './contextMenu.css.ts';
/** `checked` marks an on/off action (a label); it is then a menuitemcheckbox that says which.
 * `current` draws a trailing check (with `checked`, for one-of-several choices).
 * `separatorBefore` draws a separator above the item, to group related actions; arrow keys skip it. */
export interface ContextAction {id:string;label:string;icon:IconName;disabled?:boolean;checked?:boolean;radio?:boolean;separatorBefore?:boolean;/** Why it is disabled, or what it does: shown as a tooltip. */title?:string;
 /** A chosen option among several (a radio-like menu, t57): a trailing check is drawn; announced through `checked`. */
 current?:boolean;children?:ContextAction[];searchLabel?:string;run:()=>void|Promise<void>}
export function clampMenu(x:number,y:number,width:number,height:number,viewportWidth:number,viewportHeight:number):{left:number;top:number} {
 return {left:Math.max(8,Math.min(x,viewportWidth-width-8)),top:Math.max(8,Math.min(y,viewportHeight-height-8))};
}
/** Where a menu panel goes so it stays on screen: a submenu (`anchor`) opens beside its row and flips to the other side when
 * there is no room; either kind is shifted to fit and given a `maxHeight` of the viewport so the panel scrolls instead of overflowing. */
export function placePanel(input:{point:{x:number;y:number;above?:boolean};anchor?:{left:number;right:number;top:number};size:{width:number;height:number};viewport:{width:number;height:number};rtl?:boolean}):{left:number;top:number;maxHeight:number} {
 const {point,anchor,size,viewport,rtl=false}=input,margin=8,maxHeight=Math.max(0,viewport.height-margin*2);
 const height=Math.min(size.height,maxHeight);
 let x=anchor?(rtl?anchor.left-size.width:anchor.right):point.x;
 if(anchor&&(x+size.width>viewport.width-margin||x<margin))x=rtl?anchor.right:anchor.left-size.width;
 const y=anchor?anchor.top:point.above?point.y-height:point.y;
 return {...clampMenu(x,y,size.width,height,viewport.width,viewport.height),maxHeight};
}
/** The scope is captured at open and rechecked at execution, including identity reuse.
 * `restoreFocus` is given the trigger that is gone: an action that re-renders its row replaces the trigger, and focus would fall to the body. */
export function createContextMenu(options:{label:Accessor<string>;scope:Accessor<string>;actions:Accessor<ContextAction[]>;focusCurrent?:boolean;searchLabel?:Accessor<string>;onError?:()=>void;restoreFocus?:(trigger?:HTMLElement)=>void;ownerDocument?:Accessor<Document|undefined>}) {
 const [position,setPosition]=createSignal<{x:number;y:number;scope:string;trigger:HTMLElement;document:Document;above?:boolean;anchored?:boolean}>();
 const owner=():Document=>options.ownerDocument?.()??document;
 function announce(doc:Document):void {close(false);const event=doc.createEvent('Event');event.initEvent('mailwoman-context-open',false,false);doc.dispatchEvent(event);}
 function close(restore=true):void {const old=position();setPosition(undefined);if(restore){if(old?.trigger.isConnected)old.trigger.focus();else options.restoreFocus?.(old?.trigger);}}
 function open(event:MouseEvent|KeyboardEvent):void {
  const target=event.target as Element|null,doc=owner();if(target?.nodeType===1&&target.closest('input,textarea,[contenteditable="true"]'))return;
  if(doc.defaultView?.getSelection()?.toString()||options.actions().length===0)return;
  const trigger=(target?.nodeType===1?target.closest<HTMLElement>('button,[tabindex]'):null)??event.currentTarget as HTMLElement;const rect=trigger.getBoundingClientRect();
  event.preventDefault();event.stopPropagation();announce(doc);setPosition({x:'clientX' in event&&event.type==='contextmenu'?event.clientX:rect.left,y:'clientY' in event&&event.type==='contextmenu'?event.clientY:rect.bottom,scope:options.scope(),trigger,document:doc});
 }
 /** A toolbar dropdown uses the same themed menu without the native text-selection rules. */
 function openBelow(trigger:HTMLElement):void {
  if(options.actions().length===0)return;const rect=trigger.getBoundingClientRect();
  const doc=owner();announce(doc);setPosition({x:rect.left,y:rect.bottom+4,scope:options.scope(),trigger,anchored:true,document:doc});
 }
 /** Open from a control near the bottom of its pane (the selection bar, t58): the menu opens upward, its bottom edge just above the trigger. */
 function openAbove(trigger:HTMLElement):void {
  if(options.actions().length===0)return;const rect=trigger.getBoundingClientRect();
  const doc=owner();announce(doc);setPosition({x:rect.left,y:rect.top-4,scope:options.scope(),trigger,above:true,document:doc});
 }
 /** Already validated coordinates from an isolated frame's explicit gesture. */
 function openAt(point:{x:number;y:number;trigger:HTMLElement}):void {if(!Number.isFinite(point.x)||!Number.isFinite(point.y)||options.actions().length===0)return;const doc=owner();announce(doc);setPosition({...point,scope:options.scope(),document:doc});}
 function keydown(event:KeyboardEvent):void {if(event.key==='ContextMenu'||(event.shiftKey&&event.key==='F10'))open(event);}
 createEffect(()=>{const value=position();if(value&&(value.scope!==options.scope()||(options.ownerDocument&&options.ownerDocument()!==value.document)))close(false);});
 function MenuBody(props:{point:NonNullable<ReturnType<typeof position>>}):JSX.Element {
  let container!:HTMLDivElement;
  const doc=props.point.document,host=doc.defaultView??window;
  onMount(()=>{
   const outside=(e:PointerEvent):void=>{const node=e.target as Node|null;if(node?.nodeType&&!container.contains(node)&&!(props.point.anchored&&props.point.trigger.contains(node)))close(false);};
   const dismiss=(e:Event):void=>{const node=e.target as Node|null;if(node?.nodeType&&container.contains(node))return;close(false);};
   let frameFocusTimer:ReturnType<typeof setTimeout>|undefined;
   // Pointer events inside a message iframe never reach this document. Wait for
   // focus to settle after blur, then dismiss without taking focus off the frame.
   const dismissFrame=():void=>{const active=doc.activeElement;if(active?.tagName==='IFRAME'&&!container.contains(active))close(false);};
   const frameFocus=():void=>{clearTimeout(frameFocusTimer);frameFocusTimer=setTimeout(dismissFrame,0);};
   // Firefox can change iframe focus without emitting parent focus/blur events.
   // Check only while this menu is mounted; no access to the frame is needed.
   const frameFocusPoll=setInterval(dismissFrame,100);
   doc.addEventListener('mailwoman-context-open',dismiss);doc.addEventListener('pointerdown',outside,true);host.addEventListener('resize',dismiss);host.addEventListener('scroll',dismiss,true);host.addEventListener('blur',frameFocus);
   onCleanup(()=>{clearInterval(frameFocusPoll);clearTimeout(frameFocusTimer);doc.removeEventListener('mailwoman-context-open',dismiss);doc.removeEventListener('pointerdown',outside,true);host.removeEventListener('resize',dismiss);host.removeEventListener('scroll',dismiss,true);host.removeEventListener('blur',frameFocus);});
  });
  function Panel(panel:{actions:Accessor<ContextAction[]>;label:string;searchLabel?:string|undefined;anchor?:HTMLButtonElement;back?:()=>void;valid:()=>boolean}):JSX.Element {
   let element!:HTMLDivElement;
   let searchInput:HTMLInputElement|undefined;
   const [query,setQuery]=createSignal('');
   const visibleActions=()=>{const needle=query().trim().toLocaleLowerCase();return needle?panel.actions().filter(action=>action.label.toLocaleLowerCase().includes(needle)):panel.actions();};
   const [coords,setCoords]=createSignal<{left:number;top:number;maxHeight?:number}>({left:8,top:8});
   const [child,setChild]=createSignal<{id:string;anchor:HTMLButtonElement}>();
   const buttons=()=>Array.from(element.querySelectorAll<HTMLButtonElement>(':scope > button:not([aria-disabled="true"])'));
   const dismissChild=()=>{const old=child();setChild(undefined);old?.anchor.focus({preventScroll:true});};
   const childAction=()=>panel.actions().find(action=>action.id===child()?.id);
   createEffect(()=>{if(child()&&(!childAction()?.children?.length||childAction()?.disabled))dismissChild();});
   onMount(()=>{
    const rect=element.getBoundingClientRect(),anchor=panel.anchor?.getBoundingClientRect();
    const rtl=host.getComputedStyle(element).direction==='rtl';
    setCoords(placePanel({point:props.point,...(anchor?{anchor}:{}),size:{width:rect.width,height:rect.height},viewport:{width:host.innerWidth,height:host.innerHeight},rtl}));
    if(searchInput){searchInput.focus({preventScroll:true});return;}
    const rows=buttons();((options.focusCurrent?rows.find(row=>row.getAttribute('aria-checked')==='true')??rows[0]:rows[0])??element).focus({preventScroll:true});
   });
   function activate(id:string,button:HTMLButtonElement):void {
    const current=panel.actions().find(item=>item.id===id);
    if(props.point.scope!==options.scope()||!panel.valid()||!current||current.disabled){close(false);return;}
    if(current.children?.length){setChild({id,anchor:button});return;}
    const trigger=props.point.trigger;close();
    try{void Promise.resolve(current.run()).catch(()=>options.onError?.());}catch{options.onError?.();}
    queueMicrotask(()=>{if(!trigger.isConnected&&(doc.activeElement===null||doc.activeElement===doc.body))options.restoreFocus?.(trigger);});
   }
   function navigate(e:KeyboardEvent):void {
    const rtl=host.getComputedStyle(element).direction==='rtl';
    if(e.target===searchInput&&e.key!=='Escape'&&e.key!=='Tab'){if(e.key==='ArrowDown'){e.preventDefault();buttons()[0]?.focus();}return;}
    if(e.key==='Escape'||(panel.back&&e.key===(rtl?'ArrowRight':'ArrowLeft'))){e.preventDefault();e.stopPropagation();if(panel.back)panel.back();else close();return;}
    if(e.key==='Tab'){close();return;}
    if(e.key===(rtl?'ArrowLeft':'ArrowRight')){const button=doc.activeElement as HTMLButtonElement;const action=panel.actions().find(item=>item.id===button.dataset['actionId']);if(action?.children?.length){e.preventDefault();e.stopPropagation();activate(action.id,button);}return;}
    if(e.key.length===1&&!e.ctrlKey&&!e.metaKey&&!e.altKey&&e.key!==' '){const rows=buttons(),at=rows.indexOf(doc.activeElement as HTMLButtonElement),letter=e.key.toLocaleLowerCase();
     const hit=[...rows.slice(at+1),...rows.slice(0,at+1)].find(row=>(row.textContent??'').trim().toLocaleLowerCase().startsWith(letter));
     if(hit){e.preventDefault();e.stopPropagation();setChild(undefined);hit.focus({preventScroll:true});hit.scrollIntoView?.({block:'nearest'});}return;}
    const keys=['ArrowDown','ArrowUp','Home','End'];if(!keys.includes(e.key))return;
    e.preventDefault();e.stopPropagation();setChild(undefined);const rows=buttons(),at=rows.indexOf(doc.activeElement as HTMLButtonElement);const next=e.key==='Home'?0:e.key==='End'?rows.length-1:(at+(e.key==='ArrowDown'?1:-1)+rows.length)%rows.length;
    const target=rows[next];if(target){target.focus({preventScroll:true});const item=target.getBoundingClientRect(),rect=element.getBoundingClientRect();if(item.bottom>rect.bottom)element.scrollTop+=item.bottom-rect.bottom;else if(item.top<rect.top)element.scrollTop-=rect.top-item.top;}
   }
   return <><div ref={element} role="menu" tabIndex={-1} aria-label={panel.label} class={css.menu} style={{left:`${coords().left}px`,top:`${coords().top}px`,...(coords().maxHeight===undefined?{}:{'max-height':`${coords().maxHeight}px`})}} on:keydown={navigate} on:contextmenu={e=>{e.preventDefault();e.stopPropagation();}} on:click={e=>e.stopPropagation()}>
    <Show when={panel.searchLabel}><input ref={searchInput} type="search" aria-label={panel.searchLabel} placeholder={panel.searchLabel} value={query()} onInput={event=>{setChild(undefined);setQuery(event.currentTarget.value);}} style={{margin:'4px 8px 8px',width:'calc(100% - 16px)','box-sizing':'border-box'}}/></Show>
    <Show when={query().trim()&&visibleActions().length===0}><div role="status" style={{padding:'8px 12px'}}>{panel.label}: 0</div></Show>
    <For each={visibleActions().map(action=>action.id)}>{id=>{const action=()=>panel.actions().find(item=>item.id===id)!;return <><Show when={action().separatorBefore}><div role="separator" class={css.separator}/></Show><button type="button" data-action-id={id} data-current={action().current?'true':undefined} role={action().radio?'menuitemradio':action().checked===undefined?'menuitem':'menuitemcheckbox'} aria-checked={action().checked} aria-haspopup={action().children?.length?'menu':undefined} aria-expanded={action().children?.length?child()?.id===id:undefined} class={css.item} title={action().title} aria-disabled={action().disabled===true} on:click={e=>activate(id,e.currentTarget)} on:pointerenter={()=>{if(child()?.id!==id)setChild(undefined);}}><Icon name={action().icon}/>{action().label}<Show when={action().current||action().checked||action().children?.length}><span class={css.current} aria-hidden="true"><Icon name={action().children?.length?'chevron':'check'}/></span></Show></button></>;}}</For>
   </div><Show when={child()}>{entry=><Panel actions={()=>panel.actions().find(action=>action.id===entry().id)?.children??[]} label={childAction()?.label??''} searchLabel={childAction()?.searchLabel} anchor={entry().anchor} back={dismissChild} valid={()=>panel.valid()&&!panel.actions().find(action=>action.id===entry().id)?.disabled}/>}</Show></>;
  }
  return <div ref={container}><Panel actions={options.actions} label={options.label()} searchLabel={options.searchLabel?.()} valid={()=>true}/></div>;
 }
 const menu=():JSX.Element=><Show when={position()} keyed>{point=><Portal mount={point.document.body}><MenuBody point={point}/></Portal>}</Show>;
 return {open,openBelow,openAbove,openAt,keydown,close,menu,isOpen:()=>position()!==undefined};
}
