// Admin › Users › one account (`#/users/<accountId>[/<tab>]`). The header (initial, name,
// address, status) stays above a route-driven tablist; a tab's content mounts on its first
// visit and then stays mounted (hidden), so an unsaved edit survives a tab switch and nothing
// refetches. The tab list is filtered by the admin's capabilities.

import { createMemo, createResource, createSignal, For, Index, Show, onMount, useContext, type JSX } from 'solid-js';
import { useAdmin } from '../context.ts';
import type { UserFeatureFlags, UserSummary } from '../../../state/slices/admin.ts';
import { loadCatalog } from '../../../i18n/catalog.ts';
import { t, formatDateTime } from '../../../i18n/index.ts';
import { AsyncPending } from '../../../components/AsyncState.tsx';
import { ShellContext } from '../shell/context.ts';
import { adminRequest } from '../shell/adminApi.ts';
import { hrefFor, sectionPath } from '../shell/router.ts';
import { Feedback } from '../shell/Feedback.tsx';
import { RouteTabs, tabId, tabPanelId } from '../shell/RouteTabs.tsx';
import * as auditCss from '../shell/audit.css.ts';
import * as css from '../admin.css.ts';
import { UserAccessActions } from '../UserAccessActions.tsx';
import { UserFeatureProfile } from './UserFeatureProfile.tsx';
import { UserComplianceTab } from './UserComplianceTab.tsx';
import { resolveUserTab, visibleUserTabs, type UserTabId } from './userTabs.ts';
import { initials } from './UsersTable.tsx';
import * as ux from './userDetail.css.ts';

interface Identity { accountId: string; label: string; address: string; loginAddress?: string; directoryAddress?: string; legacyBinding?: boolean }
const PREFIX = 'user';

