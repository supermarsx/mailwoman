// A single headline figure (t32-admin.md §2.2).
//
// Shows only a value the server read. `value === null` means the server cannot
// read it in this mode or on this platform: the tile says "Not available" and the
// reason, never a zero or an estimate.

import { Show, createUniqueId, type JSX } from 'solid-js';
import { formatNumber, t } from '../../../i18n';
import * as css from './shell.css.ts';

export interface StatTileProps {
  label: string;
  value: number | string | null;
  /** Formats a numeric value (bytes, durations). Defaults to locale digits. */
  format?: ((n: number) => string) | undefined;
  /** Why the value is absent; shown when `value` is null. */
  unavailableReason?: string | undefined;
  /** A short line under the value, e.g. "of 33 expected". */
  detail?: string | undefined;
  /** Link to the section that explains or fixes the figure. */
  href?: string | undefined;
  linkLabel?: string | undefined;
  onLink?: ((e: MouseEvent) => void) | undefined;
  testId?: string | undefined;
}

export function StatTile(props: StatTileProps): JSX.Element {
  const id = createUniqueId();
  const shown = (): string | null => {
    const v = props.value;
    if (v === null) return null;
    if (typeof v === 'string') return v;
    return props.format ? props.format(v) : formatNumber(v);
  };
  return (
    <figure class={css.tile} data-testid={props.testId} aria-labelledby={`${id}-label`}>
      <figcaption id={`${id}-label`} class={css.tileLabel}>
        {props.label}
      </figcaption>
      <Show
        when={shown() !== null}
        fallback={
          <>
            <p class={css.tileNa}>{t('admin-shell-not-available')}</p>
            <Show when={props.unavailableReason}>
              {(reason) => <p class={css.tileDetail}>{reason()}</p>}
            </Show>
          </>
        }
      >
        <p class={css.tileValue}>{shown()}</p>
        <Show when={props.detail}>{(detail) => <p class={css.tileDetail}>{detail()}</p>}</Show>
      </Show>
      <Show when={props.href}>
        {(href) => (
          <a href={href()} onClick={(e) => props.onLink?.(e)}>
            {props.linkLabel ?? props.label}
          </a>
        )}
      </Show>
    </figure>
  );
}
