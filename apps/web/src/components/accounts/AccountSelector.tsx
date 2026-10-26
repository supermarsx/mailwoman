import {For,Show,type JSX} from 'solid-js';
import {t} from '../../i18n/index.ts';
import * as css from './accountSelector.css.ts';

export interface MailboxAccountChoice {
  /** Null means primary; every child ID must come from the owned server list. */
  linkId:string|null;
  address:string;
  displayName?:string;
  available:boolean;
}

/** Controlled: the selected address changes only after verified switching. */
export function AccountSelector(props:{
  choices:readonly MailboxAccountChoice[];
  activeLinkId:string|null;
  busy:boolean;
  error?:string;
  onSelect:(linkId:string|null)=>void;
  onManage:()=>void;
}):JSX.Element {
  return <div class={css.control} aria-busy={props.busy}>
    <select class={css.select} aria-label={t('linked-accounts-select')} disabled={props.busy}
      value={props.activeLinkId??''} onChange={event=>{
        const selected=event.currentTarget.value||null;
        event.currentTarget.value=props.activeLinkId??'';
        if(selected===props.activeLinkId||!props.choices.some(choice=>choice.linkId===selected&&choice.available))return;
        props.onSelect(selected);
      }}>
      <For each={props.choices}>{choice=><option value={choice.linkId??''} disabled={!choice.available}>
        {choice.address}{choice.linkId===null?` — ${t('linked-accounts-primary')}`:''}{!choice.available?` — ${t('linked-accounts-unavailable')}`:''}
      </option>}</For>
    </select>
    <button type="button" class={`btn btn--ghost ${css.manage}`} disabled={props.busy} onClick={props.onManage} data-tooltip={t('linked-accounts-manage')} aria-label={t('linked-accounts-manage')}>…</button>
    <Show when={props.error}><span role="alert" class={css.error}>{props.error}</span></Show>
  </div>;
}
