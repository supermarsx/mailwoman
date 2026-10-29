import {createSignal,For,Show,type JSX} from 'solid-js';
import type {LinkedMailbox,LinkedMailboxList} from '../../api/linkedAccounts.ts';
import {NICKNAME_MAX,cleanNickname,linkOf,orderedKeys} from '../../api/mailboxLayout.ts';
import {createPointerReorder} from '../../components/accounts/pointerReorder.ts';
import {MailboxAvatarEditor} from './MailboxAvatarEditor.tsx';
import type {MailboxAvatarSpec} from '../../api/mailboxAvatar.ts';
import {Icon} from '../../components/Icon.tsx';
import {isolate,t} from '../../i18n/index.ts';
import * as a11y from '../../components/mailA11y.css.ts';
import {ConfirmDialog} from '../Admin/shell/ConfirmDialog.tsx';
import * as ux from '../Admin/shell/ux.css.ts';
import * as indicator from '../../components/accounts/accountIndicator.css.ts';
import * as css from './styles.css.ts';
import * as table from './linkedAccounts.css.ts';
import '../../components/person-avatar.css';
import '../../components/senderIcons/palette.css';

/** Settings › Linked mailboxes (t40, t48 k): the one place mailboxes are
 *  managed. A neat table in the owner's saved order: drag a row by its slim
 *  grip (long press anywhere on the row on touch), or focus the grip and press
 *  Alt+Up/Down; a live region says where it went. Each row has the avatar,
 *  nickname over address, the primary star, an expired-status icon, and icon
 *  actions with tooltips: renew access, rename, unlink (through the shared
 *  ConfirmDialog). The avatar is a button that opens the avatar editor (t86:
 *  letter or icon, colours). The sidebar picker only switches. */
