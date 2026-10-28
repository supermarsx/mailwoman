// `<RibbonGroup>` and `<RibbonItem>`: a labelled block of ribbon controls at
// exactly three rows (t38 §2.2).
//
// Inside a `<RibbonPanel>`, a group too wide for the ribbon folds into one
// `large` dropdown button (its icon and label). The button opens a popover
// holding the same controls in the same 3-row grid; the controls are moved,
// not re-created, so their state survives folding. While the popover is open
// it holds the enclosing `<CollapsibleBar>` open.

import {
  createEffect,
  createSignal,
  createUniqueId,
  on,
  onCleanup,
  onMount,
  Show,
  type JSX,
} from 'solid-js';
import { loadCatalog, t } from '../../i18n/index.ts';
import { Icon, type IconName } from '../Icon.tsx';
import { useCollapsibleBar } from './CollapsibleBar.tsx';
import { useRibbonPanel } from './RibbonPanel.tsx';
import * as css from './ribbonGroup.css.ts';

export interface RibbonGroupProps {
  /** The caption under the group, and the folded button's text. */
  label: string;
  /** Higher stays open longer when the ribbon is too narrow. Default 0. */
  priority?: number;
  /** Shown on the folded button. */
  icon?: IconName;
  class?: string;
  /** `data-testid` of the group element. */
  testId?: string;
  children?: JSX.Element;
}

export function RibbonGroup(props: RibbonGroupProps): JSX.Element {
  onMount(() => void loadCatalog('bars'));
  const panel = useRibbonPanel();
  const bar = useCollapsibleBar();
  const popoverId = `mw-ribbon-group-${createUniqueId()}`;
  let groupEl: HTMLDivElement | undefined;
  let buttonEl: HTMLButtonElement | undefined;
  let popoverEl: HTMLDivElement | undefined;

  const registration = panel?.register({
    priority: () => props.priority ?? 0,
    element: () => groupEl,
  });
  onCleanup(() => registration?.dispose());
  const folded = (): boolean => registration?.folded() ?? false;

  const [popoverOpen, setPopoverOpen] = createSignal(false);
  const [position, setPosition] = createSignal<{ top: number; left?: number; right?: number }>({ top: 0 });
  let release: (() => void) | null = null;

  // One controls element, moved between the group and the popover.
  const controls = (
    <div class={css.controls} data-ribbon-controls="">
      {props.children}
    </div>
  ) as HTMLDivElement;

  function place(): void {
    const rect = buttonEl?.getBoundingClientRect();
    if (!rect) return;
    const rtl = getComputedStyle(buttonEl!).direction === 'rtl';
    const top = rect.bottom + 2;
    if (rtl) setPosition({ top, right: Math.max(8, window.innerWidth - rect.right) });
    else setPosition({ top, left: Math.max(8, Math.min(rect.left, window.innerWidth - 8 - (popoverEl?.offsetWidth ?? 0))) });
  }

  function openPopover(): void {
    if (popoverOpen()) return;
    place();
    setPopoverOpen(true);
    release = bar?.hold() ?? null;
    queueMicrotask(place);
  }

  function closePopover(opts: { restoreFocus?: boolean } = {}): void {
    if (!popoverOpen()) return;
    setPopoverOpen(false);
    release?.();
    release = null;
    if (opts.restoreFocus) buttonEl?.focus();
  }

  // Opening the group again (the ribbon grew) closes its popover.
  createEffect(on(folded, (isFolded) => {
    if (!isFolded) closePopover();
  }, { defer: true }));

  const onDocumentPointerDown = (event: Event): void => {
    if (!popoverOpen() || !(event.target instanceof Node) || groupEl?.contains(event.target)) return;
    closePopover();
  };
  onMount(() => document.addEventListener('pointerdown', onDocumentPointerDown, true));
  onCleanup(() => {
    document.removeEventListener('pointerdown', onDocumentPointerDown, true);
    release?.();
  });

  const onKeyDown = (event: KeyboardEvent): void => {
    if (event.key !== 'Escape' || !popoverOpen()) return;
    event.preventDefault();
    event.stopPropagation();
    closePopover({ restoreFocus: true });
  };
  const onFocusOut = (event: FocusEvent): void => {
    const next = event.relatedTarget;
    if (!popoverOpen() || (next instanceof Node && groupEl?.contains(next))) return;
    if (next instanceof Node) closePopover();
  };

  return (
    <div
      ref={groupEl}
      class={[css.group, props.class ?? ''].join(' ').trim()}
      role="group"
      aria-label={props.label}
      data-ribbon-group=""
      data-priority={props.priority ?? 0}
      data-folded={folded() ? 'true' : 'false'}
      data-testid={props.testId}
      onKeyDown={onKeyDown}
      onFocusOut={onFocusOut}
    >
      <div class={css.body} data-ribbon-body="">
        <Show when={folded()} fallback={controls}>
          <button
            ref={buttonEl}
            type="button"
            class={css.more}
            data-ribbon-more=""
            data-detached-local=""
            aria-haspopup="true"
            aria-expanded={popoverOpen() ? 'true' : 'false'}
            aria-controls={popoverId}
            aria-label={t('bars-group-more-label', { group: props.label })}
            onClick={() => (popoverOpen() ? closePopover() : openPopover())}
          >
            <Show when={props.icon}>{(name) => <Icon name={name()} />}</Show>
            <span class={css.moreText}>{t('bars-group-more', { group: props.label })}</span>
            <Icon name="chevronDown" />
          </button>
          <div
            ref={popoverEl}
            id={popoverId}
            class={css.popover}
            role="group"
            aria-label={props.label}
            data-ribbon-popover=""
            hidden={!popoverOpen()}
            style={{
              top: `${position().top}px`,
              left: position().left === undefined ? undefined : `${position().left}px`,
              right: position().right === undefined ? undefined : `${position().right}px`,
            }}
          >
            {controls}
          </div>
        </Show>
      </div>
      <div class={[css.caption, folded() ? css.captionFolded : ''].join(' ').trim()} aria-hidden="true">
        {props.label}
      </div>
    </div>
  );
}

export interface RibbonItemProps {
  /** `small` takes one of the three rows, `large` all three. */
  size?: 'small' | 'large';
  /** An explicit inline size for a wide `small` control (a font-family select). */
  inlineSize?: string;
  class?: string;
  children?: JSX.Element;
}

export function RibbonItem(props: RibbonItemProps): JSX.Element {
  return (
    <div
      class={[props.size === 'large' ? css.itemLarge : css.itemSmall, props.class ?? ''].join(' ').trim()}
      data-ribbon-item={props.size ?? 'small'}
      style={props.inlineSize ? { 'inline-size': props.inlineSize } : undefined}
    >
      {props.children}
    </div>
  );
}
