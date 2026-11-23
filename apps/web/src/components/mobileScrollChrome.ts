import {createEffect, createSignal, on, onCleanup, onMount, type Accessor} from 'solid-js';
/** Scoped to the actual scroll viewport, never wheel events or unrelated panes. */
export function createMobileScrollChrome(options:{root:()=>HTMLElement|undefined;selector:string;active:Accessor<boolean>;scope:Accessor<unknown>}){
 const [hidden,setHidden]=createSignal(false);let positions=new Map<unknown,number>(),direction=0,distance=0,settlingUntil=0;
 function seedViewports(){const root=options.root();if(!root)return;if(root.matches(options.selector))positions.set(root,root.scrollTop);for(const node of root.querySelectorAll<HTMLElement>(options.selector))positions.set(node,node.scrollTop);}
 function reveal(){direction=0;distance=0;settlingUntil=0;setHidden(false);seedViewports();}
 function reset(){positions.clear();reveal();}
 function observe(y:number,source:unknown='frame'){
  if(!options.active()||!Number.isFinite(y)||y<0)return;
  const previous=positions.get(source)??0;if(positions.size>4)positions.clear();positions.set(source,y);
  if(y<=8){setHidden(false);direction=0;distance=0;return;}
  if(performance.now()<settlingUntil)return;
  const delta=y-previous,nextDirection=Math.sign(delta);
  if(!nextDirection)return;
  if(direction!==nextDirection){direction=nextDirection;distance=0;}distance+=Math.abs(delta);
  if(distance<16)return;
  const next=direction>0;
  if(hidden()!==next){setHidden(next);settlingUntil=performance.now()+120;}
  distance=0;
 }
 createEffect(on(options.scope,reset));
 createEffect(on(options.active,reveal));
 onMount(()=>{const root=options.root();if(!root)return;seedViewports();
  const scroll=(event:Event)=>{const target=event.target;if(target instanceof HTMLElement&&target.matches(options.selector))observe(target.scrollTop,target);};
  root.addEventListener('scroll',scroll,true);onCleanup(()=>root.removeEventListener('scroll',scroll,true));
 });
 return {hidden,observe,reset};
}
