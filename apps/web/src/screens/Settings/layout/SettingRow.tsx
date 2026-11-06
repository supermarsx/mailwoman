import { Show, untrack, type JSX } from 'solid-js';
import { t } from '../../../i18n';
import { LockNote } from '../LockNote.tsx';
import * as css from './layout.css.ts';

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
export function SettingRow(props: { controlId: string; label: string; help?: string; handle: RowHandle; children: (describedBy: () => string) => JSX.Element }): JSX.Element {
  const helpId = (): string => `${props.controlId}-help`;
  const lockId = (): string => `${props.controlId}-lock`;
  const disabled = (): boolean => !props.handle.entry() || props.handle.locked() || props.handle.saving();
  return <div class={css.row} data-setting-row>
    <label class={css.rowLabel} for={props.controlId}>{props.label}</label>
    <Show when={props.help}><p id={helpId()} class={css.rowHelp}>{props.help}</p></Show>
    <div class={css.rowControl}>
      {untrack(() => props.children(() => `${props.help ? helpId() : ''} ${props.handle.locked() ? lockId() : ''}`.trim()))}
      <button type="button" class={css.rowReset} disabled={disabled()} onClick={() => void props.handle.reset()}>{t('reader-inherit')}</button>
    </div>
    <div class={css.rowNote}>
      <LockNote id={lockId()} locked={props.handle.locked()} />
      <Show when={props.handle.error()}><p role="alert">{props.handle.error()}</p></Show>
    </div>
  </div>;
}
