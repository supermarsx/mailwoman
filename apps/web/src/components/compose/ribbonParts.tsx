// Composer ribbon helpers (t38-e5).
//
// `RibbonSlot` marks where, in one ribbon tab's `RibbonPanel`, the rich-text
// editor's groups go. The editor lives in the body, not in the ribbon, so it
// renders its groups through `InRibbonSlot`: a portal into the slot that
// re-provides the panel and the bar, so the groups fold with the rest of the
// tab and their dropdowns hold the bar open. Both the slot and the portal's
// container are `display: contents`, so the groups lay out as the track's own.
//
// `RibbonPopover` is a one-row button that opens a fixed popover under itself,
// for anything that cannot fit three rows (a URL field, find and replace,
// paragraph options, the security details). While open it holds the bar open;
// Esc closes it and returns focus to the button, and so does a press outside.

import {
  createSignal,
  createUniqueId,
  onCleanup,
  onMount,
  Show,
  type JSX,
} from 'solid-js';
import { Portal } from 'solid-js/web';
import {
  CollapsibleBarContext,
  RibbonPanelContext,
  useCollapsibleBar,
  useRibbonPanel,
  type CollapsibleBarApi,
  type RibbonPanelApi,
} from '../bars/index.ts';
import { Icon, type IconName } from '../Icon.tsx';
import * as css from './composerRibbon.css.ts';

export interface RibbonSlotTarget {
  element: HTMLElement;
  panel: RibbonPanelApi | undefined;
  bar: CollapsibleBarApi | undefined;
}

/** A place in a `RibbonPanel` for groups rendered elsewhere (the editor's). */
export function RibbonSlot(props: { name: string; onTarget: (target: RibbonSlotTarget | undefined) => void }): JSX.Element {
  const panel = useRibbonPanel();
  const bar = useCollapsibleBar();
  onCleanup(() => props.onTarget(undefined));
  return <div class={css.slot} data-ribbon-slot={props.name} ref={(element) => props.onTarget({ element, panel, bar })} />;
}

function Provide(props: { target: RibbonSlotTarget; children: JSX.Element }): JSX.Element {
  const inPanel = (): JSX.Element =>
    props.target.panel ? (
      <RibbonPanelContext.Provider value={props.target.panel}>{props.children}</RibbonPanelContext.Provider>
    ) : (
      props.children
    );
  return props.target.bar ? (
    <CollapsibleBarContext.Provider value={props.target.bar}>{inPanel()}</CollapsibleBarContext.Provider>
  ) : (
    inPanel()
  );
}

/** Render `children` (ribbon groups) into a `RibbonSlot`, inside its panel and bar. */
export function InRibbonSlot(props: { target: RibbonSlotTarget; children: JSX.Element }): JSX.Element {
  return (
    <Portal mount={props.target.element} ref={(el: HTMLDivElement) => {
        // The editor's formatting controls act on the editor, so a detached window keeps them (DetachedWorkspacePane).
        el.className = css.slot;
        el.setAttribute('data-detached-local', '');
      }}>
      <Provide target={props.target}>{props.children}</Provide>
    </Portal>
  );
}

export interface RibbonPopoverProps {
  /** The button's text (also its name unless `ariaLabel` is given). */
  label: string;
  icon?: IconName;
  /** Show only the icon; the label becomes the name and the tooltip. */
  iconOnly?: boolean;
  ariaLabel?: string;
  /** Accessible name of the popover; defaults to `label`. */
  popoverLabel?: string;
  /** Id of a heading inside the popover that names it (replaces `popoverLabel`). */
  labelledBy?: string;
  /** Move focus into the popover itself when it holds no field (a read-only detail). */
  focusDialog?: boolean;
  /** Replaces the button's icon and text (the security status chip). */
  content?: JSX.Element;
  buttonClass?: string;
  testId?: string;
  buttonTestId?: string;
  disabled?: boolean;
  /** Keep the editor's selection when the button is pressed. */
  keepSelection?: boolean;
  /** Controlled open state (optional). */
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  children?: JSX.Element;
}

