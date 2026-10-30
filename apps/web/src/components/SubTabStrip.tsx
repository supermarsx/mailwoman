import {displayText} from '../i18n/presentation.ts';
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
//
// t38: the strip is a `CollapsibleBar` (`ui.bar.workspaceTabs`, overlay). In
// auto-hide mode only a slim handle with the active tab's title stays on
// screen; hover, a click or keyboard focus slides the strip out over the
// content and Esc puts it away again. A vertical mouse wheel scrolls the strip
// sideways (`horizontalWheel`).
//
// t50: the tab's context menu (right-click, long-press, the ContextMenu key or
// Shift+F10) has three groups: the close actions (this tab, the others, the tabs
// to the left or right, all), Star / Unstar, and Open in a new window / Reattach.
// The bulk closes skip starred tabs and tabs that cannot close (the Mail home
// tab); "left" and "right" follow what the user sees, so they swap sides under
// `dir="rtl"`. An item with nothing to close on its side is disabled.

import { For, Show, createEffect, onCleanup, onMount, type JSX } from 'solid-js';
import {beginWorkspaceTabDrag,workspaceTabDragId,endWorkspaceTabDrag} from '../realtime/workspaceTabDrag.ts';
import { useRealtime } from '../realtime/context.ts';
import { t,loadCatalog } from '../i18n/index.ts';
import * as a11y from './mailA11y.css.ts';
import { CollapsibleBar, useCollapsibleBar, type BarModeHandle } from './bars/index.ts';
import { horizontalWheel } from './horizontalWheel.ts';
import * as barCss from './subTabStripBar.css.ts';
import type { SubTab } from '../realtime/subTabs.ts';

/** DOM id of a sub-tab's `role="tab"` button — referenced by the tablist's aria-owns. */
function tabDomId(id: string): string {
  return `subtab-tab-${id}`;
}

/** `onCloseMany` closes several tabs as one action (one confirmation for unsaved changes); without it each closes in turn. */
export function SubTabStrip(props: { onClose?: (id:string)=>void; onCloseMany?: (ids:string[])=>void; allowTearOff?: boolean; onDetach?: (id:string)=>void; isDetached?: (id:string)=>boolean; onReattach?: (id:string)=>void; canDetach?: (id:string)=>boolean; canClose?: (id:string)=>boolean; bar?: BarModeHandle } = {}): JSX.Element {
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
  const activeTitle = (): string => subTabs.tabs().find((tab) => tab.id === subTabs.activeId())?.title ?? '';
  return (
    <Show when={subTabs.tabs().length > 0}>
      <CollapsibleBar id="workspaceTabs" revealPolicy="overlay" {...(props.bar ? { bar: props.bar } : {})} class={barCss.root} panelClass={barCss.panel} handleLabel={activeTitle()}>
      <div class={`subtab-strip ${barCss.strip}`} ref={horizontalWheel} onKeyDown={onKeyDown}>
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
      </CollapsibleBar>
    </Show>
  );
}

