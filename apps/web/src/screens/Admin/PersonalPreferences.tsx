import { AppearanceCustomization } from '../../components/AppearanceCustomization.tsx';
import { AdminTotp } from './AdminTotp.tsx';
import { sectionPath } from './shell/router.ts';
import { For, Show, createSignal, onMount, type JSX } from 'solid-js';
import { LOCALES, formatLanguageName, isKnownLocale, loadCatalog, t } from '../../i18n';
import { AdminAppearanceFields } from './shell/AdminAppearance.tsx';
import { usePersonalPreferences } from './shell/personalPreferences.tsx';
import { useShell } from './shell/context.ts';
import { adminRequest } from './shell/adminApi.ts';
import type { UiFont } from '../../theme/appearance.ts';
import * as css from '../Settings/styles.css.ts';
import * as ux from './shell/ux.css.ts';
import { AdminPasskeys } from './AdminPasskeys.tsx';
import { DashboardPreferencesFields } from './Dashboard/PreferencesFields.tsx';
import { ColorField } from '../../components/color/ColorPicker.tsx';
import { applyCustomAccent } from '../../theme/customAccent.ts';
import { isThemeName } from '../../theme/registry.ts';

/** Live accent while the picker is dragged; the store re-applies the saved one on commit. */
function previewAccent(value: string | null): void {
  const root = document.documentElement;
  const theme = root.getAttribute('data-theme');
  if (isThemeName(theme)) applyCustomAccent(root, value ?? '', theme);
}

export function PersonalPreferences(): JSX.Element {
  const prefs = usePersonalPreferences();
  const shell = useShell();
  onMount(() => void loadCatalog('admin-personal'));
  const me = () => { const grants = shell.can.grants(); return grants.kind === 'listed' ? grants.me : null; };
  const blocked = () => prefs.loading() || !!prefs.error();
  return <Show when={shell.router.route().rest[0]==='totp'} fallback={<div class={ux.page} data-testid="admin-personal-preferences">
    <header class={ux.pageHead}><div class={ux.pageTitleBlock}><h2 class={ux.pageTitle}>{t('admin-personal-title')}</h2><p class={ux.pageDescription}>{t('admin-personal-description')}</p></div></header>
    <section class={css.section}><h3 class={css.heading}>{t('admin-personal-identity')}</h3><p class={css.prose}>{me()?.displayName || shell.username()}</p><p class={css.meta}>{shell.username()}</p><Show when={me()?.roles.length}><p class={css.meta}>{me()?.roles.join(', ')}</p></Show></section>
    <p role="status" aria-live="polite" class={css.meta}>{prefs.loading() ? t('admin-personal-loading') : prefs.saving() ? t('admin-personal-saving') : prefs.error() ? t('admin-personal-unsaved') : t('admin-personal-saved')}</p>
    <Show when={prefs.error()}><div role="alert" class={css.section}><p>{prefs.error()}</p><p class={css.prose}>{t('admin-personal-reload-warning')}</p><div class={css.actions}><button class={css.button} onClick={() => void prefs.retry()}>{t('admin-personal-retry')}</button><button class={css.button} onClick={() => void prefs.reload()}>{t('admin-personal-reload')}</button></div></div></Show>
    <AppearanceCustomization owner={() => ({kind: 'admin', username: shell.username()})} />
    <fieldset disabled={blocked()} class={css.section}><legend class={css.heading}>{t('admin-personal-appearance')}</legend><AdminAppearanceFields controller={prefs.appearance} />
      <div class={css.field}><span aria-hidden="true">{t('appearance-custom-accent')}</span><ColorField label={t('appearance-custom-accent')} testId="admin-custom-accent" disabled={blocked()} value={prefs.appearance.prefs().customAccent || null} fallback="#3366cc" emptyLabel={t('color-picker-default')} recentOwner={`admin:${shell.username()}`} contrast={{against: 'bg', use: 'non-text'}} onPreview={value => previewAccent(value)} onChange={value => prefs.appearance.setCustomAccent(value ?? '')} /></div>
      <div class={css.actions}><button class={css.button} disabled={!prefs.appearance.prefs().customAccent} onClick={() => prefs.appearance.setCustomAccent('')}>{t('appearance-custom-accent-reset')}</button><button class={css.button} onClick={() => prefs.appearance.reset()}>{t('admin-personal-reset')}</button></div>
    </fieldset>
    <fieldset disabled={blocked()} class={css.section}><legend class={css.heading}>{t('admin-personal-layout')}</legend>
      <label class={css.field}>{t('admin-personal-density')}<select class={css.select} value={prefs.appearance.prefs().density ?? 'cozy'} onChange={e => prefs.appearance.setDensity(e.currentTarget.value as 'compact' | 'cozy' | 'relaxed')}><For each={['compact', 'cozy', 'relaxed'] as const}>{value => <option value={value}>{t(`admin-personal-density-${value}`)}</option>}</For></select></label>
      <label class={css.field}>{t('admin-personal-font')}<select class={css.select} value={prefs.appearance.prefs().font ?? 'default'} onChange={e => prefs.appearance.setFont(e.currentTarget.value as UiFont)}><For each={['default', 'system', 'serif', 'mono', 'inter', 'roboto', 'open-sans', 'jost'] as const}>{value => <option value={value}>{t(`admin-personal-font-${value}`)}</option>}</For></select></label>
      <Show when={prefs.i18n}>{ctx => <label class={css.field}>{t('admin-personal-language')}<select class={css.select} value={ctx().locale()} onChange={e => { const locale = e.currentTarget.value; if (isKnownLocale(locale)) { prefs.patch({ locale }); ctx().setLocale(locale); } }}><For each={LOCALES}>{locale => <option value={locale} lang={locale}>{formatLanguageName(locale)}</option>}</For></select></label>}</Show>
    </fieldset>
    <DashboardPreferencesFields />
    <section class={css.section}><h3 class={css.heading}>{t('admin-personal-security')}</h3><Show when={me()?.accountId} fallback={<p class={css.prose}>{t('admin-personal-managed-auth')}</p>}><PersonalSecurity manageTotp={()=>void shell.router.navigate(sectionPath('preferences',['totp']))} /></Show></section>
  </div>}><Show when={me()?.accountId} fallback={<p>{t('admin-personal-managed-auth')}</p>}><AdminTotp back={()=>void shell.router.navigate(sectionPath('preferences'))}/></Show></Show>;
}

