import {createResource,createSignal,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {useApp} from '../../state/context.ts';
import {observePreferenceAccount,preferenceAccount} from '../../api/accountPreferenceLifecycle.ts';
import {createLinkedAccountsApi} from '../../api/linkedAccounts.ts';
import {isolate,loadCatalog,t} from '../../i18n/index.ts';
import {confirmShellNavigation} from '../../shell/navigationGuard.ts';
import {AccountSelector} from './AccountSelector.tsx';
import {AccountIconMenu} from './AccountIconMenu.tsx';
import {linkedAccountError} from './linkedAccountError.ts';
import {linkedAccountsRevision} from '../../api/linkedMailboxSession.ts';
import {refreshHelp} from './accountSelector.css.ts';
import {address as addressStyle} from './accountIndicator.css.ts';

/** The one active-account indicator and switcher (t38 §2.3). `full` shows the
 *  address, or the account selector when linking is on; `icon` is a one-letter
 *  button that opens the same choices (compact folders, hidden sidebar). */
export type AccountControlVariant='full'|'icon';

export function LinkedAccountControl(props:{onManage:()=>void;variant?:AccountControlVariant}):JSX.Element {
  const [owner,setOwner]=createSignal(preferenceAccount());
  onCleanup(observePreferenceAccount(setOwner));
  onMount(()=>void loadCatalog('linked-accounts'));
  return <Show when={owner()} keyed>{_owner=> <OwnedControl {...props}/>}</Show>;
}
function OwnedControl(props:{onManage:()=>void;variant?:AccountControlVariant}):JSX.Element {
  const app=useApp(),api=createLinkedAccountsApi();
  onCleanup(api.dispose);
  const [list]=createResource(linkedAccountsRevision,()=>api.list());
  const [error,setError]=createSignal('');
  async function select(id:string|null):Promise<void>{
    if(!await confirmShellNavigation())return;
    setError('');
    try{await app.switchLinkedMailbox(id);}catch(cause){setError(linkedAccountError(cause));}
  }
  const active=():string=>app.me()?.username??'';
  const body=(close:()=>void):JSX.Element=>{
    const manage=():void=>{close();props.onManage();};
    return <Show when={!list.error&&list()} fallback={<button type="button" class="btn btn--ghost" onClick={manage}>{app.me()?.username??t('linked-accounts-manage')}</button>}>{value=><>
      <Show when={value().policy.enabled||value().accounts.length>0} fallback={<span class={addressStyle} title={value().primary.address} data-account-address>{isolate(value().primary.address)}</span>}>
      <AccountSelector choices={[{linkId:null,address:value().primary.address,available:true},...value().accounts.map(account=>({linkId:account.id,address:account.address??account.displayName,displayName:account.displayName,available:value().policy.enabled&&account.status==='ready'}))]}
        activeLinkId={app.linkedMailboxId()} busy={app.switchingMailbox()} error={error()} onSelect={id=>{close();void select(id);}} onManage={manage}/>
      </Show>
      <Show when={app.linkedMailboxId()}><span class={refreshHelp} role="status" tabIndex={0} aria-label={t('linked-accounts-refresh-help')} data-tooltip={t('linked-accounts-refresh-help')}>ⓘ</span></Show>
    </>}</Show>;
  };
  return <Show when={props.variant==='icon'} fallback={body(()=>undefined)}>
    <AccountIconMenu address={active()} linked={app.linkedMailboxId()!==null}>{close=>body(close)}</AccountIconMenu>
  </Show>;
}
