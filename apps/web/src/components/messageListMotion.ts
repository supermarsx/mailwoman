import * as css from './messageListMotion.css.ts';

const DURATION=160;
const MAX_GHOSTS=8;
const MAX_MOVING_ROWS=48;

export interface MessageListMotionFrame {
  host:HTMLElement|undefined;
  /** Account authority, folder/search, ordering, density and thread expansion. */
  scope:string;
  /** The complete visual query, not just the virtual window. */
  ids:readonly string[];
  busy:boolean;
}
interface PaintedRow {element:HTMLElement;top:number;height:number}
interface Snapshot {
  host:HTMLElement;
  scope:string;
  ids:Set<string>;
  rows:Map<string,PaintedRow>;
  top:number;
  left:number;
  width:number;
  height:number;
  scrollTop:number;
}

/** Animate optimistic list changes without retaining messages in live state or
 * delaying the mutation. Virtual scrolling and query replacement never animate.
 * Individual `translate` composes with the virtualizer's existing `transform`. */
export function createMessageListMotion():{
  update(frame:MessageListMotionFrame):void;
  clear():void;
  dispose():void;
} {
  let previous:Snapshot|undefined;
  let generation=0;
  let disposed=false;
  let watched:HTMLElement|undefined;
  let sourceIds:readonly string[]|undefined;
  let sourceSet=new Set<string>();
  let latest:MessageListMotionFrame|undefined;
  const running=new Set<()=>void>();
  const media=typeof matchMedia==='function'?matchMedia('(prefers-reduced-motion: reduce)'):undefined;
  const reduced=():boolean=>document.documentElement.hasAttribute('data-reduced-motion')||media?.matches===true;
  const cancelPaint=():void=>{for(const stop of [...running])stop();};
  const clear=():void=>{generation++;previous=undefined;cancelPaint();};
  const onScroll=():void=>{
    clear();
    const mine=generation;
    // Solid's delegated scroll handler may have already updated the virtual
    // window before this native listener runs. Refresh the baseline in either
    // listener order so the first real deletion after scrolling still moves.
    queueMicrotask(()=>{if(!disposed&&mine===generation&&latest&&!latest.busy&&!reduced())previous=snapshot(latest);});
  };
  const onMotion=():void=>{if(reduced())clear();};
  media?.addEventListener('change',onMotion);
  const observer=typeof MutationObserver==='undefined'?undefined:new MutationObserver(onMotion);
  observer?.observe(document.documentElement,{attributes:true,attributeFilter:['data-reduced-motion']});

  function snapshot(frame:MessageListMotionFrame):Snapshot|undefined {
    const host=frame.host;
    if(!host?.isConnected)return;
    const bounds=host.getBoundingClientRect();
    const rows=new Map<string,PaintedRow>();
    for(const element of host.querySelectorAll<HTMLElement>('[data-motion-row]')){
      const key=element.dataset.motionRow;
      const rect=element.getBoundingClientRect();
      if(key&&rect.bottom>bounds.top&&rect.top<bounds.bottom&&rect.height>0)rows.set(key,{element,top:rect.top,height:rect.height});
    }
    if(sourceIds!==frame.ids){sourceIds=frame.ids;sourceSet=new Set(frame.ids);}
    return {host,scope:frame.scope,ids:sourceSet,rows,top:bounds.top,left:bounds.left,width:bounds.width,height:bounds.height,scrollTop:host.scrollTop};
  }

  function animate(element:HTMLElement,keyframes:Keyframe[],after?:()=>void):void {
    if(typeof element.animate!=='function'){after?.();return;}
    const animation=element.animate(keyframes,{duration:DURATION,easing:'cubic-bezier(.2,.7,.3,1)'});
    let stopped=false;
    const stop=():void=>{
      if(stopped)return;
      stopped=true;
      clearTimeout(timer);
      running.delete(stop);
      animation.removeEventListener('finish',stop);
      animation.removeEventListener('cancel',stop);
      animation.cancel();
      after?.();
    };
    const timer=setTimeout(stop,DURATION+80);
    running.add(stop);
    animation.addEventListener('finish',stop,{once:true});
    animation.addEventListener('cancel',stop,{once:true});
  }

  function ghost(row:PaintedRow,host:HTMLElement):void {
    const list=host.querySelector<HTMLElement>('.list__items');
    if(!list)return;
    const clone=row.element.cloneNode(true) as HTMLElement;
    clone.classList.add(css.departure);
    clone.setAttribute('data-list-exit-ghost','');
    clone.setAttribute('aria-hidden','true');
    clone.setAttribute('inert','');
    clone.setAttribute('role','presentation');
    clone.removeAttribute('aria-posinset');
    clone.removeAttribute('aria-setsize');
    clone.querySelectorAll('.msg-more').forEach(element=>element.remove());
    for(const element of [clone,...clone.querySelectorAll<HTMLElement>('*')]){
      for(const attribute of ['id','data-motion-row','data-email-id','data-context-open','aria-current'])element.removeAttribute(attribute);
      if(element.matches('a,button,input,select,textarea,[tabindex]'))element.tabIndex=-1;
    }
    list.append(clone);
    animate(clone,[{opacity:0.8,translate:'0 0'},{opacity:0,translate:'-10px 0'}],()=>clone.remove());
  }

  return {
    update(frame):void {
      if(disposed)return;
      latest=frame;
      if(watched!==frame.host){watched?.removeEventListener('scroll',onScroll);watched=frame.host;watched?.addEventListener('scroll',onScroll,{passive:true});}
      if(frame.busy||reduced()){clear();return;}
      if(previous&&previous.scope!==frame.scope)clear();
      const mine=++generation;
      queueMicrotask(()=>{
        if(disposed||mine!==generation)return;
        // Cancel old paint before reading geometry: WAAPI translations must
        // never become the next animation's baseline.
        cancelPaint();
        const next=snapshot(frame),before=previous;
        previous=next;
        if(!before||!next||before.host!==next.host||before.scope!==next.scope||reduced())return;
        if(before.scrollTop!==next.scrollTop||Math.abs(before.top-next.top)>1||Math.abs(before.left-next.left)>1||Math.abs(before.width-next.width)>1||Math.abs(before.height-next.height)>1)return;
        if(before.ids===next.ids)return;
        const removed=[...before.ids].filter(id=>!next.ids.has(id));
        if(removed.length===0)return;
        let ghosts=0,moving=0;
        for(const id of removed){const row=before.rows.get(id);if(row&&ghosts++<MAX_GHOSTS)ghost(row,next.host);}
        for(const [id,row] of next.rows){
          const old=before.rows.get(id);
          if(!old||old.element!==row.element||moving>=MAX_MOVING_ROWS)continue;
          const delta=old.top-row.top;
          if(Math.abs(delta)<1||Math.abs(delta)>next.height)continue;
          moving++;
          animate(row.element,[{translate:`0 ${delta}px`},{translate:'0 0'}]);
        }
      });
    },
    clear,
    dispose():void {if(disposed)return;disposed=true;clear();latest=undefined;sourceIds=undefined;sourceSet.clear();watched?.removeEventListener('scroll',onScroll);media?.removeEventListener('change',onMotion);observer?.disconnect();},
  };
}
