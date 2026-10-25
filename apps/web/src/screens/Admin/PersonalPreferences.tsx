import { For, Show, createSignal, onMount, type JSX } from 'solid-js';
import { LOCALES, formatLanguageName, isKnownLocale, loadCatalog, t } from '../../i18n';
import { AdminAppearanceFields } from './shell/AdminAppearance.tsx';
import { usePersonalPreferences } from './shell/personalPreferences.tsx';
import { useShell } from './shell/context.ts';
import { adminRequest } from './shell/adminApi.ts';
import type { UiFont } from '../../theme/appearance.ts';
import * as css from '../Settings/styles.css.ts';
import * as personal from './personal.css.ts';
import * as ux from './shell/ux.css.ts';

export function PersonalPreferences(): JSX.Element {
  const prefs = usePersonalPreferences();
  const shell = useShell();
  onMount(() => void loadCatalog('admin-personal'));
  const me = () => { const grants = shell.can.grants(); return grants.kind === 'listed' ? grants.me : null; };
  const blocked = () => prefs.loading() || !!prefs.error();
  return <div class={ux.page} data-testid="admin-personal-preferences">
    <header class={ux.pageHead}><div class={ux.pageTitleBlock}><h2 class={ux.pageTitle}>{t('admin-personal-title')}</h2><p class={ux.pageDescription}>{t('admin-personal-description')}</p></div></header>
    <section class={css.section}><h3 class={css.heading}>{t('admin-personal-identity')}</h3><p class={css.prose}>{me()?.displayName || shell.username()}</p><p class={css.meta}>{shell.username()}</p><Show when={me()?.roles.length}><p class={css.meta}>{me()?.roles.join(', ')}</p></Show></section>
    <p role="status" aria-live="polite" class={css.meta}>{prefs.loading() ? t('admin-personal-loading') : prefs.saving() ? t('admin-personal-saving') : prefs.error() ? t('admin-personal-unsaved') : t('admin-personal-saved')}</p>
    <Show when={prefs.error()}><div role="alert" class={css.section}><p>{prefs.error()}</p><p class={css.prose}>{t('admin-personal-reload-warning')}</p><div class={css.actions}><button class={css.button} onClick={() => void prefs.retry()}>{t('admin-personal-retry')}</button><button class={css.button} onClick={() => void prefs.reload()}>{t('admin-personal-reload')}</button></div></div></Show>
    <fieldset disabled={blocked()} class={css.section}><legend class={css.heading}>{t('admin-personal-appearance')}</legend><AdminAppearanceFields controller={prefs.appearance} />
      <div class={css.actions}><button class={css.button} onClick={() => prefs.appearance.reset()}>{t('admin-personal-reset')}</button></div>
    </fieldset>
    <fieldset disabled={blocked()} class={css.section}><legend class={css.heading}>{t('admin-personal-layout')}</legend>
      <label class={css.field}>{t('admin-personal-density')}<select class={css.select} value={prefs.appearance.prefs().density ?? 'cozy'} onChange={e => prefs.appearance.setDensity(e.currentTarget.value as 'compact' | 'cozy' | 'relaxed')}><For each={['compact', 'cozy', 'relaxed'] as const}>{value => <option value={value}>{t(`admin-personal-density-${value}`)}</option>}</For></select></label>
      <label class={css.field}>{t('admin-personal-font')}<select class={css.select} value={prefs.appearance.prefs().font ?? 'default'} onChange={e => prefs.appearance.setFont(e.currentTarget.value as UiFont)}><For each={['default', 'system', 'serif', 'mono'] as const}>{value => <option value={value}>{t(`admin-personal-font-${value}`)}</option>}</For></select></label>
      <Show when={prefs.i18n}>{ctx => <label class={css.field}>{t('admin-personal-language')}<select class={css.select} value={ctx().locale()} onChange={e => { const locale = e.currentTarget.value; if (isKnownLocale(locale)) { prefs.patch({ locale }); ctx().setLocale(locale); } }}><For each={LOCALES}>{locale => <option value={locale} lang={locale}>{formatLanguageName(locale)}</option>}</For></select></label>}</Show>
    </fieldset>
    <section class={css.section}><h3 class={css.heading}>{t('admin-personal-security')}</h3><Show when={me()?.accountId} fallback={<p class={css.prose}>{t('admin-personal-managed-auth')}</p>}><PersonalSecurity enrolled={me()?.totpEnrolled ?? false} /></Show></section>
  </div>;
}

