// Admin > Compliance > Restrictions: accounts whose processing is restricted
// (Art.18), apply and lift with a reason, and the restriction.* settings. A
// restriction pauses processing categories while the data stays stored. These
// are configurable controls; whether a configuration meets a given law is for the
// operator to decide.

import { For, Show, createResource, createSignal, onMount, type JSX } from 'solid-js';
import { formatDateTime, loadCatalog, t } from '../../../i18n';
import { listAdminSettings, type AdminSetting } from '../../../api/admin/settings.ts';
import { adminRequest } from '../shell/adminApi.ts';
import { useShell } from '../shell/context.ts';
import { sectionPath } from '../shell/router.ts';
import { SectionTabs } from '../shell/SectionTabs.tsx';
import { AdminSubtabPanel } from '../shell/AdminSubtabPanel.tsx';
import { AdminSubsection } from '../AdminSubsection.tsx';
import { InlineSettingEditor } from '../Settings/InlineSettingEditor.tsx';
import { describeAdminError } from '../shell/Feedback.tsx';
import * as css from '../admin.css.ts';

export interface RestrictionRow {
  id: string; accountId: string; address: string; domain: string; scopes: string[];
  mailDelivery: string; reason: string; source: string; requestId: string | null;
  createdBy: string; createdAt: number; expiresAt: number | null; liftedAt: number | null;
  liftedBy: string | null; liftReason: string | null; active: boolean;
}
export interface RestrictionList { restrictions: RestrictionRow[]; scopes: string[]; mailDelivery: string[] }

export interface RestrictionsApi {
  list: (all: boolean) => Promise<RestrictionList>;
  apply: (body: { address: string; scopes: string[]; reason: string; expiresInDays?: number; mailDelivery?: string }) => Promise<void>;
  lift: (address: string, reason: string) => Promise<void>;
  settings: () => Promise<AdminSetting[]>;
}

export const restrictionsApi: RestrictionsApi = {
  list: async (all) => (await adminRequest<RestrictionList>(`/restrictions${all ? '?all=true' : ''}`)).data,
  apply: async (body) => { await adminRequest('/restrictions', { method: 'POST', body }); },
  lift: async (address, reason) => { await adminRequest('/restrictions/lift', { method: 'POST', body: { address, reason } }); },
  settings: async () => { const r = await listAdminSettings(); return r.kind === 'ok' ? r.data : []; },
};

const when = (s: number | null): string => (s === null ? '' : formatDateTime(s * 1000));

