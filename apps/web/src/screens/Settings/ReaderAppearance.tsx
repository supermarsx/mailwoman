import { For, Show, onMount, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../i18n';
import { useSetting } from './useSetting.ts';
import { LockNote } from './LockNote.tsx';
import * as css from './styles.css.ts';

const fields: { key: string; fallback: string | number; choices?: string[]; min?: number; max?: number; step?: number }[] = [
  { key: 'colorMode', fallback: 'auto', choices: ['auto', 'original', 'adaptive', 'invert'] },
  { key: 'fontSize', fallback: 14, min: 12, max: 24, step: 1 },
  { key: 'lineHeight', fallback: 1.6, min: 1.2, max: 2.2, step: 0.1 },
  { key: 'contentWidth', fallback: 'wide', choices: ['comfortable', 'wide', 'full'] },
  { key: 'loading', fallback: 'skeleton', choices: ['skeleton', 'indicator', 'none'] },
  { key: 'loadingDelayMs', fallback: 120, min: 0, max: 1000, step: 20 },
];

export function ReaderAppearanceSettings(): JSX.Element {
  onMount(() => { void loadCatalog('reader'); void loadCatalog('settings-policy'); });
  return <section class={css.section} aria-label={t('reader-settings-title')}><h3 class={css.heading}>{t('reader-settings-title')}</h3><p class={css.meta}>{t('reader-settings-help')}</p>
    <For each={fields}>{field => {
      const setting = useSetting<string | number>(`reader.${field.key}`, field.fallback, (v): v is string | number => field.choices ? typeof v === 'string' && field.choices.includes(v) : typeof v === 'number' && Number.isFinite(v));
      const disabled = () => !setting.entry() || setting.locked() || setting.saving();
      return <div class={css.field}><label for={`reader-${field.key}`}>{t(`reader-${field.key}`)}</label>
        <Show when={field.choices} fallback={<input id={`reader-${field.key}`} class={css.input} type="number" min={field.min} max={field.max} step={field.step} value={setting.value()} disabled={disabled()} onChange={e => { if (e.currentTarget.checkValidity()) void setting.set(e.currentTarget.valueAsNumber); }} aria-describedby={`reader-${field.key}-help`} />}>
          {choices => <select id={`reader-${field.key}`} class={css.select} value={setting.value()} disabled={disabled()} onChange={e => void setting.set(e.currentTarget.value)} aria-describedby={`reader-${field.key}-help`}><For each={choices()}>{choice => <option value={choice}>{t(`reader-option-${choice}`)}</option>}</For></select>}
        </Show>
        <p id={`reader-${field.key}-help`} class={css.meta}>{t(`reader-${field.key}-help`)}</p><LockNote locked={setting.locked()} />
        <Show when={setting.error()}><p role="alert">{setting.error()}</p></Show>
        <button type="button" class={css.button} disabled={disabled()} onClick={() => void setting.reset()}>{t('reader-inherit')}</button>
      </div>;
    }}</For>
  </section>;
}
