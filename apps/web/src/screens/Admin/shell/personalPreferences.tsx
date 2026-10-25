import { createContext, createEffect, createSignal, on, onCleanup, useContext, type JSX } from 'solid-js';
import { adminRequest } from './adminApi.ts';
import { AdminAppearanceContext } from './AdminAppearance.tsx';
import { createAdminAppearance, parseAdminAppearance } from './appearanceStore.ts';
import { isKnownLocale, negotiateLocale, useI18n, type I18nContext } from '../../../i18n';

export interface AdminPreferencesApi {
  load(): Promise<{ data: Record<string, unknown>; etag: string | null }>;
  save(data: Record<string, unknown>, etag: string | null): Promise<{ data: Record<string, unknown>; etag: string | null }>;
}
export const httpAdminPreferences: AdminPreferencesApi = {
  load: () => adminRequest<Record<string, unknown>>('/me/preferences'),
  save: (body, ifMatch) => adminRequest<Record<string, unknown>>('/me/preferences', { method: 'PUT', body, ifMatch }),
};

function optionalI18n(): I18nContext | undefined { try { return useI18n(); } catch { return undefined; } }

function createPreferences(username: () => string, api: AdminPreferencesApi) {
  const i18n = optionalI18n();
  const initialLocale = negotiateLocale(typeof navigator === 'undefined' ? ['en'] : navigator.languages);
  const [loading, setLoading] = createSignal(true);
  const [saving, setSaving] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [data, setData] = createSignal<Record<string, unknown>>({});
  let generation = 0;
  let etag: string | null = null;
  let revision = 0;
  let savedRevision = 0;
  const appearance = createAdminAppearance({
    store: { load: () => null, save: (value) => patch({ appearance: value }) },
    webmail: () => null,
  });

  async function load(): Promise<void> {
    const mine = ++generation;
    setLoading(true); setSaving(false); setError(null); setData({});
    revision = 0; savedRevision = 0; etag = null;
    appearance.hydrate(null);
    if (i18n && initialLocale) i18n.setLocale(initialLocale);
    try {
      const response = await api.load();
      if (mine !== generation) return;
      etag = response.etag; setData(response.data);
      appearance.hydrate(parseAdminAppearance(response.data['appearance']));
      const locale = response.data['locale'];
      if (i18n && typeof locale === 'string' && isKnownLocale(locale)) i18n.setLocale(locale);
    } catch (cause) {
      if (mine === generation) setError(cause instanceof Error ? cause.message : String(cause));
    } finally { if (mine === generation) setLoading(false); }
  }
  async function flush(): Promise<void> {
    if (loading() || saving() || revision === savedRevision) return;
    const mine = generation;
    setSaving(true); setError(null);
    try {
      while (mine === generation && revision !== savedRevision) {
        const sent = revision;
        const response = await api.save(data(), etag);
        if (mine !== generation) return;
        etag = response.etag; savedRevision = sent;
      }
    } catch (cause) {
      if (mine === generation) setError(cause instanceof Error ? cause.message : String(cause));
    } finally { if (mine === generation) setSaving(false); }
  }
  function patch(values: Record<string, unknown>): void {
    if (loading() || error()) return;
    setData((current) => ({ ...current, ...values })); revision++;
    void flush();
  }
  createEffect(on(username, () => { void load(); }));
  onCleanup(() => { generation++; appearance.dispose(); });
  return { appearance, loading, saving, error, data, i18n, patch, reload: load, retry: () => revision === savedRevision ? load() : flush() };
}
export type PersonalPreferences = ReturnType<typeof createPreferences>;
const PreferencesContext = createContext<PersonalPreferences>();
export function useOptionalPersonalPreferences(): PersonalPreferences | undefined { return useContext(PreferencesContext); }
export function usePersonalPreferences(): PersonalPreferences {
  const value = useContext(PreferencesContext);
  if (!value) throw new Error('Personal preferences require the signed-in console');
  return value;
}
export function PersonalPreferencesProvider(props: { username: string; api?: AdminPreferencesApi | undefined; children: JSX.Element }): JSX.Element {
  const preferences = createPreferences(() => props.username, props.api ?? httpAdminPreferences);
  return <PreferencesContext.Provider value={preferences}><AdminAppearanceContext.Provider value={preferences.appearance}>{props.children}</AdminAppearanceContext.Provider></PreferencesContext.Provider>;
}
