// In-app confirmations instead of the browser's window.confirm / prompt (t67).
//
// The native dialogs block the page, cannot be styled or translated beyond their
// text, open in the opener when a pane is detached into its own window, and
// cannot take part in the workspace's bulk tab close: closing several tabs asked
// once in the workspace dialog and then again natively for every Settings
// editor with changes. These helpers show the shared ConfirmDialog (t42) instead.
//
// `createConfirm()` gives an `ask(request)` that resolves true/false, plus the
// `Dialog` to render once in the component. The dialog mounts in the document
// that holds `Dialog` (a detached workspace window included), via a hidden
// anchor. Asking again while a request is open answers the open one "no".
//
// `useLeaveConfirm(navigation, dirty, title)` registers a Settings leave guard
// that asks through that dialog, and reports `dirty` to the navigation, so a
// bulk close that already asked (and chose discard) does not ask again.

import { createSignal, createUniqueId, onCleanup, Show, type JSX } from 'solid-js';
import { ConfirmDialog } from '../screens/Admin/shell/ConfirmDialog.tsx';
import { t } from '../i18n/index.ts';
import type { SettingsNavigation } from '../screens/Settings/navigation.tsx';

export interface ConfirmRequest {
  title: string;
  body?: string;
  confirmLabel: string;
  cancelLabel?: string;
  /** Red confirm button (default true). */
  danger?: boolean;
  testId?: string;
}

interface Open { request: ConfirmRequest; answer: (yes: boolean) => void }

export function createConfirm(): { ask: (request: ConfirmRequest) => Promise<boolean>; Dialog: () => JSX.Element; open: () => boolean } {
  const [current, setCurrent] = createSignal<Open | null>(null);
  function ask(request: ConfirmRequest): Promise<boolean> {
    current()?.answer(false);
    return new Promise<boolean>((resolve) => {
      const entry: Open = { request, answer: (yes) => { if (current() === entry) setCurrent(null); resolve(yes); } };
      setCurrent(entry);
    });
  }
  onCleanup(() => current()?.answer(false));
  function Dialog(): JSX.Element {
    let anchor: HTMLSpanElement | undefined;
    return (
      <>
        <span ref={anchor} hidden />
        <Show when={current()}>
          {(open) => (
            <ConfirmDialog
              open
              title={open().request.title}
              {...(open().request.body !== undefined ? { body: open().request.body } : {})}
              confirmLabel={open().request.confirmLabel}
              cancelLabel={open().request.cancelLabel ?? t('common-cancel')}
              danger={open().request.danger !== false}
              {...(open().request.testId !== undefined ? { testId: open().request.testId } : {})}
              mount={anchor?.ownerDocument.body}
              onCancel={() => open().answer(false)}
              onConfirm={() => open().answer(true)}
            />
          )}
        </Show>
      </>
    );
  }
  return { ask, Dialog, open: () => current() !== null };
}

/** A Settings editor's leave guard through the in-app dialog. `title` is the
 *  question asked ("Discard unsaved time zone changes?"). Returns the dialog to
 *  render inside the editor. */
export function useLeaveConfirm(navigation: SettingsNavigation, dirty: () => boolean, title: () => string): () => JSX.Element {
  const confirm = createConfirm();
  onCleanup(navigation.beforeLeave(
    () => !dirty() || confirm.ask({
      title: title(),
      confirmLabel: t('settings-editor-discard'),
      cancelLabel: t('settings-editor-keep'),
      testId: 'settings-leave-confirm',
    }),
    dirty,
  ));
  return confirm.Dialog;
}

/** A one-field prompt in the shared dialog (in place of window.prompt). */
export function createPrompt(): { ask: (request: ConfirmRequest & { label: string; placeholder?: string; type?: string }) => Promise<string | null>; Dialog: () => JSX.Element } {
  const id = createUniqueId();
  const [current, setCurrent] = createSignal<{ request: ConfirmRequest & { label: string; placeholder?: string; type?: string }; answer: (value: string | null) => void } | null>(null);
  const [value, setValue] = createSignal('');
  function ask(request: ConfirmRequest & { label: string; placeholder?: string; type?: string }): Promise<string | null> {
    current()?.answer(null);
    setValue('');
    return new Promise((resolve) => {
      const entry = { request, answer: (v: string | null) => { if (current() === entry) setCurrent(null); resolve(v); } };
      setCurrent(entry);
    });
  }
  onCleanup(() => current()?.answer(null));
  function Dialog(): JSX.Element {
    let anchor: HTMLSpanElement | undefined;
    return (
      <>
        <span ref={anchor} hidden />
        <Show when={current()}>
          {(open) => (
            <ConfirmDialog
              open
              title={open().request.title}
              confirmLabel={open().request.confirmLabel}
              cancelLabel={open().request.cancelLabel ?? t('common-cancel')}
              danger={open().request.danger === true}
              {...(open().request.testId !== undefined ? { testId: open().request.testId } : {})}
              mount={anchor?.ownerDocument.body}
              onCancel={() => open().answer(null)}
              onConfirm={() => open().answer(value().trim() === '' ? null : value().trim())}
            >
              <label class="field" for={`${id}-input`}>
                <span>{open().request.label}</span>
                <input
                  id={`${id}-input`}
                  type={open().request.type ?? 'text'}
                  autocomplete="off"
                  {...(open().request.placeholder !== undefined ? { placeholder: open().request.placeholder } : {})}
                  value={value()}
                  onInput={(e) => setValue(e.currentTarget.value)}
                  ref={(el) => queueMicrotask(() => el.focus())}
                />
              </label>
            </ConfirmDialog>
          )}
        </Show>
      </>
    );
  }
  return { ask, Dialog };
}
