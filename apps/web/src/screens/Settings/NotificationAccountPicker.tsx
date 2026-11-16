import {createResource,onCleanup,onMount,Show,useContext,type JSX} from 'solid-js';
import type {AppState} from '../../state/store.ts';
import {AppContext} from '../../state/context.ts';
import {createLinkedAccountsApi} from '../../api/linkedAccounts.ts';
import {MailboxPicker} from '../../components/accounts/MailboxPicker.tsx';
import {resourceValue} from '../../components/resourceValue.ts';
import {confirmShellNavigation} from '../../shell/navigationGuard.ts';
import {linkedAccountError} from '../../components/accounts/linkedAccountError.ts';
import {toast} from '../../components/toast/toast.ts';
import {t,loadCatalog} from '../../i18n';
import * as css from './styles.css.ts';
export function NotificationAccountPicker(props:{disabled?:boolean}={}):JSX.Element{const app=useContext(AppContext);return <Show when={app}>{value=><AccountPicker app={value()} disabled={props.disabled===true}/>}</Show>;}
function AccountPicker(props:{app:AppState;disabled:boolean}):JSX.Element{
 const api=createLinkedAccountsApi();onCleanup(api.dispose);onMount(()=>void loadCatalog('linked-accounts'));
 const [list]=createResource(()=>api.list());const value=()=>resourceValue(list);
 const choices=()=>{const current=value();return current?[{linkId:null,address:current.primary.address,available:true,unread:null},...current.accounts.map(account=>({linkId:account.id,address:account.address||account.displayName||account.id,available:current.policy.enabled&&account.status==='ready',unread:null}))]:[];};
 async function select(id:string|null):Promise<void>{if(props.disabled||props.app.switchingMailbox()||!await confirmShellNavigation())return;try{await props.app.switchLinkedMailbox(id);}catch(error){toast.error(linkedAccountError(error));}}
 return <Show when={value()}><div class={css.field} style={{'max-inline-size':'28rem'}}><p class={css.meta}>{t('settings-permissions-push-account-help')}</p><MailboxPicker variant="full" choices={choices()} activeLinkId={props.app.linkedMailboxId()} busy={props.disabled||props.app.switchingMailbox()} onSelect={id=>void select(id)}/></div></Show>;
}
