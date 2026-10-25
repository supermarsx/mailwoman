import { For, Show, createSignal, onMount, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../../i18n';
import { adminsApi, type AdminAccountsApi, type AdminAccount, type AdminRole, type AdminMe, type AdminSession, type RoleGrant, type Override } from '../../../api/admin/admins.ts';
import { Feedback, describeAdminError } from '../shell/Feedback.tsx';
import { ConfirmDialog } from '../shell/ConfirmDialog.tsx';
import * as css from '../shell/shell.css.ts';
import * as ux from '../shell/ux.css.ts';

export function AdminAdmins(props: { service?: AdminAccountsApi }): JSX.Element {
  const api = props.service ?? adminsApi;
  const [accounts, setAccounts] = createSignal<AdminAccount[]>([]);
  const [roles, setRoles] = createSignal<AdminRole[]>([]);
  const [me, setMe] = createSignal<AdminMe>();
  const [sessions, setSessions] = createSignal<AdminSession[]>([]);
  const [selected, setSelected] = createSignal<AdminAccount>();
  const [name, setName] = createSignal('');
  const [password, setPassword] = createSignal('');
  const [createPassword, setCreatePassword] = createSignal('');
  const [username, setUsername] = createSignal('');
  const [grants, setGrants] = createSignal<RoleGrant[]>([]);
  const [domainText, setDomainText] = createSignal<Record<string, string>>({});
  const [overrides, setOverrides] = createSignal<Override[]>([]);
  const [overrideCap, setOverrideCap] = createSignal('');
  const [overrideEffect, setOverrideEffect] = createSignal<'allow' | 'deny'>('deny');
  const [error, setError] = createSignal<string | null>(null);
  const [success, setSuccess] = createSignal<string | null>(null);
  const [busy, setBusy] = createSignal(false);
  const [loaded, setLoaded] = createSignal(false);
  const [secret, setSecret] = createSignal('');
  const [confirm, setConfirm] = createSignal<{ name: string; action: () => Promise<unknown> }>();
  const canManage = () => me()?.capabilities.includes('admins.manage') === true;
  async function reload(): Promise<void> {
    const [a, r, m, s] = await Promise.all([api.accounts(), api.roles(), api.me(), api.sessions()]);
    setAccounts(a); setRoles(r); setMe(m); setSessions(s); setLoaded(true);
    const current = selected();
    if (current) { const fresh = a.find((row) => row.id === current.id); setSelected(fresh); }
  }
  async function run(action: () => Promise<unknown>, announce = true): Promise<void> {
    setBusy(true); setError(null); setSuccess(null);
    try { await action(); await reload(); if (announce) setSuccess(t('admin-accounts-saved')); }
    catch (e) { setError(describeAdminError(e)); }
    finally { setBusy(false); }
  }
  function select(a: AdminAccount): void { setSelected(a); setName(a.displayName); setPassword(''); setSecret(''); setGrants(a.roles.map((g) => ({ ...g }))); setDomainText(Object.fromEntries(a.roles.map((g) => [g.roleId, g.domains?.join(', ') ?? '']))); setOverrides([...a.overrides]); }
  const path = () => `/admins/${encodeURIComponent(selected()!.id)}`;
  onMount(() => { void loadCatalog('admin-admins'); void run(reload, false); });
  return <section class={ux.page} aria-label={t('admin-accounts-title')}>
    <h2>{t('admin-accounts-title')}</h2>
    <Feedback kind="error" message={error()} /><Feedback kind="success" message={success()} />
    <button class="btn" type="button" disabled={busy()} onClick={() => void run(reload, false)}>{t('admin-accounts-refresh')}</button>
    <Show when={loaded()} fallback={<p role="status">{t('admin-accounts-loading')}</p>}>
      <Show when={canManage()}><form class={ux.form} onSubmit={(e) => { e.preventDefault(); void run(async () => { await api.write('/admins', 'POST', { username: username(), displayName: username(), password: createPassword(), roles: [] }); setCreatePassword(''); setUsername(''); }); }}>
        <h3>{t('admin-accounts-create')}</h3>
        <label class={css.field}>{t('admin-accounts-username')}<input class={ux.control} required maxLength={64} autocomplete="off" value={username()} onInput={(e) => setUsername(e.currentTarget.value)} /></label>
        <label class={css.field}>{t('admin-accounts-password')}<input class={ux.control} required type="password" minLength={12} autocomplete="new-password" value={createPassword()} onInput={(e) => setCreatePassword(e.currentTarget.value)} /></label>
        <button class="btn" disabled={busy()}>{t('admin-accounts-create')}</button>
      </form></Show>
      <ul><For each={accounts()}>{(a) => <li><button class="btn" type="button" onClick={() => select(a)}>{a.displayName} ({a.username})</button> — {t(a.disabled ? 'admin-accounts-disabled' : 'admin-accounts-enabled')}</li>}</For></ul>
      <Show when={selected()}>{(account) => <section aria-label={account().username}>
        <h3>{account().username}</h3>
        <p>{t(account().totpEnrolled ? 'admin-accounts-totp-on' : 'admin-accounts-totp-off')}</p>
        <fieldset class={ux.fieldset} disabled={busy() || !canManage()}>
          <legend>{t('admin-accounts-edit')}</legend>
          <label class={css.field}>{t('admin-accounts-display-name')}<input class={ux.control} value={name()} maxLength={128} onInput={(e) => setName(e.currentTarget.value)} /></label>
          <button class="btn" type="button" onClick={() => void run(() => api.write(path(), 'PUT', { displayName: name(), disabled: account().disabled }, account().etag))}>{t('admin-accounts-save')}</button>
          <button class="btn" type="button" onClick={() => setConfirm({ name: account().username, action: () => api.write(path(), 'PUT', { displayName: name(), disabled: !account().disabled }, account().etag) })}>{t(account().disabled ? 'admin-accounts-enable' : 'admin-accounts-disable')}</button>
          <button class="btn" type="button" onClick={() => setConfirm({ name: account().username, action: () => api.write(path(), 'DELETE', undefined, account().etag) })}>{t('admin-accounts-delete')}</button>
          <h4>{t('admin-accounts-roles')}</h4>
          <For each={roles()}>{(role) => <div>
            <label class={ux.checkboxRow}><input type="checkbox" checked={grants().some((g) => g.roleId === role.id)} onChange={(e) => setGrants(e.currentTarget.checked ? [...grants(), { roleId: role.id, domains: null }] : grants().filter((g) => g.roleId !== role.id))} />{role.name}</label>
            <Show when={grants().find((g) => g.roleId === role.id)}>{(grant) => <label class={css.field}>{t('admin-accounts-domains')}<input class={ux.control} aria-label={`${role.name}: ${t('admin-accounts-domains')}`} value={domainText()[grant().roleId] ?? ''} onInput={(e) => setDomainText({ ...domainText(), [role.id]: e.currentTarget.value })} /></label>}</Show>
          </div>}</For>
          <button class="btn" type="button" onClick={() => void run(() => api.write(`${path()}/roles`, 'PUT', { roles: grants().map((g) => { const ds = (domainText()[g.roleId] ?? '').split(',').map((d) => d.trim()).filter(Boolean); return { ...g, domains: ds.length ? ds : null }; }) }, account().etag))}>{t('admin-accounts-save-roles')}</button>
          <h4>{t('admin-accounts-overrides')}</h4>
          <ul><For each={overrides()}>{(o) => <li>{o.capability}: {o.effect} <button class="btn" type="button" onClick={() => setOverrides(overrides().filter((v) => v !== o))}>{t('admin-accounts-remove')}</button></li>}</For></ul>
          <label class={css.field}>{t('admin-accounts-capability')}<select class={ux.control} value={overrideCap()} onChange={(e) => setOverrideCap(e.currentTarget.value)}><option value="">{t('admin-accounts-select')}</option><For each={me()?.capabilities ?? []}>{(cap) => <option>{cap}</option>}</For></select></label>
          <label class={css.field}>{t('admin-accounts-effect')}<select class={ux.control} value={overrideEffect()} onChange={(e) => setOverrideEffect(e.currentTarget.value as 'allow' | 'deny')}><option value="deny">{t('admin-accounts-deny')}</option><option value="allow">{t('admin-accounts-allow')}</option></select></label>
          <button class="btn" type="button" disabled={!overrideCap()} onClick={() => setOverrides([...overrides().filter((o) => o.capability !== overrideCap()), { capability: overrideCap(), effect: overrideEffect() }])}>{t('admin-accounts-add')}</button>
          <button class="btn" type="button" onClick={() => void run(() => api.write(`${path()}/overrides`, 'PUT', { overrides: overrides() }, account().etag))}>{t('admin-accounts-save-overrides')}</button>
          <label class={css.field}>{t('admin-accounts-password')}<input class={ux.control} type="password" autocomplete="new-password" minLength={12} value={password()} onInput={(e) => setPassword(e.currentTarget.value)} /></label>
          <button class="btn" type="button" disabled={password().length < 12} onClick={() => setConfirm({ name: account().username, action: async () => { await api.write(`${path()}/reset-password`, 'POST', { password: password() }); setPassword(''); } })}>{t('admin-accounts-reset-password')}</button>
          <button class="btn" type="button" onClick={() => setConfirm({ name: account().username, action: async () => { const result = await api.write<{ secret: string }>(`${path()}/totp`, 'POST'); setSecret(result.secret); } })}>{t('admin-accounts-reset-totp')}</button>
          <button class="btn" type="button" disabled={!account().totpEnrolled} onClick={() => setConfirm({ name: account().username, action: () => api.write(`${path()}/totp`, 'DELETE') })}>{t('admin-accounts-remove-totp')}</button>
        </fieldset>
        <Show when={secret()}><p role="status">{t('admin-accounts-secret-once')} <code>{secret()}</code><button class="btn" type="button" onClick={() => setSecret('')}>{t('admin-accounts-dismiss')}</button></p></Show>
      </section>}</Show>
      <h3>{t('admin-accounts-sessions')}</h3>
      <ul><For each={sessions()}>{(session) => <li>{session.adminId} — {session.createdAt} {session.current ? t('admin-accounts-current') : ''}<button class="btn" type="button" disabled={busy() || !canManage()} onClick={() => setConfirm({ name: session.adminId, action: () => api.write(`/admin-sessions/${encodeURIComponent(session.id)}`, 'DELETE') })}>{t('admin-accounts-revoke')}</button></li>}</For></ul>
    </Show>
    <ConfirmDialog open={confirm() !== undefined} title={t('admin-accounts-confirm', { name: confirm()?.name ?? '' })} body={t('admin-accounts-confirm-body')} confirmLabel={t('admin-accounts-confirm-action')} onCancel={() => setConfirm(undefined)} onConfirm={async () => { const action = confirm()?.action; if (action) { await action(); setConfirm(undefined); await run(reload); } }} />
  </section>;
}
