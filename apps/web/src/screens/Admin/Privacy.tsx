import { createResource, For, onMount, Show, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../i18n';
import { listAdminSettings } from '../../api/admin/settings.ts';
import { adminRequest } from './shell/adminApi.ts';
import { describeAdminError } from './shell/Feedback.tsx';
import { useShell } from './shell/context.ts';
import { hrefFor, sectionPath } from './shell/router.ts';
import { SettingDetail } from './Settings/SettingDetail.tsx';
import { AdminSubsection } from './AdminSubsection.tsx';
import { AsyncPending } from '../../components/AsyncState.tsx';
import * as css from './admin.css.ts';
import * as layout from './privacyLayout.css.ts';
export interface PrivacyStats { cacheEntries: number; cacheBytes: number; trackersReplaced: number; traffic: { class: string; allowed: number; refused: number; failed: number; totalMs: number }[] }
export const privacyAdminApi = { settings: listAdminSettings, async stats(): Promise<PrivacyStats> { return (await adminRequest<PrivacyStats>('/egress/privacy-stats')).data; } };
export function AdminPrivacy(props: { service?: typeof privacyAdminApi }): JSX.Element {
  const api = props.service ?? privacyAdminApi; const shell = useShell();
  onMount(() => {void loadCatalog('privacy');void loadCatalog('admin-settings');});
  const [settings] = createResource(async () => shell.can.can('settings.read') ? await api.settings() : null);
  const [stats, { refetch }] = createResource(() => api.stats());
  const labels: Record<string,string> = { 'privacy.remoteContent':'privacy-remote-mode', 'privacy.trackers':'privacy-trackers', 'privacy.cleanLinks':'privacy-clean-links', 'privacy.allowDirect':'privacy-allow-direct', 'privacy.maxBytes':'privacy-max-bytes', 'privacy.cacheBytes':'privacy-cache-limit', 'privacy.ratePerMin':'privacy-rate', 'privacy.remoteImagesToast':'privacy-image-toast' };
  const editKey=()=>shell.router.route().rest[0]==='policy' && Object.hasOwn(labels,shell.router.route().rest[1]??'') && shell.can.can('settings.read') ? shell.router.route().rest[1] : undefined;
  return <Show when={editKey()} keyed fallback={<section class={css.section}><h2>{t('privacy-admin-title')}</h2><p>{t('privacy-admin-help')}</p>
    <Show when={settings.error}><p role="alert">{describeAdminError(settings.error)}</p></Show>
    <Show when={settings.state === 'ready' && settings.latest?.kind === 'ok' ? settings.latest.data.filter(row => Object.hasOwn(labels, row.key)) : []}>{rows => <AdminSubsection title={t('privacy-policy-values')}><For each={rows()}>{row => <div class={css.listRow}><div class={layout.policyText}><strong>{t(labels[row.key] ?? row.key)}</strong><p>{typeof row.effective.value === 'boolean' ? t(row.effective.value ? 'privacy-on':'privacy-off') : String(row.effective.value)}</p><Show when={row.globalLocked}><p>{t('privacy-locked')}</p></Show></div><a class={`btn ${layout.policyAction}`} href={hrefFor(sectionPath('privacy',['policy',row.key]))} onClick={e => { e.preventDefault(); void shell.router.navigate(sectionPath('privacy',['policy',row.key])); }}>{t('privacy-edit-setting')}</a></div>}</For></AdminSubsection>}</Show>
    <AdminSubsection title={t('privacy-process-stats')} actions={<button class="btn" disabled={stats.loading} onClick={() => void refetch()}>{t('privacy-refresh')}</button>}>
    <Show when={stats.error}><p role="alert">{describeAdminError(stats.error)}</p></Show>
    <Show when={stats.loading && !stats.latest}><AsyncPending /></Show>
    <Show when={!stats.error ? stats.latest : undefined}>{value => <div class={css.section} aria-busy={stats.loading}>
      <div class={css.tableWrap} tabindex="0" role="region" aria-label={t('privacy-cache-summary')}>
        <table class={css.table} aria-label={t('privacy-cache-summary')}>
          <thead><tr><th scope="col">{t('privacy-metric')}</th><th scope="col">{t('privacy-value')}</th></tr></thead>
          <tbody>
            <tr><th scope="row">{t('privacy-cache-entries')}</th><td>{value().cacheEntries}</td></tr>
            <tr><th scope="row">{t('privacy-cache-bytes')}</th><td>{value().cacheBytes}</td></tr>
            <tr><th scope="row">{t('privacy-replaced-trackers')}</th><td>{value().trackersReplaced}</td></tr>
          </tbody>
        </table>
      </div>
      <div class={css.tableWrap} tabindex="0" role="region" aria-label={t('privacy-traffic-summary')}>
        <table class={css.table} aria-label={t('privacy-traffic-summary')}>
          <thead><tr><th scope="col">{t('privacy-traffic-class')}</th><th scope="col">{t('privacy-allowed')}</th><th scope="col">{t('privacy-refused')}</th><th scope="col">{t('privacy-failed')}</th><th scope="col">{t('privacy-total-time')}</th></tr></thead>
          <tbody><For each={value().traffic} fallback={<tr><td colspan="5">{t('privacy-no-traffic')}</td></tr>}>{row => <tr><th scope="row">{row.class.replaceAll('_',' ')}</th><td>{row.allowed}</td><td>{row.refused}</td><td>{row.failed}</td><td>{row.totalMs}</td></tr>}</For></tbody>
        </table>
      </div>
    </div>}</Show>
    </AdminSubsection>
  </section>}>{key=><section class={css.section}><a class="btn btn--ghost" href={hrefFor(sectionPath('privacy'))} onClick={event=>{if(event.ctrlKey||event.metaKey||event.shiftKey||event.altKey)return;event.preventDefault();void shell.router.navigate(sectionPath('privacy'));}}>{t('privacy-back-policies')}</a><h2>{t(labels[key]??key)}</h2><SettingDetail settingKey={key}/></section>}</Show>;
}
