import { toast } from './toast/index.ts';
import { toastTarget } from './toast/context.ts';
import { Icon } from './Icon.tsx';
// In-app sub-tab strip (plan §3 e6). A thin view over the realtime controller's
// sub-tab model (realtime/subTabs.ts): a horizontal tablist of open surfaces
// (messages / composers / settings), each activatable, pinnable, closable, and
// tear-off-able into its own window via `window.open`. Keyboard: Left/Right (or
// Ctrl+[Shift+]Tab) cycles focus. Reads `useRealtime()` so it resolves the app
// singleton or a test-provided controller.

import { For, Show, type JSX } from 'solid-js';
import { useRealtime } from '../realtime/context.ts';
import { t } from '../i18n/index.ts';
import * as a11y from './mailA11y.css.ts';
import type { SubTab } from '../realtime/subTabs.ts';

/** DOM id of a sub-tab's `role="tab"` button — referenced by the tablist's aria-owns. */
function tabDomId(id: string): string {
  return `subtab-tab-${id}`;
}

export function SubTabStrip(props: { onClose?: (id:string)=>void; allowTearOff?: boolean; canClose?: (id:string)=>boolean } = {}): JSX.Element {
  const { subTabs } = useRealtime();

  function onKeyDown(e: KeyboardEvent): void {
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
          {(tab) => <SubTabButton tab={tab} active={subTabs.activeId() === tab.id} {...props} />}
        </For>
      </div>
    </Show>
  );
}

function SubTabButton(props: { tab: SubTab; active: boolean; onClose?: (id:string)=>void; allowTearOff?: boolean; canClose?: (id:string)=>boolean }): JSX.Element {
  const { subTabs } = useRealtime();
  const id = (): string => props.tab.id;
  const close=():void=>{if(props.canClose?.(id())===false) return;if(props.onClose) props.onClose(id());else subTabs.close(id());};
  return (
    <div
      class="subtab"
      classList={{ 'subtab--active': props.active, 'subtab--pinned': props.tab.pinned }}
      data-kind={props.tab.kind}
    >
      <button
        type="button"
        id={tabDomId(props.tab.id)}
        role="tab"
        class={`subtab__label ${a11y.focusable}`}
        aria-selected={props.active}
        tabindex={props.active ? 0 : -1}
        onClick={() => subTabs.activate(id())}
        onAuxClick={(e) => {
          // Middle-click closes, matching browser tab convention.
          if (e.button === 1) close();
        }}
      >
        {props.tab.title}
      </button>
      <button
        type="button"
        class={`subtab__pin ${a11y.iconButton}`}
        aria-label={props.tab.pinned ? t('mail-subtab-unpin', { title: props.tab.title }) : t('mail-subtab-pin', { title: props.tab.title })}
        aria-pressed={props.tab.pinned}
        onClick={() => {
          const pinned = !props.tab.pinned;
          const target = toastTarget(props.tab.title, t('mail-subtabs-label'));
          subTabs.togglePin(id());
          toast.success(t(pinned ? 'mail-toast-tab-pinned' : 'mail-toast-tab-unpinned', { target }));
        }}
      >
        <Icon name={props.tab.pinned ? 'star' : 'starOutline'} />
      </button>
      <Show when={props.allowTearOff !== false}><button
        type="button"
        class={`subtab__tearoff ${a11y.iconButton}`}
        aria-label={t('mail-subtab-tearoff', { title: props.tab.title })}
        onClick={() => subTabs.tearOff(id())}
      >
        <Icon name="clone" />
      </button></Show>
      <Show when={props.canClose?.(id()) !== false}><button
        type="button"
        class={`subtab__close ${a11y.iconButton}`}
        aria-label={t('mail-subtab-close', { title: props.tab.title })}
        onClick={close}
      >
        <Icon name="close" />
      </button></Show>
    </div>
  );
}
