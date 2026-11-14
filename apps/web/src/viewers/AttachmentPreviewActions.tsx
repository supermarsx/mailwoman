import {Show,type JSX} from 'solid-js';
import {AttachmentDetailsButton,type AttachmentDetails} from './AttachmentDetails.tsx';
import {ViewerToolbar,downloadPreview,type ViewerToolAction} from './ViewerToolbar.tsx';
import {Icon} from '../components/Icon.tsx';
import {attachmentActionStyle} from './previewPreferences.ts';
import {t} from '../i18n';
export function AttachmentPreviewActions(props:{url:string;name:string;canDownload:boolean;details?:AttachmentDetails|undefined}):JSX.Element{
 const actions=(open?:()=>void):ViewerToolAction[]=>[
  ...(props.canDownload?[{id:'download',label:t('common-message-preview-download'),icon:'download' as const,run:()=>downloadPreview(props.url,props.name),render:()=> <a class="btn btn--ghost attachment-tab__download" rel="noopener" href={props.url} download={props.name.replace(/[\\/]/g,'_')} aria-label={t('common-message-preview-download')} title={t('common-message-preview-download')}><Show when={attachmentActionStyle()!=='labels'}><Icon name="download"/></Show><Show when={attachmentActionStyle()!=='icons'}><span>{t('common-message-preview-download')}</span></Show></a>}]:[]),
  ...(open?[{id:'details',label:t('mail-file-details-button'),icon:'info' as const,testId:'attachment-details-button',run:open}]:[])];
 return <Show when={props.details} fallback={<ViewerToolbar scope={props.url} actions={actions()}/>}><AttachmentDetailsButton details={props.details!} renderTrigger={open=><ViewerToolbar scope={props.url} actions={actions(open)}/>}/></Show>;
}
