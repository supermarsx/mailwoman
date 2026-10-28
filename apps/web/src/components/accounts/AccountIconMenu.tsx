// The account indicator in its icon-only form (t38 §2.3): compact folder mode
// hides the names, and a hidden sidebar moves the indicator into the mail top
// bar. A one-letter button names the active address and opens the same choices
// (the address, the switcher and "Manage") in a small popover.
import { createSignal, onCleanup, onMount, Show, type JSX } from 'solid-js';
import { Portal } from 'solid-js/web';
import { isolate, loadCatalog, t } from '../../i18n/index.ts';
import { useCollapsibleBar } from '../bars/index.ts';
import * as css from './accountIndicator.css.ts';

let sequence = 0;

export function accountIndicatorLabel(address: string, linked: boolean): string {
  return linked
    ? t('mail-account-linked-active', { address: isolate(address) })
    : t('mail-account-active', { address: isolate(address) });
}

export function AccountIconMenu(props: {
  address: string;
  linked: boolean;
  /** The popover body; `close` dismisses it (call it before navigating away). */
  children: (close: () => void) => JSX.Element;
}): JSX.Element {
  onMount(() => void loadCatalog('bars'));
  const bar = useCollapsibleBar();
  const popoverId = `mw-account-menu-${++sequence}`;
  const [open, setOpen] = createSignal(false);
  const [position, setPosition] = createSignal<{ top: number; left: number }>({ top: 0, left: 0 });
  let button: HTMLButtonElement | undefined;
  let popover: HTMLDivElement | undefined;
  let release: (() => void) | undefined;

  const label = (): string => accountIndicatorLabel(props.address, props.linked);
  const initial = (): string => Array.from(props.address.trim())[0]?.toLocaleUpperCase() ?? '?';

  function place(): void {
    if (!button) return;
    const rect = button.getBoundingClientRect();
    const view = button.ownerDocument.defaultView;
    const width = popover?.offsetWidth ?? 0;
    const maxLeft = Math.max(8, (view?.innerWidth ?? rect.right) - width - 8);
    setPosition({ top: rect.bottom + 4, left: Math.min(Math.max(8, rect.left), maxLeft) });
  }

  function show(): void {
    if (open()) return;
    release = bar?.hold();
    place();
    setOpen(true);
    queueMicrotask(() => {
      place();
      popover?.querySelector<HTMLElement>('select,button,[tabindex="0"]')?.focus();
    });
  }

  function close(restoreFocus = false): void {
    if (!open()) return;
    setOpen(false);
    release?.();
    release = undefined;
    if (restoreFocus) button?.focus();
  }

  const onDocumentPointerDown = (event: Event): void => {
    const target = event.target;
    if (!open() || !(target instanceof Node) || button?.contains(target) || popover?.contains(target)) return;
    close();
  };
  onMount(() => {
    const doc = button?.ownerDocument ?? document;
    doc.addEventListener('pointerdown', onDocumentPointerDown, true);
    onCleanup(() => doc.removeEventListener('pointerdown', onDocumentPointerDown, true));
  });
  onCleanup(() => {
    release?.();
    release = undefined;
  });

  return (
    <>
      <button
        ref={button}
        type="button"
        class={`btn btn--ghost ${css.iconButton}`}
        data-account-indicator="icon"
        data-linked={props.linked ? 'true' : 'false'}
        aria-label={label()}
        data-tooltip={label()}
        aria-haspopup="dialog"
        aria-expanded={open() ? 'true' : 'false'}
        aria-controls={open() ? popoverId : undefined}
        onClick={() => (open() ? close() : show())}
      >
        <span aria-hidden="true">{initial()}</span>
      </button>
      <Show when={open()}>
        <Portal mount={button?.ownerDocument.body ?? document.body}>
          <div
            ref={popover}
            id={popoverId}
            role="dialog"
            aria-label={label()}
            class={css.popover}
            style={{ top: `${position().top}px`, left: `${position().left}px` }}
            onKeyDown={(event) => {
              if (event.key !== 'Escape') return;
              event.preventDefault();
              event.stopPropagation();
              close(true);
            }}
            onFocusOut={(event) => {
              const next = event.relatedTarget;
              if (next instanceof Node && (popover?.contains(next) || button?.contains(next))) return;
              if (next instanceof Node) close();
            }}
          >
            <p class={css.popoverTitle}>{label()}</p>
            {props.children(() => close())}
          </div>
        </Portal>
      </Show>
    </>
  );
}
