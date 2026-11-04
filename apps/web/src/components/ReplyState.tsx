import {Show,type JSX} from 'solid-js';
import {Icon} from './Icon.tsx';
import {t} from '../i18n/index.ts';
import * as css from './replyState.css.ts';
import type {ReplyDraft} from './compose/replyDrafts.ts';
export function ReplyDraftIndicator():JSX.Element{return <span class={css.indicator} role="img" aria-label={t('mail-reply-draft')} title={t('mail-reply-draft')} data-testid="row-reply-draft"><Icon name="compose"/></span>;}
export function OpenReplyDraft(props:{draft:ReplyDraft;onOpen:(id:string)=>void;row?:boolean}):JSX.Element{
 return <button type="button" class={props.row?css.rowControl:'btn btn--ghost'} aria-label={t('mail-reply-draft-open')} title={t('mail-reply-draft-open')} data-reply-draft-id={props.draft.id} onClick={event=>{event.stopPropagation();props.onOpen(props.draft.id);}}><Icon name="compose"/><Show when={!props.row}>{t('mail-reply-draft-open')}</Show></button>;
}
export function ReaderReplyState(props:{answered:boolean;draft:ReplyDraft|undefined;onOpen?:((id:string)=>void)|undefined}):JSX.Element{
 return <Show when={props.answered||props.draft}><div class={css.reader} data-testid="reader-reply-state"><Show when={props.answered}><span class={css.indicator}><Icon name="reply"/>{t('mail-answered')}</span></Show><Show when={props.draft}>{draft=><><span class={css.indicator}><Icon name="compose"/>{t('mail-reply-draft')}</span><Show when={props.onOpen}>{open=><OpenReplyDraft draft={draft()} onOpen={open()}/>}</Show></>}</Show></div></Show>;
}
