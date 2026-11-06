// The modal shell the Insert tab's dialogs share: portal into the trigger's
// document (detached composers), focus trap, Escape closes, focus returns.

import { onCleanup, onMount, Show, type JSX } from 'solid-js';
import { Portal } from 'solid-js/web';
import { t } from '../../../i18n/index.ts';
import * as css from '../insertAccountItem.css.ts';
import { dialogForm } from './insertDialog.css.ts';

export interface InsertDialogProps {
  title: string;
  /** The element to return focus to. */
  returnTo?: HTMLElement | null;
  busy?: boolean;
  onClose: () => void;
  /** Enter in a field submits. */
  onSubmit?: () => void;
  submitLabel?: string;
  submitDisabled?: boolean;
  testId: string;
  children: JSX.Element;
}

export function InsertDialog(props: InsertDialogProps): JSX.Element {
  let panel!: HTMLElement;
  const opener = (typeof document !== 'undefined' ? document.activeElement : null) as HTMLElement | null;
  onMount(() => {
    const doc = panel.ownerDocument;
    const controls = (): HTMLElement[] => Array.from(panel.querySelectorAll<HTMLElement>('button:not([disabled]),input:not([disabled]),select:not([disabled]),textarea:not([disabled]),[tabindex="0"]'));
    const keydown = (event: KeyboardEvent): void => {
      if (event.key === 'Escape') { event.preventDefault(); event.stopPropagation(); props.onClose(); return; }
      if (event.key === 'Enter' && props.onSubmit && (event.target as HTMLElement).tagName === 'INPUT' && !props.submitDisabled) { event.preventDefault(); props.onSubmit(); return; }
      if (event.key !== 'Tab') return;
      const nodes = controls(), first = nodes[0], last = nodes[nodes.length - 1];
      if (!first || !last) { event.preventDefault(); return; }
      if (event.shiftKey && doc.activeElement === first) { event.preventDefault(); last.focus(); }
      else if (!event.shiftKey && doc.activeElement === last) { event.preventDefault(); first.focus(); }
    };
    panel.addEventListener('keydown', keydown);
    (panel.querySelector<HTMLElement>('input:not([type=hidden]),select,textarea') ?? controls()[0] ?? panel).focus();
    onCleanup(() => {
      panel.removeEventListener('keydown', keydown);
      const back = props.returnTo ?? opener;
      if (back?.isConnected) back.focus();
    });
  });
  return (
    <Portal mount={(props.returnTo?.ownerDocument ?? document).body}>
      <div class={css.backdrop} onClick={(event) => { if (event.target === event.currentTarget) props.onClose(); }}>
        <section ref={panel} class={`${css.panel} ${dialogForm}`} role="dialog" aria-modal="true" aria-label={props.title} aria-busy={props.busy === true} tabIndex={-1} data-testid={props.testId}>
          <div class={css.header}><h2 style={{ margin: '0', 'font-size': '1rem' }}>{props.title}</h2></div>
          {props.children}
          <div class="mw-insert-actions">
            <button type="button" class="btn btn--ghost" data-testid={`${props.testId}-cancel`} onClick={props.onClose}>{t('ins-cancel')}</button>
            <Show when={props.onSubmit}>
              <button type="button" class="btn" data-testid={`${props.testId}-apply`} disabled={props.submitDisabled === true || props.busy === true} onClick={() => props.onSubmit?.()}>{props.submitLabel ?? t('ins-insert')}</button>
            </Show>
          </div>
        </section>
      </div>
    </Portal>
  );
}
