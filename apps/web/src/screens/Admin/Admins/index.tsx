import { For, Show, createSignal, createEffect, on, onMount, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../../i18n';
import { adminsApi, type AdminAccountsApi, type AdminAccount, type AdminRole, type AdminMe, type AdminSession, type RoleGrant, type Override } from '../../../api/admin/admins.ts';
import { Feedback, describeAdminError } from '../shell/Feedback.tsx';
import { ConfirmDialog } from '../shell/ConfirmDialog.tsx';
import { useShell } from '../shell/context.ts';
import { sectionPath } from '../shell/router.ts';
import { useDirtyGuard } from '../shell/useDirtyGuard.ts';
import * as layout from '../admin.css.ts';
import * as css from '../shell/shell.css.ts';
import * as ux from '../shell/ux.css.ts';

export function AdminAdmins(props: { service?: AdminAccountsApi }): JSX.Element {
  const api = props.service ?? adminsApi;
  const shell = useShell();
  const route = () => shell.router.route();
  const creating = () => route().rest[0] === 'new';
  const editing = () => route().rest.length > 0 && !creating();
  const go = (rest: string[] = []): void => { void shell.router.navigate(sectionPath('admins', rest)); };
  const [accounts, setAccounts] = createSignal<AdminAccount[]>([]);
  const [roles, setRoles] = createSignal<AdminRole[]>([]);
  const [me, setMe] = createSignal<AdminMe>();
  const [sessions, setSessions] = createSignal<AdminSession[]>([]);
  const [selected, setSelected] = createSignal<AdminAccount>();
  const [name, setName] = createSignal('');
  const [password, setPassword] = createSignal('');
  const [createPassword, setCreatePassword] = createSignal('');
  const [username, setUsername] = createSignal('');
  const [createName, setCreateName] = createSignal('');
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
  const [confirm, setConfirm] = createSignal<{ name: string; typed?: boolean; action: () => Promise<unknown> }>();
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
  const normalizedGrants = (): RoleGrant[] => grants().map(g => {
    const domains = (domainText()[g.roleId] ?? '').split(',').map(d => d.trim()).filter(Boolean);
    return { roleId: g.roleId, domains: domains.length ? domains : null };
  });
  const grantKey = (rows: RoleGrant[]): string => JSON.stringify([...rows].sort((a, b) => a.roleId.localeCompare(b.roleId)).map(row => [row.roleId, row.domains === null ? null : [...row.domains].sort()]));
  const overrideKey = (rows: Override[]): string => JSON.stringify([...rows].sort((a, b) => a.capability.localeCompare(b.capability)).map(row => [row.capability, row.effect]));
  const dirty = () => creating() ? Boolean(username() || createName() || createPassword()) : editing() && Boolean(selected()) && (
    name() !== selected()!.displayName || Boolean(password()) || grantKey(normalizedGrants()) !== grantKey(selected()!.roles) || overrideKey(overrides()) !== overrideKey(selected()!.overrides)
  );
  useDirtyGuard(dirty);
  createEffect(on(() => [route().rest[0], loaded()] as const, () => {
    const found = accounts().find(a => a.id === route().rest[0]);
    if (found) select(found); else setSelected(undefined);
    setSecret(''); setPassword(''); setUsername(''); setCreateName(''); setCreatePassword(''); setSuccess(null);
  }));
  const path = () => `/admins/${encodeURIComponent(selected()!.id)}`;
  onMount(() => { void loadCatalog('admin-admins'); setBusy(true); void reload().catch(e => setError(describeAdminError(e))).finally(() => setBusy(false)); });
  return <section class={ux.page} aria-label={t('admin-accounts-title')}>
    <Show when={creating() || editing()}><div class={ux.pageActions}><button class="btn btn--ghost" type="button" disabled={busy()} onClick={() => go()}>{t('admin-accounts-back')}</button></div></Show>
    <h2 class={layout.heading}>{creating() ? t('admin-accounts-create') : editing() ? `${t('admin-accounts-edit')}: ${selected()?.username ?? ''}` : t('admin-accounts-title')}</h2>
    <Feedback kind="error" message={error()} /><Feedback kind="success" message={success()} />
    <Show when={!creating() && !editing()}><p class={ux.pageDescription}>{t('admin-accounts-list-help')}</p><div class={ux.pageActions}>
      <button class="btn" type="button" disabled={busy()} onClick={() => void run(reload, false)}>{t('admin-accounts-refresh')}</button>
      <Show when={canManage()}><button class="btn btn--primary" type="button" onClick={() => go(['new'])}>{t('admin-accounts-create')}</button></Show>
    </div></Show>
    <Show when={loaded()} fallback={<p role="status">{t('admin-accounts-loading')}</p>}>
      <Show when={creating()}>
        <Show when={canManage()} fallback={<p>{t('admin-accounts-readonly')}</p>}>
          <p class={ux.pageDescription}>{t('admin-accounts-create-help')}</p>
          <form class={ux.form} aria-label={t('admin-accounts-create')} onSubmit={e => {
            e.preventDefault();
            void run(async () => {
              const created = await api.write<AdminAccount>('/admins', 'POST', { username: username(), displayName: createName().trim() || username(), password: createPassword(), roles: [] });
              setCreatePassword(''); setUsername(''); setCreateName('');
              if (created.id) { setAccounts(rows => [...rows, created]); go([created.id, 'edit']); } else go();
            });
          }}>
            <fieldset class={ux.fieldset} disabled={busy()}><legend>{t('admin-accounts-identity')}</legend>
              <label class={css.field}>{t('admin-accounts-username')}<input class={ux.control} required maxLength={64} autocomplete="off" value={username()} onInput={e => setUsername(e.currentTarget.value)} /></label>
              <label class={css.field}>{t('admin-accounts-display-name')}<input class={ux.control} maxLength={128} value={createName()} onInput={e => setCreateName(e.currentTarget.value)} /></label>
              <label class={css.field}>{t('admin-accounts-password')}<input class={ux.control} required type="password" minLength={12} autocomplete="new-password" value={createPassword()} onInput={e => setCreatePassword(e.currentTarget.value)} /></label>
            </fieldset>
            <p>{t('admin-accounts-create-access')}</p>
            <div class={ux.pageActions}><button class="btn btn--primary" disabled={busy()}>{t('admin-accounts-create')}</button></div>
          </form>
        </Show>
      </Show>
      <Show when={!creating() && !editing()}><div class={layout.tableWrap}><table class={layout.table}>
        <thead><tr><th scope="col">{t('admin-accounts-username')}</th><th scope="col">{t('admin-accounts-status')}</th><th scope="col">{t('admin-accounts-authenticator')}</th></tr></thead>
        <tbody><For each={accounts()}>{a => <tr><th scope="row"><button class="btn" type="button" onClick={() => go([a.id, 'edit'])}>{a.displayName} ({a.username})</button></th><td>{t(a.disabled ? 'admin-accounts-disabled' : 'admin-accounts-enabled')}</td><td>{t(a.totpEnrolled ? 'admin-accounts-totp-on' : 'admin-accounts-totp-off')}</td></tr>}</For></tbody>
      </table></div></Show>
      <Show when={editing() && !selected()}><p role="alert">{t('admin-accounts-not-found')}</p></Show>
      <Show when={editing()}>
      <Show when={selected()}>{(account) => <section class={layout.section} aria-label={account().username}>
        <p class={ux.pageDescription}>{t('admin-accounts-edit-help')}</p><Show when={!canManage()}><p>{t('admin-accounts-readonly')}</p></Show>
        <p>{t(account().totpEnrolled ? 'admin-accounts-totp-on' : 'admin-accounts-totp-off')}</p>
        <fieldset class={ux.fieldset} disabled={busy() || !canManage()}>
          <legend>{t('admin-accounts-identity')}</legend>
          <label class={css.field}>{t('admin-accounts-display-name')}<input class={ux.control} value={name()} maxLength={128} onInput={(e) => setName(e.currentTarget.value)} /></label>
          <button class="btn" type="button" onClick={() => void run(() => api.write(path(), 'PUT', { displayName: name(), disabled: account().disabled }, account().etag))}>{t('admin-accounts-save')}</button>
        </fieldset>
        <fieldset class={ux.fieldset} disabled={busy() || !canManage()}><legend>{t('admin-accounts-roles')}</legend>
          <p>{t('admin-accounts-role-help')}</p>
          <For each={roles()}>{(role) => <div>
            <label class={ux.checkboxRow}><input type="checkbox" checked={grants().some((g) => g.roleId === role.id)} onChange={(e) => setGrants(e.currentTarget.checked ? [...grants(), { roleId: role.id, domains: null }] : grants().filter((g) => g.roleId !== role.id))} />{role.name}</label>
            <Show when={grants().find((g) => g.roleId === role.id)}>{(grant) => <label class={css.field}>{t('admin-accounts-domains')}<input class={ux.control} aria-label={`${role.name}: ${t('admin-accounts-domains')}`} value={domainText()[grant().roleId] ?? ''} onInput={(e) => setDomainText({ ...domainText(), [role.id]: e.currentTarget.value })} /></label>}</Show>
          </div>}</For>
          <button class="btn" type="button" onClick={() => void run(() => api.write(`${path()}/roles`, 'PUT', { roles: grants().map((g) => { const ds = (domainText()[g.roleId] ?? '').split(',').map((d) => d.trim()).filter(Boolean); return { ...g, domains: ds.length ? ds : null }; }) }, account().etag))}>{t('admin-accounts-save-roles')}</button>
        </fieldset>
        <fieldset class={ux.fieldset} disabled={busy() || !canManage()}><legend>{t('admin-accounts-overrides')}</legend>
          <p>{t('admin-accounts-override-help')}</p>
          <ul><For each={overrides()}>{(o) => <li>{o.capability}: {o.effect} <button class="btn" type="button" onClick={() => setOverrides(overrides().filter((v) => v !== o))}>{t('admin-accounts-remove')}</button></li>}</For></ul>
          <label class={css.field}>{t('admin-accounts-capability')}<select class={ux.control} value={overrideCap()} onChange={(e) => setOverrideCap(e.currentTarget.value)}><option value="">{t('admin-accounts-select')}</option><For each={me()?.capabilities ?? []}>{(cap) => <option>{cap}</option>}</For></select></label>
          <label class={css.field}>{t('admin-accounts-effect')}<select class={ux.control} value={overrideEffect()} onChange={(e) => setOverrideEffect(e.currentTarget.value as 'allow' | 'deny')}><option value="deny">{t('admin-accounts-deny')}</option><option value="allow">{t('admin-accounts-allow')}</option></select></label>
          <button class="btn" type="button" disabled={!overrideCap()} onClick={() => setOverrides([...overrides().filter((o) => o.capability !== overrideCap()), { capability: overrideCap(), effect: overrideEffect() }])}>{t('admin-accounts-add')}</button>
          <button class="btn" type="button" onClick={() => void run(() => api.write(`${path()}/overrides`, 'PUT', { overrides: overrides() }, account().etag))}>{t('admin-accounts-save-overrides')}</button>
        </fieldset>
        <fieldset class={ux.fieldset} disabled={busy() || !canManage()}><legend>{t('admin-accounts-credentials')}</legend>
          <p>{t('admin-accounts-credentials-help')}</p>
          <Show when={account().passkeyCount !== undefined}><p>{t('admin-accounts-passkey-count', { count: account().passkeyCount ?? 0 })}</p><button class="btn" type="button" disabled={!account().passkeyCount} onClick={() => setConfirm({ name: account().username, typed: true, action: () => api.write(`${path()}/passkeys`, 'DELETE') })}>{t('admin-accounts-revoke-passkeys')}</button><p>{t('admin-accounts-revoke-passkeys-help')}</p></Show>
          <label class={css.field}>{t('admin-accounts-password')}<input class={ux.control} type="password" autocomplete="new-password" minLength={12} value={password()} onInput={(e) => setPassword(e.currentTarget.value)} /></label>
          <button class="btn" type="button" disabled={password().length < 12} onClick={() => setConfirm({ name: account().username, action: async () => { await api.write(`${path()}/reset-password`, 'POST', { password: password() }); setPassword(''); } })}>{t('admin-accounts-reset-password')}</button>
          <button class="btn" type="button" onClick={() => setConfirm({ name: account().username, action: async () => { const result = await api.write<{ secret: string }>(`${path()}/totp`, 'POST'); setSecret(result.secret); } })}>{t('admin-accounts-reset-totp')}</button>
          <button class="btn" type="button" disabled={!account().totpEnrolled} onClick={() => setConfirm({ name: account().username, action: () => api.write(`${path()}/totp`, 'DELETE') })}>{t('admin-accounts-remove-totp')}</button>
        </fieldset>
        <Show when={secret()}><p role="status">{t('admin-accounts-secret-once')} <code>{secret()}</code><button class="btn" type="button" onClick={() => setSecret('')}>{t('admin-accounts-dismiss')}</button></p></Show>
        <fieldset class={ux.fieldset} disabled={busy() || !canManage()}><legend>{t('admin-accounts-access-state')}</legend>
          <p>{t('admin-accounts-access-state-help')}</p>
          <button class="btn" type="button" onClick={() => setConfirm({ name: account().username, action: () => api.write(path(), 'PUT', { displayName: account().displayName, disabled: !account().disabled }, account().etag) })}>{t(account().disabled ? 'admin-accounts-enable' : 'admin-accounts-disable')}</button>
          <button class="btn" type="button" onClick={() => setConfirm({ name: account().username, action: async () => { await api.write(path(), 'DELETE', undefined, account().etag); setSelected(undefined); go(); } })}>{t('admin-accounts-delete')}</button>
        </fieldset>
      </section>}</Show></Show>
      <Show when={!creating()}><h3>{t('admin-accounts-sessions')}</h3>
      <ul><For each={editing() ? sessions().filter(s => s.adminId === selected()?.id) : sessions()}>{(session) => <li>{session.adminId} — {session.createdAt} {session.current ? t('admin-accounts-current') : ''}<button class="btn" type="button" disabled={busy() || !canManage()} onClick={() => setConfirm({ name: session.adminId, action: () => api.write(`/admin-sessions/${encodeURIComponent(session.id)}`, 'DELETE') })}>{t('admin-accounts-revoke')}</button></li>}</For></ul></Show>
    </Show>
    <ConfirmDialog {...(confirm()?.typed ? { typedConfirmation: confirm()?.name ?? '' } : {})} open={confirm() !== undefined} title={t('admin-accounts-confirm', { name: confirm()?.name ?? '' })} body={t('admin-accounts-confirm-body')} confirmLabel={t('admin-accounts-confirm-action')} onCancel={() => setConfirm(undefined)} onConfirm={async () => { const action = confirm()?.action; if (action) { await action(); setConfirm(undefined); await run(reload); } }} />
  </section>;
}
