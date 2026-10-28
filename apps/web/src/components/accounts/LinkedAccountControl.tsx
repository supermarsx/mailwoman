import {createEffect,createResource,createSignal,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {useApp} from '../../state/context.ts';
import {observePreferenceAccount,preferenceAccount} from '../../api/accountPreferenceLifecycle.ts';
import {createLinkedAccountsApi,type LinkedMailboxList} from '../../api/linkedAccounts.ts';
import {loadCatalog,t} from '../../i18n/index.ts';
import {confirmShellNavigation} from '../../shell/navigationGuard.ts';
import {MailboxPicker,MailboxPlain,type MailboxChoice} from './MailboxPicker.tsx';
import {linkedAccountError} from './linkedAccountError.ts';
import {invalidateLinkedAccounts,linkedAccountsRevision} from '../../api/linkedMailboxSession.ts';
import {keyOf,layoutChange,linkOf,mailboxNickname,orderedKeys,saveLayout} from '../../api/mailboxLayout.ts';
import {moveTo} from './pointerReorder.ts';
import * as css from './accountIndicator.css.ts';

/** The one active-account indicator and switcher (t38 §2.3, t40). With one
 *  mailbox it is the plain address or nickname (`full`) or a static letter
 *  (`icon`); with linked mailboxes it is the mailbox picker, in the owner's
 *  order, where mailboxes can be switched, reordered, renamed and marked as the
 *  one that opens first. Linking itself is managed in Settings › Linked mailboxes. */
export type AccountControlVariant='full'|'icon';

// Last inbox unread count seen per mailbox in this tab (key: layout key). The
// active mailbox is live; the others show what they had when last open, since
// a mailbox is only queried while it is active (t46 adds live child counts).
const [unreadSeen,setUnreadSeen]=createSignal<Readonly<Record<string,number>>>({});

/** The picker entries in the owner's order (also the shape t46 feeds counts into). */
export function mailboxChoices(list:LinkedMailboxList,unread:(linkId:string|null)=>number|null):MailboxChoice[]{
  const nick=(linkId:string|null)=>list.layout?.nicknames[keyOf(linkId)];
  return orderedKeys(list).map(key=>{
    const linkId=linkOf(key);
    if(linkId===null){const nickname=nick(null);return {linkId,address:list.primary.address,...(nickname?{nickname}:{}),signIn:true,available:true,unread:unread(null)};}
    const account=list.accounts.find(item=>item.id===linkId)!;
    const nickname=nick(linkId);
    return {linkId,address:account.address??account.displayName,name:account.displayName,...(nickname?{nickname}:{}),
      available:list.policy.enabled&&account.status==='ready',unread:unread(linkId)};
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
  const [list,{mutate}]=createResource(linkedAccountsRevision,()=>api.list());
  const [error,setError]=createSignal('');
  const [saving,setSaving]=createSignal(false);
  const activeInboxUnread=():number|null=>{
    const inbox=app.mailboxes().find(box=>box.role==='inbox');
    return inbox&&Number.isFinite(inbox.unreadEmails)?inbox.unreadEmails:null;
  };
  createEffect(()=>{
    const count=activeInboxUnread(),key=keyOf(app.linkedMailboxId());
    if(count!==null&&unreadSeen()[key]!==count)setUnreadSeen(seen=>({...seen,[key]:count}));
  });
  const unreadFor=(linkId:string|null):number|null=>linkId===app.linkedMailboxId()?activeInboxUnread():unreadSeen()[keyOf(linkId)]??null;
  async function select(id:string|null):Promise<void>{
    if(!await confirmShellNavigation())return;
    setError('');
    try{await app.switchLinkedMailbox(id);}catch(cause){setError(linkedAccountError(cause));}
  }
  async function run(action:()=>Promise<void>):Promise<boolean>{
    if(saving())return false;
    setSaving(true);setError('');
    try{await action();invalidateLinkedAccounts();return true;}
    catch(cause){setError(linkedAccountError(cause));invalidateLinkedAccounts();return false;}
    finally{setSaving(false);}
  }
  const setFirst=(id:string|null)=>void run(async()=>{await api.setPrimary(id);});
  function move(from:number,slot:number):void{
    const current=list();if(!current)return;
    const order=moveTo(orderedKeys(current),from,slot);
    // Show the new order at once; the refetch after saving confirms it.
    mutate({...current,layout:{revision:current.layout?.revision??'',order,nicknames:current.layout?.nicknames??{}}});
    void run(async()=>{await saveLayout(api,current,()=>({order,nicknames:current.layout?.nicknames??{}}));});
  }
  const rename=(linkId:string|null,nickname:string):Promise<boolean>=>{
    const current=list();if(!current)return Promise.resolve(false);
    return run(async()=>{await saveLayout(api,current,fresh=>layoutChange(fresh,{nickname:{key:keyOf(linkId),value:nickname}}));});
  };
  const address=():string=>app.me()?.username??'';
  const plain=():JSX.Element=><MailboxPlain variant={props.variant??'full'} address={address()} nickname={mailboxNickname(app.linkedMailboxId())} unread={activeInboxUnread()}/>;
  return <Show when={!list.error&&list()&&list()!.accounts.length>0?list():undefined} fallback={plain()}>{value=>
    <div class={css.control}>
      <MailboxPicker variant={props.variant??'full'} choices={mailboxChoices(value(),unreadFor)} activeLinkId={app.linkedMailboxId()} firstLinkId={value().primaryMailbox??null}
        busy={app.switchingMailbox()||saving()} error={error()} onSelect={id=>void select(id)} onSetFirst={setFirst} onMove={move} onRename={rename}/>
      <Show when={props.variant!=='icon'&&app.linkedMailboxId()}><span class={css.refreshHelp} role="status" tabIndex={0} aria-label={t('linked-accounts-refresh-help')} data-tooltip={t('linked-accounts-refresh-help')}>ⓘ</span></Show>
    </div>}</Show>;
}
