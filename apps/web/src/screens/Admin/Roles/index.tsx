import { For, Show, createSignal, onMount, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../../i18n';
import { adminsApi, type AdminAccountsApi, type AdminRole, type AdminMe } from '../../../api/admin/admins.ts';
import { Feedback, describeAdminError } from '../shell/Feedback.tsx';
import { ConfirmDialog } from '../shell/ConfirmDialog.tsx';
import * as css from '../shell/shell.css.ts';
import * as ux from '../shell/ux.css.ts';

export function AdminRoles(props: { service?: AdminAccountsApi }): JSX.Element {
  const api = props.service ?? adminsApi;
  const [roles, setRoles] = createSignal<AdminRole[]>([]);
  const [me, setMe] = createSignal<AdminMe>();
  const [selected, setSelected] = createSignal<AdminRole>();
  const [name, setName] = createSignal('');
  const [caps, setCaps] = createSignal<string[]>([]);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [success, setSuccess] = createSignal<string | null>(null);
  const [deleting, setDeleting] = createSignal(false);
  const canManage = () => me()?.capabilities.includes('roles.manage') === true;
  const editable = () => canManage() && !selected()?.builtin;
  async function reload(): Promise<void> { const [r, m] = await Promise.all([api.roles(), api.me()]); setRoles(r); setMe(m); }
  async function run(action: () => Promise<unknown>, announce = true): Promise<void> {
    setBusy(true); setError(null); setSuccess(null);
    try { await action(); await reload(); if (announce) setSuccess(t('admin-accounts-saved')); }
    catch (e) { setError(describeAdminError(e)); }
    finally { setBusy(false); }
  }
  function select(role?: AdminRole): void { setSelected(role); setName(role?.name ?? ''); setCaps(role ? [...role.capabilities] : []); }
  onMount(() => { void loadCatalog('admin-admins'); void run(reload, false); });
  return <section class={ux.page} aria-label={t('admin-accounts-roles')}>
    <h2>{t('admin-accounts-roles')}</h2><Feedback kind="error" message={error()} /><Feedback kind="success" message={success()} />
    <button class="btn" type="button" disabled={busy()} onClick={() => void run(reload, false)}>{t('admin-accounts-refresh')}</button>
    <button class="btn" type="button" disabled={!canManage() || busy()} onClick={() => select()}>{t('admin-roles-create')}</button>
    <ul><For each={roles()}>{(role) => <li><button class="btn" type="button" onClick={() => select(role)}>{role.name}</button> {role.builtin ? t('admin-roles-builtin') : ''} — {t('admin-roles-holders', { count: role.holders })}</li>}</For></ul>
    <Show when={selected()?.builtin}><p>{t('admin-roles-readonly')}</p><button class="btn" type="button" disabled={!canManage() || busy()} onClick={() => { setSelected(undefined); setName(`${name()} ${t('admin-roles-copy')}`); }}>{t('admin-roles-clone')}</button></Show>
    <form class={ux.form} onSubmit={(e) => { e.preventDefault(); if (!editable()) return; void run(async () => { const role = selected(); const saved = await api.write<AdminRole>(role ? `/roles/${encodeURIComponent(role.id)}` : '/roles', role ? 'PUT' : 'POST', { name: name(), capabilities: caps() }, role?.etag); select(saved); }); }}>
      <fieldset class={ux.fieldset} disabled={busy() || !editable()}><legend>{t('admin-roles-edit')}</legend>
        <label class={css.field}>{t('admin-accounts-display-name')}<input class={ux.control} required maxLength={64} value={name()} onInput={(e) => setName(e.currentTarget.value)} /></label>
        <For each={[...new Set([...(me()?.capabilities ?? []), ...caps()])].sort()}>{(cap) => <label class={css.field}><span><input type="checkbox" checked={caps().includes(cap)} disabled={!me()?.capabilities.includes(cap)} onChange={(e) => setCaps(e.currentTarget.checked ? [...caps(), cap] : caps().filter((c) => c !== cap))} />{cap}</span></label>}</For>
        <button class="btn">{t('admin-accounts-save')}</button>
      </fieldset>
    </form>
    <Show when={selected() && !selected()?.builtin}><button class="btn" type="button" disabled={!canManage() || busy()} onClick={() => setDeleting(true)}>{t('admin-accounts-delete')}</button></Show>
    <ConfirmDialog open={deleting()} title={t('admin-accounts-confirm', { name: selected()?.name ?? '' })} body={t('admin-roles-delete-impact', { count: selected()?.holders ?? 0 })} confirmLabel={t('admin-accounts-delete')} onCancel={() => setDeleting(false)} onConfirm={async () => { const role = selected(); if (!role) return; await api.write(`/roles/${encodeURIComponent(role.id)}`, 'DELETE', undefined, role.etag); setDeleting(false); select(); await run(reload); }} />
  </section>;
}
