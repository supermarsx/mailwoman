import { For, Show, createSignal, onMount, type JSX } from 'solid-js';
import { loadCatalog,t } from '../../i18n';
import { adminRequest } from './shell/adminApi.ts';
import { registerPasskey,passkeySupported } from '../Settings/webauthn.ts';
import * as css from '../Settings/styles.css.ts';

interface Key {credentialId:string;label:string;createdAt:string;backupEligible:boolean;backedUp:boolean}
export function AdminPasskeys():JSX.Element {
  const [keys,setKeys]=createSignal<Key[]>([]);
  const [codes,setCodes]=createSignal<string[]>([]);
  const [label,setLabel]=createSignal('');
  const [remaining,setRemaining]=createSignal(0);
  const [busy,setBusy]=createSignal(false);
  const [error,setError]=createSignal('');
  async function refresh():Promise<void>{const response=await adminRequest<{passkeys:Key[];recoveryRemaining:number}>('/me/passkeys');setKeys(response.data.passkeys);setRemaining(response.data.recoveryRemaining);}
  async function run(action:()=>Promise<void>):Promise<void>{setBusy(true);setError('');try{await action();await refresh();}catch(cause){setError(cause instanceof Error?cause.message:String(cause));}finally{setBusy(false);}}
  onMount(()=>{void loadCatalog('admin-passkeys');void run(refresh);});
  async function enroll():Promise<void>{
    const challenge=await adminRequest<Parameters<typeof registerPasskey>[0]>('/me/passkeys/begin',{method:'POST',body:{}});
    const credential=await registerPasskey(challenge.data);
    const saved=await adminRequest<{recoveryCodes:string[]}>('/me/passkeys/finish',{method:'POST',body:{...credential,label:label()}});
    setCodes(saved.data.recoveryCodes);setLabel('');
  }
  return <section class={css.section} aria-label={t('admin-passkeys-title')}>
    <h3 class={css.heading}>{t('admin-passkeys-title')}</h3><p class={css.prose}>{t('admin-passkeys-description')}</p>
    <Show when={error()}><p role="alert">{error()}</p></Show>
    <For each={keys()}>{key=><div class={css.stack}><strong>{key.label||t('admin-passkeys-unnamed')}</strong><p class={css.meta}>{t(key.backupEligible?'admin-passkeys-synced':'admin-passkeys-device')}</p><div class={css.actions}><button class={css.button} disabled={busy()} onClick={()=>void run(async()=>{await adminRequest('/me/passkeys/remove',{method:'POST',body:{credentialId:key.credentialId}});})}>{t('admin-passkeys-remove')}</button></div></div>}</For>
    <Show when={passkeySupported()} fallback={<p>{t('admin-passkeys-unavailable')}</p>}><label class={css.field}>{t('admin-passkeys-label')}<input class={css.input} maxlength={128} value={label()} onInput={e=>setLabel(e.currentTarget.value)} disabled={busy()}/></label><div class={css.actions}><button class={css.button} disabled={busy()} onClick={()=>void run(enroll)}>{t('admin-passkeys-add')}</button></div></Show>
    <Show when={keys().length>0}><p class={css.meta}>{t('admin-passkeys-recovery-count',{count:remaining()})}</p><div class={css.actions}><button class={css.button} disabled={busy()} onClick={()=>void run(async()=>{const r=await adminRequest<{recoveryCodes:string[]}>('/me/recovery',{method:'POST',body:{}});setCodes(r.data.recoveryCodes);})}>{t('admin-passkeys-recovery-rotate')}</button></div></Show>
    <Show when={codes().length>0}><div role="status"><p>{t('admin-passkeys-recovery-save')}</p><ul><For each={codes()}>{code=><li><code>{code}</code></li>}</For></ul><button class={css.button} onClick={()=>setCodes([])}>{t('admin-passkeys-recovery-dismiss')}</button></div></Show>
  </section>;
}
