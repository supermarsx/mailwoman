/** Attachment classification shared by reader, paperclips and download menus.
 * Only renderable image resources are inline. A PDF/document's Content-ID does
 * not turn it into part of the HTML body; explicit attachment always wins.
 * Keep this rule aligned with mw-mime's is_inline_resource. */
export interface AttachmentMetadata {
 type?: string | null | undefined;
 disposition?: string | null | undefined;
 cid?: string | null | undefined;
 name?: string | null | undefined;
}
export function isInlineResourcePart(part:AttachmentMetadata):boolean {
 const disposition=typeof part.disposition==='string'?part.disposition.trim().toLowerCase():'';
 const mime=typeof part.type==='string'?part.type.trim().toLowerCase():'';
 if(disposition==='attachment')return false;
 if(!mime.startsWith('image/'))return false;
 if(typeof part.cid==='string'&&part.cid.length>0)return true;
 return disposition==='inline'&&!(typeof part.name==='string'&&part.name.length>0);
}
/** Preserve a positive upstream flag and recover a stale false flag only from
 * actual JMAP part metadata. Security verdict filenames never supply blob IDs. */
export function hasFileAttachments(message:{hasAttachment?:boolean;attachments?:readonly AttachmentMetadata[]|null}):boolean {
 return message.hasAttachment===true||(Array.isArray(message.attachments)&&message.attachments.some(part=>part!==null&&typeof part==='object'&&!isInlineResourcePart(part)));
}
