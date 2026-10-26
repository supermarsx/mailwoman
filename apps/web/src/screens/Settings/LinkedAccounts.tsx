import {createResource,createSignal,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {createLinkedAccountsApi,type LinkedMailbox,type LinkedMailboxChallenge,type LinkMailboxInput,type LinkMailboxProof} from '../../api/linkedAccounts.ts';
import {publicConfig} from '../../api/publicConfig.ts';
import {invalidateLinkedAccounts} from '../../api/linkedMailboxSession.ts';
import {SigninChallenge} from '../../components/SigninChallenge.tsx';
import {AsyncPending} from '../../components/AsyncState.tsx';
import {linkedAccountError} from '../../components/accounts/linkedAccountError.ts';
import {useApp} from '../../state/context.ts';
import {loadCatalog,t} from '../../i18n/index.ts';
import {confirmShellNavigation} from '../../shell/navigationGuard.ts';
import {useEntityNavigation} from './entityEditor.tsx';
import {PersonalAccountBoundary} from './PersonalAccountBoundary.tsx';
import {LinkedAccountList} from './LinkedAccountList.tsx';
import {LinkMailboxEditor} from './LinkMailboxEditor.tsx';
import {LinkedMailboxFactor} from './LinkedMailboxFactor.tsx';
import {useSetting} from './useSetting.ts';
import {LockNote} from './LockNote.tsx';
import * as css from './styles.css.ts';

export function LinkedAccounts():JSX.Element{return <PersonalAccountBoundary>{()=> <OwnedLinkedAccounts/>}</PersonalAccountBoundary>;}
function OwnedLinkedAccounts():JSX.Element {
  const app=useApp(),api=createLinkedAccountsApi(),navigation=useEntityNavigation('linked-accounts');
  let disposed=false;
  onCleanup(()=>{disposed=true;api.dispose();});
  onMount(()=>{void loadCatalog('linked-accounts');void loadCatalog('settings');});
  const [list,{refetch}]=createResource(()=>api.list());
  const [config]=createResource(()=>publicConfig());
  const [challenge,setChallenge]=createSignal<LinkedMailboxChallenge>();
  const [proof,setProof]=createSignal<string>();
  const [reset,setReset]=createSignal(0);
  const [busy,setBusy]=createSignal(false);
  const [error,setError]=createSignal('');
  const editing=()=>navigation.route().section==='linked-accounts'&&navigation.route().editor?.kind==='new';
  const back=async():Promise<void>=>{setChallenge(undefined);await navigation.navigate({section:'linked-accounts'});};
  async function complete():Promise<void>{if(disposed)return;invalidateLinkedAccounts();await refetch();if(!disposed)await back();}
  async function link(input:LinkMailboxInput):Promise<void>{
    try{const result=await api.link(input);if(disposed)return;if('twofaRequired'in result)setChallenge(result);}
    finally{setProof(undefined);setReset(value=>value+1);}
  }
  async function confirm(value:LinkMailboxProof):Promise<void>{const pending=challenge();if(!pending)return;await api.confirm(pending.challengeId,value);await complete();}
  async function switchTo(id:string|null):Promise<void>{if(!await confirmShellNavigation())return;setError('');try{await app.switchLinkedMailbox(id);}catch(cause){if(!disposed)setError(linkedAccountError(cause));}}
  async function remove(account:LinkedMailbox):Promise<void>{
    if(busy())return;
    if(app.linkedMailboxId()===account.id&&!await confirmShellNavigation())return;
    setBusy(true);setError('');
    try{await api.unlink(account);if(disposed)return;invalidateLinkedAccounts();if(app.linkedMailboxId()===account.id)await app.switchLinkedMailbox(null);else await refetch();}
    catch(cause){if(!disposed)setError(linkedAccountError(cause));}
    finally{if(!disposed)setBusy(false);}
  }
  return <Show when={challenge()} keyed fallback={<Show when={editing()} fallback={<>
    <Show when={list.loading&&!list()}><AsyncPending variant="list"/></Show>
    <Show when={list.error}><p role="alert" class={css.error}>{linkedAccountError(list.error)}</p><button class={css.button} onClick={()=>void refetch()}>{t('linked-accounts-retry')}</button></Show>
    <Show when={!list.error&&list()}>{value=><>
      <Show when={app.linkedMailboxId()===null} fallback={<p class={css.meta}>{t('linked-accounts-primary-preference')}</p>}><LinkedPreference onChanged={()=>{invalidateLinkedAccounts();void refetch();}}/></Show>
      <Show when={value().policy.reason==='linkedBackendUnsupported'}><p class={css.meta}>{t('linked-accounts-backend-unsupported')}</p></Show>
      <LinkedAccountList value={value()} activeLinkId={app.linkedMailboxId()} busy={busy()||app.switchingMailbox()} error={error()} onAdd={()=>void navigation.navigate({section:'linked-accounts',editor:{kind:'new'}})} onSwitch={id=>void switchTo(id)} onUnlink={account=>void remove(account)}/>
    </>}</Show>
  </>}><LinkMailboxEditor enabled={!!list()?.policy.enabled&&!config.loading&&!config.error} onVerify={link} onVerified={complete} onCancel={back} errorText={linkedAccountError}
      verificationRequired={config()?.antiBot?.mailbox===true} {...(proof()?{verificationProof:proof()!}:{})}
      verification={<Show when={config()?.antiBot?.mailbox}><SigninChallenge surface="mailbox" reset={reset()} onProof={setProof}/></Show>}/>
    <Show when={config.error}><p role="alert" class={css.error}>{t('linked-accounts-error')}</p></Show>
  </Show>}>{pending=><LinkedMailboxFactor challenge={pending} onConfirm={confirm} onCancel={()=>void back()} errorText={linkedAccountError}/>}</Show>;
}
function LinkedPreference(props:{onChanged:()=>void}):JSX.Element {
  const setting=useSetting<boolean>('accounts.useLinked',true);
  return <div class={css.field}><label class={css.check}><input class={css.checkbox} type="checkbox" checked={setting.value()} disabled={!setting.entry()||setting.locked()||setting.saving()} onChange={event=>void setting.set(event.currentTarget.checked).then(saved=>{if(saved)props.onChanged();})}/>{t('linked-accounts-use')}</label><p class={css.meta}>{t('linked-accounts-use-help')}</p><LockNote locked={setting.locked()}/><Show when={setting.error()}><p role="alert" class={css.error}>{setting.error()}</p></Show></div>;
}
export default LinkedAccounts;
