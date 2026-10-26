import {createRoot,createSignal,onCleanup} from 'solid-js';
import {observePreferenceAccount,preferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import type {SliceContext} from './slices/context.ts';
import {apiFetch} from '../api/http.ts';

export function accountScopedFetcher(fetcher:(input:string,init?:RequestInit)=>Promise<Response>=apiFetch):(input:string,init?:RequestInit)=>Promise<Response> {
  const owner=preferenceAccount();
  return async(input,init)=>{
    if(owner!==preferenceAccount())throw new DOMException('Mailbox changed','AbortError');
    const response=await fetcher(input,{credentials:'same-origin',...init});
    if(owner!==preferenceAccount())throw new DOMException('Mailbox changed','AbortError');
    return response;
  };
}

/** Stable public accessors/actions over freshly owned private caches. Old async
 * actions retain a fenced client: they cannot dispatch into a newer account. */
export function accountScopedSlice<T extends object>(ctx:SliceContext,factory:(context:SliceContext)=>T):T {
  let owner=preferenceAccount();
  let dispose=()=>{};
  function build():T {
    const captured=owner;
    const check=()=>{if(captured!==preferenceAccount())throw new DOMException('Mailbox changed','AbortError');};
    const client=new Proxy(ctx.client,{get(target,key,receiver){
      const value=Reflect.get(target,key,receiver) as unknown;
      if(typeof value!=='function'||!['session','jmap','sanitize'].includes(String(key)))return value;
      return async(...args:unknown[])=>{check();const result=await Reflect.apply(value,target,args);check();return result;};
    }});
    return createRoot(stop=>{dispose=stop;return factory({...ctx,client,showToast:(...args)=>{if(captured===preferenceAccount())ctx.showToast(...args);}});});
  }
  const [current,setCurrent]=createSignal(build());
  const stop=observePreferenceAccount(next=>{
    if(next===owner)return;
    dispose();owner=next;setCurrent(()=>build());
  });
  onCleanup(()=>{stop();dispose();});
  const facade={} as T;
  for(const key of Reflect.ownKeys(current())){
    const initial=Reflect.get(current(),key) as unknown;
    Object.defineProperty(facade,key,{enumerable:true,get:()=>typeof initial==='function'
      ?(...args:unknown[])=>{const slice=current();return Reflect.apply(Reflect.get(slice,key) as (...values:unknown[])=>unknown,slice,args);}
      :Reflect.get(current(),key)});
  }
  return facade;
}
