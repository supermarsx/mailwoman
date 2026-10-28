// `<CollapsibleBar>`: a bar the user pins open or sets to auto-hide, like the
// Office ribbon (t38 §2.1).
//
// Pinned: the panel sits in the flow and is always shown. Auto: only the
// `header` (a ribbon's tab names) or, without one, a slim handle stays on
// screen, and the panel opens on
//   - pointer hover over the bar, after a 150 ms intent delay (not bound on
//     touch-only devices, and not while the user is typing in a field outside
//     the bar);
//   - a click or tap on the handle, or a tab calling `open()` from the context;
//   - keyboard focus entering the bar.
// It closes when the pointer has been outside for 400 ms and focus is not
// inside, when focus leaves, on Esc (focus goes back to what opened it), and
// on a tap or click outside. It never closes while something inside holds it
// open (`hold()`: a group dropdown, a menu, a popover).
//
// `overlay` lays the open panel over the content below; `push` puts it in the
// flow so the content shrinks and is never covered (the composer), and keeps
// the caret in view once the panel is open.

import {
  children as resolveChildren,
  createContext,
  createEffect,
  createSignal,
  on,
  onCleanup,
  onMount,
  Show,
  useContext,
  type Accessor,
  type JSX,
} from 'solid-js';
import { loadCatalog, t } from '../../i18n/index.ts';
import { Icon } from '../Icon.tsx';
import { useBarMode, type BarId, type BarMode, type BarModeHandle } from './useBarMode.ts';
import * as css from './collapsibleBar.css.ts';

export type RevealPolicy = 'overlay' | 'push';

export const HOVER_OPEN_MS = 150;
export const LEAVE_CLOSE_MS = 400;
export const TYPING_GUARD_MS = 1500;

interface BarStrings {
  pin: string;
  unpin: string;
  show: string;
  hide: string;
}

// Static ids so the catalog gate sees every one of them.
const STRINGS: Record<BarId, () => BarStrings> = {
  mailTopBar: () => ({
    pin: t('bars-pin-mail-top-bar'),
    unpin: t('bars-unpin-mail-top-bar'),
    show: t('bars-show-mail-top-bar'),
    hide: t('bars-hide-mail-top-bar'),
  }),
  workspaceTabs: () => ({
    pin: t('bars-pin-workspace-tabs'),
    unpin: t('bars-unpin-workspace-tabs'),
    show: t('bars-show-workspace-tabs'),
    hide: t('bars-hide-workspace-tabs'),
  }),
  mailRibbon: () => ({
    pin: t('bars-pin-mail-ribbon'),
    unpin: t('bars-unpin-mail-ribbon'),
    show: t('bars-show-mail-ribbon'),
    hide: t('bars-hide-mail-ribbon'),
  }),
  composerRibbon: () => ({
    pin: t('bars-pin-composer-ribbon'),
    unpin: t('bars-unpin-composer-ribbon'),
    show: t('bars-show-composer-ribbon'),
    hide: t('bars-hide-composer-ribbon'),
  }),
};

export interface CollapsibleBarApi {
  readonly id: BarId;
  /** The DOM id of the panel (for `aria-controls` on tab buttons). */
  readonly panelId: string;
  mode: Accessor<BarMode>;
  /** The panel is on screen: pinned, or revealed in auto mode. */
  expanded: Accessor<boolean>;
  /** Reveal the panel (auto mode); `trigger` gets focus back on Esc. No-op when pinned. */
  open(trigger?: HTMLElement | null): void;
  /** Hide the panel (auto mode). `restoreFocus` moves focus to the element that opened it. */
  close(opts?: { restoreFocus?: boolean }): void;
  /** Keep the panel open until the returned function runs (menus, popovers, dropdowns). */
  hold(): () => void;
}

const BarContext = createContext<CollapsibleBarApi>();

/** The enclosing bar, or `undefined` outside one (a ribbon group used on its own). */
export function useCollapsibleBar(): CollapsibleBarApi | undefined {
  return useContext(BarContext);
}

