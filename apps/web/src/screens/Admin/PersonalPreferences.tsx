import {TextModePreference} from '../Settings/TextModePreference.tsx';
import { AppearanceCustomization } from '../../components/AppearanceCustomization.tsx';
import { AdminTotp } from './AdminTotp.tsx';
import { sectionPath } from './shell/router.ts';
import { For, Show, createSignal, createUniqueId, onMount, onCleanup, type JSX } from 'solid-js';
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
import { horizontalWheel } from '../../components/horizontalWheel.ts';
import * as tabsCss from './shell/sectionTabs.css.ts';
import {toast} from '../../components/toast/toast.ts';

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
  onMount(() => { void loadCatalog('appearance'); void loadCatalog('color-picker'); }); // t112: the accent picker labels
  const me = () => { const grants = shell.can.grants(); return grants.kind === 'listed' ? grants.me : null; };
  const blocked = () => prefs.loading() || !!prefs.error();
  const tabs=['appearance','layout','dashboard','security'] as const;
  type Tab=typeof tabs[number];
  const panelId=createUniqueId();
  const active=():Tab=>{const route=shell.router.route().rest[0];return tabs.includes(route as Tab)?route as Tab:'appearance';};
  const tabButtons:Partial<Record<Tab,HTMLButtonElement>>={};
  async function select(tab:Tab):Promise<void>{
    // The shell focuses its main region after navigation. Restore tab focus only
    // after that completes, and only when the navigation guard allowed it.
    if(await shell.router.navigate(sectionPath('preferences',[tab]))&&active()===tab)tabButtons[tab]?.focus();
  }
  function tabKey(event:KeyboardEvent,tab:Tab):void {
    const rtl=getComputedStyle(event.currentTarget as HTMLElement).direction==='rtl';
    const index=tabs.indexOf(tab);
    const next=event.key==='Home'?0:event.key==='End'?tabs.length-1:event.key===(rtl?'ArrowLeft':'ArrowRight')?(index+1)%tabs.length:event.key===(rtl?'ArrowRight':'ArrowLeft')?(index+tabs.length-1)%tabs.length:null;
    if(next===null)return;event.preventDefault();void select(tabs[next]!);
  }
  return <Show when={shell.router.route().rest[0]==='totp'} fallback={<div class={ux.page} data-testid="admin-personal-preferences">
    <header class={ux.pageHead}><div class={ux.pageTitleBlock}><h2 class={ux.pageTitle}>{t('admin-personal-title')}</h2><p class={ux.pageDescription}>{t('admin-personal-description')}</p></div></header>
    <div class={tabsCss.tabs} ref={horizontalWheel}><div class={tabsCss.strip} role="tablist" aria-label={t('admin-personal-title')}><For each={tabs}>{tab=><button type="button" class={tabsCss.tab} role="tab" id={`${panelId}-${tab}-tab`} aria-controls={`${panelId}-${tab}-panel`} aria-selected={active()===tab} tabindex={active()===tab?0:-1} ref={el=>{tabButtons[tab]=el;}} onClick={()=>select(tab)} onKeyDown={event=>tabKey(event,tab)}>{t(`admin-personal-${tab}`)}</button>}</For></div></div>
    <Show when={prefs.error()}><div role="alert" class={css.section}><p class={css.prose}>{t('admin-personal-reload-warning')}</p><div class={css.actions}><button class={css.button} onClick={() => void prefs.retry()}>{t('admin-personal-retry')}</button><button class={css.button} onClick={() => void prefs.reload()}>{t('admin-personal-reload')}</button></div></div></Show>
    <div role="tabpanel" id={`${panelId}-appearance-panel`} aria-labelledby={`${panelId}-appearance-tab`} hidden={active()!=='appearance'}>
    <AppearanceCustomization owner={() => ({kind: 'admin', username: shell.username()})} />
    <fieldset disabled={blocked()} class={css.section}><legend class={css.heading}>{t('admin-personal-appearance')}</legend><AdminAppearanceFields controller={prefs.appearance} />
      <div class={css.field}><span aria-hidden="true">{t('appearance-custom-accent')}</span><ColorField label={t('appearance-custom-accent')} testId="admin-custom-accent" disabled={blocked()} value={prefs.appearance.prefs().customAccent || null} fallback="#3366cc" emptyLabel={t('color-picker-default')} recentOwner={`admin:${shell.username()}`} contrast={{against: 'bg', use: 'non-text'}} onPreview={value => previewAccent(value)} onChange={value => prefs.appearance.setCustomAccent(value ?? '')} /></div>
      <div class={css.actions}><button class={css.button} disabled={!prefs.appearance.prefs().customAccent} onClick={() => prefs.appearance.setCustomAccent('')}>{t('appearance-custom-accent-reset')}</button><button class={css.button} onClick={() => prefs.appearance.reset()}>{t('admin-personal-reset')}</button></div>
    </fieldset>
    </div>
    <div role="tabpanel" id={`${panelId}-layout-panel`} aria-labelledby={`${panelId}-layout-tab`} hidden={active()!=='layout'}>
    <fieldset disabled={blocked()} class={css.section}><legend class={css.heading}>{t('admin-personal-layout')}</legend>
      <label class={css.field}>{t('admin-personal-density')}<select class={css.select} value={prefs.appearance.prefs().density ?? 'cozy'} onChange={e => prefs.appearance.setDensity(e.currentTarget.value as 'compact' | 'cozy' | 'relaxed')}><For each={['compact', 'cozy', 'relaxed'] as const}>{value => <option value={value}>{t(`admin-personal-density-${value}`)}</option>}</For></select></label>
      <label class={css.field}>{t('admin-personal-font')}<select class={css.select} value={prefs.appearance.prefs().font ?? 'default'} onChange={e => prefs.appearance.setFont(e.currentTarget.value as UiFont)}><For each={['default', 'system', 'serif', 'mono', 'inter', 'roboto', 'open-sans', 'jost'] as const}>{value => <option value={value}>{t(`admin-personal-font-${value}`)}</option>}</For></select></label>
      <TextModePreference enabled={prefs.data()['leetspeak']===true} disabled={blocked()} busy={prefs.saving()} onChange={leetspeak=>prefs.patch({leetspeak})}/>
      <Show when={prefs.i18n}>{ctx => <label class={css.field}>{t('admin-personal-language')}<select class={css.select} value={ctx().locale()} onChange={e => { const locale = e.currentTarget.value; if (isKnownLocale(locale)) { prefs.patch({ locale }); ctx().setLocale(locale); } }}><For each={LOCALES}>{locale => <option value={locale} lang={locale}>{formatLanguageName(locale)}</option>}</For></select></label>}</Show>
    </fieldset>
    </div>
    <div role="tabpanel" id={`${panelId}-dashboard-panel`} aria-labelledby={`${panelId}-dashboard-tab`} hidden={active()!=='dashboard'}>
    <DashboardPreferencesFields />
    </div>
    <div role="tabpanel" id={`${panelId}-security-panel`} aria-labelledby={`${panelId}-security-tab`} hidden={active()!=='security'}>
    <section class={css.section}><h3 class={css.heading}>{t('admin-personal-identity')}</h3><p class={css.prose}>{me()?.displayName || shell.username()}</p><p class={css.meta}>{shell.username()}</p><Show when={me()?.roles.length}><p class={css.meta}>{me()?.roles.join(', ')}</p></Show></section>
    <section class={css.section}><h3 class={css.heading}>{t('admin-personal-security')}</h3><Show when={me()?.accountId} fallback={<p class={css.prose}>{t('admin-personal-managed-auth')}</p>}><PersonalSecurity manageTotp={()=>void shell.router.navigate(sectionPath('preferences',['totp']))} /></Show></section>
    </div>
  </div>}><Show when={me()?.accountId} fallback={<p>{t('admin-personal-managed-auth')}</p>}><AdminTotp back={()=>void shell.router.navigate(sectionPath('preferences',['security']))}/></Show></Show>;
}