function PersonalSecurity(props: { manageTotp: () => void }): JSX.Element {
  const [currentPassword, setCurrentPassword] = createSignal('');
  const [newPassword, setNewPassword] = createSignal('');
  const [passwordChanged, setPasswordChanged] = createSignal(false);
  const [busy, setBusy] = createSignal(false);
  const [message, setMessage] = createSignal<string | null>(null);
  async function run(action: () => Promise<void>): Promise<void> { setBusy(true); setMessage(null); try { await action(); setMessage(t('admin-personal-security-saved')); } catch (cause) { setMessage(cause instanceof Error ? cause.message : String(cause)); } finally { setBusy(false); } }
  return <div class={css.stack}>
    <AdminPasskeys />
    <section class={css.section}><h3 class={css.heading}>{t('admin-personal-totp')}</h3><p class={css.prose}>{t('admin-totp-description')}</p><div class={css.actions}><button class={css.button} onClick={props.manageTotp}>{t('admin-totp-manage')}</button></div></section>
    <form class={css.stack} onSubmit={e => { e.preventDefault(); void run(async () => { await adminRequest('/me/password', { method: 'PUT', body: { currentPassword: currentPassword(), newPassword: newPassword() } }); setCurrentPassword(''); setNewPassword(''); setPasswordChanged(true); }); }}><h4 class={css.subheading}>{t('admin-personal-password')}</h4><label class={css.field}>{t('admin-personal-password-current')}<input class={css.input} type="password" autocomplete="current-password" required disabled={busy()} value={currentPassword()} onInput={e => setCurrentPassword(e.currentTarget.value)} /></label><label class={css.field}>{t('admin-personal-password-new')}<input class={css.input} type="password" autocomplete="new-password" required disabled={busy()} value={newPassword()} onInput={e => setNewPassword(e.currentTarget.value)} /></label><div class={css.actions}><button class={css.button} type="submit" disabled={busy()}>{t('admin-personal-password-save')}</button></div></form>
    <Show when={passwordChanged()}><p role="alert" class={css.prose}>{t('admin-personal-password-signin')}</p></Show>
    <Show when={message()}><p role="status" class={css.prose}>{message()}</p></Show>
  </div>;
}
