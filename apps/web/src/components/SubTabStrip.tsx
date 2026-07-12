// In-app sub-tab strip (plan §3 e6). A thin view over the realtime controller's
// sub-tab model (realtime/subTabs.ts): a horizontal tablist of open surfaces
// (messages / composers / settings), each activatable, pinnable, closable, and
// tear-off-able into its own window via `window.open`. Keyboard: Left/Right (or
// Ctrl+[Shift+]Tab) cycles focus. Reads `useRealtime()` so it resolves the app
// singleton or a test-provided controller.

import { For, Show, type JSX } from 'solid-js';
import { useRealtime } from '../realtime/context.ts';
import type { SubTab } from '../realtime/subTabs.ts';

export function SubTabStrip(): JSX.Element {
  const { subTabs } = useRealtime();

  function onKeyDown(e: KeyboardEvent): void {
    const next = e.key === 'ArrowRight' || (e.ctrlKey && e.key === 'Tab' && !e.shiftKey);
    const prev = e.key === 'ArrowLeft' || (e.ctrlKey && e.key === 'Tab' && e.shiftKey);
    if (next) {
      subTabs.cycle(1);
      e.preventDefault();
    } else if (prev) {
      subTabs.cycle(-1);
      e.preventDefault();
    }
  }

  return (
    <Show when={subTabs.tabs().length > 0}>
      <div class="subtab-strip" role="tablist" aria-label="Open tabs" onKeyDown={onKeyDown}>
        <For each={subTabs.tabs()}>
          {(tab) => <SubTabButton tab={tab} active={subTabs.activeId() === tab.id} />}
        </For>
      </div>
    </Show>
  );
}

function SubTabButton(props: { tab: SubTab; active: boolean }): JSX.Element {
  const { subTabs } = useRealtime();
  const id = (): string => props.tab.id;
  return (
    <div
      class="subtab"
      classList={{ 'subtab--active': props.active, 'subtab--pinned': props.tab.pinned }}
      data-kind={props.tab.kind}
    >
      <button
        type="button"
        role="tab"
        class="subtab__label"
        aria-selected={props.active}
        tabindex={props.active ? 0 : -1}
        onClick={() => subTabs.activate(id())}
        onAuxClick={(e) => {
          // Middle-click closes, matching browser tab convention.
          if (e.button === 1) subTabs.close(id());
        }}
      >
        {props.tab.title}
      </button>
      <button
        type="button"
        class="subtab__pin"
        aria-label={props.tab.pinned ? `Unpin ${props.tab.title}` : `Pin ${props.tab.title}`}
        aria-pressed={props.tab.pinned}
        onClick={() => subTabs.togglePin(id())}
      >
        {props.tab.pinned ? '★' : '☆'}
      </button>
      <button
        type="button"
        class="subtab__tearoff"
        aria-label={`Open ${props.tab.title} in a new window`}
        onClick={() => subTabs.tearOff(id())}
      >
        ⧉
      </button>
      <button
        type="button"
        class="subtab__close"
        aria-label={`Close ${props.tab.title}`}
        onClick={() => subTabs.close(id())}
      >
        ×
      </button>
    </div>
  );
}
