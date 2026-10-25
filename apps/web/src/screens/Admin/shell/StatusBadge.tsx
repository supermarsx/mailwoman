// A state shown as a small badge (enabled, loaded, signed, expired …).
//
// The state is in the text and the icon; the tone only adds a coloured edge, so
// nothing is told by colour alone (t32-cfg UX13). Forced-colours mode keeps the
// border.

import type { JSX } from 'solid-js';
import { Icon, type IconName } from './icons.tsx';
import * as css from './extras.css.ts';

export type StatusTone = 'ok' | 'warn' | 'danger' | 'off' | 'neutral';

const ICON: Record<StatusTone, IconName> = {
  ok: 'check',
  warn: 'alert',
  danger: 'alert',
  off: 'close',
  neutral: 'info',
};

export interface StatusBadgeProps {
  tone: StatusTone;
  children: JSX.Element;
  testId?: string;
  /** A stable value for tests and styles (`data-status`). */
  status?: string;
}

export function StatusBadge(props: StatusBadgeProps): JSX.Element {
  return (
    <span class={css.statusBadge} data-tone={props.tone} data-status={props.status} data-testid={props.testId}>
      <Icon name={ICON[props.tone]} />
      <span>{props.children}</span>
    </span>
  );
}