export function UserDetail(props: { id: string; tab?: string }): JSX.Element {
  const shell = useContext(ShellContext);
  const { api } = useAdmin();
  const [identity] = createResource(() => props.id, async (id) => (await adminRequest<Identity>(`/users/${encodeURIComponent(id)}/identity`)).data);
  const [users, setUsers] = createSignal<UserSummary[]>([]);
  const [error, setError] = createSignal<string | null>(null);
  const [failure, setFailure] = createSignal<string | null>(null);
  const [openRequests, setOpenRequests] = createSignal<number | undefined>();
  const fail = (message: string): void => { setFailure(null); setFailure(message); };

  async function reload(): Promise<void> {
    try { setUsers(await api.listUsers()); setError(null); } catch { setError(t('admin-users-load-error')); }
  }
  onMount(() => { void loadCatalog('admin-users'); void reload(); });
  // `.latest` throws on a failed load; the page still works without the identity.
  const ident = (): Identity | undefined => (identity.error ? undefined : identity.latest);
  const user = createMemo(() => users().find((u) => u.accountId === props.id));
  const one = (): UserSummary[] => { const u = user(); return u ? [u] : []; };

  const tabs = createMemo(() => visibleUserTabs(shell?.can));
  const selected = (): UserTabId => resolveUserTab(props.tab ?? shell?.router.route().rest[1], tabs());
  // Panels open on first visit and stay mounted afterwards.
  const [visited, setVisited] = createSignal<ReadonlySet<UserTabId>>(new Set());
  const seen = (id: UserTabId): boolean => visited().has(id) || selected() === id;
  createMemo(() => { const id = selected(); if (!visited().has(id)) setVisited(new Set([...visited(), id])); });
  const items = () => tabs().map((tab) => ({ id: tab.id, label: t(tab.label), path: sectionPath('users', [props.id, tab.id]), badge: tab.id === 'compliance' ? openRequests() : undefined }));
  const open = (path: string): void => void shell?.router.navigate(path);

  const account = (u: UserSummary): string => u.domain ? `${u.username}@${u.domain}` : u.username;
  const addresses = (): string[] => {
    const i = ident();
    return [...new Set([i?.loginAddress, i?.directoryAddress, i?.address].filter((a): a is string => typeof a === 'string' && a !== ''))];
  };
  const statusKey = (u: UserSummary | undefined): string => `admin-user-status-${u?.status ?? 'active'}`;

  async function patchFlag(u: UserSummary, key: keyof UserFeatureFlags, box: HTMLInputElement): Promise<void> {
    const value = box.checked;
    try {
      if (key === 'zeroAccess') await api.toggleZeroAccess(u.accountId, value);
      else await api.setFlags(u.accountId, { ...u.flags, [key]: value });
      await reload();
      const stored = users().find((x) => x.accountId === u.accountId);
      if (stored) box.checked = stored.flags[key];
    } catch { box.checked = !value; fail(t('admin-users-flag-error')); }
  }
  async function onRevoke(u: UserSummary): Promise<void> {
    try { await api.revokeSessions(u.accountId); await reload(); } catch { fail(t('admin-users-revoke-error')); }
  }

  return <section class={css.section} aria-label={t('admin-users-title')}>
    <a href={hrefFor(sectionPath('users'))}>{t('admin-users-back')}</a>
    <h2 class={css.heading}>{t('admin-users-identity')}</h2>
    <Show when={identity.loading}><AsyncPending variant="status" /></Show>
    <Show when={identity.error}><p role="alert">{t('admin-users-load-error')}</p></Show>
    <Show when={error()}><p class={css.error} role="alert">{error()}</p></Show>
    <Feedback kind="error" as="toast" message={failure()} />
    <Show when={!identity.error && identity.state === 'ready' ? identity.latest : undefined}>{(value) =>       <div class={`${css.card} ${ux.header}`} data-testid="user-header">
        <div class={ux.headline}>
          <span class={ux.avatar} aria-hidden="true">{initials({ displayName: null, username: (value().loginAddress ?? value().address).split('@')[0] ?? '?' })}</span>
          <div class={ux.who}>
            <h3 dir="auto">{value().legacyBinding ? value().loginAddress ?? value().label : value().label}</h3>
            <p dir="auto">{value().loginAddress ?? value().address}</p>
          </div>
          <Show when={user()}>{(u) => <span class={ux.status} data-status={u().status ?? 'active'}>{t(statusKey(u()))}</span>}</Show>
        </div>
      </div>}</Show>
      <RouteTabs label={t('admin-user-tabs-label')} prefix={PREFIX} items={items()} selected={selected()} navigate={open} />
      <For each={tabs()}>{(tab) => <div role="tabpanel" class={ux.panel} id={tabPanelId(PREFIX, tab.id)} aria-labelledby={tabId(PREFIX, tab.id)} hidden={selected() !== tab.id} data-testid={`user-panel-${tab.id}`}>
        <Show when={seen(tab.id)}>
          <Show when={tab.id === 'overview'}>
            <section class={css.card}>
              <Show when={ident()?.legacyBinding && ident()?.directoryAddress}><p class={css.note}>{t('admin-users-legacy-alias', { address: ident()!.directoryAddress! })}</p></Show>
              <dl class={ux.facts}>
                <dt>{t('admin-users-id')}</dt><dd><code class={auditCss.actor}>{props.id}</code></dd>
                <Show when={user()}>{(u) => <>
                  <dt>{t('admin-user-overview-status')}</dt><dd>{t(statusKey(u()))}</dd>
                  <dt>{t('admin-user-overview-last-signin')}</dt><dd>{u().lastSigninAt ? formatDateTime(u().lastSigninAt!) : t('admin-user-never')}</dd>
                </>}</Show>
              </dl>
            </section>
            <Index each={one()}>{(u) => <section class={css.card}><h3>{t('admin-users-col-quota')}</h3><p>{u().quota ? `${u().quota?.bytesLimit} / ${u().quota?.msgLimit}` : '—'}</p></section>}</Index>
          </Show>
          <Show when={tab.id === 'profile'}><UserFeatureProfile accountId={props.id} /></Show>
          <Show when={tab.id === 'security'}>
            <Index each={one()}>{(u) => <section class={css.card}><h3>{t('admin-users-col-flags')}</h3><For each={[
              ['forcePasswordChange', 'admin-users-force-change-for', 'admin-users-force-change-help'],
              ['zeroAccess', 'admin-users-zeroaccess-for', 'admin-users-zeroaccess-help'],
            ] as const}>{([key, label, help]) => <div class="field"><label class="field"><input type="checkbox" checked={u().flags[key]} disabled={u().status === 'deleted' || u().status === 'deleting'} aria-describedby={`user-${key}-help`} onChange={(event) => void patchFlag(u(), key, event.currentTarget)} />{t(label, { account: account(u()) })}</label><p class={css.note} id={`user-${key}-help`}>{t(help)}</p></div>}</For></section>}</Index>
          </Show>
          <Show when={tab.id === 'sessions'}>
            <Index each={one()}>{(u) => <section class={css.card}><h3>{t('admin-users-col-sessions')}</h3><button type="button" class="btn btn--ghost" disabled={!(shell?.can.can('users.revoke_sessions') ?? true)} aria-label={t('admin-users-revoke-for', { account: u().accountId })} onClick={() => void onRevoke(u())}>{t('admin-revoke')}</button></section>}</Index>
          </Show>
          <Show when={tab.id === 'compliance'}><UserComplianceTab addresses={addresses()} onCount={setOpenRequests} /></Show>
          <Show when={tab.id === 'access'}>
            <Index each={one()}>{(u) => <section class={css.card}><h3>{t('admin-users-access-title')}</h3><p class={css.note}>{t('admin-users-access-help')}</p><UserAccessActions user={u()} onChanged={reload} /></section>}</Index>
          </Show>
        </Show>
      </div>}</For>
  </section>;
}
