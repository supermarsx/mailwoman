// "Set by an environment variable" marker (t32-admin.md §2.1).
//
// A value pinned by the server's environment cannot be changed from the console.
// The badge names the variable, and the control next to it is rendered disabled
// with `aria-describedby` pointing here, so the reason is read with the control.

import type { JSX } from 'solid-js';
import { t } from '../../../i18n';
import * as css from './shell.css.ts';

export interface PinnedBadgeProps {
  /** The environment variable, e.g. `MW_LOG_LEVEL`. */
  envVar: string;
  id?: string;
}

export function PinnedBadge(props: PinnedBadgeProps): JSX.Element {
  return (
    <span id={props.id} class={css.pinned} data-testid="pinned-badge">
      {t('admin-shell-pinned-prefix')} <code class={css.pinnedVar}>{props.envVar}</code>
      <span class={css.srOnly}>{t('admin-shell-pinned-sr')}</span>
    </span>
  );
}
