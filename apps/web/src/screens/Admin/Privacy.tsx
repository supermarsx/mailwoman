// Admin console > Privacy. Five tabs like the Security centre (SectionTabs, one
// route segment per tab, per-tab capability gating, lazy tab code): Overview,
// Remote content, Image proxy, Fetch limits and Traffic. Every setting is edited
// in place (Settings/InlineSettingEditor.tsx). `#/privacy/policy/<key>` links of
// earlier versions open the tab that owns the key and focus its row.

import { createEffect, createMemo, createResource, createSignal, lazy, on, onCleanup, onMount, Match, Show, Switch, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../i18n';
import { listAdminSettings, putGlobal, type AdminSetting } from '../../api/admin/settings.ts';
import { IMAGE_CONCURRENCY_KEY, type ImageConcurrencyMode } from './AdminImageConcurrency.tsx';
import { adminRequest } from './shell/adminApi.ts';
import { useShell } from './shell/context.ts';
import { hrefFor, sectionPath } from './shell/router.ts';
import { SectionTabs } from './shell/SectionTabs.tsx';
import { AdminSubtabPanel } from './shell/AdminSubtabPanel.tsx';
import { PRIVACY_TABS, canOpenPrivacyTab, privacyRouteTab } from './shell/privacyNavigation.ts';
import { SettingDetail } from './Settings/SettingDetail.tsx';
import { settingLabel } from './Settings/labels.ts';
import { tabForKey, type HistoryRange, type PrivacyStats, type PrivacyTabContext } from './Privacy/model.ts';
import * as css from './admin.css.ts';

export { FETCH_LIMIT_KEYS, fetchLimitValue, type PrivacyStats } from './Privacy/model.ts';

export const privacyAdminApi = {
  settings: listAdminSettings,
  async stats(range?: HistoryRange): Promise<PrivacyStats> {
    return (await adminRequest<PrivacyStats>(`/egress/privacy-stats${range ? `?range=${range}` : ''}`)).data;
  },
  async saveMode(mode: ImageConcurrencyMode, etag: string): Promise<AdminSetting> {
    return (await putGlobal(IMAGE_CONCURRENCY_KEY, { globalValue: mode }, etag)).setting;
  },
};
export type PrivacyAdminApi = Pick<typeof privacyAdminApi, 'settings'> & { stats: (range?: HistoryRange) => Promise<PrivacyStats> } & Partial<Pick<typeof privacyAdminApi, 'saveMode'>>;

const OverviewTab = lazy(() => import('./Privacy/OverviewTab.tsx'));
const RemoteTab = lazy(() => import('./Privacy/RemoteTab.tsx'));
const ImageProxyTab = lazy(() => import('./Privacy/ImageProxyTab.tsx'));
const FetchLimitsTab = lazy(() => import('./Privacy/FetchLimitsTab.tsx'));
const TrafficTab = lazy(() => import('./Privacy/TrafficTab.tsx'));

export function AdminPrivacy(props: { service?: PrivacyAdminApi }): JSX.Element {
  const api = props.service ?? privacyAdminApi;
  const shell = useShell();
  onMount(() => { void loadCatalog('privacy'); void loadCatalog('admin-settings'); void loadCatalog('admin-stats'); });
  const canReadSettings = (): boolean => shell.can.can('settings.read');
  const [settings, { mutate: mutateSettings, refetch: refetchSettings }] = createResource(async () => (canReadSettings() ? await api.settings() : null));
  const rows = createMemo((): AdminSetting[] => (!settings.error && settings.latest?.kind === 'ok' ? settings.latest.data : []));
  const [range, setRange] = createSignal<HistoryRange>('1h');
  const [stats, { refetch }] = createResource(range, (r) => api.stats(r));
  const [updatedAt, setUpdatedAt] = createSignal<number | null>(null);
  createEffect(on(() => (stats.error ? undefined : stats.latest), (value) => { if (value) setUpdatedAt(Date.now()); }));

  // Polling pauses while the tab is hidden or any editor holds unsaved edits.
  const dirtyKeys = new Set<string>();
  const [dirtyCount, setDirtyCount] = createSignal(0);
  const onDirty = (key: string, dirty: boolean): void => {
    if (dirty) dirtyKeys.add(key); else dirtyKeys.delete(key);
    setDirtyCount(dirtyKeys.size);
  };
  onMount(() => {
    const timer = setInterval(() => { if (document.visibilityState !== 'hidden' && dirtyCount() === 0 && !stats.loading) void refetch(); }, 5000);
    onCleanup(() => clearInterval(timer));
  });

  const tabs = () => PRIVACY_TABS.filter((tab) => canOpenPrivacyTab(shell.can, tab));
  const route = () => privacyRouteTab(shell.router.route());
  const focusKey = (): string | undefined => { const key = route().focusKey; return key && key.includes('.') ? key : undefined; };
  const selected = (): string => route().tab ?? (focusKey() ? tabForKey(focusKey()!, rows()) : undefined) ?? tabs()[0]?.id ?? '';
  const allowed = (): boolean => tabs().some((tab) => tab.id === selected());
  /** A `policy/<key>` link of a key no tab owns keeps the full detail page. */
  const detailKey = (): string | undefined => {
    const r = shell.router.route();
    const key = r.rest[0] === 'policy' ? r.rest[1] : undefined;
    return key && canReadSettings() && tabForKey(key, rows()) === undefined && rows().some((row) => row.key === key) ? key : undefined;
  };

  const ctx: PrivacyTabContext = {
    rows,
    settingsError: () => settings.error,
    canReadSettings,
    stats: () => (stats.error ? undefined : stats.latest),
    statsError: () => stats.error,
    statsLoading: () => stats.loading,
    updatedAt,
    range,
    setRange,
    refresh: () => void refetch(),
    focusKey,
    onDirty,
    onSaved: (row) => {
      mutateSettings((value) => (value?.kind === 'ok' ? { ...value, data: value.data.map((current) => (current.key === row.key ? row : current)) } : value));
      if (row.key === IMAGE_CONCURRENCY_KEY) void refetch();
    },
    reloadSettings: () => void refetchSettings(),
    saveMode: api.saveMode,
  };
  const open = (tab: string): void => void shell.router.navigate(sectionPath('privacy', [tab]));

  return (
    <Show
      when={detailKey()}
      keyed
      fallback={
        <section class={css.section}>
          <h2>{t('privacy-admin-title')}</h2>
          <SectionTabs label={t('privacy-admin-title')} items={tabs().map((tab) => ({ id: tab.id, label: t(tab.label), path: sectionPath('privacy', [tab.id]) }))} selected={selected()} navigate={(path) => void shell.router.navigate(path)} />
          <AdminSubtabPanel>
            <Show when={allowed()} fallback={<p role="alert">{t('privacy-tab-denied')}</p>}>
              <Switch>
                <Match when={selected() === 'overview'}><OverviewTab ctx={ctx} open={open} /></Match>
                <Match when={selected() === 'remote'}><RemoteTab ctx={ctx} /></Match>
                <Match when={selected() === 'image-proxy'}><ImageProxyTab ctx={ctx} canWrite={shell.can.can('settings.write')} /></Match>
                <Match when={selected() === 'fetch-limits'}><FetchLimitsTab ctx={ctx} /></Match>
                <Match when={selected() === 'traffic'}><TrafficTab ctx={ctx} /></Match>
              </Switch>
            </Show>
          </AdminSubtabPanel>
        </section>
      }
    >
      {(key) => (
        <section class={css.section}>
          <a class="btn btn--ghost" href={hrefFor(sectionPath('privacy'))} onClick={(event) => { if (event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) return; event.preventDefault(); open('overview'); }}>{t('privacy-back-policies')}</a>
          <h2>{settingLabel(key)}</h2>
          <SettingDetail settingKey={key} />
        </section>
      )}
    </Show>
  );
}