function PersonalSecurity(props: { enrolled: boolean }): JSX.Element {
  const [enrolled, setEnrolled] = createSignal(props.enrolled);
  const [secret, setSecret] = createSignal<string | null>(null);
  const [code, setCode] = createSignal('');
  const [currentPassword, setCurrentPassword] = createSignal('');
  const [newPassword, setNewPassword] = createSignal('');
  const [passwordChanged, setPasswordChanged] = createSignal(false);
  const [busy, setBusy] = createSignal(false);
  const [message, setMessage] = createSignal<string | null>(null);
  async function run(action: () => Promise<void>): Promise<void> { setBusy(true); setMessage(null); try { await action(); setMessage(t('admin-personal-security-saved')); } catch (cause) { setMessage(cause instanceof Error ? cause.message : String(cause)); } finally { setBusy(false); } }
  return <div class={css.stack}>
    <fieldset disabled={busy()} class={[css.stack, personal.formGroup].join(' ')}><legend class={css.subheading}>{t('admin-personal-totp')}</legend><p class={css.prose}>{t(enrolled() ? 'admin-personal-totp-enabled' : 'admin-personal-totp-disabled')}</p>
      <Show when={!enrolled() && !secret()}><div class={css.actions}><button class={css.button} onClick={() => void run(async () => { const response = await adminRequest<{ secret: string }>('/me/totp', { method: 'POST' }); setSecret(response.data.secret); })}>{t('admin-personal-totp-start')}</button></div></Show>
      <Show when={secret()}><p class={css.prose}>{t('admin-personal-totp-secret')}</p><code class={personal.setupKey}>{secret()}</code></Show>
      <Show when={secret() || enrolled()}><label class={css.field}>{t('admin-personal-totp-code')}<input class={css.input} autocomplete="one-time-code" inputmode="numeric" value={code()} onInput={e => setCode(e.currentTarget.value)} /></label><div class={css.actions}><button class={css.button} disabled={!code().trim()} onClick={() => void run(async () => { if (enrolled()) { await adminRequest('/me/totp', { method: 'DELETE', body: { code: code() } }); setEnrolled(false); } else { await adminRequest('/me/totp/confirm', { method: 'POST', body: { code: code() } }); setEnrolled(true); } setSecret(null); setCode(''); })}>{t(enrolled() ? 'admin-personal-totp-remove' : 'admin-personal-totp-confirm')}</button></div></Show>
    </fieldset>
    <form class={css.stack} onSubmit={e => { e.preventDefault(); void run(async () => { await adminRequest('/me/password', { method: 'PUT', body: { currentPassword: currentPassword(), newPassword: newPassword() } }); setCurrentPassword(''); setNewPassword(''); setPasswordChanged(true); }); }}><h4 class={css.subheading}>{t('admin-personal-password')}</h4><label class={css.field}>{t('admin-personal-password-current')}<input class={css.input} type="password" autocomplete="current-password" required disabled={busy()} value={currentPassword()} onInput={e => setCurrentPassword(e.currentTarget.value)} /></label><label class={css.field}>{t('admin-personal-password-new')}<input class={css.input} type="password" autocomplete="new-password" required disabled={busy()} value={newPassword()} onInput={e => setNewPassword(e.currentTarget.value)} /></label><div class={css.actions}><button class={css.button} type="submit" disabled={busy()}>{t('admin-personal-password-save')}</button></div></form>
    <Show when={passwordChanged()}><p role="alert" class={css.prose}>{t('admin-personal-password-signin')}</p></Show>
    <Show when={message()}><p role="status" class={css.prose}>{message()}</p></Show>
  </div>;
}
