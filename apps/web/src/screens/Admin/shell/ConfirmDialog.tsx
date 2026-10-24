// Confirmation for destructive or far-reaching actions (t32-admin.md §2.1,
// t32-ext.md §K): revoke sessions, revoke an API key, disable a user, delete a
// domain, remove a block or allowlist entry, the AI assistant kill switch, ...
//
// The title names the object ("Revoke all sessions of alice@example.org?").
// `typedConfirmation` makes the admin type the target's name before the confirm
// button works (delete domain, removing oneself from an allowlist).
//
// A modal `alertdialog`: focus moves in (to the typed field, else to Cancel, the
// safe choice), Tab stays inside, Escape cancels, and focus returns to the control
// that opened it. A failed action keeps the dialog open and says what failed.

import { Show, createSignal, createUniqueId, type JSX } from 'solid-js';
import { Portal } from 'solid-js/web';
import { isolate, t } from '../../../i18n';
import { createFocusTrap } from '../../../components/a11y';
import { Feedback, describeAdminError } from './Feedback.tsx';
import * as css from './shell.css.ts';

export interface ConfirmDialogProps {
  open: boolean;
  title: string;
  /** What will happen and what will not. */
  body?: string;
  confirmLabel: string;
  /** Red confirm button (default true: this dialog is for destructive actions). */
  danger?: boolean;
  /** When set, the confirm button works only after this exact text is typed. */
  typedConfirmation?: string;
  /** May return a promise; a rejection keeps the dialog open with the error. */
  onConfirm: () => void | Promise<void>;
  onCancel: () => void;
}

export function ConfirmDialog(props: ConfirmDialogProps): JSX.Element {
  return (
    <Show when={props.open}>
      <ConfirmDialogOpen {...props} />
    </Show>
  );
}

function ConfirmDialogOpen(props: ConfirmDialogProps): JSX.Element {
  const id = createUniqueId();
  const [typed, setTyped] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [el, setEl] = createSignal<HTMLElement>();
  let typedInput: HTMLInputElement | undefined;
  let cancelButton: HTMLButtonElement | undefined;

  const matches = (): boolean => props.typedConfirmation === undefined || typed() === props.typedConfirmation;

  function cancel(): void {
    if (!busy()) props.onCancel();
  }

  createFocusTrap(el, {
    onEscape: cancel,
    initialFocus: () => typedInput ?? cancelButton,
  });

  async function confirm(e: Event): Promise<void> {
    e.preventDefault();
    if (!matches() || busy()) return;
    setBusy(true);
    setError(null);
    try {
      await props.onConfirm();
    } catch (err) {
      setError(describeAdminError(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <Portal>
      <div class={css.dialogScrim} onClick={cancel} />
      <form
        ref={setEl}
        class={css.dialog}
        role="alertdialog"
        aria-modal="true"
        aria-labelledby={`${id}-title`}
        aria-describedby={props.body ? `${id}-body` : undefined}
        tabindex="-1"
        onSubmit={(e) => void confirm(e)}
      >
        <h2 id={`${id}-title`} class={css.dialogTitle}>
          {props.title}
        </h2>
        <Show when={props.body}>
          {(body) => (
            <p id={`${id}-body`} class={css.help}>
              {body()}
            </p>
          )}
        </Show>
        <Show when={props.typedConfirmation}>
          {(target) => (
            <label class="field">
              <span>{t('admin-shell-confirm-type', { name: isolate(target()) })}</span>
              <input
                ref={typedInput}
                type="text"
                autocomplete="off"
                spellcheck={false}
                value={typed()}
                onInput={(e) => setTyped(e.currentTarget.value)}
              />
            </label>
          )}
        </Show>
        <Feedback kind="error" message={error()} />
        <div class={css.dialogActions}>
          <button ref={cancelButton} type="button" class="btn btn--ghost" onClick={cancel} disabled={busy()}>
            {t('admin-shell-cancel')}
          </button>
          <button
            type="submit"
            class={props.danger === false ? 'btn btn--primary' : 'btn btn--danger'}
            disabled={!matches() || busy()}
          >
            {props.confirmLabel}
          </button>
        </div>
      </form>
    </Portal>
  );
}
