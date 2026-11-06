import { Show, untrack, type JSX } from 'solid-js';
import { t } from '../../../i18n';
import { LockNote } from '../LockNote.tsx';
import * as css from './layout.css.ts';
import { loadCatalog } from '../../../i18n/catalog.ts';
// i18n: the ids this file shows live in these catalogs, which load with it.
void loadCatalog('reader');

/** The slice of a useSetting handle a row needs. */
export interface RowHandle {
  locked(): boolean;
  saving(): boolean;
  error(): string | undefined | null;
  entry(): unknown;
  reset(): unknown;
}

/** One compact row: label (and muted hint) at the start, control at the end,
 * then the administrator lock note and any error. `controlId` is the id the
 * control must carry; `children` receives `aria-describedby` for it. */
export function SettingRow(props: {
  controlId: string; label: string; help?: string; handle: RowHandle; children: (describedBy: () => string) => JSX.Element;
  /** Control sits under the label at full width (textareas, long lists). */
  stacked?: boolean;
  /** Replace the default reset (handle.reset) and its text/enabled state. */
  onReset?: () => void; resetLabel?: string; resetDisabled?: boolean;
  /** Show the administrator lock note even when the handle itself is free. */
  lockNote?: boolean;
}): JSX.Element {
  const helpId = (): string => `${props.controlId}-help`;
  const lockId = (): string => `${props.controlId}-lock`;
  const disabled = (): boolean => props.resetDisabled ?? (!props.handle.entry() || props.handle.locked() || props.handle.saving());
  return <div class={props.stacked ? `${css.row} ${css.rowStacked}` : css.row} data-setting-row>
    <label class={css.rowLabel} for={props.controlId}>{props.label}</label>
    <Show when={props.help}><p id={helpId()} class={css.rowHelp}>{props.help}</p></Show>
    <div class={css.rowControl}>
      {untrack(() => props.children(() => `${props.help ? helpId() : ''} ${(props.lockNote ?? props.handle.locked()) ? lockId() : ''}`.trim()))}
      <button type="button" class={css.rowReset} disabled={disabled()} onClick={() => (props.onReset ? props.onReset() : void props.handle.reset())}>{props.resetLabel ?? t('reader-inherit')}</button>
    </div>
    <div class={css.rowNote}>
      <LockNote id={lockId()} locked={props.lockNote ?? props.handle.locked()} />
      <Show when={props.handle.error()}><p role="alert">{props.handle.error()}</p></Show>
    </div>
  </div>;
}

/** A compact row for a control that is not a useSetting handle (local state,
 * button groups, time fields, per-device browser state): label and hint at the
 * start, the control at the end, then an optional lock note / status below.
 * `group` labels a role="group" control (the label is a span carrying
 * `controlId`, which the group references with aria-labelledby); otherwise the label targets `controlId`. */
export function ControlRow(props: {
  controlId: string; label: string; help?: string; group?: boolean; stacked?: boolean; note?: JSX.Element;
  children: (describedBy: () => string) => JSX.Element;
}): JSX.Element {
  const helpId = (): string => `${props.controlId}-help`;
  // Button groups are as wide as their labels: they sit under the row label rather than squeezing it.
  const stacked = (): boolean => props.stacked ?? props.group === true;
  return <div class={stacked() ? `${css.row} ${css.rowStacked}` : css.row} data-setting-row>
    {props.group
      ? <span class={css.rowLabel} id={props.controlId}>{props.label}</span>
      : <label class={css.rowLabel} for={props.controlId}>{props.label}</label>}
    <Show when={props.help}><p id={helpId()} class={css.rowHelp}>{props.help}</p></Show>
    <div class={css.rowControl}>{untrack(() => props.children(() => (props.help ? helpId() : '')))}</div>
    <Show when={props.note}><div class={css.rowNote}>{props.note}</div></Show>
  </div>;
}
