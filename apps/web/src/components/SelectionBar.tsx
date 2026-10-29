// The floating selection bar (t58).
//
// While one or more messages are selected, a compact bar floats over the bottom
// centre of the list pane: the count, Select all / Select none, the selection's
// actions as icon buttons, a More menu for whatever does not fit on one line,
// and × to clear the selection. It is NON-MODAL: no backdrop, no focus trap, and
// it never takes focus by itself (t51), so clicking, Ctrl/Shift-clicking, the
// arrow keys, Space and scrolling keep working on the list underneath.
//
// Keyboard: the bar is one `role="toolbar"` tab stop after the list (arrow keys
// move between its buttons); F6 in the list jumps to it, and Esc or F6 in the
// bar returns to the list. The count is announced politely from a live region
// that outlives the bar, so the first selection is announced too.
//
// The bar does not know what a message is: a list hands it a model (count,
// select all, clear, actions) — MessageList through `MailSelectionBar`, a folder
// tab through FolderPane.

import { createEffect, createSignal, For, on, onCleanup, onMount, Show, type JSX } from 'solid-js';
import { Icon, type IconName } from './Icon.tsx';
import { createContextMenu, type ContextAction } from './ContextMenu.tsx';
import { loadCatalog, t } from '../i18n/index.ts';
import * as css from './selectionBar.css.ts';
import { registerToastObstacle } from './toast/toast.ts';

/** One action on the selection. `menu` makes it open a menu (Move, Labels). */
export interface SelectionAction {
  id: string;
  label: string;
  icon: IconName;
  run?: () => void | Promise<void>;
  menu?: () => ContextAction[];
  disabled?: boolean;
  testId?: string;
}

export interface SelectionBarProps {
  /** Selected messages; the bar shows while this is above zero. */
  count: number;
  /** Selectable messages in the view; Select all turns into Select none at this count. */
  total: number;
  selectAll: () => void;
  clear: () => void;
  /** In priority order: the first ones stay on the bar, the rest go to More. */
  actions: SelectionAction[];
  /** Put focus back in the list (Esc / F6 in the bar, or the bar closing under focus). */
  returnFocus: () => void;
  /** Scope for the menus: a change (account, folder) closes an open menu. */
  scope: string;
  busy?: boolean;
}

/**
 * How many actions fit on the bar. `fixed` is the width of everything that is
 * not an action (count, select all, ×, padding, gaps); `slot` one action button
 * plus its gap. When they do not all fit, a More button takes one slot. An
 * unmeasured pane (no layout yet, or jsdom) shows every action.
 */
export function inlineCount(total: number, available: number, fixed: number, slot: number): number {
  if (!(available > 0) || !(slot > 0)) return total;
  if (fixed + total * slot <= available) return total;
  return Math.max(0, Math.min(total, Math.floor((available - fixed - slot) / slot)));
}

export function SelectionBar(props: SelectionBarProps): JSX.Element {
  void loadCatalog('selection-bar');
  return (
    <>
      <span class={css.live} role="status" aria-live="polite" aria-atomic="true" data-testid="selection-bar-status">
        {props.count > 0 ? t('selection-bar-announce', { count: props.count }) : ''}
      </span>
      <Show when={props.count > 0}>
        <Bar {...props} />
      </Show>
    </>
  );
}

