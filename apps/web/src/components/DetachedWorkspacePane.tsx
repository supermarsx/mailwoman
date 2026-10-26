import {withBase} from '../api/basePath.ts';
import {Show,createEffect,createSignal,onCleanup,onMount,type JSX} from 'solid-js';
import {Portal,delegateEvents,clearDelegatedEvents,DelegatedEvents} from 'solid-js/web';
import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import {toast} from './toast/index.ts';
import {TooltipLayer} from './TooltipLayer.tsx';
import {t} from '../i18n/index.ts';
import './detachedWorkspace.css.ts';

export interface DetachedPaneHandle {detach():void;reattach():void;isDetached():boolean}
/** The parent owns the component lifetime. Only its existing DOM changes document. */
export function DetachedWorkspacePane(props:{title:()=>string;register:(handle:DetachedPaneHandle)=>void;children:()=>JSX.Element}):JSX.Element {
  const owner=preferenceAccount();
  const [mount,setMount]=createSignal<HTMLElement>();
  const [detached,setDetached]=createSignal(false);
  let home:HTMLDivElement|undefined;
  let child:Window|null=null;
  let cleanupChild:(()=>void)|undefined;
  let disposed=false;
  let popupLabel:HTMLElement|undefined;
  createEffect(()=>{const title=props.title();if(detached()&&child){child.document.title=title;if(popupLabel)popupLabel.textContent=title;}});
  createEffect(()=>{const target=mount();if(target)queueMicrotask(()=>{for(const editor of target.querySelectorAll('.compose-rt__surface'))editor.dispatchEvent(new Event('mailwoman:documentchange'));});});
  function reattach():void {
    const previous=child;child=null;cleanupChild?.();cleanupChild=undefined;
    setMount(home);setDetached(false);
    if(previous&&!previous.closed){try{previous.close();}catch{/* Already navigated/closed. */}}
  }
  function detach():void {
    if(disposed||!owner||!preferenceAccountIsCurrent(owner))return;
    if(child&&!child.closed){child.focus();return;}
    const popupUrl=new URL(withBase('/workspace-popup.html'),location.href).href;
    const popup=window.open(popupUrl,'_blank','popup,width=1050,height=760');
    if(!popup){toast.error(t('mail-popup-blocked'));return;}
    child=popup;
    let initialized=false;

    const initialize=():void=>{
      if(initialized)return;
      try {if(popup.location.href!==popupUrl)return;}catch{return;}
      if(popup.document.readyState!=='complete')return;
      initialized=true;clearTimeout(pendingTimer);popup.removeEventListener('load',initialize);
      if(disposed||!preferenceAccountIsCurrent(owner)){reattach();return;}
      // A matching URL can still contain a server/proxy error page. Never move
      // the live editor into that document or initialize before navigation ends.
      if(popup.document.documentElement.getAttribute('data-mailwoman-workspace-popup')!=='1'){
        reattach();toast.error(t('mail-popup-unavailable'));return;
      }
    try {
      const doc=popup.document;doc.title=props.title();
      const syncTheme=():void=>{
        for(const name of ['class','lang','dir','data-theme','data-font']){
          const value=document.documentElement.getAttribute(name);
          if(value===null)doc.documentElement.removeAttribute(name);else doc.documentElement.setAttribute(name,value);
        }
        doc.documentElement.style.cssText=document.documentElement.style.cssText;
        doc.body.className=document.body.className+' detached-workspace';
      };
      syncTheme();
      for(const source of document.head.querySelectorAll('style,link[rel="stylesheet"]')){
        if(source instanceof HTMLLinkElement&&new URL(source.href).origin!==location.origin)continue;
        if(source instanceof HTMLLinkElement){
          const copy=doc.createElement('link');copy.rel='stylesheet';copy.href=source.href;copy.media=source.media;
          if(source.crossOrigin)copy.crossOrigin=source.crossOrigin;
          if(source.nonce)copy.nonce=source.nonce;
          doc.head.appendChild(copy);
        }else{
          const copy=doc.createElement('style');copy.textContent=source.textContent;
          if(source instanceof HTMLStyleElement&&source.nonce)copy.nonce=source.nonce;
          doc.head.appendChild(copy);
        }
      }
      const header=doc.createElement('header');header.className='detached-workspace__header';
      const label=doc.createElement('span');label.textContent=props.title();popupLabel=label;
      const button=doc.createElement('button');button.type='button';button.className='btn btn--ghost';button.textContent=t('mail-popup-reattach');button.addEventListener('click',reattach);
      header.append(label,button);
      const body=doc.createElement('main');body.className='detached-workspace__content';doc.body.append(header,body);
      delegateEvents([...DelegatedEvents],doc);
      // Global confirmations and account actions belong to the main application.
      // Move back before invoking them; formatting controls remain local.
      const action=(event:Event):void=>{
        const target=event.target as Element|null;
        const control=target?.closest<HTMLElement>('button,a[href],input[type="submit"]');
        if(!control||!body.contains(control)||control.closest('[data-detached-local]'))return;
        event.preventDefault();event.stopImmediatePropagation();reattach();window.focus();
        toast.info(t('mail-popup-action-reattached'));
        queueMicrotask(()=>{if(owner&&preferenceAccountIsCurrent(owner)&&control.isConnected){control.focus();control.click();}});
      };
      const submit=(event:Event):void=>{
        const form=event.target as HTMLFormElement;if(!body.contains(form))return;
        const submitter=(event as SubmitEvent).submitter;
        event.preventDefault();event.stopImmediatePropagation();reattach();window.focus();toast.info(t('mail-popup-action-reattached'));
        queueMicrotask(()=>{if(owner&&preferenceAccountIsCurrent(owner)&&form.isConnected)form.requestSubmit(submitter);});
      };
      const change=(event:Event):void=>{
        const control=event.target as HTMLElement;
        if(!body.contains(control)||!control.matches('select,input[type="checkbox"],input[type="radio"],input[type="file"]')||control.closest('[data-detached-local]'))return;
        event.stopImmediatePropagation();reattach();window.focus();toast.info(t('mail-popup-action-reattached'));
        queueMicrotask(()=>{if(owner&&preferenceAccountIsCurrent(owner)&&control.isConnected)control.dispatchEvent(new Event('change',{bubbles:true}));});
      };
      doc.addEventListener('click',action,true);doc.addEventListener('submit',submit,true);doc.addEventListener('change',change,true);
      const leave=():void=>reattach();popup.addEventListener('pagehide',leave);
      const theme=new MutationObserver(syncTheme);theme.observe(document.documentElement,{attributes:true});theme.observe(document.body,{attributes:true,attributeFilter:['class']});
      const poll=window.setInterval(()=>{if(popup.closed)reattach();},500);
      cleanupChild=()=>{window.clearInterval(poll);theme.disconnect();popup.removeEventListener('pagehide',leave);doc.removeEventListener('click',action,true);doc.removeEventListener('submit',submit,true);doc.removeEventListener('change',change,true);clearDelegatedEvents(doc);};
      child=popup;setMount(body);setDetached(true);popup.focus();
    }catch{reattach();toast.error(t('mail-popup-unavailable'));}
    };
    popup.addEventListener('load',initialize);
    const pendingTimer=setTimeout(()=>{if(!initialized){reattach();toast.error(t('mail-popup-unavailable'));}},10000);
    cleanupChild=()=>{clearTimeout(pendingTimer);popup.removeEventListener('load',initialize);};
    initialize();
  }
  props.register({detach,reattach,isDetached:detached});
  const stop=observePreferenceAccount(current=>{if(owner&&current?.generation!==owner.generation)reattach();});
  const parentLeaving=(event:BeforeUnloadEvent):void=>{if(child&&!child.closed){event.preventDefault();event.returnValue='';}};
  onMount(()=>{setMount(home);window.addEventListener('beforeunload',parentLeaving);window.addEventListener('pagehide',reattach);});
  onCleanup(()=>{disposed=true;stop();reattach();window.removeEventListener('beforeunload',parentLeaving);window.removeEventListener('pagehide',reattach);});
  return <><div class="detached-workspace__home" ref={home} hidden={detached()}/><Show when={mount()}><Portal mount={mount()!} ref={element=>element.classList.add('detached-workspace__portal')}>{props.children()}</Portal></Show><Show when={detached()&&mount()?.ownerDocument!==document?mount()?.ownerDocument:undefined}>{doc=><TooltipLayer document={doc()}/>}</Show><Show when={detached()}><div class="detached-workspace__placeholder"><p>{t('mail-popup-detached')}</p><button type="button" class="btn btn--ghost" onClick={reattach}>{t('mail-popup-reattach')}</button></div></Show></>;
}