function PersonalSecurity(props: { manageTotp: () => void }): JSX.Element {
  const [currentPassword, setCurrentPassword] = createSignal('');
  const [newPassword, setNewPassword] = createSignal('');
  const [passwordChanged, setPasswordChanged] = createSignal(false);
  const [busy, setBusy] = createSignal(false);
  const saveToast=`admin-personal-security-${createUniqueId()}`;
  const shell=useShell();let alive=true;
  onCleanup(()=>{alive=false;toast.dismiss(saveToast);});
  async function run(action: () => Promise<void>): Promise<void> { const owner=shell.username();const current=()=>alive&&shell.username()===owner;setBusy(true); toast.info(t('admin-personal-saving'),{id:saveToast,busy:true,persistent:true,duration:Infinity}); try { await action(); if(current())toast.success(t('admin-personal-security-saved'),{id:saveToast}); } catch (cause) { if(current())toast.error(cause instanceof Error ? cause.message : String(cause),{id:saveToast}); } finally { if(alive)setBusy(false); } }
  return <div class={css.stack}>
    <AdminPasskeys />
    <section class={css.section}><h3 class={css.heading}>{t('admin-personal-totp')}</h3><p class={css.prose}>{t('admin-totp-description')}</p><div class={css.actions}><button class={css.button} onClick={props.manageTotp}>{t('admin-totp-manage')}</button></div></section>
    <form class={css.stack} onSubmit={e => { e.preventDefault(); void run(async () => { await adminRequest('/me/password', { method: 'PUT', body: { currentPassword: currentPassword(), newPassword: newPassword() } }); setCurrentPassword(''); setNewPassword(''); setPasswordChanged(true); }); }}><h4 class={css.subheading}>{t('admin-personal-password')}</h4><label class={css.field}>{t('admin-personal-password-current')}<input class={css.input} type="password" autocomplete="current-password" required disabled={busy()} value={currentPassword()} onInput={e => setCurrentPassword(e.currentTarget.value)} /></label><label class={css.field}>{t('admin-personal-password-new')}<input class={css.input} type="password" autocomplete="new-password" required disabled={busy()} value={newPassword()} onInput={e => setNewPassword(e.currentTarget.value)} /></label><div class={css.actions}><button class={css.button} type="submit" disabled={busy()}>{t('admin-personal-password-save')}</button></div></form>
    <Show when={passwordChanged()}><p role="alert" class={css.prose}>{t('admin-personal-password-signin')}</p></Show>
  </div>;
}
