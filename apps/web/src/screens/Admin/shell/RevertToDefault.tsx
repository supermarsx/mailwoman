// "Revert to default" for one field (t32-cfg UX7, DC13).
//
// For a registry setting the action deletes the stored value, so the built-in
// default (or a narrower scope's value) applies again; for a field of an object
// (a domain, an integration) it resets the form field. The caller's `onRevert`
// does either. The button is not shown while the default is in use, and it is
// disabled with the reason when the value cannot be changed here (env pin, lock).
// The result is announced in a live region next to the button.

import { Show, createSignal, createUniqueId, type JSX } from 'solid-js';
import { isolate, t } from '../../../i18n';
import { Feedback, describeAdminError } from './Feedback.tsx';
import * as css from './shell.css.ts';
import * as ux from './ux.css.ts';

export interface RevertToDefaultProps {
  /** The field label, added to the button's accessible name ("Revert to default (Port)"). */
  label: string;
  /** The default as shown to the admin ("587", "Off"). */
  defaultDisplay: string;
  /** True while the default is in use: nothing to revert. */
  isDefault: boolean;
  /** Deletes the stored value, or resets the form field. */
  onRevert: () => void | Promise<void>;
  /** Why the value cannot be changed here, or null. */
  disabledReason?: string | null;
}

export function RevertToDefault(props: RevertToDefaultProps): JSX.Element {
  const id = createUniqueId();
  const [busy, setBusy] = createSignal(false);
  const [done, setDone] = createSignal<string | null>(null);
  const [err, setErr] = createSignal<string | null>(null);

  async function revert(): Promise<void> {
    setBusy(true);
    setDone(null);
    setErr(null);
    try {
      await props.onRevert();
      setDone(t('admin-ux-revert-done', { value: isolate(props.defaultDisplay) }));
    } catch (e) {
      setErr(describeAdminError(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div class={ux.stateRow} data-testid="revert-to-default">
      <span id={`${id}-default`}>{t('admin-ux-revert-default-is', { value: isolate(props.defaultDisplay) })}</span>
      <Show when={!props.isDefault}>
        <button
          type="button"
          class={ux.linkButton}
          disabled={busy() || Boolean(props.disabledReason)}
          aria-describedby={`${id}-default`}
          title={props.disabledReason ?? undefined}
          onClick={() => void revert()}
        >
          {t('admin-ux-revert')}
          <span class={css.srOnly}> {t('admin-ux-revert-field', { label: props.label })}</span>
        </button>
      </Show>
      <Feedback kind="success" message={done()} />
      <Feedback kind="error" as="toast" message={err()} />
    </div>
  );
}
