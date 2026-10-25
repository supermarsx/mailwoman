// A labelled admin form field (t32-admin.md §2.1, t32-ext.md §K).
//
// Plain-language label with the technical name secondary ("Connection security —
// STARTTLS"), an optional one-sentence help note, and an inline validation or
// server error. The control is rendered by the caller through `children`, which
// receives the ids to wire: `id` for the control, `describedBy` for
// `aria-describedby` (help + error), `invalid` for `aria-invalid`.

import { Show, createUniqueId, type JSX } from 'solid-js';
import { HelpNote } from './HelpNote.tsx';
import { Icon } from './icons.tsx';
import * as css from './shell.css.ts';

export interface FieldWiring {
  id: string;
  describedBy: string | undefined;
  invalid: boolean;
}

export interface FormFieldProps {
  label: string;
  /** The technical name shown after the label, e.g. "STARTTLS". */
  technical?: string;
  help?: string;
  /** Validation or mapped server error; null when the value is fine. */
  error?: string | null;
  children: (wiring: FieldWiring) => JSX.Element;
}

export function FormField(props: FormFieldProps): JSX.Element {
  const id = createUniqueId();
  const helpId = `${id}-help`;
  const errorId = `${id}-error`;
  const describedBy = (): string | undefined => {
    const ids = [props.help ? helpId : '', props.error ? errorId : ''].filter((s) => s !== '');
    return ids.length > 0 ? ids.join(' ') : undefined;
  };
  return (
    <div class={css.field} data-form-field>
      <label class={css.fieldLabel} for={id}>
        {props.label}
        <Show when={props.technical}>
          {(tech) => <span class={css.fieldTech}> — {tech()}</span>}
        </Show>
      </label>
      {props.children({
        id,
        get describedBy() {
          return describedBy();
        },
        get invalid() {
          return Boolean(props.error);
        },
      })}
      <Show when={props.help}>{(help) => <HelpNote id={helpId}>{help()}</HelpNote>}</Show>
      <Show when={props.error}>
        {(error) => (
          <p id={errorId} class={css.fieldError}>
            <Icon name="alert" />
            <span>{error()}</span>
          </p>
        )}
      </Show>
    </div>
  );
}
