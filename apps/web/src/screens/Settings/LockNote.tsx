// "Set by your administrator" next to a Settings control the settings model
// holds (t32-admin §2.8, worker f01). Renders nothing while the control is free.
// Give the control `aria-describedby={id}` so a screen reader reads the note
// with it; the lock glyph is decorative.

import { Show, type JSX } from 'solid-js';
import { t } from '../../i18n';
import * as css from './settingsPolicy.css.ts';

export interface LockNoteProps {
  /** Whether the control is held. */
  locked: boolean;
  /** The note's id, for the control's `aria-describedby`. */
  id?: string;
}

export function LockNote(props: LockNoteProps): JSX.Element {
  return (
    <Show when={props.locked}>
      <p class={css.lockNote} id={props.id} data-testid="setting-lock-note">
        <svg class={css.lockIcon} viewBox="0 0 16 16" aria-hidden="true">
          <rect x="3" y="7" width="10" height="7" rx="1" fill="none" stroke="currentColor" stroke-width="1.5" />
          <path d="M5.5 7V5a2.5 2.5 0 0 1 5 0v2" fill="none" stroke="currentColor" stroke-width="1.5" />
        </svg>
        <span>{t('settings-policy-locked')}</span>
      </p>
    </Show>
  );
}
