import {createContextMenu,type ContextAction} from './ContextMenu.tsx';
import {preferenceAccount,observePreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import { toast } from './toast/index.ts';
import { toastTarget } from './toast/context.ts';
import { Icon } from './Icon.tsx';
// In-app sub-tab strip (plan §3 e6). A thin view over the realtime controller's
// sub-tab model (realtime/subTabs.ts): a horizontal tablist of open surfaces
// (messages / composers / settings), each activatable, pinnable, closable, and
// tear-off-able into its own window via `window.open`. Keyboard: Left/Right (or
// Ctrl+[Shift+]Tab) cycles focus. Reads `useRealtime()` so it resolves the app
// singleton or a test-provided controller.

import { For, Show, createEffect, onCleanup, onMount, type JSX } from 'solid-js';
import {beginWorkspaceTabDrag,workspaceTabDragId,endWorkspaceTabDrag} from '../realtime/workspaceTabDrag.ts';
import { useRealtime } from '../realtime/context.ts';
import { t,loadCatalog } from '../i18n/index.ts';
import * as a11y from './mailA11y.css.ts';
import type { SubTab } from '../realtime/subTabs.ts';

/** DOM id of a sub-tab's `role="tab"` button — referenced by the tablist's aria-owns. */
function tabDomId(id: string): string {
  return `subtab-tab-${id}`;
}

export function SubTabStrip(props: { onClose?: (id:string)=>void; allowTearOff?: boolean; onDetach?: (id:string)=>void; isDetached?: (id:string)=>boolean; onReattach?: (id:string)=>void; canDetach?: (id:string)=>boolean; canClose?: (id:string)=>boolean } = {}): JSX.Element {
  const { subTabs } = useRealtime();
  onCleanup(endWorkspaceTabDrag);
  onMount(()=>void loadCatalog('workspace-tabs'));
  function canDrop(event:DragEvent):boolean {
    const id=workspaceTabDragId(event);return id!==null&&subTabs.tabs().some(tab=>tab.id===id);
  }
  function drop(event:DragEvent,id:string):void {
    const dragged=workspaceTabDragId(event,true);if(!dragged||!canDrop(event))return;
    event.preventDefault();const index=subTabs.tabs().findIndex(tab=>tab.id===id);
    if(index>=0)subTabs.move(dragged,index);endWorkspaceTabDrag();
  }

  function onKeyDown(e: KeyboardEvent): void {
    if(e.ctrlKey&&e.shiftKey&&(e.key==='ArrowLeft'||e.key==='ArrowRight')){
      const active=subTabs.activeId();const index=subTabs.tabs().findIndex(tab=>tab.id===active);
      if(active&&index>=0){subTabs.move(active,index+(e.key==='ArrowRight'?1:-1));queueMicrotask(()=>document.getElementById(tabDomId(active))?.focus());e.preventDefault();}
      return;
    }
    const next = e.key === 'ArrowRight' || (e.ctrlKey && e.key === 'Tab' && !e.shiftKey);
    const prev = e.key === 'ArrowLeft' || (e.ctrlKey && e.key === 'Tab' && e.shiftKey);
    if (next) {
      const id=subTabs.cycle(1); if(id) document.getElementById(tabDomId(id))?.focus();
      e.preventDefault();
    } else if (prev) {
      const id=subTabs.cycle(-1); if(id) document.getElementById(tabDomId(id))?.focus();
      e.preventDefault();
    }
  }

  // WCAG 1.3.1 (aria-required-children): a `role="tablist"` may only own `role="tab"`
  // elements. Each sub-tab pairs its tab with pin/tear-off/close controls (buttons),
  // so the tablist can't physically contain the tab groups. Instead it owns the tab
  // buttons via `aria-owns` (leaving the controls outside its required-children
  // contract), while the visible strip carries the roving keyboard model — keydown
  // from a focused tab bubbles here. Controls are NOT nested inside the tabs either,
  // which would trip `nested-interactive` (tab has childrenPresentational).
  return (
    <Show when={subTabs.tabs().length > 0}>
      <div class="subtab-strip" onKeyDown={onKeyDown}>
        <div
          class={`subtab-tablist ${a11y.srOnly}`}
          role="tablist"
          aria-label={t('mail-subtabs-label')}
          aria-owns={subTabs.tabs().map((tab) => tabDomId(tab.id)).join(' ')}
        />
        <For each={subTabs.tabs()}>
          {(tab) => <SubTabButton tab={tab} active={subTabs.activeId() === tab.id} {...props}
            onDragStart={event=>beginWorkspaceTabDrag(event,tab.id)} onDragEnd={endWorkspaceTabDrag}
            onDragOver={event=>{if(canDrop(event)){event.preventDefault();if(event.dataTransfer)event.dataTransfer.dropEffect='move';}}}
            onDrop={event=>drop(event,tab.id)}/>}
        </For>
      </div>
    </Show>
  );
}

function SubTabButton(props: { tab: SubTab; active: boolean; onClose?: (id:string)=>void; allowTearOff?: boolean; onDetach?: (id:string)=>void; isDetached?: (id:string)=>boolean; onReattach?: (id:string)=>void; canDetach?: (id:string)=>boolean; canClose?: (id:string)=>boolean;onDragStart:(event:DragEvent)=>void;onDragEnd:()=>void;onDragOver:(event:DragEvent)=>void;onDrop:(event:DragEvent)=>void }): JSX.Element {
  const { subTabs } = useRealtime();
  const id = (): string => props.tab.id;
  let label:HTMLButtonElement|undefined;
  createEffect(()=>{if(props.active)queueMicrotask(()=>{if(props.active)label?.scrollIntoView?.({block:'nearest',inline:'nearest'});});});
  const close=():void=>{if(props.canClose?.(id())===false) return;if(props.onClose) props.onClose(id());else subTabs.close(id());};
  const context=createContextMenu({label:()=>props.tab.title,scope:()=>`${preferenceAccount()?.generation}:${id()}:${subTabs.tabs().some(tab=>tab.id===id())}`,actions:()=>{
    if(!subTabs.tabs().some(tab=>tab.id===id()))return [];
    const actions:ContextAction[]=[{id:'star',label:t(props.tab.pinned?'workspace-tabs-unstar':'workspace-tabs-star'),icon:props.tab.pinned?'star':'starOutline',run:()=>{
      const pinned=!props.tab.pinned;subTabs.togglePin(id());toast.success(t(pinned?'mail-toast-tab-pinned':'mail-toast-tab-unpinned',{target:toastTarget(props.tab.title,t('mail-subtabs-label'))}));
    }}];
    if(props.isDetached?.(id())){
      if(props.onReattach)actions.push({id:'reattach',label:t('mail-popup-reattach'),icon:'clone',run:()=>props.onReattach?.(id())});
    }else if(props.onDetach&&props.allowTearOff!==false&&props.canDetach?.(id())!==false)actions.push({id:'detach',label:t('mail-subtab-tearoff',{title:props.tab.title}),icon:'clone',run:()=>props.onDetach?.(id())});
    if(props.canClose?.(id())!==false)actions.push({id:'close',label:t('mail-subtab-close',{title:props.tab.title}),icon:'close',run:close});
    return actions;
  }});
  onCleanup(observePreferenceAccount(()=>context.close(false)));
  return (
    <div
      class="subtab"
      classList={{ 'subtab--active': props.active, 'subtab--pinned': props.tab.pinned }}
      data-kind={props.tab.kind}
      onContextMenu={context.open} onKeyDown={context.keydown}
      onDragOver={props.onDragOver} onDrop={props.onDrop}
    >
      <button
        type="button"
        ref={label} id={tabDomId(props.tab.id)}
        role="tab"
        class={`subtab__label ${a11y.focusable}`}
        aria-selected={props.active}
        tabindex={props.active ? 0 : -1}
        draggable="true" onDragStart={props.onDragStart} onDragEnd={props.onDragEnd}
        data-tooltip={t('mail-subtab-reorder-help')}
        onClick={() => subTabs.activate(id())}
        onAuxClick={(e) => {
          // Middle-click closes, matching browser tab convention.
          if (e.button === 1) close();
        }}
      >
        {props.tab.title}
      </button>
      <Show when={props.tab.pinned}><span class="subtab__star" role="img" aria-label={t('workspace-tabs-starred')} data-tooltip={t('workspace-tabs-starred')}><Icon name="star"/></span></Show>
      <Show when={props.active && props.canClose?.(id()) !== false}><button
        type="button"
        class={`subtab__close ${a11y.iconButton}`}
        aria-label={t('mail-subtab-close', { title: props.tab.title })}
        data-tooltip={t('mail-subtab-close', {title:props.tab.title})}
        onClick={close}
      >
        <Icon name="close" />
      </button></Show>
      {context.menu()}
    </div>
  );
}