export interface CollapsibleBarProps {
  id: BarId;
  revealPolicy: RevealPolicy;
  /** The mode handle; defaults to `useBarMode(id)`. Pass one to share it or to test. */
  bar?: BarModeHandle;
  /** An always-visible row above the panel (a ribbon's tab names). Replaces the slim handle. */
  header?: JSX.Element;
  /** Text on the slim handle (the workspace strip shows the active tab's title). */
  handleLabel?: string;
  /** Accessible name of the panel; omit when the content names itself (role=toolbar). */
  label?: string;
  class?: string;
  panelClass?: string;
  children?: JSX.Element;
}

type Query = MediaQueryList | null;
function media(query: string): Query {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') return null;
  try {
    return window.matchMedia(query);
  } catch {
    return null;
  }
}

/** Touch-only devices: no hover reveal; a tap is the only pointer path. */
function hoverUnavailable(): boolean {
  return media('(hover: none)')?.matches === true || media('(pointer: coarse)')?.matches === true;
}

function reducedMotion(): boolean {
  return (
    document.documentElement.hasAttribute('data-reduced-motion') ||
    media('(prefers-reduced-motion: reduce)')?.matches === true
  );
}

function isEditable(element: Element | null): boolean {
  if (!(element instanceof HTMLElement)) return false;
  if (element.isContentEditable || element.closest('[contenteditable=""],[contenteditable="true"]')) return true;
  if (element instanceof HTMLTextAreaElement) return !element.readOnly;
  if (element instanceof HTMLInputElement) {
    return !element.readOnly && !['button', 'checkbox', 'radio', 'submit', 'reset', 'range', 'color', 'file'].includes(element.type);
  }
  return false;
}

// Typing guard: the last key pressed anywhere (one listener for every bar).
let lastKeyAt = Number.NEGATIVE_INFINITY;
let keyListeners = 0;
const recordKey = (): void => {
  lastKeyAt = Date.now();
};