export default function Restrictions(props: { api?: RestrictionsApi }): JSX.Element {
  const api = props.api ?? restrictionsApi;
  const shell = useShell();
  const tabs = [
    { id: 'list', label: 'admin-restrictions-list' },
    { id: 'apply', label: 'admin-restrictions-apply' },
    { id: 'settings', label: 'admin-restrictions-settings' },
  ];
  const selected = () => {
    const id = shell.router.route().rest[1];
    return tabs.some((tab) => tab.id === id) ? id! : 'list';
  };

  onMount(() => void loadCatalog('admin-restrictions'));
  const [showAll, setShowAll] = createSignal(false);
  const [list, { refetch }] = createResource(() => ({ all: showAll() }), (k) => api.list(k.all));
  const [rows] = createResource(() => api.settings().catch(() => [] as AdminSetting[]));
  const [address, setAddress] = createSignal('');
  const [reason, setReason] = createSignal('');
  const [days, setDays] = createSignal('');
  const [chosen, setChosen] = createSignal<string[] | null>(null);
  const [failure, setFailure] = createSignal<unknown>(null);
  const [message, setMessage] = createSignal('');
  const scopes = (): string[] => chosen() ?? list.latest?.scopes ?? [];
  const settingRows = (): AdminSetting[] => (rows.error ? [] : (rows.latest ?? []).filter((r) => r.key.startsWith('restriction.')));

  async function run(work: () => Promise<void>, done: string): Promise<void> {
    setFailure(null); setMessage('');
    try { await work(); setMessage(done); await refetch(); } catch (e) { setFailure(e); }
  }
  const toggle = (s: string, on: boolean): void => {
    const cur = new Set(scopes());
    if (on) cur.add(s); else cur.delete(s);
    setChosen([...cur]);
  };

  return (
    <section class={css.section} data-testid="admin-restrictions">
      <h2 class={css.heading}>{t('admin-restrictions-title')}</h2>
      <p class={css.note}>{t('admin-restrictions-intro')}</p>

      <SectionTabs label={t('admin-restrictions-title')} selected={selected()}
        items={tabs.map((tab) => ({ id: tab.id, label: t(tab.label), path: sectionPath('compliance', ['restrictions', tab.id]) }))}
        navigate={(path) => void shell.router.navigate(path)} />
      <AdminSubtabPanel>
      <Show when={message()}><p role="status">{message()}</p></Show>
      <Show when={failure()}><p role="alert">{describeAdminError(failure())}</p></Show>
      <Show when={selected() === 'apply'}>
      <AdminSubsection title={t('admin-restrictions-apply')}>
        <form onSubmit={(e) => {
          e.preventDefault();
          const d = Number(days());
          void run(() => api.apply({ address: address().trim(), scopes: scopes(), reason: reason().trim(), ...(days() !== '' && Number.isFinite(d) ? { expiresInDays: d } : {}) }), t('admin-restrictions-applied'));
        }}>
          <p><label>{t('admin-restrictions-address')} <input type="email" required value={address()} onInput={(e) => setAddress(e.currentTarget.value)} /></label></p>
          <fieldset>
            <legend>{t('admin-restrictions-scopes')}</legend>
            <For each={list.latest?.scopes ?? []}>{(s) => (
              <label><input type="checkbox" checked={scopes().includes(s) || chosen() === null} onChange={(e) => toggle(s, e.currentTarget.checked)} /> {t(`admin-restrictions-scope-${s}`)}</label>
            )}</For>
          </fieldset>
          <p><label>{t('admin-restrictions-reason')} <input required value={reason()} onInput={(e) => setReason(e.currentTarget.value)} /></label></p>
          <p><label>{t('admin-restrictions-days')} <input type="number" min="0" max="3650" value={days()} onInput={(e) => setDays(e.currentTarget.value)} /></label></p>
          <button type="submit" class="btn" data-testid="restriction-apply">{t('admin-restrictions-apply-button')}</button>
        </form>
      </AdminSubsection>
      </Show>

      <Show when={selected() === 'list'}>
      <AdminSubsection title={t('admin-restrictions-list')}
        actions={<label><input type="checkbox" checked={showAll()} onChange={(e) => setShowAll(e.currentTarget.checked)} /> {t('admin-restrictions-show-history')}</label>}>
        <Show when={(list.latest?.restrictions.length ?? 0) > 0} fallback={<p>{t('admin-restrictions-none')}</p>}>
          <table class={css.table} data-testid="restriction-table">
            <thead><tr>
              <th scope="col">{t('admin-restrictions-col-person')}</th><th scope="col">{t('admin-restrictions-col-scopes')}</th>
              <th scope="col">{t('admin-restrictions-col-since')}</th><th scope="col">{t('admin-restrictions-col-until')}</th>
              <th scope="col">{t('admin-restrictions-col-source')}</th><th scope="col">{t('admin-restrictions-col-reason')}</th>
              <th scope="col">{t('admin-restrictions-col-actions')}</th>
            </tr></thead>
            <tbody><For each={list.latest?.restrictions ?? []}>{(r) => (
              <tr data-testid={`restriction-${r.id}`}>
                <th scope="row">{r.address}</th>
                <td>{r.scopes.map((s) => t(`admin-restrictions-scope-${s}`)).join(', ')}</td>
                <td>{when(r.createdAt)}</td>
                <td>{r.active ? (r.expiresAt ? when(r.expiresAt) : t('admin-restrictions-until-lifted')) : t('admin-restrictions-lifted', { date: when(r.liftedAt), reason: r.liftReason ?? '' })}</td>
                <td>{t(`admin-restrictions-source-${r.source}`)}{r.requestId ? ` (${r.requestId.slice(0, 8)})` : ''}</td>
                <td>{r.reason}</td>
                <td><Show when={r.active}>
                  <button type="button" class="btn btn--ghost" data-testid={`restriction-lift-${r.id}`}
                    onClick={() => { const why = window.prompt(t('admin-restrictions-lift-reason')); if (why && why.trim() !== '') void run(() => api.lift(r.address, why.trim()), t('admin-restrictions-lifted-done')); }}>
                    {t('admin-restrictions-lift')}
                  </button>
                </Show></td>
              </tr>
            )}</For></tbody>
          </table>
        </Show>
      </AdminSubsection>

      </Show>
      <Show when={selected() === 'settings'}>
      <AdminSubsection title={t('admin-restrictions-settings')} description={t('admin-restrictions-settings-help')}>
        <For each={settingRows()}>{(row) => <InlineSettingEditor settingKey={row.key} setting={row} noScoped />}</For>
      </AdminSubsection>
      </Show>
      </AdminSubtabPanel>
    </section>
  );
}
