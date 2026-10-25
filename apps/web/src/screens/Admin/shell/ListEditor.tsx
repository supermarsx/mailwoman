// An editor for a list of short entries (accounts, folders, host names, scopes),
// in place of a textarea with one entry per line (t32-cfg §4.12).
//
// The entries are a list, each with its own Remove button that names it. Below
// is one input and an Add button; Enter in the input adds too (it never submits
// the surrounding form). An entry is trimmed, refused when empty, already listed
// or rejected by `validate`, and the reason is shown under the input and tied to
// it with `aria-describedby`. Removing an entry moves focus to the next Remove
// button, else to the input, so keyboard users are not dropped to the page.
//
// `id` is given to the input, so a `<label for>` (SettingsFormPage, FormField)
// names it; `describedBy` is added to the input's description.

import { For, Show, createSignal, createUniqueId, type JSX } from 'solid-js';
import { isolate, t } from '../../../i18n';
import { Icon } from './icons.tsx';
import * as shell from './shell.css.ts';
import * as ux from './ux.css.ts';
import * as css from './extras.css.ts';

export interface ListEditorProps {
  /** The input's id (for an outside `<label for>`). */
  id?: string | undefined;
  /** What the list holds, for the Add button and the list's name ("Allowed hosts"). */
  label: string;
  items: readonly string[];
  onChange: (items: string[]) => void;
  /** Add button text. Default "Add". */
  addLabel?: string | undefined;
  placeholder?: string | undefined;
  /** The sentence to show for a refused entry, or null when it is fine. */
  validate?: ((entry: string) => string | null) | undefined;
  /** Extra ids for the input's `aria-describedby` (field help, field error). */
  describedBy?: string | undefined;
  invalid?: boolean | undefined;
  disabled?: boolean | undefined;
  /** Shown when the list is empty ("Every account"). */
  emptyText?: string | undefined;
  /** Entries in a monospace face (host names, ids). */
  mono?: boolean | undefined;
  /** Lower-case entries before comparing and storing (host names). */
  lowerCase?: boolean | undefined;
  /** `words` (default): a pasted "a b, c" becomes three entries; `none`: the input is one
   *  entry as typed (program arguments, names with spaces or commas). */
  split?: 'words' | 'none' | undefined;
  testId?: string | undefined;
}

export function ListEditor(props: ListEditorProps): JSX.Element {
  const own = createUniqueId();
  const inputId = (): string => props.id ?? `${own}-input`;
  const [draft, setDraft] = createSignal('');
  const [error, setError] = createSignal<string | null>(null);
  let input: HTMLInputElement | undefined;
  let list: HTMLUListElement | undefined;

  const describedBy = (): string | undefined => {
    const ids = [props.describedBy ?? '', error() ? `${own}-error` : ''].filter((s) => s !== '');
    return ids.length > 0 ? ids.join(' ') : undefined;
  };

  function normalise(raw: string): string {
    const v = raw.trim();
    return props.lowerCase ? v.toLowerCase() : v;
  }

  function add(): void {
    const parts = (props.split === 'none' ? [draft()] : draft().split(/[\s,]+/)).map(normalise).filter((p) => p !== '');
    if (parts.length === 0) {
      setError(t('admin-ux-list-empty-entry'));
      input?.focus();
      return;
    }
    const next = [...props.items];
    for (const entry of parts) {
      if (next.includes(entry)) {
        setError(t('admin-ux-list-duplicate', { item: isolate(entry) }));
        input?.focus();
        return;
      }
      const refused = props.validate?.(entry) ?? null;
      if (refused !== null) {
        setError(refused);
        input?.focus();
        return;
      }
      next.push(entry);
    }
    setError(null);
    setDraft('');
    props.onChange(next);
    input?.focus();
  }

  function remove(index: number): void {
    const next = props.items.filter((_, i) => i !== index);
    props.onChange(next);
    queueMicrotask(() => {
      const buttons = list?.isConnected ? list.querySelectorAll<HTMLButtonElement>('button') : undefined;
      const target = buttons?.[Math.min(index, (buttons?.length ?? 1) - 1)];
      if (target) target.focus();
      else input?.focus();
    });
  }

  return (
    <div class={css.listEditor} data-testid={props.testId}>
      <Show when={props.items.length > 0} fallback={<p class={css.listEmpty}>{props.emptyText ?? t('admin-ux-list-none')}</p>}>
        <ul class={css.listItems} ref={list} aria-label={props.label}>
          <For each={props.items}>
            {(entry, i) => (
              <li class={css.listItem}>
                <span class={props.mono ? `${css.listText} ${css.listMono}` : css.listText} dir="auto">
                  {entry}
                </span>
                <button
                  type="button"
                  class="btn btn--ghost"
                  disabled={props.disabled}
                  aria-label={t('admin-ux-list-remove', { item: isolate(entry) })}
                  onClick={() => remove(i())}
                >
                  <Icon name="close" />
                </button>
              </li>
            )}
          </For>
        </ul>
      </Show>
      <div class={css.listAdd}>
        <input
          ref={input}
          id={inputId()}
          class={`${ux.control} ${css.listInput}`}
          type="text"
          autocomplete="off"
          spellcheck={false}
          placeholder={props.placeholder}
          disabled={props.disabled}
          value={draft()}
          aria-describedby={describedBy()}
          aria-invalid={error() || props.invalid ? true : undefined}
          onInput={(e) => {
            setDraft(e.currentTarget.value);
            if (error()) setError(null);
          }}
          onKeyDown={(e) => {
            if (e.key === 'Enter') {
              e.preventDefault();
              add();
            }
          }}
        />
        <button type="button" class="btn" disabled={props.disabled} onClick={add}>
          {props.addLabel ?? t('admin-ux-list-add')}
          <span class={shell.srOnly}> {props.label}</span>
        </button>
      </div>
      <Show when={error()}>
        {(msg) => (
          <p id={`${own}-error`} class={shell.fieldError} role="alert">
            <Icon name="alert" />
            <span>{msg()}</span>
          </p>
        )}
      </Show>
    </div>
  );
}
