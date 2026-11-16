import {onCleanup,onMount} from 'solid-js';

/** Modal dialogs make nodes outside their subtree inert, regardless of z-index. */
export function floatingMount(trigger:HTMLElement):HTMLElement {
  return trigger.closest<HTMLDialogElement>('dialog[open]') ?? trigger.ownerDocument.body;
}

/** Manual popovers escape clipping and stacking contexts while preserving the
 * component's own focus, outside-click and Escape behavior. */
export function useFloatingLayer(target:HTMLElement|(()=>HTMLElement),anchor?:()=>HTMLElement):void {
  onMount(()=>{
    const element=typeof target==='function'?target():target;
    const doc=element.ownerDocument,view=doc.defaultView;
    if(!view)return;
    const supported=typeof element.showPopover==='function';
    if(supported){
      element.setAttribute('popover','manual');
      element.style.margin='0';element.style.inset='auto';
      try{element.showPopover();}catch{element.removeAttribute('popover');}
    }
    const place=()=>{
      if(!anchor||!element.isConnected)return;
      const rect=anchor().getBoundingClientRect(),margin=8,viewport=view.visualViewport;
      const left=viewport?.offsetLeft??0,top=viewport?.offsetTop??0;
      const width=viewport?.width??view.innerWidth,height=viewport?.height??view.innerHeight;
      element.style.position='fixed';element.style.inset='auto';element.style.margin='0';element.style.boxSizing='border-box';
      element.style.width=`${Math.min(rect.width,Math.max(0,width-margin*2))}px`;
      const below=Math.max(0,top+height-rect.bottom-margin),above=Math.max(0,rect.top-top-margin);
      const up=below<Math.min(element.scrollHeight,256)&&above>below;
      element.style.maxHeight=`${Math.min(256,up?above:below)}px`;
      const bounds=element.getBoundingClientRect();
      element.style.left=`${Math.max(left+margin,Math.min(rect.left,left+width-bounds.width-margin))}px`;
      element.style.top=`${up?rect.top-bounds.height:rect.bottom}px`;
    };
    place();
    const scroll=(event:Event)=>{const target=event.target as Node|null;if(target?.nodeType&&element.contains(target))return;place();};
    if(anchor){view.addEventListener('resize',place);doc.addEventListener('scroll',scroll,true);view.visualViewport?.addEventListener('resize',place);view.visualViewport?.addEventListener('scroll',place);}
    onCleanup(()=>{
      view.removeEventListener('resize',place);doc.removeEventListener('scroll',scroll,true);view.visualViewport?.removeEventListener('resize',place);view.visualViewport?.removeEventListener('scroll',place);
      if(supported){try{element.hidePopover();}catch{/* Already removed. */}}
    });
  });
}
