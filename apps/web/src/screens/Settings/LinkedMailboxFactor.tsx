import {createSignal,onCleanup,Show,type JSX} from 'solid-js';
import type {LinkedMailboxChallenge,LinkMailboxProof} from '../../api/linkedAccounts.ts';
import {preferenceAccount,preferenceAccountIsCurrent} from '../../api/accountPreferenceLifecycle.ts';
import {assertPasskey,passkeySupported} from './webauthn.ts';
import {t} from '../../i18n/index.ts';
import * as css from './styles.css.ts';

/** Child MFA never invokes the ordinary login endpoint or replaces its cookie. */
export function LinkedMailboxFactor(props:{challenge:LinkedMailboxChallenge;onConfirm:(proof:LinkMailboxProof)=>Promise<void>;onCancel:()=>void;errorText:(error:unknown)=>string}):JSX.Element {
  const owner=preferenceAccount();
  const has=(method:string)=>props.challenge.factors.includes(method);
  const [method,setMethod]=createSignal<'totp'|'recovery'>(has('totp')?'totp':'recovery');
  const [code,setCode]=createSignal('');
  const [busy,setBusy]=createSignal(false);
  const [error,setError]=createSignal('');
  let disposed=false;
  onCleanup(()=>{disposed=true;setCode('');});
  const current=()=>!disposed&&owner!==null&&preferenceAccountIsCurrent(owner);
  async function verify(passkey=false):Promise<void> {
    if(busy()||!current())return;
    setBusy(true);setError('');
    try {
      let proof:LinkMailboxProof;
      if(passkey){
        if(!has('webauthn')||!props.challenge.webauthn)return;
        const assertion=await assertPasskey(props.challenge.webauthn);
        if(!current())return;
        proof={method:'webauthn',credentialId:assertion.credentialId,clientDataJson:assertion.clientDataJson,authenticatorData:assertion.authenticatorData,signature:assertion.signature};
      }else{
        if(!has(method())||!code().trim())return;
        proof={method:method(),code:code().trim()};
      }
      await props.onConfirm(proof);
    }catch(cause){if(current())setError(props.errorText(cause));}
    finally{setCode('');if(!disposed)setBusy(false);}
  }
  return <section class={css.section} aria-label={t('linked-accounts-factor-title')} aria-busy={busy()}>
    <h2 class={css.heading}>{t('linked-accounts-factor-title')}</h2>
    <p class={css.prose}>{t('linked-accounts-factor-help')}</p>
    <Show when={has('webauthn')&&props.challenge.webauthn&&passkeySupported()}><button type="button" class={css.button} disabled={busy()} onClick={()=>void verify(true)}>{t('settings-2fa-challenge-use-passkey')}</button></Show>
    <Show when={has('totp')||has('recovery')}>
      <form onSubmit={event=>{event.preventDefault();void verify();}}>
        <label class={css.field}><span class={css.label}>{t('settings-2fa-challenge-method')}</span><select class={css.select} disabled={busy()} value={method()} onChange={event=>setMethod(event.currentTarget.value==='totp'?'totp':'recovery')}>
          <Show when={has('totp')}><option value="totp">{t('settings-2fa-challenge-totp-tab')}</option></Show>
          <Show when={has('recovery')}><option value="recovery">{t('settings-2fa-challenge-recovery-tab')}</option></Show>
        </select></label>
        <label class={css.field}><span class={css.label}>{t(method()==='totp'?'settings-2fa-totp-code-label':'settings-2fa-recovery-code-label')}</span><input class={css.input} disabled={busy()} autocomplete="one-time-code" inputmode={method()==='totp'?'numeric':'text'} value={code()} onInput={event=>setCode(event.currentTarget.value)}/></label>
        <button type="submit" class={css.button} disabled={busy()||!code().trim()}>{t('settings-2fa-challenge-verify')}</button>
      </form>
    </Show>
    <Show when={error()}><p role="alert" class={css.error}>{error()}</p></Show>
    <button type="button" class={css.ghost} disabled={busy()} onClick={props.onCancel}>{t('linked-accounts-cancel')}</button>
  </section>;
}