function Bar(props: SelectionBarProps): JSX.Element {
  let bar!: HTMLDivElement;
  let moreButton: HTMLButtonElement | undefined;
  const [fit, setFit] = createSignal(props.actions.length);
  const [expanded, setExpanded] = createSignal<string | null>(null);
  const all = (): boolean => props.total > 0 && props.count >= props.total;
  const inline = (): SelectionAction[] => props.actions.slice(0, fit());
  const overflow = (): SelectionAction[] => props.actions.slice(fit());

  // One menu at a time: an action's own menu (Move, Labels) or More.
  const [menuItems, setMenuItems] = createSignal<() => ContextAction[]>(() => []);
  const [menuLabel, setMenuLabel] = createSignal('');
  const menu = createContextMenu({
    label: menuLabel,
    scope: () => props.scope,
    actions: () => menuItems()(),
    restoreFocus: () => props.returnFocus(),
  });
  createEffect(() => { if (!menu.isOpen()) setExpanded(null); });
  function openMenu(id: string, label: string, items: () => ContextAction[], trigger: HTMLElement): void {
    if (menu.isOpen() && expanded() === id) { menu.close(); return; }
    setMenuLabel(label);
    setMenuItems(() => items);
    setExpanded(id);
    menu.openAbove(trigger);
  }
  function activate(action: SelectionAction, trigger: HTMLElement): void {
    if (action.disabled || props.busy) return;
    if (action.menu) openMenu(action.id, action.label, action.menu, trigger);
    else void action.run?.();
  }
  const moreItems = (): ContextAction[] =>
    overflow().map((action) => ({
      id: action.id,
      label: action.menu ? `${action.label}…` : action.label,
      icon: action.icon,
      disabled: action.disabled === true || props.busy === true,
      // A submenu opens from the More button, once More has closed.
      run: () => {
        if (action.menu && moreButton) {
          const trigger = moreButton;
          queueMicrotask(() => openMenu(action.id, action.label, action.menu!, trigger));
        } else return action.run?.();
      },
    }));

  // ── fitting the bar to its pane ──────────────────────────────────────────
  function measure(): void {
    const pane = bar.parentElement;
    if (!pane) return;
    const available = pane.clientWidth - 16;
    const style = getComputedStyle(bar);
    const gap = parseFloat(style.columnGap) || 0;
    const children = [...bar.children] as HTMLElement[];
    const slots = children.filter((el) => el.hasAttribute('data-slot'));
    const others = children.filter((el) => !el.hasAttribute('data-slot'));
    const slot = (slots[0]?.offsetWidth ?? 0) + gap;
    // Not `bar.offsetWidth`: the bar is capped at the pane's width, so it would
    // hide exactly the overflow this is measuring.
    const edges = ['paddingLeft', 'paddingRight', 'borderLeftWidth', 'borderRightWidth'] as const;
    const fixed = edges.reduce((sum, key) => sum + (parseFloat(style[key]) || 0), 0)
      + others.reduce((sum, el) => sum + el.offsetWidth, 0) + Math.max(0, others.length - 1) * gap;
    setFit(inlineCount(props.actions.length, available, fixed, slot));
  }
  onMount(() => {
    measure();
    if (typeof ResizeObserver === 'function' && bar.parentElement) {
      const observer = new ResizeObserver(() => measure());
      observer.observe(bar.parentElement);
      onCleanup(() => observer.disconnect());
    }
  });
  // The count's width and the number of actions change what fits.
  createEffect(on(() => [props.count, props.actions.length, props.busy], () => queueMicrotask(measure), { defer: true }));

  // ── keyboard ─────────────────────────────────────────────────────────────
  const controls = (): HTMLButtonElement[] => [...bar.querySelectorAll<HTMLButtonElement>('button')];
  let stop = 0;
  function syncStops(): void {
    const list = controls();
    if (list.length === 0) return;
    if (stop >= list.length) stop = list.length - 1;
    list.forEach((el, i) => { el.tabIndex = i === stop ? 0 : -1; });
  }
  createEffect(on(() => [fit(), props.actions.length, all()], () => queueMicrotask(syncStops)));
  function focusBar(): void {
    syncStops();
    controls()[stop]?.focus();
  }
  function onKeyDown(e: KeyboardEvent): void {
    if (e.key === 'Escape' || e.key === 'F6') {
      e.preventDefault();
      e.stopPropagation();
      props.returnFocus();
      return;
    }
    const list = controls();
    const at = list.indexOf(e.target as HTMLButtonElement);
    if (at < 0) return;
    const rtl = getComputedStyle(bar).direction === 'rtl';
    const forward = rtl ? 'ArrowLeft' : 'ArrowRight';
    const back = rtl ? 'ArrowRight' : 'ArrowLeft';
    let next: number;
    if (e.key === forward) next = (at + 1) % list.length;
    else if (e.key === back) next = (at - 1 + list.length) % list.length;
    else if (e.key === 'Home') next = 0;
    else if (e.key === 'End') next = list.length - 1;
    else return;
    e.preventDefault();
    stop = next;
    syncStops();
    list[next]?.focus();
  }
  // F6 from anywhere else in the pane (the list) jumps to the bar.
  onMount(() => {
    const pane = bar.parentElement;
    if (!pane) return;
    const onPaneKey = (e: KeyboardEvent): void => {
      if (e.key !== 'F6' || e.defaultPrevented || e.altKey || e.ctrlKey || e.metaKey) return;
      if (e.target instanceof Node && bar.contains(e.target)) return;
      e.preventDefault();
      focusBar();
    };
    pane.addEventListener('keydown', onPaneKey);
    onCleanup(() => pane.removeEventListener('keydown', onPaneKey));
  });
  // Toasts sit above the bar while it is shown, not over its end (t67).
  onMount(() => onCleanup(registerToastObstacle(bar)));
  // The bar going away under focus (the last selected message archived) hands
  // focus back to the list instead of dropping it on <body>.
  onCleanup(() => {
    if (bar.contains(document.activeElement)) queueMicrotask(() => props.returnFocus());
  });

  return (
    <div
      ref={bar}
      class={css.bar}
      role="toolbar"
      aria-label={t('selection-bar-label')}
      aria-keyshortcuts="F6"
      data-testid="bulk-bar"
      onKeyDown={onKeyDown}
      onFocusIn={(e) => { const at = controls().indexOf(e.target as HTMLButtonElement); if (at >= 0) { stop = at; syncStops(); } }}
    >
      <span class={css.count} data-testid="bulk-count">{t('selection-bar-count', { count: props.count })}</span>
      <button
        type="button"
        class={css.button}
        aria-label={all() ? t('selection-bar-select-none') : t('selection-bar-select-all')}
        data-testid="bulk-select-all"
        onClick={() => (all() ? props.clear() : props.selectAll())}
      >
        <Icon name={all() ? 'minus' : 'check'} />
      </button>
      <span class={css.separator} aria-hidden="true" />
      {/* Keyed by id, not by object: the actions are rebuilt on every change
          (a label flips to "Mark unread"), and a re-created button would drop
          the focus it holds. */}
      <For each={inline().map((action) => action.id)}>
        {(id) => {
          const action = (): SelectionAction => props.actions.find((item) => item.id === id) ?? { id, label: '', icon: 'more', disabled: true };
          return (
          <button
            type="button"
            class={css.button}
            data-slot=""
            data-testid={action().testId}
            aria-label={action().label}
            aria-haspopup={action().menu ? 'menu' : undefined}
            aria-expanded={action().menu ? expanded() === id : undefined}
            disabled={action().disabled === true || props.busy === true}
            onClick={(e) => activate(action(), e.currentTarget)}
          >
            <Icon name={action().icon} />
          </button>
          );
        }}
      </For>
      <Show when={overflow().length > 0}>
        <button
          ref={moreButton}
          type="button"
          class={css.button}
          data-slot=""
          data-testid="bulk-more"
          aria-label={t('selection-bar-more')}
          aria-haspopup="menu"
          aria-expanded={expanded() === 'more'}
          onClick={(e) => openMenu('more', t('selection-bar-more'), moreItems, e.currentTarget)}
        >
          <Icon name="more" />
        </button>
      </Show>
      <span class={css.separator} aria-hidden="true" />
      <button type="button" class={css.button} aria-label={t('selection-bar-clear')} data-testid="bulk-clear" onClick={() => props.clear()}>
        <Icon name="close" />
      </button>
      {menu.menu()}
    </div>
  );
}
