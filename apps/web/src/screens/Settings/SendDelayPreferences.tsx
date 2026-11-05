// Undo-send window (mail.send.recallDelaySeconds): a friendly selector within
// the administrator's cap, with a custom value for anything between presets.

import { Show, createSignal, onMount, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../i18n';
import { useSetting } from './useSetting.ts';
import { LockNote } from './LockNote.tsx';
import {
  DEFAULT_RECALL_SECONDS,
  RECALL_DELAY_KEY,
  RECALL_PRESETS,
  recallAllowed,
  recallCapSeconds,
  recallChoices,
} from '../../state/sendDelay.ts';
import * as css from './styles.css.ts';

const CUSTOM = 'custom';

/** "30 seconds", "2 minutes"; whole minutes read as minutes. */
export function delayLabel(seconds: number): string {
  if (seconds === 0) return t('send-delay-off');
  if (seconds >= 60 && seconds % 60 === 0) return t('send-delay-minutes', { n: seconds / 60 });
  return t('send-delay-seconds', { n: seconds });
}

export function SendDelayPreferences(): JSX.Element {
  const delay = useSetting<number>(RECALL_DELAY_KEY, DEFAULT_RECALL_SECONDS);
  const [custom, setCustom] = createSignal(false);
  const [invalid, setInvalid] = createSignal(false);
  onMount(() => void loadCatalog('send-delay'));
  const cap = (): number => recallCapSeconds();
  const off = (): boolean => !recallAllowed();
  const current = (): number => Math.min(delay.value(), cap());
  const isPreset = (): boolean => RECALL_PRESETS.includes(current());
  const selected = (): string => (custom() || !isPreset() ? CUSTOM : String(current()));
  const disabled = (): boolean => off() || !delay.entry() || delay.locked() || delay.saving();
  return (
    <section class={css.section} aria-label={t('send-delay-title')} data-testid="send-delay">
      <h3 class={css.heading}>{t('send-delay-title')}</h3>
      <p class={css.meta}>{t('send-delay-help')}</p>
      <Show when={off()}>
        <p class={css.meta} role="status">{t('send-delay-disabled')}</p>
      </Show>
      <label class={css.field}>
        {t('send-delay-label')}
        <select
          class={css.input}
          value={selected()}
          disabled={disabled()}
          onChange={(e) => {
            const v = e.currentTarget.value;
            setInvalid(false);
            if (v === CUSTOM) {
              setCustom(true);
              return;
            }
            setCustom(false);
            void delay.set(Number(v));
          }}
        >
          {recallChoices(cap()).map((s) => (
            <option value={String(s)}>{delayLabel(s)}</option>
          ))}
          <option value={CUSTOM}>{t('send-delay-custom')}</option>
        </select>
      </label>
      <Show when={selected() === CUSTOM}>
        <label class={css.field}>
          {t('send-delay-custom-label')}
          <input
            class={css.input}
            type="number"
            min="0"
            max={cap()}
            step="1"
            value={current()}
            disabled={disabled()}
            onChange={(e) => {
              const n = e.currentTarget.valueAsNumber;
              if (Number.isInteger(n) && n >= 0 && n <= cap()) {
                setInvalid(false);
                void delay.set(n);
              } else setInvalid(true);
            }}
          />
        </label>
        <p class={css.meta}>{t('send-delay-custom-help', { max: cap() })}</p>
      </Show>
      <Show when={invalid()}>
        <p role="alert">{t('send-delay-invalid', { max: cap() })}</p>
      </Show>
      <Show when={cap() < 1800 && !off()}>
        <p class={css.meta}>{t('send-delay-capped', { max: delayLabel(cap()) })}</p>
      </Show>
      <LockNote locked={delay.locked()} />
      <Show when={delay.error()}>
        <p role="alert">{delay.error()}</p>
      </Show>
    </section>
  );
}
