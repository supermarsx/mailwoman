// "Send later" split-button next to Send: quick presets and a date-time picker
// showing the viewer's time zone. Picking a time calls `onPick` with the local
// `datetime-local` value; the composer schedules it through its normal send
// path, so pre-send checks run before the message is queued.

import { For, Show, createSignal, onMount, type JSX } from 'solid-js';
import './SendLaterMenu.css';
import { formatDateTime, loadCatalog, t } from '../../i18n';
import {
  checkSendAt,
  horizonSeconds,
  localTimeZone,
  sendLaterPresets,
  toLocalInput,
  type SendAtCheck,
} from '../../state/sendDelay.ts';

export interface SendLaterMenuProps {
  disabled?: boolean;
  onPick: (localValue: string) => void;
}

function problem(check: SendAtCheck): string {
  switch (check) {
    case 'invalid': return t('send-later-err-invalid');
    case 'past': return t('send-later-err-past');
    case 'off': return t('send-later-err-off');
    case 'horizon': return t('send-later-err-horizon', { days: Math.max(1, Math.floor(horizonSeconds() / 86_400)) });
    default: return '';
  }
}

export function SendLaterMenu(props: SendLaterMenuProps): JSX.Element {
  const [open, setOpen] = createSignal(false);
  const [picking, setPicking] = createSignal(false);
  const [value, setValue] = createSignal('');
  const [error, setError] = createSignal('');
  const [now, setNow] = createSignal(new Date());
  onMount(() => void loadCatalog('send-delay'));

  function toggle(): void {
    setNow(new Date());
    setOpen(!open());
    setPicking(false);
    setError('');
  }
  function choose(at: Date): void {
    const check = checkSendAt(at, new Date());
    if (check !== 'ok') {
      setError(problem(check));
      return;
    }
    setOpen(false);
    props.onPick(toLocalInput(at));
  }
  function submitCustom(): void {
    choose(new Date(value()));
  }

  return (
    <span class="send-later" data-testid="send-later">
      <button
        type="button"
        class="btn btn--ghost send-later__toggle"
        aria-haspopup="menu"
        aria-expanded={open()}
        aria-label={t('send-later-menu')}
        data-tooltip={t('send-later-button')}
        disabled={props.disabled === true}
        onClick={toggle}
      >
        <span aria-hidden="true">▾</span>
      </button>
      <Show when={open()}>
        <div class="send-later__menu" role="menu" aria-label={t('send-later-button')}>
          <For each={sendLaterPresets(now())}>
            {(preset) => (
              <button type="button" role="menuitem" class="send-later__item" data-preset={preset.id} onClick={() => choose(preset.at)}>
                <span>{t(`send-later-preset-${preset.id}`)}</span>
                <span class="send-later__when">{formatDateTime(preset.at.toISOString())}</span>
              </button>
            )}
          </For>
          <button type="button" role="menuitem" class="send-later__item" data-preset="custom" onClick={() => { setPicking(true); setValue(toLocalInput(new Date(now().getTime() + 3_600_000))); }}>
            {t('send-later-custom')}
          </button>
          <Show when={picking()}>
            <div class="send-later__picker">
              <label>
                {t('send-later-custom-label')}
                <input
                  type="datetime-local"
                  value={value()}
                  min={toLocalInput(now())}
                  onInput={(e) => { setValue(e.currentTarget.value); setError(''); }}
                />
              </label>
              <p class="send-later__zone">{t('send-later-zone', { zone: localTimeZone() })}</p>
              <button type="button" class="btn btn--primary" data-testid="send-later-schedule" onClick={submitCustom}>
                {t('send-later-schedule')}
              </button>
            </div>
          </Show>
          <Show when={error() !== ''}>
            <p class="send-later__error" role="alert">{error()}</p>
          </Show>
        </div>
      </Show>
    </span>
  );
}
