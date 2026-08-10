// Admin › Appearance (§19). Brand name + default theme + accent for the served
// SPA. Persisted via PUT; audited. This is the DEPLOYMENT default appearance —
// distinct from a user's per-account theme in Settings.
//
// t19 e13: it is a DEFAULT, not a policy. A user who has chosen an appearance
// keeps it, and changing this afterwards does not move them — the server serves
// these values alongside the account's own at `GET /api/account/appearance` and
// the client prefers the account's (SPEC §17.3). The theme set includes the
// high-contrast packs, so an enforceable deployment theme would be an enforceable
// accessibility regression; the reasoning is recorded in
// `crates/mw-admin/src/config.rs`. The panel now says so instead of leaving an
// operator to infer it from a control that looks like a switch.
//
// The theme list is read from the theme registry rather than hardcoded here: it
// was a stale literal of 7 ids that silently omitted every pack shipped since.

import { createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { useAdmin } from './context.ts';
import type { Appearance as AppearanceCfg } from '../../state/slices/admin.ts';
import { THEME_GROUPS } from '../../theme/registry.ts';
import { t, loadCatalog } from '../../i18n';
import * as css from './admin.css.ts';

const EMPTY: AppearanceCfg = { theme: 'light', brandName: 'Mailwoman', accent: null };

export function Appearance(): JSX.Element {
  const { api } = useAdmin();
  const [cfg, setCfg] = createSignal<AppearanceCfg>(EMPTY);
  const [error, setError] = createSignal<string | null>(null);
  const [saved, setSaved] = createSignal(false);

  onMount(() => {
    // `appearance.ftl` (t19 e13) rather than `admin.ftl`: the note below is the
    // theme-ownership copy, which belongs with the rest of that feature area.
    void loadCatalog('appearance');
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
      <p class={css.note}>{t('appearance-admin-default-note')}</p>
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
            <For each={THEME_GROUPS}>
              {(group) => (
                <Show when={group.themes.length > 0}>
                  <optgroup label={group.label}>
                    <For each={group.themes}>
                      {(entry) => <option value={entry.id}>{entry.label}</option>}
                    </For>
                  </optgroup>
                </Show>
              )}
            </For>
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
