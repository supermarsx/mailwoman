import { createResource, For, onMount, Show, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../i18n';
import { listAdminSettings } from '../../api/admin/settings.ts';
import { adminRequest } from './shell/adminApi.ts';
import { describeAdminError } from './shell/Feedback.tsx';
import { useShell } from './shell/context.ts';
import { hrefFor, sectionPath } from './shell/router.ts';
import { AdminSubsection } from './AdminSubsection.tsx';
import * as css from './admin.css.ts';
export interface PrivacyStats { cacheEntries: number; cacheBytes: number; trackersReplaced: number; traffic: { class: string; allowed: number; refused: number; failed: number; totalMs: number }[] }
export const privacyAdminApi = { settings: listAdminSettings, async stats(): Promise<PrivacyStats> { return (await adminRequest<PrivacyStats>('/egress/privacy-stats')).data; } };
export function AdminPrivacy(props: { service?: typeof privacyAdminApi }): JSX.Element {
  const api = props.service ?? privacyAdminApi; const shell = useShell();
  onMount(() => void loadCatalog('privacy'));
  const [settings] = createResource(async () => shell.can.can('settings.read') ? await api.settings() : null);
  const [stats, { refetch }] = createResource(() => api.stats());
  const labels: Record<string,string> = { 'privacy.remoteContent':'privacy-remote-mode', 'privacy.trackers':'privacy-trackers', 'privacy.cleanLinks':'privacy-clean-links', 'privacy.allowDirect':'privacy-allow-direct', 'privacy.maxBytes':'privacy-max-bytes', 'privacy.cacheBytes':'privacy-cache-limit', 'privacy.ratePerMin':'privacy-rate' };
  return <section class={css.section}><h2>{t('privacy-admin-title')}</h2><p>{t('privacy-admin-help')}</p>
    <Show when={settings.error}><p role="alert">{describeAdminError(settings.error)}</p></Show>
    <Show when={settings.state === 'ready' && settings.latest?.kind === 'ok' ? settings.latest.data.filter(row => Object.hasOwn(labels, row.key)) : []}>{rows => <AdminSubsection title={t('privacy-policy-values')}><For each={rows()}>{row => <div class={css.listRow}><div><strong>{t(labels[row.key] ?? row.key)}</strong><p>{typeof row.effective.value === 'boolean' ? t(row.effective.value ? 'privacy-on':'privacy-off') : String(row.effective.value)}</p><Show when={row.globalLocked}><p>{t('privacy-locked')}</p></Show></div><a class="btn" href={hrefFor(sectionPath('settings',[row.key]))} onClick={e => { e.preventDefault(); void shell.router.navigate(sectionPath('settings',[row.key])); }}>{t('privacy-edit-setting')}</a></div>}</For></AdminSubsection>}</Show>
    <AdminSubsection title={t('privacy-process-stats')} actions={<button class="btn" disabled={stats.loading} onClick={() => void refetch()}>{t('privacy-refresh')}</button>}>
    <Show when={stats.error}><p role="alert">{describeAdminError(stats.error)}</p></Show>
    <Show when={stats.state === 'ready' ? stats.latest : undefined}>{value => <><dl><dt>{t('privacy-cache-entries')}</dt><dd>{value().cacheEntries}</dd><dt>{t('privacy-cache-bytes')}</dt><dd>{value().cacheBytes}</dd><dt>{t('privacy-replaced-trackers')}</dt><dd>{value().trackersReplaced}</dd></dl><table><thead><tr><th>{t('privacy-traffic-class')}</th><th>{t('privacy-allowed')}</th><th>{t('privacy-refused')}</th><th>{t('privacy-failed')}</th></tr></thead><tbody><For each={value().traffic}>{row => <tr><th scope="row">{row.class.replaceAll('_',' ')}</th><td>{row.allowed}</td><td>{row.refused}</td><td>{row.failed}</td></tr>}</For></tbody></table></>}</Show>
    </AdminSubsection>
  </section>;
}
