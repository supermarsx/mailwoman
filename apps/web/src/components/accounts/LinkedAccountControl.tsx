import {createEffect,createResource,createSignal,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {useApp} from '../../state/context.ts';
import {observePreferenceAccount,preferenceAccount} from '../../api/accountPreferenceLifecycle.ts';
import {createLinkedAccountsApi,type LinkedMailboxList} from '../../api/linkedAccounts.ts';
import {loadCatalog,t} from '../../i18n/index.ts';
import {confirmShellNavigation} from '../../shell/navigationGuard.ts';
import {MailboxPicker,MailboxPlain,type MailboxChoice} from './MailboxPicker.tsx';
import {dismissFailure,isTransientFailure,showFailure} from '../recoverableAction.ts';
import {linkedAccountError} from './linkedAccountError.ts';
import {RenewAccessDialog} from './RenewAccessDialog.tsx';
import {resourceValue} from '../resourceValue.ts';
import {invalidateLinkedAccounts,linkedAccountsRevision,primaryMailboxAuthority} from '../../api/linkedMailboxSession.ts';
import {keyOf,linkOf,mailboxAvatar,mailboxNickname,orderedKeys} from '../../api/mailboxLayout.ts';
import * as css from './accountIndicator.css.ts';

/** The one active-account indicator and switcher (t38 §2.3, t40, t48 k). With
 *  one mailbox it is the plain address or nickname (`full`) or a static letter
 *  (`icon`); with linked mailboxes it is the mailbox picker, in the owner's
 *  order, which only switches. Linking, order, nicknames, the primary mailbox,
 *  renewing access and unlinking are managed in Settings › Linked mailboxes. */
export type AccountControlVariant='full'|'icon';

// Last inbox unread count seen per mailbox in this tab (key: layout key). The
// active mailbox reads its folder list; the others read the live count the
// realtime stream carries for them (t46), and fall back to this only until the
// first frame for that mailbox arrives.
const [unreadSeen,setUnreadSeen]=createSignal<Readonly<Record<string,number>>>({});

// The last list this tab verified, by the sign-in mailbox's storage authority.
// Switching mailboxes remounts the control under the new mailbox's preference
// account, and its first fetch is in flight then: without this the picker
// became the plain address for that moment and the sidebar head changed size
// twice (t71). The list belongs to the sign-in mailbox, so it is the same list
// whichever of its mailboxes is open.
let lastList:{authority:string;value:LinkedMailboxList}|null=null;

/** The picker entries in the owner's order (also the shape t46 feeds counts into). */
export function mailboxChoices(list:LinkedMailboxList,unread:(linkId:string|null)=>number|null):MailboxChoice[]{
  const nick=(linkId:string|null)=>list.layout?.nicknames[keyOf(linkId)];
  const face=(linkId:string|null)=>{const avatar=list.layout?.avatars?.[keyOf(linkId)];return avatar?{avatar}:{};};
  return orderedKeys(list).map(key=>{
    const linkId=linkOf(key);
    if(linkId===null){const nickname=nick(null);return {linkId,address:list.primary.address,...(nickname?{nickname}:{}),...face(null),signIn:true,available:true,unread:unread(null)};}
    const account=list.accounts.find(item=>item.id===linkId)!;
    const nickname=nick(linkId);
    return {linkId,address:account.address??account.displayName,name:account.displayName,...(nickname?{nickname}:{}),...face(linkId),
      available:list.policy.enabled&&account.status==='ready',unread:unread(linkId),
      ...(list.policy.enabled&&account.status==='reauthRequired'?{renew:true}:{})};
  });
}

export function LinkedAccountControl(props:{variant?:AccountControlVariant}):JSX.Element {
  const [owner,setOwner]=createSignal(preferenceAccount());
  onCleanup(observePreferenceAccount(setOwner));
  onMount(()=>void loadCatalog('linked-accounts'));
  return <Show when={owner()} keyed>{_owner=> <OwnedControl {...props}/>}</Show>;
}
function OwnedControl(props:{variant?:AccountControlVariant}):JSX.Element {
  const app=useApp(),api=createLinkedAccountsApi();
  onCleanup(api.dispose);
  // Read through resourceValue: calling list() while it loads registers with
  // the route's <Suspense> around the whole mailbox shell, which then swaps the
  // shell for its loading card on every mailbox switch (t71, as t60 for Reader).
  const [list]=createResource(linkedAccountsRevision,async()=>{
    const value=await api.list();
    lastList={authority:value.primary.storageAuthority,value};
    return value;
  });
  const listed=():LinkedMailboxList|undefined=>{
    const value=resourceValue(list);
    if(value)return value;
    if(list.state==='errored'||!lastList)return undefined;
    // Only the list verified for this same sign-in mailbox stands in.
    const authority=primaryMailboxAuthority()??preferenceAccount()?.storageAuthority;
    return lastList.authority===authority?lastList.value:undefined;
  };
  const [error,setError]=createSignal('');
  // t48: the expired link being renewed from the picker (its dialog is open).
  const [renewing,setRenewing]=createSignal<string>();
  const activeInboxUnread=():number|null=>{
    const inbox=app.mailboxes().find(box=>box.role==='inbox');
    return inbox&&Number.isFinite(inbox.unreadEmails)?inbox.unreadEmails:null;
  };
  createEffect(()=>{
    const count=activeInboxUnread(),key=keyOf(app.linkedMailboxId());
    if(count!==null&&unreadSeen()[key]!==count)setUnreadSeen(seen=>({...seen,[key]:count}));
  });
  const unreadFor=(linkId:string|null):number|null=>{
    if(linkId===app.linkedMailboxId())return activeInboxUnread();
    // A live count is read only under the authority this list verified for the link.
    const authority=linkId===null?undefined:listed()?.accounts.find(account=>account.id===linkId&&account.status==='ready')?.storageAuthority;
    const live=linkId===null||authority?app.mailboxUnread(linkId,authority):null;
    return live??unreadSeen()[keyOf(linkId)]??null;
  };
  async function select(id:string|null):Promise<void>{
    if(!await confirmShellNavigation())return;
    setError('');
    // t91: the switch is retried by the transport; a final transient failure gets one toast with Retry.
    try{await app.switchLinkedMailbox(id);dismissFailure('linked:switch');}
    catch(cause){if(isTransientFailure(cause))showFailure({key:'linked:switch',error:cause,message:reason=>t('linked-accounts-switch-failed-reason',{reason}),retry:()=>app.switchLinkedMailbox(id)});else setError(linkedAccountError(cause));}
  }
  const renewAccount=()=>{const id=renewing();return id?listed()?.accounts.find(account=>account.id===id):undefined;};
  const renewLabel=(id:string):string=>{const account=listed()?.accounts.find(item=>item.id===id);return mailboxNickname(id)||account?.address||account?.displayName||'';};
  // Choosing an expired mailbox renews its access in place, then opens it:
  // never a dead end (t48 k). Managing links stays in Settings.
  function renewed():void{
    const id=renewing();
    setRenewing(undefined);
    invalidateLinkedAccounts();
    if(id)void select(id);
  }
  const address=():string=>app.me()?.username??'';
  const plain=():JSX.Element=><MailboxPlain variant={props.variant??'full'} address={address()} nickname={mailboxNickname(app.linkedMailboxId())} avatar={mailboxAvatar(app.linkedMailboxId())} unread={activeInboxUnread()}/>;
  return <Show when={(listed()?.accounts.length??0)>0?listed():undefined} fallback={plain()}>{value=>
    <div class={css.control}>
      <MailboxPicker variant={props.variant??'full'} choices={mailboxChoices(value(),unreadFor)} activeLinkId={app.linkedMailboxId()}
        busy={app.switchingMailbox()} error={error()} onSelect={id=>void select(id)} onRenew={id=>setRenewing(id)}/>
      <Show when={renewAccount()}>{account=>
        <RenewAccessDialog account={account()} label={renewLabel(account().id)} api={api} onRenewed={renewed} onCancel={()=>setRenewing(undefined)}/>}</Show>
    </div>}</Show>;
}
