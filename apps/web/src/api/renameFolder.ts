// Rename a folder (t65): "Rename…" in the folder menu, F2 on a focused folder,
// and the folder tab's toolbar.
//
// One standard call, `Mailbox/set` with `update: {id: {name}}` (RFC 8621 §2.5).
// On the native engine (crates/mw-engine/src/folders.rs) that is an IMAP RENAME:
// children move with the folder, subscriptions follow it, the cached folder and
// message ids stay, and the new name is sent in modified UTF-7. The engine
// refuses role folders, a sibling's name, the hierarchy delimiter and folders of
// another account; the checks here only say so before the round trip.
//
// Offered only where it can work: a real folder (no role, not a saved search)
// whose `myRights.mayRename` is true, on an account whose session does not say
// `mailwomanMailboxSet: false` (POP3 and bridge accounts).
import type {Client} from './client.ts';
import {CAP_CORE,CAP_MAIL,type JmapSession,type Mailbox} from './jmap-types.ts';

/** RFC 8621 `maxSizeMailboxName` when the session does not give one. */
export const DEFAULT_MAX_NAME=255;
export interface RenameAbility {manages:boolean;maxLength:number}
export interface RenameAuthority {current:()=>boolean;signal?:AbortSignal}
type Transport=Pick<Client,'jmap'>;

export function renameAbility(session:Pick<JmapSession,'accounts'>|null|undefined,accountId:string):RenameAbility {
 const mail=session?.accounts?.[accountId]?.accountCapabilities?.[CAP_MAIL] as {mailwomanMailboxSet?:unknown;maxSizeMailboxName?:unknown}|undefined;
 const max=mail?.maxSizeMailboxName;
 return {manages:mail?.mailwomanMailboxSet!==false,maxLength:typeof max==='number'&&Number.isSafeInteger(max)&&max>0?max:DEFAULT_MAX_NAME};
}

/** Whether "Rename…" is offered for `box`. */
export function canRename(box:Mailbox,ability:RenameAbility|null):boolean {
 if(!ability?.manages||box.mailwomanSearchQuery!==undefined||box.role!==null)return false;
 return box.myRights?.mayRename===true;
}

export type NameProblem='empty'|'long'|'separator'|'reserved'|'duplicate';

const utf8Length=(value:string):number=>new TextEncoder().encode(value).length;

/** Why `name` cannot be `box`'s new name, or null. Siblings share `box.parentId`;
 *  the comparison is exact, as on the IMAP server (only INBOX is case-blind). */
export function nameProblem(name:string,box:Mailbox,mailboxes:readonly Mailbox[],delimiter:string|null,maxLength:number):NameProblem|null {
 if(name.trim()==='')return 'empty';
 if(utf8Length(name)>maxLength)return 'long';
 if(delimiter&&name.includes(delimiter))return 'separator';
 // Control characters and the LIST wildcards are refused by the engine.
 if(/[\p{Cc}*%]/u.test(name))return 'reserved';
 const siblings=mailboxes.filter(other=>other.id!==box.id&&other.parentId===box.parentId&&other.mailwomanSearchQuery===undefined);
 if(siblings.some(other=>other.name===name||(box.parentId===null&&other.role==='inbox'&&name.toLowerCase()==='inbox')))return 'duplicate';
 return null;
}

/** The server's hierarchy delimiter (`Mailbox/get` response, t65), or null when
 *  it has none or does not say (an upstream JMAP server names folders apart). */
export async function folderDelimiter(client:Transport,accountId:string,signal?:AbortSignal):Promise<string|null> {
 const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Mailbox/get',{accountId,ids:[],properties:['id']},'delim']]},signal?{signal}:undefined);
 const found=response.methodResponses.find(item=>item[2]==='delim');
 const value=found?.[0]==='Mailbox/get'?(found[1] as {mailwomanHierarchyDelimiter?:unknown}).mailwomanHierarchyDelimiter:undefined;
 return typeof value==='string'&&[...value].length===1?value:null;
}

export type RenameOutcome={ok:true}|{ok:false;problem:NameProblem|'forbidden'|'notFound'|'failed';description?:string};

/** Ask the server to rename `mailboxId` to `name`. */
export async function renameFolder(client:Transport,accountId:string,mailboxId:string,name:string,authority:RenameAuthority):Promise<RenameOutcome> {
 if(!authority.current())return {ok:false,problem:'failed'};
 const response=await client.jmap({using:[CAP_CORE,CAP_MAIL],methodCalls:[['Mailbox/set',{accountId,update:{[mailboxId]:{name}}},'rename']]},authority.signal?{signal:authority.signal}:undefined);
 if(!authority.current())return {ok:false,problem:'failed'};
 const found=response.methodResponses.find(item=>item[2]==='rename');
 if(!found||found[0]!=='Mailbox/set')return {ok:false,problem:'failed'};
 const reply=found[1] as {accountId?:string;updated?:Record<string,unknown>|null;notUpdated?:Record<string,{type?:string;description?:string;properties?:string[]}>|null};
 if(reply.accountId!==accountId)return {ok:false,problem:'failed'};
 if(reply.updated&&Object.prototype.hasOwnProperty.call(reply.updated,mailboxId))return {ok:true};
 const refusal=reply.notUpdated?.[mailboxId];
 const description=typeof refusal?.description==='string'?{description:refusal.description}:{};
 switch(refusal?.type){
  case 'alreadyExists':return {ok:false,problem:'duplicate',...description};
  case 'forbidden':return {ok:false,problem:'forbidden',...description};
  case 'notFound':return {ok:false,problem:'notFound',...description};
  case 'invalidProperties':return {ok:false,problem:'reserved',...description};
  default:return {ok:false,problem:'failed',...description};
 }
}
