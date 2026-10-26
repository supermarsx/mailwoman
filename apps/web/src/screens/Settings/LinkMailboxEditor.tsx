import {createSignal,onCleanup,Show,type JSX} from 'solid-js';
import type {LinkMailboxInput} from '../../api/linkedAccounts.ts';
import {preferenceAccount,preferenceAccountIsCurrent} from '../../api/accountPreferenceLifecycle.ts';
import {t} from '../../i18n/index.ts';
import {useEditorGuard} from './entityEditor.tsx';
import * as css from './styles.css.ts';
import * as layout from './editor.css.ts';

/** Credentials live only in this editor. Parent owns the separate MFA step. */
export function LinkMailboxEditor(props:{
  enabled:boolean;
  onVerify:(input:LinkMailboxInput)=>Promise<void>;
  onVerified?:()=>void|Promise<void>;
  onCancel:()=>void|Promise<void>;
  errorText:(error:unknown)=>string;
  verification?:JSX.Element;
  verificationRequired?:boolean;
  verificationProof?:string;
}):JSX.Element {
  const owner=preferenceAccount();
  const [username,setUsername]=createSignal('');
  const [password,setPassword]=createSignal('');
  const [name,setName]=createSignal('');
  const [busy,setBusy]=createSignal(false);
  const [saved,setSaved]=createSignal(false);
  const [error,setError]=createSignal('');
  let disposed=false;
  const guard=useEditorGuard(()=>!saved()&&!!(username()||password()||name()),busy);
  onCleanup(()=>{disposed=true;setPassword('');});
  async function submit():Promise<void> {
    if(busy()||!props.enabled||(props.verificationRequired&&!props.verificationProof))return;
    if(!owner?.storageAuthority||!preferenceAccountIsCurrent(owner)){setPassword('');setError(t('linked-accounts-changed'));return;}
    const address=username().trim();
    if(!address||address.length>320||[...address].some(char=>char.charCodeAt(0)<32||char.charCodeAt(0)===127)||!password()||name().length>128){setError(t('linked-accounts-fields-required'));return;}
    setBusy(true);setError('');
    try {
      await props.onVerify({username:address,password:password(),...(name().trim()?{displayName:name().trim()}:{}),...(props.verificationProof?{altcha:props.verificationProof}:{})});
      if(!disposed&&preferenceAccountIsCurrent(owner)){setSaved(true);setBusy(false);await props.onVerified?.();}
    } catch(cause) {
      if(!disposed&&preferenceAccountIsCurrent(owner))setError(props.errorText(cause));
    } finally {
      setPassword('');
      if(!disposed)setBusy(false);
    }
  }
  return <section class={css.section} aria-label={t('linked-accounts-add')}>
    <h2 class={css.heading}>{t('linked-accounts-add')}</h2>
    <p class={css.prose}>{t('linked-accounts-verify-help')}</p>
    <form onSubmit={event=>{event.preventDefault();void submit();}}>
      <fieldset class={layout.fields} disabled={busy()||!props.enabled}>
        <label class={css.field}><span class={css.label}>{t('linked-accounts-address')}</span><input class={css.input} autocomplete="username" autocapitalize="none" spellcheck={false} required maxlength={320} value={username()} onInput={event=>setUsername(event.currentTarget.value)}/></label>
        <label class={css.field}><span class={css.label}>{t('linked-accounts-password')}</span><input class={css.input} type="password" autocomplete="current-password" required value={password()} onInput={event=>setPassword(event.currentTarget.value)}/></label>
        <label class={css.field}><span class={css.label}>{t('linked-accounts-name')}</span><input class={css.input} maxlength={128} value={name()} onInput={event=>setName(event.currentTarget.value)}/></label>
        {props.verification}
        <div class={css.actions}><button type="submit" class={css.button} disabled={!!props.verificationRequired&&!props.verificationProof}>{t('linked-accounts-submit')}</button></div>
      </fieldset>
      <button type="button" class={css.ghost} disabled={busy()} onClick={()=>void guard.leave(props.onCancel)}>{t('linked-accounts-cancel')}</button>
    </form>
    <Show when={error()}><p class={css.error} role="alert">{error()}</p></Show>
    {guard.dialog()}
  </section>;
}
