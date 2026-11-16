import {Show,type JSX} from 'solid-js';
import {settings} from '../../api/settings.ts';
import {primaryMailboxAuthority} from '../../api/linkedMailboxSession.ts';
import {accountHealth} from '../../realtime/accountHealth.ts';
import {loadCatalog,t} from '../../i18n/index.ts';
import {Icon,type IconName} from '../Icon.tsx';
import * as css from './mailboxConnectionStatus.css.ts';
void loadCatalog('linked-accounts');

/** Observed mailbox health, never inferred from the app's push connection. */
export function MailboxConnectionStatus(props:{linkId:string|null;available?:boolean|undefined;renew?:boolean|undefined;detail?:boolean;root?:string|undefined}):JSX.Element {
 const health=()=>{const root=props.root??primaryMailboxAuthority();return root?accountHealth().find(row=>row.root===root&&row.linkId===props.linkId):undefined;};
 const state=()=>props.renew?'auth-expired':props.available===false?'unavailable':health()?.state??'unknown';
 const label=()=>state()==='unknown'?t('linked-accounts-connection-unknown'):state()==='unavailable'?t('linked-accounts-unavailable'):t(`common-conn-account-${state()}`);
 const icon=():IconName=>state()==='online'?'check':state()==='syncing'?'restore':state()==='auth-expired'?'lock':state()==='unknown'?'info':'alert';
 return <Show when={props.detail||settings()['accounts.showConnectionStatus']?.value===true}><span class={css.status} data-detail={props.detail?'':undefined} data-mailbox-connection={state()} role={props.detail?undefined:'img'} aria-label={props.detail?undefined:label()} data-tooltip={label()}><Icon name={icon()}/><Show when={props.detail}><span>{label()}</span></Show></span></Show>;
}
