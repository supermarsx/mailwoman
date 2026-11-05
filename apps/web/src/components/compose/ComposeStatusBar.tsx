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
  /** Overrides the measured width (tests, where there is no layout). */
  width?: number;
}

function ItemView(props: { item: StatusItem }): JSX.Element {
  const item = props.item;
  const body = (
    <>
      <span class="compose-status__icon" classList={{ 'compose-status__icon--spin': item.severity === 'busy' }}><Icon name={item.icon} /></span>
      <span class="compose-status__text">{item.label}</span>
      <Show when={item.detail}><span class="compose-status__sr"> — {item.detail}</span></Show>
    </>
  );
  return (
    <span
      class="compose-status__item"
      data-severity={item.severity}
      data-subdued={item.subdued ? '' : undefined}
      data-status-id={item.id}
      data-testid={item.testId ?? `compose-status-${item.id}`}
      role={item.assertive ? 'alert' : undefined}
    >
      <Show
        when={item.onClick}
        fallback={<span class="compose-status__body" data-tooltip={item.detail}>{body}</span>}
      >
        <button type="button" class="compose-status__body compose-status__btn" data-tooltip={item.detail} onClick={() => item.onClick?.()}>{body}</button>
      </Show>
      <Show when={item.actionLabel && item.onAction}>
        <button type="button" class="compose-status__action" onClick={() => item.onAction?.()}>{item.actionLabel}</button>
      </Show>
    </span>
  );
}

export function ComposeStatusBar(props: ComposeStatusBarProps): JSX.Element {
  const popId = `compose-status-pop-${createUniqueId()}`;
  const [measured, setMeasured] = createSignal(0);
  const [open, setOpen] = createSignal(false);
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
  createEffect(() => {
    if (split().overflow.length === 0) setOpen(false);
  });
  const onPointer = (event: Event): void => {
    if (open() && event.target instanceof Node && !root?.contains(event.target)) setOpen(false);
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
        if (event.key === 'Escape' && open()) {
          event.stopPropagation();
          setOpen(false);
        }
      }}
    >
      <div class="compose-status__row">
        <For each={split().inline}>{(item) => <ItemView item={item} />}</For>
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
        <For each={split().overflow}>{(item) => <ItemView item={item} />}</For>
      </div>
    </div>
  );
}
