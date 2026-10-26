import {createSignal,For,Show,type JSX} from 'solid-js';
import type {LinkedMailbox,LinkedMailboxList} from '../../api/linkedAccounts.ts';
import {t} from '../../i18n/index.ts';
import * as css from './styles.css.ts';

/** Management remains a primary-session operation even while viewing a child. */
export function LinkedAccountList(props:{
  value:LinkedMailboxList;
  activeLinkId:string|null;
  busy:boolean;
  error?:string;
  onAdd:()=>void;
  onSwitch:(id:string|null)=>void;
  onUnlink:(account:LinkedMailbox)=>void;
}):JSX.Element {
  const [confirm,setConfirm]=createSignal<string>();
  const full=()=>props.value.accounts.length>=props.value.policy.maxAccounts;
  return <section class={css.section} aria-label={t('linked-accounts-title')} aria-busy={props.busy}>
    <h2 class={css.heading}>{t('linked-accounts-title')}</h2>
    <p class={css.prose}>{t('linked-accounts-description')}</p>
    <Show when={!props.value.policy.enabled}><p role="status" class={css.meta}>{t('linked-accounts-disabled')}</p></Show>
    <Show when={props.value.policy.enabled&&full()}><p role="status" class={css.meta}>{t('linked-accounts-limit')}</p></Show>
    <Show when={props.error}><p role="alert" class={css.error}>{props.error}</p></Show>
    <ul class={css.list}>
      <li class={css.item}>
        <div class={css.itemMain}><span class={css.itemName}>{props.value.primary.address}</span><span class={css.meta}>{t('linked-accounts-primary')}</span></div>
        <button type="button" class={css.ghost} disabled={props.busy||props.activeLinkId===null} onClick={()=>props.onSwitch(null)}>{t('linked-accounts-open')}</button>
      </li>
      <For each={props.value.accounts}>{account=><li class={css.item}>
        <div class={css.itemMain}><span class={css.itemName}>{account.displayName||account.address}</span><span class={css.meta}>{account.address}</span>
          <Show when={account.status!=='ready'}><span class={css.meta}>{t(account.status==='reauthRequired'?'linked-accounts-reauth':'linked-accounts-unavailable')}</span></Show>
          <Show when={confirm()===account.id}><p class={css.meta}>{t('linked-accounts-unlink-help')}</p></Show>
        </div>
        <div class={css.actions}>
          <Show when={confirm()===account.id} fallback={<>
            <button type="button" class={css.ghost} disabled={props.busy||account.status!=='ready'||props.activeLinkId===account.id||!props.value.policy.enabled} onClick={()=>props.onSwitch(account.id)}>{t('linked-accounts-open')}</button>
            <button type="button" class={css.danger} disabled={props.busy} onClick={()=>setConfirm(account.id)}>{t('linked-accounts-unlink')}</button>
          </>}>
            <button type="button" class={css.danger} disabled={props.busy} onClick={()=>{props.onUnlink(account);setConfirm(undefined);}}>{t('linked-accounts-unlink-confirm')}</button>
            <button type="button" class={css.ghost} disabled={props.busy} onClick={()=>setConfirm(undefined)}>{t('linked-accounts-cancel')}</button>
          </Show>
        </div>
      </li>}</For>
    </ul>
    <div class={css.actions}><button type="button" class={css.button} disabled={props.busy||!props.value.policy.enabled||full()} onClick={props.onAdd}>{t('linked-accounts-add')}</button></div>
  </section>;
}
