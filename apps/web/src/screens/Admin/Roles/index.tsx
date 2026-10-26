import {AsyncPending} from '../../../components/AsyncState.tsx';
import { For, Show, createSignal, onMount, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../../i18n';
import { adminsApi, type AdminAccountsApi, type AdminRole, type AdminMe } from '../../../api/admin/admins.ts';
import { Feedback, describeAdminError } from '../shell/Feedback.tsx';
import { ConfirmDialog } from '../shell/ConfirmDialog.tsx';
import { useShell } from '../shell/context.ts';
import { sectionPath } from '../shell/router.ts';
import { RoleEditor } from './RoleEditor.tsx';
import { rolePurpose, permissionPurpose } from './purpose.ts';
import * as css from '../admin.css.ts';
import * as ux from '../shell/ux.css.ts';

export function AdminRoles(props: { service?: AdminAccountsApi }): JSX.Element {
  const api = props.service ?? adminsApi;
  const shell = useShell();
  const [roles, setRoles] = createSignal<AdminRole[]>([]);
  const [me, setMe] = createSignal<AdminMe>();
  const [loaded, setLoaded] = createSignal(false);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [deleting, setDeleting] = createSignal<AdminRole>();
  const canManage = () => me()?.capabilities.includes('roles.manage') === true;
  const route = () => shell.router.route();
  const creating = () => route().rest[0] === 'new';
  const editing = () => route().rest.length > 0 && !creating();
  const selected = () => roles().find(r => r.id === route().rest[0]);
  const clone = () => roles().find(r => r.id === route().query?.['from']);
  const go = (rest: string[] = [], query?: Record<string, string>): void => { void shell.router.navigate(sectionPath('roles', rest, query)); };
  async function reload(): Promise<void> {
    setBusy(true); setError(null);
    try { const [r, m] = await Promise.all([api.roles(), api.me()]); setRoles(r); setMe(m); setLoaded(true); }
    catch (cause) { setError(describeAdminError(cause)); }
    finally { setBusy(false); }
  }
  async function save(name: string, capabilities: string[], description: string, role?: AdminRole): Promise<void> {
    const saved = await api.write<AdminRole>(role ? `/roles/${encodeURIComponent(role.id)}` : '/roles', role ? 'PUT' : 'POST', { name, capabilities, description }, role?.etag);
    setRoles(current => role ? current.map(r => r.id === role.id ? saved : r) : [...current, saved]);
  }
  onMount(() => { void loadCatalog('admin-admins'); void loadCatalog('admin-role-purpose'); void reload(); });
  return <>
    <Feedback kind="error" message={error()} /><Show when={loaded() && busy()}><AsyncPending variant="status"/></Show>
    <Show when={loaded()} fallback={<Show when={busy()} fallback={<button class="btn" type="button" onClick={()=>void reload()}>{t('common-retry')}</button>}><AsyncPending variant="list"/></Show>}>
      <Show when={creating() || editing()} fallback={<section class={css.section} aria-label={t('admin-accounts-roles')}>
        <h2 class={css.heading}>{t('admin-accounts-roles')}</h2>
        <p class={css.note}>{t('admin-roles-list-help')}</p>
        <div class={ux.pageActions}>
          <button class="btn" type="button" disabled={busy()} onClick={() => void reload()}>{t('admin-accounts-refresh')}</button>
          <button class="btn btn--primary" type="button" disabled={!canManage() || busy()} onClick={() => go(['new'])}>{t('admin-roles-create')}</button>
        </div>
        <ul><For each={roles()}>{role => <li><button class="btn" type="button" onClick={() => go([role.id, 'edit'])}>{role.name}</button> {role.builtin ? t('admin-roles-builtin') : ''} — {t('admin-roles-holders', { count: role.holders })}<p class={css.note}>{rolePurpose(role)}</p></li>}</For></ul>
        <p class={css.note}>{t('admin-role-purpose-scope')}</p>
      </section>}>
        <Show when={creating()} fallback={<Show when={selected()} keyed fallback={<section class={css.section}><h2>{t('admin-roles-not-found')}</h2><button class="btn" type="button" onClick={() => go()}>{t('admin-roles-back')}</button></section>}>{role => <>
          <Show when={canManage() && !role.builtin} fallback={<section class={css.section}>
            <div class={ux.pageActions}><button class="btn" type="button" onClick={() => go()}>{t('admin-roles-back')}</button></div>
            <h2>{role.name}</h2><p>{role.builtin ? t('admin-roles-readonly') : t('admin-roles-no-manage')}</p>
            <p>{rolePurpose(role)}</p><p class={css.note}>{t('admin-role-purpose-scope')}</p>
            <ul><For each={role.capabilities}>{cap => <li><code>{cap}</code><p class={css.note}>{permissionPurpose(cap)}</p></li>}</For></ul>
            <Show when={canManage()}><button class="btn" type="button" onClick={() => go(['new'], { from: role.id })}>{t('admin-roles-clone')}</button></Show>
          </section>}>
            <RoleEditor initial={role} role={role} available={me()?.capabilities ?? []} save={(name, capabilities, description) => save(name, capabilities, description, role)} back={() => go()} />
            <div class={ux.pageActions}><button class="btn btn--danger" type="button" onClick={() => setDeleting(role)}>{t('admin-accounts-delete')}</button></div>
          </Show>
        </>}</Show>}>
          <Show when={canManage()} fallback={<section class={css.section}><p>{t('admin-roles-no-manage')}</p><button class="btn" type="button" onClick={() => go()}>{t('admin-roles-back')}</button></section>}>
            <RoleEditor initial={{ name: clone() ? `${clone()!.name} ${t('admin-roles-copy')}` : '', description: clone() ? rolePurpose(clone()!) : '', capabilities: clone()?.capabilities ?? [] }} available={me()?.capabilities ?? []} save={(name, capabilities, description) => save(name, capabilities, description)} back={() => go()} />
          </Show>
        </Show>
      </Show>
    </Show>
    <ConfirmDialog open={!!deleting()} title={t('admin-accounts-confirm', { name: deleting()?.name ?? '' })} body={t('admin-roles-delete-impact', { count: deleting()?.holders ?? 0 })} confirmLabel={t('admin-accounts-delete')} onCancel={() => setDeleting(undefined)} onConfirm={async () => { const role = deleting(); if (!role) return; await api.write(`/roles/${encodeURIComponent(role.id)}`, 'DELETE', undefined, role.etag); setDeleting(undefined); await reload(); go(); }} />
  </>;
}
