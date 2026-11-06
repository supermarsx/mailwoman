// The composer's bottom status bar (see statusBar.ts). A fixed-height strip,
// always present under the editor. Items are compact icon + text; a count chip
// folds what does not fit into a popover that opens upward over the editor, so
// the layout never moves. Severity is carried by the icon and the text as well
// as the colour. The strip is a polite live region; an item flagged assertive
// (an error) is announced at once.

import { createEffect, createSignal, createUniqueId, For, onCleanup, onMount, Show, type JSX } from 'solid-js';
import { Icon } from '../Icon.tsx';
import { t } from '../../i18n/index.ts';
import { inlineCapacity, splitItems, type StatusItem } from './statusBar.ts';
import './compose-chrome.css';

export interface ComposeStatusBarProps {
  items: () => readonly StatusItem[];
  /** Counters and the like, on the leading edge of the same line; dropped first on a narrow bar. */
  start?: JSX.Element;
  /** Overrides the measured width (tests, where there is no layout). */
  width?: number;
}

interface ItemViewProps {
  item: StatusItem;
  /** Id of the item whose detail popover is open (hover, focus or click). */
  openId: () => string | null;
  popId: string;
  onHover: (id: string | null) => void;
  onToggle: (id: string) => void;
  inPopover?: boolean;
}

/** A detail the chip cannot hold goes in the popover, together with a non-urgent action ("Dismiss"). */
function hasPopover(item: StatusItem): boolean {
  return !!item.detail && !item.onClick && (!!item.actionLabel || (item.severity !== 'success' && item.severity !== 'busy'));
}
function actionInPopover(item: StatusItem): boolean {
  return !!item.detail && !!item.actionLabel && item.severity === 'neutral';
}

function ItemView(props: ItemViewProps): JSX.Element {
  const item = props.item;
  const popover = () => !props.inPopover && hasPopover(item);
  const body = (
    <>
      <span class="compose-status__icon" classList={{ 'compose-status__icon--spin': item.severity === 'busy' }}><Icon name={item.icon} /></span>
      <span class="compose-status__text">{item.label}</span>
      <Show when={item.detail}><span class="compose-status__sr"> — {item.detail}</span></Show>
    </>
  );
  const hover = {
    onMouseEnter: () => { if (popover()) props.onHover(item.id); },
    onMouseLeave: () => props.onHover(null),
    onFocusIn: () => { if (popover()) props.onHover(item.id); },
    onFocusOut: () => props.onHover(null),
  };
  return (
    <span
      class="compose-status__item"
      data-severity={item.severity}
      data-subdued={item.subdued ? '' : undefined}
      data-status-id={item.id}
      data-testid={item.testId ?? `compose-status-${item.id}`}
      role={item.assertive ? 'alert' : undefined}
      {...hover}
    >
      <Show
        when={item.onClick || popover()}
        fallback={<span class="compose-status__body" title={item.detail}>{body}</span>}
      >
        <button
          type="button"
          class="compose-status__body compose-status__btn"
          title={item.detail && !popover() ? item.detail : undefined}
          aria-haspopup={popover() ? 'dialog' : undefined}
          aria-expanded={popover() ? (props.openId() === item.id ? 'true' : 'false') : undefined}
          aria-controls={popover() ? props.popId : undefined}
          onClick={() => (item.onClick ? item.onClick() : props.onToggle(item.id))}
        >{body}</button>
      </Show>
      <Show when={item.actionLabel && item.onAction && (props.inPopover || !actionInPopover(item))}>
        <button type="button" class="compose-status__action" onClick={() => item.onAction?.()}>{item.actionLabel}</button>
      </Show>
    </span>
  );
}

export function ComposeStatusBar(props: ComposeStatusBarProps): JSX.Element {
  const popId = `compose-status-pop-${createUniqueId()}`;
  const [measured, setMeasured] = createSignal(0);
  const [open, setOpen] = createSignal(false);
  const [pinned, setPinned] = createSignal<string | null>(null);
  const [hovered, setHovered] = createSignal<string | null>(null);
  const detailId = `${popId}-detail`;
  const openId = () => pinned() ?? hovered();
  let root: HTMLDivElement | undefined;
  onMount(() => {
    if (!root) return;
    setMeasured(root.clientWidth);
    if (typeof ResizeObserver === 'undefined') return;
    const observer = new ResizeObserver(() => setMeasured(root?.clientWidth ?? 0));
    observer.observe(root);
    onCleanup(() => observer.disconnect());
  });
  const split = () => splitItems(props.items(), inlineCapacity(props.width ?? measured()));
  const shared = {
    openId,
    popId: detailId,
    onHover: (id: string | null) => setHovered(id),
    onToggle: (id: string) => setPinned(pinned() === id ? null : id),
  };
  const shownDetail = () => {
    const id = openId();
    return id ? props.items().find((item) => item.id === id && hasPopover(item)) : undefined;
  };
  createEffect(() => {
    if (split().overflow.length === 0) setOpen(false);
  });
  const onPointer = (event: Event): void => {
    if (event.target instanceof Node && !root?.contains(event.target)) { setOpen(false); setPinned(null); }
  };
  onMount(() => {
    const doc = root?.ownerDocument ?? document;
    doc.addEventListener('pointerdown', onPointer, true);
    onCleanup(() => doc.removeEventListener('pointerdown', onPointer, true));
  });
  return (
    <div
      ref={root}
      class="compose-status"
      role="status"
      aria-live="polite"
      aria-label={t('mail-status-label')}
      data-testid="compose-status"
      data-detached-local=""
      onKeyDown={(event) => {
        if (event.key === 'Escape' && (open() || openId())) {
          event.stopPropagation();
          setOpen(false);
          setPinned(null);
          setHovered(null);
        }
      }}
    >
      <Show when={props.start}><div class="compose-status__start" data-detached-local="">{props.start}</div></Show>
      <div class="compose-status__row">
        <For each={split().inline}>{(item) => <ItemView item={item} {...shared} />}</For>
      </div>
      <Show when={split().overflow.length > 0}>
        <button
          type="button"
          class="compose-status__more"
          data-testid="compose-status-more"
          aria-expanded={open() ? 'true' : 'false'}
          aria-controls={popId}
          aria-label={t('mail-status-more', { count: split().overflow.length })}
          onClick={() => setOpen(!open())}
        >
          <Icon name="chevronUp" />
          <span>+{split().overflow.length}</span>
        </button>
      </Show>
      <div id={popId} class="compose-status__pop" role="group" aria-label={t('mail-status-more-label')} hidden={!open()} data-testid="compose-status-pop">
        <For each={split().overflow}>{(item) => <ItemView item={item} {...shared} inPopover />}</For>
      </div>
      <Show when={shownDetail()}>
        {(item) => (
          <div id={detailId} class="compose-status__pop compose-status__detail" role="dialog" aria-label={item().label} data-testid="compose-status-detail" onMouseEnter={() => setHovered(item().id)} onMouseLeave={() => setHovered(null)}>
            <p>{item().detail}</p>
            <Show when={item().actionLabel && item().onAction}>
              <button type="button" class="compose-status__action" onClick={() => { item().onAction?.(); setPinned(null); setHovered(null); }}>{item().actionLabel}</button>
            </Show>
          </div>
        )}
      </Show>
    </div>
  );
}
