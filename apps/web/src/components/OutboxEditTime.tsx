// "Edit send time" for a message scheduled in the Outbox: the same quick
// choices and date-time picker as the composer's Send later menu, checked
// against the administrator's furthest allowed time before anything is sent
// to the server. The save is a compare-and-set on the time the row showed, so
// a row that changed meanwhile (sent, moved from another window) says so.

import { For, Show, createSignal, onCleanup, onMount, type JSX } from 'solid-js';
import './compose/SendLaterMenu.css';
import './OutboxReader.css';
import { formatDateTime, loadCatalog, t } from '../i18n';
import {
  checkSendAt,
  horizonSeconds,
  localTimeZone,
  sendLaterPresets,
  toLocalInput,
  type SendAtCheck,
} from '../state/sendDelay.ts';
import type { RescheduleResult } from '../state/slices/outbox.ts';

export interface OutboxEditTimeProps {
  /** The send time the row has now (ISO), the starting value. */
  current: string | null;
  save: (isoUtc: string) => Promise<RescheduleResult>;
  onClose: () => void;
}

export function sendAtProblem(check: SendAtCheck, horizon = horizonSeconds()): string {
  switch (check) {
    case 'invalid': return t('send-later-err-invalid');
    case 'past': return t('send-later-err-past');
    case 'off': return t('send-later-err-off');
    case 'horizon': return t('send-later-err-horizon', { days: Math.max(1, Math.floor(horizon / 86_400)) });
    default: return '';
  }
}

export function OutboxEditTime(props: OutboxEditTimeProps): JSX.Element {
  const start = props.current !== null && !Number.isNaN(new Date(props.current).getTime()) ? new Date(props.current) : new Date(Date.now() + 3_600_000);
  const [value, setValue] = createSignal(toLocalInput(start));
  const [error, setError] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const [now] = createSignal(new Date());
  let panel: HTMLDivElement | undefined;
  onMount(() => {
    void loadCatalog('send-delay');
    panel?.querySelector<HTMLElement>('input,button')?.focus();
    const onKey = (event: KeyboardEvent): void => { if (event.key === 'Escape') props.onClose(); };
    document.addEventListener('keydown', onKey);
    onCleanup(() => document.removeEventListener('keydown', onKey));
  });

  async function apply(at: Date): Promise<void> {
    const check = checkSendAt(at, new Date());
    if (check !== 'ok') { setError(sendAtProblem(check)); return; }
    setBusy(true);
    setError('');
    try {
      const result = await props.save(at.toISOString());
      if (result.ok) props.onClose();
      else setError(result.kind === 'conflict' ? t('outbox-edit-time-conflict') : t('outbox-edit-time-refused', { error: result.reason }));
    } catch {
      setError(t('outbox-edit-time-refused', { error: '' }));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div class="outbox-edit-time__scrim" onClick={(event) => { if (event.target === event.currentTarget) props.onClose(); }}>
      <div class="outbox-edit-time" role="dialog" aria-modal="true" aria-label={t('outbox-edit-time')} ref={panel} data-testid="outbox-edit-time">
        <h3 class="outbox-edit-time__title">{t('outbox-edit-time')}</h3>
        <Show when={props.current !== null}>
          <p class="send-later__zone">{t('outbox-edit-time-current', { when: formatDateTime(props.current ?? '') })}</p>
        </Show>
        <For each={sendLaterPresets(now())}>
          {(preset) => (
            <button type="button" class="send-later__item" data-preset={preset.id} disabled={busy()} onClick={() => void apply(preset.at)}>
              <span>{t(`send-later-preset-${preset.id}`)}</span>
              <span class="send-later__when">{formatDateTime(preset.at.toISOString())}</span>
            </button>
          )}
        </For>
        <div class="send-later__picker">
          <label>
            {t('send-later-custom-label')}
            <input type="datetime-local" value={value()} min={toLocalInput(now())} onInput={(event) => { setValue(event.currentTarget.value); setError(''); }} />
          </label>
          <p class="send-later__zone">{t('send-later-zone', { zone: localTimeZone() })}</p>
        </div>
        <Show when={error() !== ''}>
          <p class="send-later__error" role="alert">{error()}</p>
        </Show>
        <div class="outbox-edit-time__actions">
          <button type="button" class="btn btn--ghost" onClick={() => props.onClose()}>{t('send-later-cancel')}</button>
          <button type="button" class="btn btn--primary" data-testid="outbox-edit-time-save" disabled={busy()} onClick={() => void apply(new Date(value()))}>{t('outbox-edit-time-save')}</button>
        </div>
      </div>
    </div>
  );
}
