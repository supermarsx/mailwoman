// Fixture-only copies of public UI request properties. The source-contract test
// rejects drift; importing the UI runtime would initialize browser-only state.
export const HEADER_PROPERTIES=Object.freeze(["id","blobId","from","to","subject","receivedAt","preview","keywords","mailboxIds","threadId","pinned","snoozedUntil","followUpAt","hasAttachment","size","mailwomanFocus","mailwomanAuth","mailwomanPriority","mailwomanMentions"]);
export const BODY_PROPERTIES=Object.freeze(["id","blobId","from","to","cc","bcc","replyTo","subject","receivedAt","sentAt","messageId","inReplyTo","references","preview","htmlBody","textBody","bodyValues","attachments","keywords","mailboxIds","threadId","pinned","snoozedUntil","followUpAt","hasAttachment","size","mailwomanPriority","mailwomanMentions"]);
export const DELIVERY_HEADER_PROPERTIES=Object.freeze(["header:Delivered-To:asAddresses:all","header:X-Original-To:asAddresses:all","header:Envelope-To:asAddresses:all","header:X-Forwarded-To:asAddresses:all","header:Resent-To:asAddresses:all"]);
export const OPTIONAL_HEADER_PROPERTIES=Object.freeze(["header:Expires:asText","header:Expiry-Date:asText","header:Reply-By:asText","header:Sensitivity:asText","header:Supersedes:asText"]);
export const HEADER_PRELOAD_PROPERTIES=Object.freeze(['cc','bcc','replyTo','sentAt','messageId','inReplyTo','references']);
export const DEFAULT_HEADER_PROPERTIES=Object.freeze([...HEADER_PROPERTIES,...HEADER_PRELOAD_PROPERTIES,...DELIVERY_HEADER_PROPERTIES,...OPTIONAL_HEADER_PROPERTIES]);
export const READER_PROPERTIES=Object.freeze([...BODY_PROPERTIES,...DELIVERY_HEADER_PROPERTIES,...OPTIONAL_HEADER_PROPERTIES]);
export const MINIMAL_HEADER_PROPERTIES=Object.freeze(['id','subject','from','receivedAt','preview']);
export const READER_FETCH_OPTIONS=Object.freeze({fetchHTMLBodyValues:true,fetchTextBodyValues:true,maxBodyValueBytes:1000000,mailwomanBodyUse:true});
export function fixturePropertyProfile(kind){
 if(!['baseline','representative'].includes(kind))throw new Error('Unknown fixture property profile');
 const representative=kind==='representative';
 return {headerProfile:representative?'mailwoman-default-preloaded-headers':'minimal-five-field-diagnostic',baselineHeaderProperties:[...(representative?DEFAULT_HEADER_PROPERTIES:MINIMAL_HEADER_PROPERTIES)],pagedHeaderProperties:representative?[...DEFAULT_HEADER_PROPERTIES]:null,reader:representative?{properties:[...READER_PROPERTIES],...READER_FETCH_OPTIONS}:null};
}
