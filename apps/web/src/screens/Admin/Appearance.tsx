// Admin › Appearance (§19). Brand name + default theme + accent for the served
// SPA. Persisted via PUT; audited. This is the DEPLOYMENT default appearance —
// distinct from a user's per-account theme in Settings.

import { createSignal, Show, onMount, type JSX } from 'solid-js';
import { useAdmin } from './context.ts';
import type { Appearance as AppearanceCfg } from '../../state/slices/admin.ts';
import { t } from '../../i18n';
import * as css from './admin.css.ts';

const THEMES = ['light', 'dark', 'hc-light', 'hc-dark', 'amoled', 'grove-light', 'grove-dark'] as const;

const EMPTY: AppearanceCfg = { theme: 'light', brandName: 'Mailwoman', accent: null };

export function Appearance(): JSX.Element {
  const { api } = useAdmin();
  const [cfg, setCfg] = createSignal<AppearanceCfg>(EMPTY);
  const [error, setError] = createSignal<string | null>(null);
  const [saved, setSaved] = createSignal(false);

  onMount(() => {
    void (async () => {
      try {
        setCfg(await api.getAppearance());
      } catch {
        setError(t('admin-appearance-load-error'));
      }
    })();
  });

  function patch<K extends keyof AppearanceCfg>(key: K, value: AppearanceCfg[K]): void {
    setCfg({ ...cfg(), [key]: value });
    setSaved(false);
  }

  async function onSave(e: Event): Promise<void> {
    e.preventDefault();
    try {
      await api.setAppearance(cfg());
      setSaved(true);
      setError(null);
    } catch {
      setError(t('admin-appearance-save-error'));
    }
  }

  return (
    <section class={css.section} aria-label={t('admin-appearance-title')}>
      <h2 class={css.heading}>{t('admin-appearance-title')}</h2>
      <Show when={error()}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>

      <form class={css.card} onSubmit={(e) => void onSave(e)}>
        <label class="field">
          <span>{t('admin-appearance-brand')}</span>
          <input type="text" value={cfg().brandName} onInput={(e) => patch('brandName', e.currentTarget.value)} />
        </label>
        <label class="field">
          <span>{t('admin-appearance-theme')}</span>
          <select value={cfg().theme} onChange={(e) => patch('theme', e.currentTarget.value)}>
            {THEMES.map((name) => (
              <option value={name}>{name}</option>
            ))}
          </select>
        </label>
        <label class="field">
          <span>{t('admin-appearance-accent')}</span>
          <input
            type="text"
            value={cfg().accent ?? ''}
            placeholder={t('admin-appearance-accent-placeholder')}
            onInput={(e) => patch('accent', e.currentTarget.value === '' ? null : e.currentTarget.value)}
          />
        </label>
        <button type="submit" class="btn btn--primary">
          {t('admin-appearance-save')}
        </button>
        <Show when={saved()}>
          <p class={css.note} role="status">
            {t('admin-saved')}
          </p>
        </Show>
      </form>
    </section>
  );
}