export function RibbonPopover(props: RibbonPopoverProps): JSX.Element {
  const bar = useCollapsibleBar();
  const id = `mw-ribbon-pop-${createUniqueId()}`;
  const [localOpen, setLocalOpen] = createSignal(false);
  const open = (): boolean => props.open ?? localOpen();
  const [position, setPosition] = createSignal<{ top: number; left?: number; right?: number }>({ top: 0 });
  let rootEl: HTMLDivElement | undefined;
  let buttonEl: HTMLButtonElement | undefined;
  let popEl: HTMLDivElement | undefined;
  let release: (() => void) | null = null;

  function place(): void {
    const rect = buttonEl?.getBoundingClientRect();
    if (!rect || !buttonEl) return;
    const view = buttonEl.ownerDocument.defaultView ?? window;
    const width = popEl?.offsetWidth ?? 0;
    const height = popEl?.offsetHeight ?? 0;
    // Under the button; above it when it would run off the bottom and there is more room there.
    const below = view.innerHeight - rect.bottom;
    const top = height > 0 && rect.bottom + 2 + height > view.innerHeight - 8 && rect.top > below
      ? Math.max(8, rect.top - 2 - height)
      : rect.bottom + 2;
    if (getComputedStyle(buttonEl).direction === 'rtl') {
      setPosition({ top, right: Math.max(8, Math.min(view.innerWidth - rect.right, view.innerWidth - 8 - width)) });
    } else {
      setPosition({ top, left: Math.max(8, Math.min(rect.left, view.innerWidth - 8 - width)) });
    }
  }

  function setOpen(next: boolean, opts: { restoreFocus?: boolean } = {}): void {
    if (next === open()) return;
    if (props.open === undefined) setLocalOpen(next);
    props.onOpenChange?.(next);
    if (next) {
      release = bar?.hold() ?? null;
      place();
      queueMicrotask(() => {
        place();
        const field = popEl?.querySelector<HTMLElement>('input:not([type=hidden]):not([disabled]), select:not([disabled]), textarea');
        if (field) field.focus();
        else if (props.focusDialog) popEl?.focus();
      });
    } else {
      release?.();
      release = null;
      if (opts.restoreFocus) buttonEl?.focus();
    }
  }

  const onDocumentPointerDown = (event: Event): void => {
    if (!open() || !(event.target instanceof Node) || rootEl?.contains(event.target)) return;
    setOpen(false);
  };
  // A fixed popover follows its button when the page around it scrolls or resizes.
  const onViewportChange = (): void => {
    if (open()) place();
  };
  onMount(() => {
    const doc = rootEl?.ownerDocument ?? document;
    const view = doc.defaultView ?? window;
    doc.addEventListener('pointerdown', onDocumentPointerDown, true);
    doc.addEventListener('scroll', onViewportChange, true);
    view.addEventListener('resize', onViewportChange);
    onCleanup(() => {
      doc.removeEventListener('pointerdown', onDocumentPointerDown, true);
      doc.removeEventListener('scroll', onViewportChange, true);
      view.removeEventListener('resize', onViewportChange);
    });
  });
  onCleanup(() => release?.());

  return (
    <div
      ref={rootEl}
      class={css.popoverRoot}
      data-testid={props.testId}
      onKeyDown={(event) => {
        if (event.key !== 'Escape' || !open()) return;
        event.preventDefault();
        event.stopPropagation();
        setOpen(false, { restoreFocus: true });
      }}
      onFocusOut={(event) => {
        const next = event.relatedTarget;
        if (open() && next instanceof Node && !rootEl?.contains(next)) setOpen(false);
      }}
    >
      <button
        ref={buttonEl}
        type="button"
        class={props.buttonClass ?? 'compose-rt__btn'}
        aria-haspopup="dialog"
        aria-expanded={open() ? 'true' : 'false'}
        aria-controls={id}
        aria-label={props.ariaLabel ?? (props.iconOnly ? props.label : undefined)}
        data-tooltip={props.iconOnly ? props.label : undefined}
        data-testid={props.buttonTestId}
        data-detached-local=""
        disabled={props.disabled}
        onMouseDown={(event) => {
          if (props.keepSelection) event.preventDefault();
        }}
        onClick={() => setOpen(!open())}
      >
        <Show
          when={props.content}
          fallback={
            <>
              <Show when={props.icon}>{(name) => <Icon name={name()} />}</Show>
              <Show when={!props.iconOnly}>
                <span class="compose-rib__text">{props.label}</span>
              </Show>
            </>
          }
        >
          {props.content}
        </Show>
      </button>
      <div
        ref={popEl}
        id={id}
        class={css.popover}
        role="dialog"
        aria-label={props.labelledBy ? undefined : (props.popoverLabel ?? props.label)}
        aria-labelledby={props.labelledBy}
        tabIndex={props.focusDialog ? -1 : undefined}
        hidden={!open()}
        data-ribbon-popover=""
        style={{
          top: `${position().top}px`,
          left: position().left === undefined ? undefined : `${position().left}px`,
          right: position().right === undefined ? undefined : `${position().right}px`,
        }}
      >
        {props.children}
      </div>
    </div>
  );
}
