// Renders the shared toast store (./toast.ts). Mount it once per app shell:
// App.tsx for the webmail, screens/Admin/index.tsx for the admin console.
//
// Live regions: an error toast is `role="alert"`, which screen readers announce
// when it is inserted. A success, info or warning toast is `role="status"`, but
// a status inserted together with its text is not announced reliably (NVDA with
// Chrome often stays silent), so these go into a polite `aria-live` container
// that stays in the page from the start, and the toast's own live behaviour is
// turned off (`aria-live="off"`) so the text is not read twice. The container
// has no role: tests and pages that look up `status` or `alert` find toasts only
// while one is shown. `aria-atomic="false"` keeps a new toast from re-reading
// the ones already shown.
//
// Keyboard: every toast has a close button; Escape closes the newest toast
// when focus is in the region, or anywhere else when no dialog, menu or text
// field would take that Escape first. Closing a toast that holds focus moves
// focus to the next toast, or back to where it was before it entered the region.
//
// The clocks stop while the pointer rests on a toast, while focus is inside the
// region and while the window is hidden (setToastPause).

import { For, Show, createEffect, createMemo, createSignal, onCleanup, onMount, type JSX } from 'solid-js';
import { t } from '../../i18n/index.ts';
import {
  registerToastRegion,
  setToastPause,
  toast,
  toastTimeLeft,
  toasts,
  toastsPaused,
  type ToastItem,
  type ToastTone,
} from './toast.ts';
import * as css from './toast.css.ts';

const ICON_PATHS: Record<ToastTone, string> = {
  success: 'M3 8.5l3.2 3.2L13 4.8',
  info: 'M8 7.2v4.6M8 4.4v.2M8 15A7 7 0 1 0 8 1a7 7 0 0 0 0 14z',
  warning: 'M8 1.8L15 14H1L8 1.8zM8 6.2v3.8M8 11.8v.2',
  error: 'M5.2 5.2l5.6 5.6M10.8 5.2l-5.6 5.6M8 15A7 7 0 1 0 8 1a7 7 0 0 0 0 14z',
};

function ToneIcon(props: { tone: ToastTone }): JSX.Element {
  return (
    <svg
      class={`${css.icon} ${css.iconTone[props.tone]}`}
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      stroke-width="1.6"
      stroke-linecap="round"
      stroke-linejoin="round"
      aria-hidden="true"
    >
      <path d={ICON_PATHS[props.tone]} />
    </svg>
  );
}

function isTextEntry(el: Element | null): boolean {
  if (el === null) return false;
  if (el instanceof HTMLElement && el.isContentEditable) return true;
  const tag = el.tagName;
  return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT';
}

/** Something on the page that answers Escape before the toasts do. */
function escapeTakenElsewhere(target: Element | null): boolean {
  if (isTextEntry(target)) return true;
  if (target?.closest('[aria-expanded="true"]')) return true;
  return document.querySelector('[aria-modal="true"], dialog[open]') !== null;
}

