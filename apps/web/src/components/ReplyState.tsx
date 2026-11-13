import {requestDraftPreview} from './compose/draftPreview.ts';
import {Show,createSignal,type JSX} from 'solid-js';
import {Icon} from './Icon.tsx';
import {t} from '../i18n/index.ts';
import * as css from './replyState.css.ts';
import {discardReplyDraft,type ReplyDraft} from './compose/replyDrafts.ts';
import {preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import {createConfirm} from './confirm.tsx';
import {toast} from './toast/toast.ts';
export function ReplyDraftIndicator():JSX.Element{return <span class={css.indicator} role="img" aria-label={t('mail-reply-draft')} title={t('mail-reply-draft')} data-testid="row-reply-draft"><Icon name="compose"/></span>;}
export function OpenReplyDraft(props:{draft:ReplyDraft;onOpen:(id:string)=>void}):JSX.Element{
 return <button type="button" class="btn btn--ghost" aria-label={t('mail-reply-draft-open')} title={t('mail-reply-draft-open')} data-reply-draft-id={props.draft.id} onClick={event=>{event.stopPropagation();props.onOpen(props.draft.id);}}><Icon name="compose"/></button>;
}
export function ReaderReplyState(props:{answered:boolean;forwarded?:boolean;draft:ReplyDraft|undefined;onOpen?:((id:string)=>void)|undefined}):JSX.Element{
 const confirm=createConfirm();const [busy,setBusy]=createSignal(false);
 async function discard():Promise<void>{
  const draft=props.draft,owner=preferenceAccount();if(!draft||!owner||busy())return;
  if(!await confirm.ask({title:t('mail-reply-draft-discard-title'),body:t('mail-reply-draft-discard-detail'),confirmLabel:t('mail-draft-discard-action'),cancelLabel:t('common-cancel'),testId:'reply-discard-confirm'}))return;
  if(props.draft?.id!==draft.id||!preferenceAccountIsCurrent(owner))return;
  setBusy(true);try{await discardReplyDraft(owner,draft);}catch{if(preferenceAccountIsCurrent(owner))toast.error(t('mail-draft-discard-pending'));}finally{setBusy(false);}
 }
 return <><confirm.Dialog/><Show when={props.answered||props.forwarded||props.draft}><div class={css.reader} data-testid="reader-reply-state"><Show when={props.answered}><span class={css.indicator}><Icon name="reply"/>{t('mail-answered')}</span></Show><Show when={props.forwarded}><span class={css.indicator} data-testid="reader-forwarded"><Icon name="forward"/>{t('mail-forwarded')}</span></Show><Show when={props.draft}>{draft=><div class={css.notice} role="status" data-testid="reply-in-progress-notice" onClick={()=>props.onOpen?.(draft().id)} classList={{[css.noticeClickable]:!!props.onOpen}}><span class={css.noticeText}><Icon name="compose"/>{t('mail-reply-draft')}</span><span class={css.noticeActions}><button type="button" class="btn btn--ghost" aria-label={t('mail-reply-draft-view')} title={t('mail-reply-draft-view')} data-testid="reader-view-reply" onClick={event=>{event.stopPropagation();requestDraftPreview(draft().id);}}><Icon name="eye"/></button><Show when={props.onOpen}>{open=><OpenReplyDraft draft={draft()} onOpen={open()}/>}</Show><button type="button" class="btn btn--ghost" aria-label={t('mail-reply-draft-discard')} title={t('mail-reply-draft-discard')} disabled={busy()} data-testid="reader-discard-reply" onClick={event=>{event.stopPropagation();void discard();}}><Icon name="delete"/></button></span></div>}</Show></div></Show></>;
}
