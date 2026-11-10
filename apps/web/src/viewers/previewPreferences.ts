import {settings,settingsLoadStatus} from '../api/settings.ts';
export const ATTACHMENT_ACTION_STYLE='reader.attachmentActionStyle';
export const ATTACHMENT_THUMBNAILS='reader.attachmentThumbnails';
export type AttachmentActionStyle='icons'|'labels'|'icons-labels';
export function attachmentActionStyle():AttachmentActionStyle{
 const value=settings()[ATTACHMENT_ACTION_STYLE]?.value;return value==='labels'||value==='icons-labels'?value:'icons';
}
export function attachmentThumbnailsEnabled():boolean{return settingsLoadStatus()==='ready'&&settings()[ATTACHMENT_THUMBNAILS]?.value===true;}
