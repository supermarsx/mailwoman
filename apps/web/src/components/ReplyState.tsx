import {Show,type JSX} from 'solid-js';
import {Icon} from './Icon.tsx';
import {t} from '../i18n/index.ts';
import * as css from './replyState.css.ts';
import type {ReplyDraft} from './compose/replyDrafts.ts';
export function ReplyDraftIndicator():JSX.Element{return <span class={css.indicator} role="img" aria-label={t('mail-reply-draft')} title={t('mail-reply-draft')} data-testid="row-reply-draft"><Icon name="compose"/></span>;}
export function OpenReplyDraft(props:{draft:ReplyDraft;onOpen:(id:string)=>void}):JSX.Element{
 return <button type="button" class="btn btn--ghost" aria-label={t('mail-reply-draft-open')} title={t('mail-reply-draft-open')} data-reply-draft-id={props.draft.id} onClick={event=>{event.stopPropagation();props.onOpen(props.draft.id);}}><Icon name="compose"/>{t('mail-reply-draft-open')}</button>;
}
export function ReaderReplyState(props:{answered:boolean;forwarded?:boolean;draft:ReplyDraft|undefined;onOpen?:((id:string)=>void)|undefined}):JSX.Element{
 return <Show when={props.answered||props.forwarded||props.draft}><div class={css.reader} data-testid="reader-reply-state"><Show when={props.answered}><span class={css.indicator}><Icon name="reply"/>{t('mail-answered')}</span></Show><Show when={props.forwarded}><span class={css.indicator} data-testid="reader-forwarded"><Icon name="forward"/>{t('mail-forwarded')}</span></Show><Show when={props.draft}>{draft=><div class={css.notice} role="status" data-testid="reply-in-progress-notice" onClick={()=>props.onOpen?.(draft().id)} classList={{[css.noticeClickable]:!!props.onOpen}}><span class={css.noticeText}><Icon name="compose"/>{t('mail-reply-draft')}</span><Show when={props.onOpen}>{open=><OpenReplyDraft draft={draft()} onOpen={open()}/>}</Show></div>}</Show></div></Show>;
}
