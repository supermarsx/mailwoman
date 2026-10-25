// Field-level errors for admin forms (t32-cfg UX5, UX13).
//
// One store per form holds an error per field id: the form's own validation
// writes it before any request, and a refused write lands here too. The server
// refuses a value with `422 {code, detail, field}` where `field` is a JSON pointer
// into the request body (`/smtp/port`); `fromError` turns the pointer into the
// form's field id (`smtp.port` by default, or the form's own mapping) and shows the
// message at that field. An error the form cannot place stays as the form error.
//
// The field ids are the same ids the field anchors use (`fieldAnchors.ts`), so
// `focusFirst()` moves focus to the first field in error.

import { createSignal, type Accessor } from 'solid-js';
import { t } from '../../../i18n';
import { AdminProblemError } from './adminApi.ts';
import { describeAdminError } from './Feedback.tsx';
import { focusField } from './fieldAnchors.ts';

/** `/smtp/port` → `smtp.port`; `~1` and `~0` unescaped (RFC 6901). Empty pointer → undefined. */
export function pointerToFieldId(pointer: string): string | undefined {
  if (pointer === '' || pointer === '/') return undefined;
  const parts = pointer
    .replace(/^#/, '')
    .split('/')
    .slice(1)
    .map((p) => p.replace(/~1/g, '/').replace(/~0/g, '~'));
  return parts.length > 0 ? parts.join('.') : undefined;
}

/** Codes the server sends for a refused value, with the sentence shown at the field. */
const FIELD_CODE_MESSAGES: Record<string, () => string> = {
  required: () => t('admin-ux-err-required'),
  out_of_range: () => t('admin-ux-err-out-of-range'),
  outOfRange: () => t('admin-ux-err-out-of-range'),
  too_long: () => t('admin-ux-err-too-long'),
  tooLong: () => t('admin-ux-err-too-long'),
  invalid: () => t('admin-ux-err-invalid'),
  pinnedByEnv: () => t('admin-ux-err-pinned'),
  pinned_by_env: () => t('admin-ux-err-pinned'),
  setting_locked: () => t('admin-ux-err-locked'),
};

/** The sentence for a refused value: by code, else the server's detail, else a plain default. */
export function fieldMessage(err: AdminProblemError): string {
  const byCode = err.code !== undefined ? FIELD_CODE_MESSAGES[err.code] : undefined;
  if (byCode) return byCode();
  if (err.detail) return err.detail;
  return t('admin-ux-err-invalid');
}

export interface FieldErrorsOptions {
  /** Map a JSON pointer to a field id when the default (`/a/b` → `a.b`) does not fit. */
  fieldFor?: (pointer: string) => string | undefined;
  /** Field ids the form shows; an error for another id becomes the form error. */
  known?: () => readonly string[];
}

export interface FieldErrors {
  /** The error shown at `id`, or null. */
  error(id: string): string | null;
  /** Every field error, by id. */
  all: Accessor<Readonly<Record<string, string>>>;
  /** The error that belongs to no field (network, 403, ...), or null. */
  formError: Accessor<string | null>;
  /** Set or clear one field's error. */
  set(id: string, message: string | null): void;
  /** Replace every field error (the form's validation pass). */
  replace(errors: Record<string, string>): void;
  /** Clear one field, or everything. */
  clear(id?: string): void;
  /** Place a failed request: at its field when it names one, else as the form error.
   *  Returns the field id it landed on, or null. */
  fromError(err: unknown): string | null;
  /** Focus the first field in error, in `order` (default: insertion order). */
  focusFirst(order?: readonly string[]): boolean;
  /** True while any field or the form has an error. */
  hasErrors: Accessor<boolean>;
}

export function useFieldErrors(opts: FieldErrorsOptions = {}): FieldErrors {
  const [errors, setErrors] = createSignal<Record<string, string>>({});
  const [formError, setFormError] = createSignal<string | null>(null);

  function set(id: string, message: string | null): void {
    setErrors((prev) => {
      const next = { ...prev };
      if (message === null) delete next[id];
      else next[id] = message;
      return next;
    });
  }

  return {
    error: (id) => errors()[id] ?? null,
    all: errors,
    formError,
    set,
    replace(next) {
      setErrors({ ...next });
      setFormError(null);
    },
    clear(id) {
      if (id === undefined) {
        setErrors({});
        setFormError(null);
      } else set(id, null);
    },
    fromError(err) {
      if (err instanceof AdminProblemError && err.field !== undefined && (err.status === 422 || err.status === 400 || err.status === 409)) {
        const id = opts.fieldFor?.(err.field) ?? pointerToFieldId(err.field);
        const known = opts.known?.();
        if (id !== undefined && (known === undefined || known.includes(id))) {
          set(id, fieldMessage(err));
          setFormError(null);
          return id;
        }
      }
      setFormError(describeAdminError(err));
      return null;
    },
    focusFirst(order) {
      const ids = order?.filter((id) => errors()[id] !== undefined) ?? Object.keys(errors());
      const first = ids[0];
      return first !== undefined ? focusField(first) : false;
    },
    hasErrors: () => Object.keys(errors()).length > 0 || formError() !== null,
  };
}
