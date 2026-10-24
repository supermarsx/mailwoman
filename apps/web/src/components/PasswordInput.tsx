// A password input with a show/hide button (t32-h17), used by both sign-in screens
// and the other password and secret fields.
//
// The field is a native <input>; the button only switches its type between
// `password` and `text`, so the caller's `autocomplete`, `name`, `id` and <label>
// stay as they were and password managers still find the field.
//
// - The button is a toggle: `aria-pressed`, `aria-controls` = the input id, and the
//   fixed name "Show password" (a toggle's name does not change with its state).
// - The input is named by `label` through `aria-label`: the <label> around it also
//   holds the button, and without it the input's name would end in "Show password".
// - A pointer press on the button does not take focus from the input, and the
//   caret and selection stay where they were.
// - The password is hidden again when its form is submitted, when `hideWhen`
//   changes (a sign-in error, say) and, since the state lives here only, when the
//   field unmounts. Nothing is stored.

import { createEffect, createSignal, createUniqueId, on, onCleanup, onMount, splitProps, type JSX } from 'solid-js';
import { t } from '../i18n';
import { Icon } from '../screens/Admin/shell/icons.tsx';
import * as css from './passwordInput.css.ts';

export interface PasswordInputProps extends Omit<JSX.InputHTMLAttributes<HTMLInputElement>, 'type' | 'aria-label'> {
  /** The field's name for assistive technology; the same text as its visible label. */
  label: string;
  /** Hide the password again whenever this value changes. */
  hideWhen?: () => unknown;
}

export function PasswordInput(props: PasswordInputProps): JSX.Element {
  const [own, rest] = splitProps(props, ['label', 'hideWhen', 'class', 'id', 'ref']);
  const fallbackId = createUniqueId();
  const id = (): string => own.id ?? `pw-${fallbackId}`;
  const [shown, setShown] = createSignal(false);
  let input: HTMLInputElement | undefined;

  const hide = (): void => {
    setShown(false);
  };

  onMount(() => {
    const form = input?.form;
    if (!form) return;
    form.addEventListener('submit', hide);
    onCleanup(() => form.removeEventListener('submit', hide));
  });

  if (own.hideWhen) createEffect(on(own.hideWhen, hide, { defer: true }));

  function toggle(): void {
    const el = input;
    const focused = el !== undefined && el.ownerDocument.activeElement === el;
    const start = el?.selectionStart ?? null;
    const end = el?.selectionEnd ?? null;
    setShown((v) => !v);
    if (!focused || !el || start === null || end === null) return;
    // Put the caret back where it was. Chromium moves it to the start once the frame
    // after a pointer click that switched the type is laid out, so again then, unless
    // the field changed or lost focus in between.
    const value = el.value;
    const restore = (): void => {
      if (el.ownerDocument.activeElement === el && el.value === value) el.setSelectionRange(start, end);
    };
    restore();
    requestAnimationFrame(restore);
  }

  return (
    <span class={css.wrap}>
      <input
        {...rest}
        ref={(el) => {
          input = el;
          const ref = own.ref as ((el: HTMLInputElement) => void) | undefined;
          if (typeof ref === 'function') ref(el);
        }}
        id={id()}
        class={[own.class, css.input].filter(Boolean).join(' ')}
        type={shown() ? 'text' : 'password'}
        aria-label={own.label}
        autocapitalize="off"
        autocorrect="off"
        spellcheck={false}
      />
      <button
        type="button"
        class={css.toggle}
        aria-pressed={shown()}
        aria-controls={id()}
        aria-label={t('common-password-show')}
        title={shown() ? t('common-password-hide') : t('common-password-show')}
        disabled={rest.disabled}
        onMouseDown={(e) => e.preventDefault()}
        onClick={toggle}
      >
        <Icon name={shown() ? 'eyeOff' : 'eye'} />
      </button>
    </span>
  );
}
