import { storageAuthorityHeaders } from '../../api/storageAuthority.ts';
import { createSignal, onMount, onCleanup, Show, type JSX } from 'solid-js';
import { apiFetch } from '../../api/http.ts';
import { withBase } from '../../api/basePath.ts';
import { preferenceAccount, preferenceAccountIsCurrent, observePreferenceAccount } from '../../api/accountPreferenceLifecycle.ts';
import { getPlatform } from '../../platform/index.ts';
import { t } from '../../i18n';
import * as css from './styles.css.ts';

export function BrowserPermissions(props: { enabled: boolean; pushAllowed: boolean; persistentStorageAllowed: boolean }): JSX.Element {
  const [busy, setBusy] = createSignal(false), [error, setError] = createSignal('');
  const [persistent, setPersistent] = createSignal(false), [subscribed, setSubscribed] = createSignal(false);
  const [browserSubscription,setBrowserSubscription]=createSignal(false);
  const [usage, setUsage] = createSignal<number | undefined>();
  const storageSupported = typeof navigator !== 'undefined' && typeof navigator.storage?.persist === 'function';
  const pushSupported = typeof navigator !== 'undefined' && 'serviceWorker' in navigator && typeof PushManager !== 'undefined';
  async function currentSubscription(): Promise<PushSubscription | null> {
    return pushSupported ? (await navigator.serviceWorker.getRegistration())?.pushManager.getSubscription() ?? null : null;
  }
  onMount(() => {
    if (storageSupported) {
      void navigator.storage.persisted().then(setPersistent).catch(() => undefined);
      void navigator.storage.estimate().then(value => setUsage(value.usage)).catch(() => undefined);
    }
    onCleanup(observePreferenceAccount(()=>{void refreshPushRegistration();}));
  });
  async function refreshPushRegistration():Promise<void>{
    const account=preferenceAccount();setSubscribed(false);setBrowserSubscription(false);
    if(!account?.storageAuthority)return;
    try{
      const subscription=await currentSubscription();if(!subscription||!preferenceAccountIsCurrent(account))return;setBrowserSubscription(true);
      const digest=await crypto.subtle.digest('SHA-256',new TextEncoder().encode(subscription.endpoint));
      const hash=Array.from(new Uint8Array(digest),byte=>byte.toString(16).padStart(2,'0')).join('');
      const response=await apiFetch(withBase(`/api/push/permission?endpointHash=${hash}`),{headers:storageAuthorityHeaders(account.storageAuthority)});
      if(!response.ok)return;const status=await response.json() as {registered?:boolean};
      if(preferenceAccountIsCurrent(account))setSubscribed(status.registered===true);
    }catch{if(preferenceAccountIsCurrent(account))setSubscribed(false);}
  }
  async function run(action: () => Promise<void>): Promise<void> {
    setBusy(true); setError('');
    try { await action(); } catch { setError(t('settings-permissions-error')); } finally { setBusy(false); }
  }
  async function registerPush(): Promise<void> {
    const account = preferenceAccount();
    if (!account || !props.enabled || !props.pushAllowed || typeof Notification === 'undefined' || Notification.permission !== 'granted') throw new Error('Permission required');
    const subscription = await getPlatform().pushSubscribe();
    if (!subscription || !preferenceAccountIsCurrent(account)) throw new Error('Account changed');
    const response = await apiFetch(withBase('/api/push/subscribe'), { method: 'POST', headers: { 'content-type': 'application/json', ...storageAuthorityHeaders(account.storageAuthority) }, body: JSON.stringify(subscription) });
    if (!response.ok) throw new Error('Registration refused');
    if (preferenceAccountIsCurrent(account)) {setSubscribed(true);setBrowserSubscription(true);}
  }
  async function unregisterPush(): Promise<void> {
    const account = preferenceAccount(); if (!account) throw new Error('Sign in required');
    const subscription = await currentSubscription();
    if (!preferenceAccountIsCurrent(account)) throw new Error('Account changed');
    if (subscription) {
      const response = await apiFetch(withBase('/api/push/unsubscribe'), { method: 'POST', headers: { 'content-type': 'application/json', ...storageAuthorityHeaders(account.storageAuthority) }, body: JSON.stringify({ endpoint: subscription.endpoint }) });
      if (!response.ok) throw new Error('Removal refused');
      await subscription.unsubscribe();
    }
    if (preferenceAccountIsCurrent(account)) {setSubscribed(false);setBrowserSubscription(false);}
  }
  return <div class={css.field}>
    <h3 class={css.subheading}>{t('settings-permissions-background-title')}</h3>
    <p class={css.prose}>{t('settings-permissions-background-help')}</p>
    <Show when={pushSupported} fallback={<p>{t('settings-permissions-push-unsupported')}</p>}>
      <p>{t(subscribed() ? 'settings-permissions-push-registered' : 'settings-permissions-push-unregistered')}</p>
      <Show when={browserSubscription()&&!subscribed()}><p>{t('settings-permissions-push-other-registration')}</p></Show>
      <div class={css.actions}>
        <button type="button" class={css.button} disabled={busy() || !props.enabled || !props.pushAllowed || (browserSubscription()&&!subscribed())} onClick={() => void run(registerPush)}>{t('settings-permissions-push-register')}</button>
        <button type="button" class={css.ghost} disabled={busy() || !browserSubscription()} onClick={() => void run(unregisterPush)}>{t('settings-permissions-push-remove')}</button>
      </div>
      <Show when={!props.pushAllowed}><p>{t('settings-permissions-push-policy')}</p></Show>
    </Show>
    <h3 class={css.subheading}>{t('settings-permissions-storage-title')}</h3>
    <p class={css.prose}>{t('settings-permissions-storage-help')}</p>
    <Show when={storageSupported} fallback={<p>{t('settings-permissions-storage-unsupported')}</p>}>
      <p role="status">{t(persistent() ? 'settings-permissions-storage-persistent' : 'settings-permissions-storage-evictable')}</p>
      <Show when={usage() !== undefined}><p>{t('settings-permissions-storage-usage', { mib: ((usage() ?? 0) / 1048576).toFixed(1) })}</p></Show>
      <button type="button" class={css.button} disabled={busy() || persistent() || !props.persistentStorageAllowed} onClick={() => void run(async () => { if (props.persistentStorageAllowed) setPersistent(await navigator.storage.persist()); })}>{t('settings-permissions-storage-request')}</button>
      <Show when={!props.persistentStorageAllowed}><p>{t('settings-permissions-storage-policy')}</p></Show>
    </Show>
    <Show when={error()}><p role="alert" class={css.error}>{error()}</p></Show>
  </div>;
}