function SubTabButton(props: { tab: SubTab; active: boolean; onClose?: (id:string)=>void; onCloseMany?: (ids:string[])=>void; allowTearOff?: boolean; onDetach?: (id:string)=>void; isDetached?: (id:string)=>boolean; onReattach?: (id:string)=>void; canDetach?: (id:string)=>boolean; canClose?: (id:string)=>boolean;onDragStart:(event:DragEvent)=>void;onDragEnd:()=>void;onDragOver:(event:DragEvent)=>void;onDrop:(event:DragEvent)=>void }): JSX.Element {
  const { subTabs } = useRealtime();
  const id = (): string => props.tab.id;
  let label:HTMLButtonElement|undefined;
  createEffect(()=>{if(props.active)queueMicrotask(()=>{if(props.active)label?.scrollIntoView?.({block:'nearest',inline:'nearest'});});});
  // Closing removes the control that had focus (the tab or its close button). Focus
  // then goes to the tab that is active afterwards, as in the ARIA tabs pattern,
  // instead of falling to <body> where a keyboard or screen-reader user is lost.
  const close=():void=>{if(props.canClose?.(id())===false) return;const owned=label?.parentElement?.contains(document.activeElement)??false;if(props.onClose) props.onClose(id());else subTabs.close(id());
    if(owned)queueMicrotask(()=>{const next=subTabs.activeId();if(next&&next!==id()&&(document.activeElement===null||document.activeElement===document.body))document.getElementById(tabDomId(next))?.focus();});};
  /** Tabs a bulk close may take: not starred, and allowed to close (the Mail home tab is not). */
  const bulk=(tab:SubTab):boolean=>!tab.pinned&&props.canClose?.(tab.id)!==false;
  /** The strip's visual direction at the tab: under `dir="rtl"` the first tab sits on the right. */
  const rtl=():boolean=>(label?.closest('[dir="rtl"],[dir="ltr"]')?.getAttribute('dir')??document.documentElement.dir)==='rtl';
  function sides():{others:string[];left:string[];right:string[];all:string[]} {
    const tabs=subTabs.tabs(),at=tabs.findIndex(tab=>tab.id===id());
    const before=tabs.slice(0,Math.max(0,at)).filter(bulk).map(tab=>tab.id),after=tabs.slice(at+1).filter(bulk).map(tab=>tab.id);
    return {others:[...before,...after],left:rtl()?after:before,right:rtl()?before:after,all:tabs.filter(bulk).map(tab=>tab.id)};
  }
  function closeMany(ids:string[]):void {
    if(ids.length===0)return;
    const owned=label?.parentElement?.contains(document.activeElement)??false;
    if(props.onCloseMany)props.onCloseMany(ids);else for(const each of ids){if(props.onClose)props.onClose(each);else subTabs.close(each);}
    if(owned)queueMicrotask(()=>{const next=subTabs.activeId();if(next&&(document.activeElement===null||document.activeElement===document.body))document.getElementById(tabDomId(next))?.focus();});
  }
  const context=createContextMenu({label:()=>displayText(props.tab.title),scope:()=>`${preferenceAccount()?.generation}:${id()}:${subTabs.tabs().some(tab=>tab.id===id())}`,actions:()=>{
    if(!subTabs.tabs().some(tab=>tab.id===id()))return [];
    const side=sides(),actions:ContextAction[]=[];
    if(props.canClose?.(id())!==false)actions.push({id:'close',label:t('mail-subtab-close',{title:props.tab.title}),icon:'close',run:close});
    actions.push(
      {id:'close-others',label:t('workspace-tabs-close-others'),icon:'close',disabled:side.others.length===0,run:()=>closeMany(sides().others)},
      {id:'close-left',label:t('workspace-tabs-close-left'),icon:'close',disabled:side.left.length===0,run:()=>closeMany(sides().left)},
      {id:'close-right',label:t('workspace-tabs-close-right'),icon:'close',disabled:side.right.length===0,run:()=>closeMany(sides().right)},
      {id:'close-all',label:t('workspace-tabs-close-all'),icon:'close',disabled:side.all.length===0,run:()=>closeMany(sides().all)},
      {id:'star',separatorBefore:true,label:t(props.tab.pinned?'workspace-tabs-unstar':'workspace-tabs-star'),icon:props.tab.pinned?'star':'starOutline',run:()=>{
        const pinned=!props.tab.pinned;subTabs.togglePin(id());toast.success(t(pinned?'mail-toast-tab-pinned':'mail-toast-tab-unpinned',{target:toastTarget(props.tab.title,t('mail-subtabs-label'))}));
      }},
    );
    if(props.isDetached?.(id())){
      if(props.onReattach)actions.push({id:'reattach',separatorBefore:true,label:t('mail-popup-reattach'),icon:'clone',run:()=>props.onReattach?.(id())});
    }else if(props.onDetach&&props.allowTearOff!==false&&props.canDetach?.(id())!==false)actions.push({id:'detach',separatorBefore:true,label:t('mail-subtab-tearoff',{title:props.tab.title}),icon:'clone',run:()=>props.onDetach?.(id())});
    return actions;
  }});
  onCleanup(observePreferenceAccount(()=>context.close(false)));
  // The menu is portaled out of the bar: keep an auto-hide strip open under it.
  const bar=useCollapsibleBar();
  createEffect(()=>{if(context.isOpen()&&bar)onCleanup(bar.hold());});
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
        {displayText(props.tab.title)}
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