export function CollapsibleBar(props: CollapsibleBarProps): JSX.Element {
  onMount(() => void loadCatalog('bars'));
  const bar = props.bar ?? useBarMode(props.id);
  const panelId = `mw-bar-${props.id}`;
  const strings = (): BarStrings => STRINGS[props.id]();
  const pinned = (): boolean => bar.mode() === 'pinned';

  const [revealed, setRevealed] = createSignal(false);
  const [motion, setMotion] = createSignal<'full' | 'reduced'>('full');
  const expanded = (): boolean => pinned() || revealed();

  let rootEl: HTMLDivElement | undefined;
  let handleEl: HTMLButtonElement | undefined;
  let trigger: HTMLElement | null = null;
  let holds = 0;
  let pointerInside = false;
  let pointerPressed = false;
  let restoringFocus = false;
  let openedByHover = false;
  let openTimer: ReturnType<typeof setTimeout> | undefined;
  let closeTimer: ReturnType<typeof setTimeout> | undefined;
  let caretTimer: ReturnType<typeof setTimeout> | undefined;

  const clearTimers = (): void => {
    clearTimeout(openTimer);
    clearTimeout(closeTimer);
    openTimer = closeTimer = undefined;
  };
  const focusInside = (): boolean => rootEl?.contains(document.activeElement) === true;

  // Push mode: once the panel has slid in, scroll the caret (outside the bar) back into view.
  function keepCaretVisible(): void {
    clearTimeout(caretTimer);
    caretTimer = setTimeout(
      () => {
        const node = window.getSelection?.()?.focusNode ?? null;
        const element = node instanceof Element ? node : (node?.parentElement ?? null);
        if (element && element.isConnected && !rootEl?.contains(element)) element.scrollIntoView?.({ block: 'nearest' });
      },
      motion() === 'reduced' ? 0 : css.REVEAL_MS,
    );
  }

  function open(by?: HTMLElement | null): void {
    if (pinned()) return;
    clearTimers();
    if (by) trigger = by;
    openedByHover = false;
    if (revealed()) return;
    setMotion(reducedMotion() ? 'reduced' : 'full');
    setRevealed(true);
    if (props.revealPolicy === 'push') keepCaretVisible();
  }

  function close(opts: { restoreFocus?: boolean } = {}): void {
    clearTimers();
    if (!revealed()) return;
    setMotion(reducedMotion() ? 'reduced' : 'full');
    setRevealed(false);
    if (opts.restoreFocus) {
      const target = trigger?.isConnected ? trigger : handleEl;
      restoringFocus = true;
      target?.focus();
      restoringFocus = false;
    }
  }

  /** Close after the leave delay, unless focus, the pointer or a hold keeps it. */
  function scheduleClose(): void {
    clearTimeout(closeTimer);
    closeTimer = setTimeout(() => {
      closeTimer = undefined;
      if (!pointerInside && !focusInside() && holds === 0) close();
    }, LEAVE_CLOSE_MS);
  }

  function hold(): () => void {
    holds++;
    let released = false;
    return () => {
      if (released) return;
      released = true;
      holds--;
      if (holds === 0 && revealed() && !pointerInside && !focusInside()) scheduleClose();
    };
  }

  function typingElsewhere(): boolean {
    const active = document.activeElement;
    return isEditable(active) && !rootEl?.contains(active) && Date.now() - lastKeyAt < TYPING_GUARD_MS;
  }

  // ── pointer hover (not bound on touch-only devices) ─────────────────────────
  const onPointerEnter = (event: Event): void => {
    if ((event as PointerEvent).pointerType === 'touch') return;
    pointerInside = true;
    clearTimeout(closeTimer);
    closeTimer = undefined;
    if (expanded() || typingElsewhere()) return;
    clearTimeout(openTimer);
    openTimer = setTimeout(() => {
      openTimer = undefined;
      if (pointerInside && !typingElsewhere()) {
        open();
        openedByHover = true;
      }
    }, HOVER_OPEN_MS);
  };
  const onPointerLeave = (event: Event): void => {
    if ((event as PointerEvent).pointerType === 'touch') return;
    pointerInside = false;
    clearTimeout(openTimer);
    openTimer = undefined;
    if (revealed()) scheduleClose();
  };

  let hoverBound = false;
  function bindHover(enable: boolean): void {
    if (!rootEl || enable === hoverBound) return;
    hoverBound = enable;
    if (enable) {
      rootEl.addEventListener('pointerenter', onPointerEnter);
      rootEl.addEventListener('pointerleave', onPointerLeave);
    } else {
      rootEl.removeEventListener('pointerenter', onPointerEnter);
      rootEl.removeEventListener('pointerleave', onPointerLeave);
      pointerInside = false;
    }
  }

  // A tap or click outside closes an open auto bar.
  const onDocumentPointerDown = (event: Event): void => {
    const target = event.target;
    if (!revealed() || holds > 0 || !(target instanceof Node) || rootEl?.contains(target)) return;
    close();
  };

  onMount(() => {
    bindHover(!hoverUnavailable());
    const queries = [media('(hover: none)'), media('(pointer: coarse)')];
    const onQuery = (): void => bindHover(!hoverUnavailable());
    for (const q of queries) q?.addEventListener?.('change', onQuery);
    document.addEventListener('pointerdown', onDocumentPointerDown, true);
    if (keyListeners++ === 0) document.addEventListener('keydown', recordKey, true);
    onCleanup(() => {
      bindHover(false);
      for (const q of queries) q?.removeEventListener?.('change', onQuery);
      document.removeEventListener('pointerdown', onDocumentPointerDown, true);
      if (--keyListeners === 0) document.removeEventListener('keydown', recordKey, true);
    });
  });
  onCleanup(() => {
    clearTimers();
    clearTimeout(caretTimer);
  });

  // Pinning closes the transient reveal; the panel is in the flow from then on.
  createEffect(on(pinned, (isPinned) => {
    if (isPinned) {
      clearTimers();
      setRevealed(false);
    }
  }, { defer: true }));

  // ── focus and keyboard ──────────────────────────────────────────────────────
  const onPointerDownInside = (): void => {
    pointerPressed = true;
    setTimeout(() => (pointerPressed = false), 0);
  };
  const onFocusIn = (event: FocusEvent): void => {
    // A click decides for itself (the handle toggles, a tab opens); a keyboard
    // focus opens the bar.
    if (restoringFocus || pointerPressed || pinned() || revealed()) return;
    open(event.target instanceof HTMLElement && event.target !== rootEl ? event.target : null);
  };
  const onFocusOut = (event: FocusEvent): void => {
    const next = event.relatedTarget;
    if (next instanceof Node && rootEl?.contains(next)) return;
    // Wait a tick: a click on a non-focusable part of the bar blurs to <body> first.
    setTimeout(() => {
      if (revealed() && !focusInside() && !pointerInside && holds === 0) close();
    }, 0);
  };
  const onKeyDown = (event: KeyboardEvent): void => {
    if (event.key !== 'Escape' || event.defaultPrevented || pinned() || !revealed() || holds > 0) return;
    event.preventDefault();
    event.stopPropagation();
    close({ restoreFocus: true });
  };

  const onHandleClick = (event: MouseEvent): void => {
    const target = event.currentTarget as HTMLElement;
    // A click on a bar that hover has just opened keeps it open (and makes it stay).
    if (!revealed() || openedByHover) open(target);
    else close();
  };

  async function togglePin(): Promise<void> {
    if (bar.locked()) return;
    if (pinned()) {
      // Stay open under the user's pointer; the usual leave and blur rules close it.
      trigger = handleEl ?? null;
      setRevealed(true);
      await bar.setMode('auto');
    } else {
      await bar.setMode('pinned');
    }
  }

  const api: CollapsibleBarApi = {
    get id() {
      return props.id;
    },
    panelId,
    mode: () => bar.mode(),
    expanded,
    open,
    close,
    hold,
  };

  // Rendered inside the provider, so the header (ribbon tabs) can reach the bar.
  const Frame = (): JSX.Element => {
    const header = resolveChildren(() => props.header);
    return (
      <div
        ref={rootEl}
        class={[css.root, props.class ?? ''].join(' ').trim()}
        data-bar={props.id}
        data-mode={bar.mode()}
        data-expanded={expanded() ? 'true' : 'false'}
        data-reveal={props.revealPolicy}
        data-motion={motion()}
        onPointerDown={onPointerDownInside}
        onFocusIn={onFocusIn}
        onFocusOut={onFocusOut}
        onKeyDown={onKeyDown}
      >
        <Show
          when={header.toArray().length > 0}
          fallback={
            <Show when={!pinned()}>
              <button
                ref={handleEl}
                type="button"
                class={css.handle}
                data-bar-handle={props.id}
                aria-expanded={expanded() ? 'true' : 'false'}
                aria-controls={panelId}
                aria-label={expanded() ? strings().hide : strings().show}
                data-tooltip={expanded() ? strings().hide : strings().show}
                onClick={onHandleClick}
              >
                <Show when={props.handleLabel}>{(text) => <span class={css.handleText}>{text()}</span>}</Show>
              </button>
            </Show>
          }
        >
          <div class={css.header} data-bar-header={props.id}>
            {header()}
          </div>
        </Show>
        <div
          ref={(el) => createEffect(() => el.toggleAttribute('inert', !expanded()))}
          id={panelId}
          class={[css.panel, props.panelClass ?? ''].join(' ').trim()}
          data-bar-panel={props.id}
          role={props.label ? 'region' : undefined}
          aria-label={props.label}
        >
          <div class={css.content}>{props.children}</div>
          <button
            type="button"
            class={css.pin}
            data-bar-pin={props.id}
            aria-pressed={pinned() ? 'true' : 'false'}
            aria-label={pinned() ? strings().unpin : strings().pin}
            data-tooltip={bar.locked() ? t('bars-locked') : pinned() ? strings().unpin : strings().pin}
            disabled={bar.locked()}
            onClick={() => void togglePin()}
          >
            <Icon name="pin" />
          </button>
        </div>
      </div>
    );
  };

  return (
    <BarContext.Provider value={api}>
      <Frame />
    </BarContext.Provider>
  );
}