export function LinkedAccountList(props:{
  value:LinkedMailboxList;
  busy:boolean;
  error?:string;
  onAdd:()=>void;
  onUnlink:(account:LinkedMailbox)=>void;
  /** Mark the mailbox that opens first (null = the sign-in account). */
  onSetFirst:(id:string|null)=>void;
  /** Move row `from` into insertion slot `slot` (0..n). */
  onMove:(from:number,slot:number)=>void;
  /** Save a nickname (`''` clears it); resolves false when refused. */
  onRename:(id:string|null,nickname:string)=>Promise<boolean>;
  /** Renew an expired link's access in place. */
  onRenew?:(account:LinkedMailbox)=>void;
  /** t86: save a mailbox's avatar (`undefined` = the generated one); resolves false when refused. */
  onAvatar?:(id:string|null,avatar:MailboxAvatarSpec|undefined)=>Promise<boolean>;
}):JSX.Element {
  const [confirm,setConfirm]=createSignal<string>();
  const [renaming,setRenaming]=createSignal<string>();
  const [announcement,setAnnouncement]=createSignal('');
  const [nicknameError,setNicknameError]=createSignal<{key:string;text:string}>();
  let listEl:HTMLUListElement|undefined;
  const full=()=>props.value.accounts.length>=props.value.policy.maxAccounts;
  const first=():string|null=>props.value.primaryMailbox??null;
  const keys=():string[]=>orderedKeys(props.value);
  const account=(key:string):LinkedMailbox|undefined=>props.value.accounts.find(item=>item.id===key);
  const addressOf=(key:string):string=>linkOf(key)===null?props.value.primary.address:(account(key)?.address??account(key)?.displayName??key);
  const nicknameOf=(key:string):string=>props.value.layout?.nicknames[key]??'';
  const labelOf=(key:string):string=>nicknameOf(key)||(linkOf(key)!==null&&account(key)?.displayName&&account(key)!.displayName!==addressOf(key)?account(key)!.displayName:addressOf(key));
  /** The mailbox as action labels name it: "Work (work@example.org)" or the address. */
  const nameFor=(key:string):string=>isolate(labelOf(key)===addressOf(key)?addressOf(key):`${labelOf(key)} (${addressOf(key)})`);
  function move(from:number,slot:number):void{
    const key=keys()[from];if(key===undefined||props.busy)return;
    props.onMove(from,slot);
    setAnnouncement(t('linked-accounts-moved',{name:labelOf(key),position:slot>from?slot:slot+1,total:keys().length}));
  }
  const reorder=createPointerReorder({
    rows:()=>Array.from(listEl?.querySelectorAll<HTMLElement>('[data-linked-row]')??[]),
    scroller:()=>listEl?.closest<HTMLElement>('[data-settings-scroll]')??listEl?.ownerDocument.scrollingElement as HTMLElement|null,
    onDrop:(from,slot)=>move(from,slot),
  });
  const drop=(index:number):'before'|'after'|undefined=>{
    const state=reorder.dragging();
    if(!state||state.over===state.from||state.over===state.from+1)return undefined;
    if(state.over===index)return 'before';
    if(state.over===keys().length&&index===keys().length-1)return 'after';
    return undefined;
  };
  async function saveNickname(key:string,input:HTMLInputElement):Promise<void>{
    const value=cleanNickname(input.value);
    if(value===null){setNicknameError({key,text:t('linked-accounts-nickname-invalid',{max:NICKNAME_MAX})});return;}
    setNicknameError(undefined);
    if(value!==nicknameOf(key)&&!await props.onRename(linkOf(key),value)){setNicknameError({key,text:t('linked-accounts-nickname-failed')});return;}
    setRenaming(undefined);
    queueMicrotask(()=>listEl?.querySelector<HTMLElement>(`[data-key="${CSS.escape(key)}"] [data-linked-rename]`)?.focus());
  }
  const icon=`btn btn--ghost ${ux.iconAction}`;
  const confirming=():LinkedMailbox|undefined=>{const key=confirm();return key?account(key):undefined;};
  const linkButton=():JSX.Element=><button type="button" class={`${css.button} ${table.addButton}`} data-linked-add disabled={props.busy||!props.value.policy.enabled||full()} onClick={props.onAdd}>{t('linked-accounts-add')}</button>;
  return <section class={css.section} aria-label={t('linked-accounts-title')} aria-busy={props.busy}>
    <div class={table.header}>
      <div class={table.headerText}>
        <h2 class={css.heading}>{t('linked-accounts-title')}</h2>
        <p class={css.meta}>{t('linked-accounts-description')}</p>
      </div>
      {linkButton()}
    </div>
    <Show when={!props.value.policy.enabled}><p role="status" class={css.meta}>{t('linked-accounts-disabled')}</p></Show>
    <Show when={props.value.policy.enabled&&full()}><p role="status" class={css.meta}>{t('linked-accounts-limit')}</p></Show>
    <Show when={props.error}><p role="alert" class={css.error}>{props.error}</p></Show>
    <span class={a11y.srOnly} role="status" aria-live="polite">{announcement()}</span>
    <Show when={props.value.accounts.length>0} fallback={<p class={table.empty} data-linked-empty>{t('linked-accounts-empty')}</p>}>
      <ul class={table.table} ref={listEl} aria-label={t('linked-accounts-title')}>
        <For each={keys()}>{(key,index)=>{
          const linkId=linkOf(key),child=()=>account(key);
          const ready=():boolean=>linkId===null||(child()?.status==='ready'&&props.value.policy.enabled);
          const expired=():boolean=>linkId!==null&&child()?.status==='reauthRequired';
          const primary=():boolean=>first()===linkId;
          const nicknameId=`linked-nickname-${key}`;
          return <li class={table.row} data-linked-row data-key={key} data-drop={drop(index())}
            onPointerDown={event=>{
              // Touch: a long press anywhere on the row lifts it (the grip is slim).
              if(event.pointerType!=='touch'||props.busy||(event.target as HTMLElement).closest('button,input,a'))return;
              reorder.start(event,index(),false);
            }}>
            <button type="button" class={table.grip} data-linked-grip disabled={props.busy||keys().length<2}
              aria-label={t('linked-accounts-move-handle',{name:labelOf(key)})} aria-describedby="linked-accounts-reorder-keys" data-tooltip={t('linked-accounts-drag')}
              onPointerDown={event=>{if(!props.busy)reorder.start(event,index(),true);}}
              onKeyDown={event=>{
                if(!event.altKey||(event.key!=='ArrowUp'&&event.key!=='ArrowDown'))return;
                event.preventDefault();
                const slot=event.key==='ArrowUp'?index()-1:index()+2;
                if(slot<0||slot>keys().length)return;
                move(index(),slot);
                queueMicrotask(()=>listEl?.querySelector<HTMLElement>(`[data-key="${CSS.escape(key)}"] [data-linked-grip]`)?.focus());
              }}/>
            <MailboxAvatarEditor mailboxKey={key} label={labelOf(key)} address={addressOf(key)} avatar={props.value.layout?.avatars?.[key]} disabled={props.busy||!props.onAvatar}
              onSave={avatar=>props.onAvatar?.(linkId,avatar)??Promise.resolve(false)}/>
            <Show when={renaming()===key} fallback={
              <div class={table.who} title={nicknameOf(key)?`${nicknameOf(key)}\n${addressOf(key)}`:addressOf(key)}>
                <span class={table.name} data-linked-name>{isolate(labelOf(key))}</span>
                <Show when={labelOf(key)!==addressOf(key)}><span class={table.address}>{isolate(addressOf(key))}</span></Show>
                <Show when={expired()}><span class={a11y.srOnly}>{t('linked-accounts-expired')}</span></Show>
              </div>}>
              <div class={`${table.who} ${table.rename}`}>
                <label class={a11y.srOnly} for={nicknameId}>{t('linked-accounts-nickname-for',{address:isolate(addressOf(key))})}</label>
                <input id={nicknameId} class={css.input} type="text" data-linked-nickname maxLength={NICKNAME_MAX*2} value={nicknameOf(key)} placeholder={addressOf(key)}
                  aria-invalid={nicknameError()?.key===key?'true':undefined} aria-describedby={nicknameError()?.key===key?`${nicknameId}-error`:undefined} disabled={props.busy}
                  ref={input=>queueMicrotask(()=>{input.focus();input.select();})}
                  onKeyDown={event=>{
                    if(event.key==='Enter'){event.preventDefault();void saveNickname(key,event.currentTarget);}
                    else if(event.key==='Escape'){event.preventDefault();event.stopPropagation();setNicknameError(undefined);setRenaming(undefined);}
                  }}/>
                <Show when={nicknameError()?.key===key}><span id={`${nicknameId}-error`} role="alert" class={css.error}>{nicknameError()!.text}</span></Show>
              </div>
            </Show>
            <div class={table.actions} data-linked-actions>
              <Show when={expired()}>
                <span class={table.status} role="img" data-linked-status aria-label={t('linked-accounts-expired')} data-tooltip={t('linked-accounts-expired')}><Icon name="alert"/></span>
              </Show>
              <button type="button" class={icon} data-linked-set-first data-primary={primary()?'true':undefined} aria-pressed={primary()?'true':'false'}
                aria-label={primary()?t('linked-accounts-primary-mailbox-for',{address:isolate(addressOf(key))}):t('linked-accounts-set-first',{address:isolate(addressOf(key))})}
                data-tooltip={primary()?t('linked-accounts-primary-mailbox'):t('linked-accounts-set-first-short')}
                disabled={props.busy||(!primary()&&!ready())} onClick={()=>{if(!primary())props.onSetFirst(linkId);}}><Icon name={primary()?'star':'starOutline'}/></button>
              <Show when={expired()&&props.onRenew&&props.value.policy.enabled}>
                <button type="button" class={`${icon} ${indicator.iconWarning}`} data-linked-renew disabled={props.busy}
                  aria-label={t('linked-accounts-renew-for',{address:nameFor(key)})}
                  data-tooltip={t('linked-accounts-renew-tooltip')} onClick={()=>props.onRenew?.(child()!)}><Icon name="refresh"/></button>
              </Show>
              <button type="button" class={icon} data-linked-rename disabled={props.busy} aria-expanded={renaming()===key?'true':'false'}
                aria-label={t('linked-accounts-rename',{address:nameFor(key)})} data-tooltip={t('linked-accounts-rename-short')}
                onClick={()=>{setNicknameError(undefined);setRenaming(renaming()===key?undefined:key);}}><Icon name="edit"/></button>
              <Show when={linkId!==null}>
                <button type="button" class={`btn btn--danger ${ux.iconAction}`} data-linked-unlink disabled={props.busy}
                  aria-label={t('linked-accounts-unlink-for',{address:nameFor(key)})} data-tooltip={t('linked-accounts-unlink')} onClick={()=>setConfirm(key)}><Icon name="delete"/></button>
              </Show>
            </div>
          </li>;
        }}</For>
      </ul>
      <Show when={keys().length>1}><p class={css.meta}>{t('linked-accounts-reorder-help')}</p></Show>
    </Show>
    <p id="linked-accounts-reorder-keys" class={a11y.srOnly}>{t('linked-accounts-reorder-keys')}</p>
    <ConfirmDialog open={confirming()!==undefined} title={t('linked-accounts-unlink-title',{address:isolate(confirming()?.address??confirming()?.displayName??'')})}
      body={t('linked-accounts-unlink-help')} confirmLabel={t('linked-accounts-unlink-confirm')} cancelLabel={t('linked-accounts-cancel')} testId="linked-unlink-confirm"
      onConfirm={()=>{const target=confirming();setConfirm(undefined);if(target)props.onUnlink(target);}} onCancel={()=>setConfirm(undefined)}/>
  </section>;
}
