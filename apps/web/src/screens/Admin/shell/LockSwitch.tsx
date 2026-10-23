// "Lock for users" switch for a setting (t32-admin.md §2.4, scope C).
//
// A `role=switch` button: when on, users cannot change the setting in their own
// settings and the global or domain value applies to them. The name stays the
// same ("Lock for users", with the setting's name for screen readers); the state
// is `aria-checked`, and on screen the knob moves to the other end and fills, so
// it is not shown by colour alone.

import type { JSX } from 'solid-js';
import { t } from '../../../i18n';
import * as css from './shell.css.ts';

export interface LockSwitchProps {
  locked: boolean;
  onChange: (locked: boolean) => void;
  /** The setting's name, read after the switch's label by screen readers. */
  settingLabel: string;
  disabled?: boolean;
  /** Id of a note explaining why the switch is disabled (pinned, no capability). */
  describedBy?: string;
}

export function LockSwitch(props: LockSwitchProps): JSX.Element {
  return (
    <button
      type="button"
      role="switch"
      class={css.lock}
      aria-checked={props.locked}
      aria-describedby={props.describedBy}
      disabled={props.disabled}
      onClick={() => props.onChange(!props.locked)}
    >
      <span class={css.lockTrack} aria-hidden="true">
        <span class={css.lockKnob} />
      </span>
      <span>{t('admin-shell-lock-text')}</span>
      <span class={css.srOnly}>{t('admin-shell-lock-setting', { setting: props.settingLabel })}</span>
    </button>
  );
}