export function ToastRegion(): JSX.Element {
  let regionEl: HTMLDivElement | undefined;
  let returnFocus: Element | null = null;

  const inRegion = (n: EventTarget | null): boolean => regionEl !== undefined && n instanceof Node && regionEl.contains(n);
  const byId = createMemo(() => new Map(toasts().map((i) => [i.id, i])));
  const alertIds = createMemo(() => toasts().filter((i) => i.tone === 'error').map((i) => i.id));
  const statusIds = createMemo(() => toasts().filter((i) => i.tone !== 'error').map((i) => i.id));

  // The countdown text ticks only while a countdown toast is shown.
  const [now, setNow] = createSignal(0);
  createEffect(() => {
    if (!toasts().some((i) => i.countdown)) return;
    const h = setInterval(() => setNow(Date.now()), 250);
    onCleanup(() => clearInterval(h));
  });

  // A toast that held focus or the pointer can leave without a focusout or a
  // pointerout; recheck after every change.
  createEffect(() => {
    toasts();
    queueMicrotask(() => {
      if (!inRegion(document.activeElement)) setToastPause('focus', false);
      if (toasts().length === 0) setToastPause('hover', false);
    });
  });

  /** Close `id`; if focus was inside it, hand focus on rather than drop it on <body>. */
  function closeToast(id: string, reason: 'dismiss' | 'action' = 'dismiss'): void {
    const el = regionEl?.querySelector(`[data-toast-id="${CSS.escape(id)}"]`);
    const hadFocus = el?.contains(document.activeElement) === true;
    toast.dismiss(id, reason);
    if (!hadFocus) return;
    const next = regionEl?.querySelector<HTMLElement>('[data-toast-close]');
    if (next) next.focus();
    else if (returnFocus instanceof HTMLElement && returnFocus.isConnected) returnFocus.focus();
  }

  function onKeyDown(e: KeyboardEvent): void {
    if (e.key !== 'Escape' || e.defaultPrevented || toasts().length === 0) return;
    const target = e.target instanceof Element ? e.target : null;
    if (!inRegion(target) && escapeTakenElsewhere(target)) return;
    const newest = toasts().at(-1);
    if (newest === undefined) return;
    e.preventDefault();
    closeToast(newest.id);
  }
  function onFocusIn(e: FocusEvent): void {
    const inside = inRegion(e.target);
    if (inside && !inRegion(e.relatedTarget)) returnFocus = e.relatedTarget instanceof Element ? e.relatedTarget : null;
    setToastPause('focus', inside);
  }
  function onPointer(e: PointerEvent): void {
    if (e.pointerType === 'touch' || toasts().length === 0) return;
    setToastPause('hover', inRegion(e.target));
  }
  function onPointerOut(e: PointerEvent): void {
    // Leaving the window: no element takes the pointer.
    if (e.relatedTarget === null && e.pointerType !== 'touch') setToastPause('hover', false);
  }
  function onVisibility(): void {
    setToastPause('hidden', document.visibilityState === 'hidden');
  }

  onCleanup(registerToastRegion());
  onMount(() => {
    document.addEventListener('keydown', onKeyDown);
    document.addEventListener('focusin', onFocusIn);
    document.addEventListener('pointerover', onPointer);
    document.addEventListener('pointermove', onPointer);
    document.addEventListener('pointerout', onPointerOut);
    document.addEventListener('visibilitychange', onVisibility);
    onVisibility();
  });
  onCleanup(() => {
    document.removeEventListener('keydown', onKeyDown);
    document.removeEventListener('focusin', onFocusIn);
    document.removeEventListener('pointerover', onPointer);
    document.removeEventListener('pointermove', onPointer);
    document.removeEventListener('pointerout', onPointerOut);
    document.removeEventListener('visibilitychange', onVisibility);
    setToastPause('hover', false);
    setToastPause('focus', false);
    setToastPause('hidden', false);
  });

  const renderToast = (id: string): JSX.Element => (
    <Show when={byId().get(id)}>
      {(item) => <Toast item={item()} now={now} onClose={(reason) => closeToast(id, reason)} />}
    </Show>
  );

  return (
    <div ref={regionEl} class={css.region} data-testid="toast-region">
      <div class={css.stack}>
        <For each={alertIds()}>{renderToast}</For>
      </div>
      <div class={css.stack} aria-live="polite" aria-atomic="false" aria-relevant="additions text">
        <For each={statusIds()}>{renderToast}</For>
      </div>
    </div>
  );
}

function Toast(props: {
  item: ToastItem;
  now: () => number;
  onClose: (reason: 'dismiss' | 'action') => void;
}): JSX.Element {
  const seconds = (): number => {
    props.now();
    return Math.ceil(toastTimeLeft(props.item.id) / 1000);
  };
  return (
    <div
      class={`toast ${css.item} ${css.tone[props.item.tone]}`}
      role={props.item.tone === 'error' ? 'alert' : 'status'}
      aria-live={props.item.tone === 'error' ? undefined : 'off'}
      data-testid={props.item.testId ?? 'toast'}
      data-tone={props.item.tone}
      data-toast-id={props.item.id}
      data-paused={toastsPaused() && props.item.pausable ? 'true' : undefined}
    >
      <ToneIcon tone={props.item.tone} />
      <span class={css.message}>{props.item.message}</span>
      <Show when={props.item.countdown && Number.isFinite(props.item.duration)}>
        <span class={css.countdown} aria-hidden="true">
          {t('common-toast-seconds', { seconds: seconds() })}
        </span>
      </Show>
      <Show when={props.item.action}>
        {(action) => (
          <button
            type="button"
            class={css.action}
            onClick={() => {
              const run = action().run;
              props.onClose('action');
              void Promise.resolve()
                .then(run)
                .catch(() => undefined);
            }}
          >
            {action().label}
          </button>
        )}
      </Show>
      <button
        type="button"
        class={css.close}
        data-toast-close=""
        aria-label={t('common-toast-close')}
        onClick={() => props.onClose('dismiss')}
      >
        <svg
          class={css.closeIcon}
          viewBox="0 0 12 12"
          fill="none"
          stroke="currentColor"
          stroke-width="1.6"
          stroke-linecap="round"
          aria-hidden="true"
        >
          <path d="M2 2l8 8M10 2l-8 8" />
        </svg>
      </button>
    </div>
  );
}
