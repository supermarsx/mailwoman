import {createSignal,For,Show,type JSX} from 'solid-js';
import type {LinkedMailbox,LinkedMailboxList} from '../../api/linkedAccounts.ts';
import {NICKNAME_MAX,cleanNickname,linkOf,orderedKeys} from '../../api/mailboxLayout.ts';
import {createPointerReorder} from '../../components/accounts/pointerReorder.ts';
import {Icon} from '../../components/Icon.tsx';
import {isolate,t} from '../../i18n/index.ts';
import * as a11y from '../../components/mailA11y.css.ts';
import * as reorderCss from '../../components/accounts/accountIndicator.css.ts';
import * as css from './styles.css.ts';

/** Management remains a primary-session operation even while viewing a child.
 *  Rows follow the owner's saved order: drag a row by its grip (long press on
 *  touch), or focus the grip and press Alt+Up/Down (the keyboard fallback); a live region
 *  says where it went. Each row has a display-only nickname field. */
export function LinkedAccountList(props:{
  value:LinkedMailboxList;
  activeLinkId:string|null;
  busy:boolean;
  error?:string;
  onAdd:()=>void;
  onSwitch:(id:string|null)=>void;
  onUnlink:(account:LinkedMailbox)=>void;
  /** Mark the mailbox that opens first (null = the sign-in account). */
  onSetFirst:(id:string|null)=>void;
  /** Move row `from` into insertion slot `slot` (0..n). */
  onMove:(from:number,slot:number)=>void;
  /** Save a nickname (`''` clears it); resolves false when refused. */
  onRename:(id:string|null,nickname:string)=>Promise<boolean>;
}):JSX.Element {
  const [confirm,setConfirm]=createSignal<string>();
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
    if(value===nicknameOf(key)){input.value=value;return;}
    if(!await props.onRename(linkOf(key),value))setNicknameError({key,text:t('linked-accounts-nickname-failed')});
  }
  const firstControl=(id:string|null,address:string,ready:boolean):JSX.Element=><Show when={first()===id} fallback={
    <button type="button" class={css.ghost} data-linked-set-first aria-label={t('linked-accounts-set-first',{address})} disabled={props.busy||!ready} onClick={()=>props.onSetFirst(id)}>{t('linked-accounts-set-first-short')}</button>}>
    <span class={css.meta} data-linked-first>{t('linked-accounts-first')} · {t('linked-accounts-first-help')}</span>
  </Show>;
  return <section class={css.section} aria-label={t('linked-accounts-title')} aria-busy={props.busy}>
    <h2 class={css.heading}>{t('linked-accounts-title')}</h2>
    <p class={css.prose}>{t('linked-accounts-description')}</p>
    <Show when={!props.value.policy.enabled}><p role="status" class={css.meta}>{t('linked-accounts-disabled')}</p></Show>
    <Show when={props.value.policy.enabled&&full()}><p role="status" class={css.meta}>{t('linked-accounts-limit')}</p></Show>
    <Show when={props.error}><p role="alert" class={css.error}>{props.error}</p></Show>
    <Show when={keys().length>1}><p class={css.meta}>{t('linked-accounts-reorder-help')}</p></Show>
    <span class={a11y.srOnly} role="status" aria-live="polite">{announcement()}</span>
    <ul class={css.list} ref={listEl}>
      <For each={keys()}>{(key,index)=>{
        const linkId=linkOf(key),child=()=>account(key);
        const ready=():boolean=>linkId===null||(child()?.status==='ready'&&props.value.policy.enabled);
        const nicknameId=`linked-nickname-${key}`;
        return <li class={`${css.item} ${reorderCss.row}`} data-linked-row data-key={key} data-drop={drop(index())}>
          <Show when={keys().length>1}>
            <span class={reorderCss.grip} role="button" tabIndex={0} data-linked-grip
              aria-label={t('linked-accounts-move-handle',{name:labelOf(key)})} aria-describedby="linked-accounts-reorder-keys"
              onPointerDown={event=>{if(!props.busy)reorder.start(event,index(),true);}}
              onKeyDown={event=>{
                if(!event.altKey||(event.key!=='ArrowUp'&&event.key!=='ArrowDown'))return;
                event.preventDefault();
                const slot=event.key==='ArrowUp'?index()-1:index()+2;
                if(slot<0||slot>keys().length)return;
                move(index(),slot);
                queueMicrotask(()=>listEl?.querySelector<HTMLElement>(`[data-key="${CSS.escape(key)}"] [data-linked-grip]`)?.focus());
              }}>
              <Icon name="menu"/>
            </span>
          </Show>
          <div class={css.itemMain}>
            <span class={css.itemName} title={nicknameOf(key)?`${nicknameOf(key)}\n${addressOf(key)}`:addressOf(key)}>{isolate(labelOf(key))}</span>
            <Show when={labelOf(key)!==addressOf(key)}><span class={css.meta}>{isolate(addressOf(key))}</span></Show>
            <Show when={linkId===null}><span class={css.meta}>{t('linked-accounts-primary')}</span></Show>
            <Show when={linkId!==null&&child()?.status!=='ready'}><span class={css.meta}>{t(child()?.status==='reauthRequired'?'linked-accounts-reauth':'linked-accounts-unavailable')}</span></Show>
            <label class={css.meta} for={nicknameId}>{t('linked-accounts-nickname')}</label>
            <input id={nicknameId} class={css.input} type="text" data-linked-nickname maxLength={NICKNAME_MAX*2} value={nicknameOf(key)} placeholder={addressOf(key)}
              aria-invalid={nicknameError()?.key===key?'true':undefined} aria-describedby={nicknameError()?.key===key?`${nicknameId}-error`:undefined} disabled={props.busy}
              onKeyDown={event=>{
                if(event.key==='Enter'){event.preventDefault();void saveNickname(key,event.currentTarget);}
                else if(event.key==='Escape'){event.preventDefault();event.stopPropagation();event.currentTarget.value=nicknameOf(key);setNicknameError(undefined);}
              }}
              onChange={event=>void saveNickname(key,event.currentTarget)}/>
            <Show when={nicknameError()?.key===key}><span id={`${nicknameId}-error`} role="alert" class={css.error}>{nicknameError()!.text}</span></Show>
            <Show when={linkId!==null&&confirm()===key}><p class={css.meta}>{t('linked-accounts-unlink-help')}</p></Show>
          </div>
          <div class={css.actions}>
            <Show when={linkId!==null&&confirm()===key} fallback={<>
              {firstControl(linkId,addressOf(key),ready())}
              <button type="button" class={css.ghost} disabled={props.busy||!ready()||props.activeLinkId===linkId} onClick={()=>props.onSwitch(linkId)}>{t('linked-accounts-open')}</button>
              <Show when={linkId!==null}><button type="button" class={css.danger} disabled={props.busy} onClick={()=>setConfirm(key)}>{t('linked-accounts-unlink')}</button></Show>
            </>}>
              <button type="button" class={css.danger} disabled={props.busy} onClick={()=>{props.onUnlink(child()!);setConfirm(undefined);}}>{t('linked-accounts-unlink-confirm')}</button>
              <button type="button" class={css.ghost} disabled={props.busy} onClick={()=>setConfirm(undefined)}>{t('linked-accounts-cancel')}</button>
            </Show>
          </div>
        </li>;
      }}</For>
    </ul>
    <p id="linked-accounts-reorder-keys" class={a11y.srOnly}>{t('linked-accounts-reorder-keys')}</p>
    <div class={css.actions}><button type="button" class={css.button} disabled={props.busy||!props.value.policy.enabled||full()} onClick={props.onAdd}>{t('linked-accounts-add')}</button></div>
  </section>;
}
