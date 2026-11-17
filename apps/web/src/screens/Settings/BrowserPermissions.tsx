import {browserStorageStatus,requestBrowserPersistence,storageSize} from '../../offline/storagePermission.ts';
import {settings} from '../../api/settings.ts';
import {NotificationAccountPicker} from './NotificationAccountPicker.tsx';
import {permissionBody} from './notificationSettings.css.ts';
import {PushRegistrationError} from '../../platform/pushRegistrationError.ts';
import {toast} from '../../components/toast/toast.ts';
import { storageAuthorityHeaders } from '../../api/storageAuthority.ts';
import { createSignal, onMount, onCleanup, Show, type JSX } from 'solid-js';
import { apiFetch } from '../../api/http.ts';
import { withBase,shellBase } from '../../api/basePath.ts';
import { preferenceAccount, preferenceAccountIsCurrent, observePreferenceAccount } from '../../api/accountPreferenceLifecycle.ts';
import { getPlatform } from '../../platform/index.ts';
import { t } from '../../i18n';
import { SettingsCard } from './layout/SettingsCard.tsx';
import * as css from './styles.css.ts';

export function BrowserPermissions(props: { enabled: boolean; pushAllowed: boolean; persistentStorageAllowed: boolean; disabled?: boolean }): JSX.Element {
  const [busy, setBusy] = createSignal(false);
  let disposed=false;onCleanup(()=>{disposed=true;});
  const [persistent, setPersistent] = createSignal(false), [subscribed, setSubscribed] = createSignal(false);
  const [browserSubscription,setBrowserSubscription]=createSignal(false),[bindingConflict,setBindingConflict]=createSignal(false);
  const [usage, setUsage] = createSignal<number | undefined>(),[quota,setQuota]=createSignal<number>();
  const storageSupported = typeof navigator !== 'undefined' && typeof navigator.storage?.persist === 'function';
  const pushSupported = typeof navigator !== 'undefined' && 'serviceWorker' in navigator && typeof PushManager !== 'undefined';
  async function currentSubscription(): Promise<PushSubscription | null> {
    if(!pushSupported)return null;
    const registration=await navigator.serviceWorker.getRegistration();
    if(registration&&typeof registration.scope==='string'&&registration.scope!==new URL(shellBase(),location.href).href)return null;
    return registration?.pushManager.getSubscription()??null;
  }
  onMount(() => {
    if (storageSupported) {
      void navigator.storage.persisted().then(setPersistent).catch(() => undefined);
      void browserStorageStatus().then(value => {setUsage(value.usage);setQuota(value.quota);}).catch(() => undefined);
    }
    onCleanup(observePreferenceAccount(()=>{void refreshPushRegistration();}));
  });
  async function refreshPushRegistration():Promise<void>{
    const account=preferenceAccount();setSubscribed(false);setBrowserSubscription(false);setBindingConflict(false);
    if(!account?.storageAuthority)return;
    try{
      const subscription=await currentSubscription();if(!subscription||!preferenceAccountIsCurrent(account))return;setBrowserSubscription(true);
      const digest=await crypto.subtle.digest('SHA-256',new TextEncoder().encode(subscription.endpoint));
      const hash=Array.from(new Uint8Array(digest),byte=>byte.toString(16).padStart(2,'0')).join('');
      const response=await apiFetch(withBase(`/api/push/permission?endpointHash=${hash}`),{headers:storageAuthorityHeaders(account.storageAuthority)});
      if(!response.ok)return;const status=await response.json() as {registered?:boolean;accountRegistered?:boolean};
      if(preferenceAccountIsCurrent(account))setSubscribed((status.accountRegistered??status.registered)===true);
    }catch{if(preferenceAccountIsCurrent(account))setSubscribed(false);}
  }
  async function run(action: () => Promise<void>): Promise<void> {
    if(busy())return;const owner=preferenceAccount();setBusy(true);
    try { await action(); } catch(error) { if(!disposed&&(owner?preferenceAccountIsCurrent(owner):!preferenceAccount())){if(error instanceof PushRegistrationError&&error.reason==='bound')setBindingConflict(true);toast.error(t(error instanceof PushRegistrationError?error.reason==='bound'?'settings-permissions-push-other-registration':error.reason==='permission'?'settings-notif-permission-denied':error.reason==='worker'?'settings-permissions-push-worker-error':error.reason==='server'?'settings-permissions-push-server-error':'settings-permissions-push-browser-error':'settings-permissions-error'));} } finally { if(!disposed)setBusy(false); }
  }
  async function registerPush(): Promise<void> {
    const account = preferenceAccount();
    if (!account || !props.enabled || !props.pushAllowed || typeof Notification === 'undefined') throw new PushRegistrationError('permission');
    const permission=Notification.permission==='default'?await Notification.requestPermission():Notification.permission;
    window.dispatchEvent(new CustomEvent('mw:notification-permission-changed',{detail:permission}));
    if(!preferenceAccountIsCurrent(account))return;
    if(permission!=='granted')throw new PushRegistrationError('permission');
    const subscription = await getPlatform().pushSubscribe();
    if(!preferenceAccountIsCurrent(account))return;
    if (!subscription) throw new PushRegistrationError('browser');
    setBrowserSubscription(true);
    const response = await apiFetch(withBase('/api/push/subscribe'), { method: 'POST', headers: { 'content-type': 'application/json', ...storageAuthorityHeaders(account.storageAuthority) }, body: JSON.stringify(subscription) });
    if (!response.ok) throw new PushRegistrationError(response.status===409?'bound':'server');
    if (preferenceAccountIsCurrent(account)) {setSubscribed(true);setBrowserSubscription(true);}
  }
  async function unregisterPush(): Promise<void> {
    const account = preferenceAccount(); if (!account) throw new Error('Sign in required');
    const subscription = await currentSubscription();
    if (!preferenceAccountIsCurrent(account)) throw new Error('Account changed');
    if (subscription) {
      const response = await apiFetch(withBase('/api/push/unsubscribe'), { method: 'POST', headers: { 'content-type': 'application/json', ...storageAuthorityHeaders(account.storageAuthority) }, body: JSON.stringify({ endpoint: subscription.endpoint }) });
      if (!response.ok) throw new Error('Removal refused');
      // Keep the browser endpoint alive for other linked mailbox registrations.
    }
    if (preferenceAccountIsCurrent(account)) {setSubscribed(false);setBrowserSubscription(subscription!==null);setBindingConflict(false);}
  }
  // Two groups (background notifications, browser storage) so the Settings
  // card rules each off like its other subgroups.
  return <>
    <SettingsCard id="notif-card-background" title={t('settings-permissions-background-title')} description={t('settings-permissions-background-help')}>
    <NotificationAccountPicker disabled={props.disabled===true}/><fieldset class={`${css.bareFieldset} ${permissionBody}`} disabled={props.disabled}>
    <Show when={pushSupported} fallback={<p>{t('settings-permissions-push-unsupported')}</p>}>
      <p role="status">{t(subscribed() ? 'settings-permissions-push-registered' : browserSubscription()?'settings-permissions-push-unlinked':'settings-permissions-push-unregistered')}</p>
      <Show when={bindingConflict()}><p>{t('settings-permissions-push-other-registration')}</p></Show>
      <div class={css.actions}>
        <button type="button" class={css.button} disabled={busy() || subscribed() || !props.enabled || !props.pushAllowed || bindingConflict()} onClick={() => void run(registerPush)}>{t('settings-permissions-push-register')}</button>
        <button type="button" class={css.ghost} disabled={busy() || !subscribed()} onClick={() => void run(unregisterPush)}>{t('settings-permissions-push-remove')}</button>
      </div>
      <Show when={!props.pushAllowed}><p>{t('settings-permissions-push-policy')}</p></Show>
    </Show>
    </fieldset>
    </SettingsCard>
    <SettingsCard id="notif-card-storage" title={t('settings-permissions-storage-title')} description={t('settings-permissions-storage-help')}>
    <fieldset class={`${css.bareFieldset} ${permissionBody}`} disabled={props.disabled}>
    <Show when={storageSupported} fallback={<p>{t('settings-permissions-storage-unsupported')}</p>}>
      <p role="status">{t(persistent() ? 'settings-permissions-storage-persistent' : 'settings-permissions-storage-evictable')}</p>
      <p>{t('settings-storage-quota',{used:storageSize(usage()),quota:storageSize(quota())})}</p><p>{t('settings-storage-quota-help')}</p><Show when={quota()!==undefined&&Number(settings()['offline.budgetMb']?.value??250)*1048576>quota()!}><p role="status">{t('settings-storage-budget-warning')}</p></Show>
      <div class={css.actions}>
        <button type="button" class={css.button} disabled={busy() || persistent() || !props.persistentStorageAllowed} onClick={() => void run(async () => { if (props.persistentStorageAllowed) setPersistent(await requestBrowserPersistence());const status=await browserStorageStatus();setUsage(status.usage);setQuota(status.quota); })}>{t('settings-permissions-storage-request')}</button>
      </div>
      <Show when={!props.persistentStorageAllowed}><p>{t('settings-permissions-storage-policy')}</p></Show>
    </Show>
    </fieldset>
    </SettingsCard>
  </>;
}
