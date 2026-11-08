// What the export dialog decides before any byte moves: which formats a scope
// offers, how files are named, the size estimate against the administrator's
// caps, header privacy for eml/mbox, and the two formats built in the page
// (CSV summary, JSON). Pure; the dialog and exportRun.ts call it.
import {settings} from '../../api/settings.ts';
import type {Email} from '../../api/jmap-types.ts';

export type ExportScope='message'|'conversation'|'selection'|'folder';
export type ServerFormat='eml'|'mbox'|'txt'|'md'|'msg'|'docx';
export type ExportFormatId=ServerFormat|'csv'|'json'|'pdf';
export const SCOPES:readonly ExportScope[]=['message','conversation','selection','folder'];
export const ALL_FORMATS:readonly ExportFormatId[]=['eml','mbox','txt','md','msg','docx','csv','json','pdf'];
const SERVER_FORMATS:readonly ExportFormatId[]=['eml','mbox','txt','md','msg','docx'];
/** One file can hold many messages only in these formats. */
const MULTI_IN_ONE:readonly ExportFormatId[]=['mbox','csv','json'];
export const EXT:Readonly<Record<ExportFormatId,string>>={eml:'eml',mbox:'mbox',txt:'txt',md:'md',msg:'msg',docx:'docx',csv:'csv',json:'json',pdf:'pdf'};
export const MIME:Readonly<Record<ExportFormatId,string>>={eml:'message/rfc822',mbox:'application/mbox',txt:'text/plain',md:'text/markdown',msg:'application/vnd.ms-outlook',docx:'application/vnd.openxmlformats-officedocument.wordprocessingml.document',csv:'text/csv',json:'application/json',pdf:'application/pdf'};
export const isServerFormat=(format:ExportFormatId):format is ServerFormat=>SERVER_FORMATS.includes(format);

export type Packaging='single'|'zip';
export const DEFAULT_PATTERN='{date}-{from}-{subject}';
export const DEFAULT_MAX_NAME=80;
export const MIN_NAME=20;
export const MAX_NAME=200;

// ── Policy ──────────────────────────────────────────────────────────────────
type Read=(key:string)=>unknown;
const live:Read=key=>settings()[key]?.value;
export interface ExportPolicy {
  enabled:boolean;downloadEnabled:boolean;folderEnabled:boolean;formats:readonly ExportFormatId[];
  defaultFormat:ExportFormatId;defaultScope:ExportScope;maxBytes:number;maxMessages:number;
}
const flag=(value:unknown,fallback=true)=>typeof value==='boolean'?value:fallback;
const count=(value:unknown,fallback:number)=>typeof value==='number'&&value>0?value:fallback;
export function exportPolicy(read:Read=live):ExportPolicy {
  const listed=read('export.formats');
  // Unset (settings not loaded yet) means the shipped list: everything.
  const formats=Array.isArray(listed)?ALL_FORMATS.filter(format=>listed.includes(format)):ALL_FORMATS;
  const wantedFormat=read('export.defaultFormat'),wantedScope=read('export.defaultScope');
  const defaultFormat=ALL_FORMATS.find(format=>format===wantedFormat&&formats.includes(format))??formats[0]??'eml';
  return {
    enabled:flag(read('export.dialog.enabled')),
    downloadEnabled:flag(read('export.message.download.enabled')),
    folderEnabled:flag(read('export.folder.enabled')),
    formats,defaultFormat,
    defaultScope:SCOPES.find(scope=>scope===wantedScope)??'message',
    maxBytes:count(read('export.folder.maxBytes'),2147483648),
    maxMessages:count(read('export.folder.maxMessages'),100000),
  };
}

// ── Scope and formats ───────────────────────────────────────────────────────
export interface ScopeSource {
  /** Loaded rows per scope; a scope that is absent is not offered. */
  message:readonly Email[];conversation?:readonly Email[];selection?:readonly Email[];
  folder?:{id:string;name:string;hasSubfolders?:boolean};
}
export function offeredScopes(source:ScopeSource,policy:Pick<ExportPolicy,'folderEnabled'>):ExportScope[] {
  const scopes:ExportScope[]=[];
  if(source.message.length>0)scopes.push('message');
  if((source.conversation?.length??0)>1)scopes.push('conversation');
  if((source.selection?.length??0)>1)scopes.push('selection');
  if(source.folder&&policy.folderEnabled)scopes.push('folder');
  return scopes;
}
export function startScope(scopes:readonly ExportScope[],wanted:ExportScope):ExportScope {
  return scopes.includes(wanted)?wanted:scopes[0]??'message';
}
/** Formats the scope can produce, in the administrator's list. A folder is a server ZIP of .eml or mbox only; PDF is a print hint for one message. */
export function scopeFormats(scope:ExportScope,policy:Pick<ExportPolicy,'formats'|'downloadEnabled'>):ExportFormatId[] {
  if(!policy.downloadEnabled)return [];
  return policy.formats.filter(format=>scope==='folder'?format==='eml'||format==='mbox':format!=='pdf'||scope==='message');
}
export function packagings(scope:ExportScope,format:ExportFormatId,messages:number):Packaging[] {
  if(format==='pdf')return [];
  if(scope==='folder')return ['zip'];
  if(messages<=1)return ['single','zip'];
  return MULTI_IN_ONE.includes(format)?['single','zip']:['zip'];
}

// ── File names ──────────────────────────────────────────────────────────────
// Control characters are what is being replaced here.
// eslint-disable-next-line no-control-regex
const UNSAFE=/[\\/:*?"<>|\u0000-\u001f\u007f]+/g;
const RESERVED=/^(con|prn|aux|nul|com[1-9]|lpt[1-9])$/i;
/** A name safe on every file system: no separators or control characters, no leading dots, trailing dots or spaces, no reserved device names, cut to `max` characters. */
export function sanitizeName(name:string,max=DEFAULT_MAX_NAME):string {
  const limit=Math.max(MIN_NAME,Math.min(MAX_NAME,Math.floor(max)||DEFAULT_MAX_NAME));
  let clean=name.normalize('NFC').replace(UNSAFE,'_').replace(/\s+/g,' ').trim();
  clean=Array.from(clean).slice(0,limit).join('').replace(/^[. ]+/,'').replace(/[. ]+$/,'');
  if(clean==='')return 'message';
  return RESERVED.test(clean)?`_${clean}`:clean;
}
const datePart=(email:Pick<Email,'receivedAt'>)=>{const time=Date.parse(email.receivedAt??'');return Number.isFinite(time)?new Date(time).toISOString().slice(0,10):'undated';};
const senderPart=(email:Pick<Email,'from'>)=>{const from=email.from?.[0];return from?(from.name||from.email):'unknown';};
/** Expand `{date}`, `{from}`, `{subject}`, `{id}`, `{n}` for one message; `n` is its 1-based position. */
export function fileStem(pattern:string,email:Pick<Email,'id'|'receivedAt'|'from'|'subject'>,n:number,max=DEFAULT_MAX_NAME):string {
  const tokens:Record<string,string>={date:datePart(email),from:senderPart(email),subject:email.subject?.trim()||'no subject',id:email.id,n:String(n)};
  const expanded=(pattern.trim()===''?DEFAULT_PATTERN:pattern).replace(/\{(date|from|subject|id|n)\}/g,(_,key:string)=>tokens[key]!);
  return sanitizeName(expanded,max);
}
/** Names for a list of messages: stem plus extension, numbered where two would collide (case-insensitively). */
export function fileNames(emails:readonly Pick<Email,'id'|'receivedAt'|'from'|'subject'>[],pattern:string,ext:string,max=DEFAULT_MAX_NAME):string[] {
  const seen=new Map<string,number>();
  return emails.map((email,index)=>{
    const stem=fileStem(pattern,email,index+1,max),key=stem.toLowerCase(),times=(seen.get(key)??0)+1;
    seen.set(key,times);
    return `${times===1?stem:`${stem} (${times})`}.${ext}`;
  });
}

// ── Estimate and caps ───────────────────────────────────────────────────────
export interface Estimate {messages:number;bytes:number;allowed:boolean;reason?:'tooManyMessages'|'tooLarge'|'nothing'}
export function estimateFor(emails:readonly {size?:number|undefined}[],policy:Pick<ExportPolicy,'maxBytes'|'maxMessages'>):Estimate {
  const bytes=emails.reduce((sum,email)=>sum+(typeof email.size==='number'&&Number.isFinite(email.size)&&email.size>0?email.size:0),0),messages=emails.length;
  if(messages===0)return {messages,bytes,allowed:false,reason:'nothing'};
  if(messages>policy.maxMessages)return {messages,bytes,allowed:false,reason:'tooManyMessages'};
  if(bytes>policy.maxBytes)return {messages,bytes,allowed:false,reason:'tooLarge'};
  return {messages,bytes,allowed:true};
}

// ── Header privacy (eml and mbox) ───────────────────────────────────────────
export interface PrivacyOptions {stripReceived:boolean;redactBcc:boolean}
/** Drop `Received`/`Bcc` header fields (with their folded lines) from the header block of an RFC 5322 message; the body is copied untouched. A leading mbox `From ` line is kept. A signed message stops verifying. */
export function stripHeaders(bytes:Uint8Array,options:PrivacyOptions):Uint8Array {
  const drop=[...(options.stripReceived?['received']:[]),...(options.redactBcc?['bcc']:[])];
  if(drop.length===0)return bytes;
  // Latin-1 keeps every byte one character, so the round trip is lossless.
  let text='';
  for(let i=0;i<bytes.length;i+=8192)text+=String.fromCharCode(...bytes.subarray(i,i+8192));
  const blank=/(?:^|\n)\r?\n/.exec(text);
  const headEnd=blank?blank.index+blank[0].length:text.length;
  let skipping=false;const kept:string[]=[];
  for(const line of text.slice(0,headEnd).split(/(?<=\n)/)){
    if(/^\r?\n$/.test(line)){kept.push(line);continue;}
    if(/^[ \t]/.test(line)){if(!skipping)kept.push(line);continue;}
    const name=/^([^:\s]+):/.exec(line)?.[1]?.toLowerCase();
    skipping=name!==undefined&&drop.includes(name);
    if(!skipping)kept.push(line);
  }
  const out=kept.join('')+text.slice(headEnd);
  const result=new Uint8Array(out.length);
  for(let i=0;i<out.length;i++)result[i]=out.charCodeAt(i)&0xff;
  return result;
}

// ── Formats built in the page ───────────────────────────────────────────────
/** A byte order mark, so a spreadsheet reads the file as UTF-8. */
const BOM=String.fromCharCode(0xfeff);
const csvCell=(value:string)=>{
  // A cell that starts like a formula is prefixed so a spreadsheet shows it as text.
  const safe=/^[=+\-@\t\r]/.test(value)?`'${value}`:value;
  return /[",\r\n]/.test(safe)?`"${safe.replace(/"/g,'""')}"`:safe;
};
const addresses=(list:readonly {name?:string|null;email:string}[]|null|undefined)=>(list??[]).map(a=>a.name?`${a.name} <${a.email}>`:a.email).join('; ');
export function buildCsv(emails:readonly Email[]):string {
  const rows=[['date','from','to','subject','size'],...emails.map(e=>[e.receivedAt??'',addresses(e.from),addresses(e.to),e.subject??'',String(e.size??0)])];
  return `${BOM}${rows.map(row=>row.map(csvCell).join(',')).join('\r\n')}\r\n`;
}
export function buildJson(emails:readonly Email[],options:Pick<PrivacyOptions,'redactBcc'>={redactBcc:false}):string {
  return `${JSON.stringify(emails.map(e=>({
    id:e.id,threadId:e.threadId,receivedAt:e.receivedAt,subject:e.subject,from:e.from,to:e.to,cc:e.cc,
    ...(options.redactBcc?{}:{bcc:e.bcc}),size:e.size,hasAttachment:e.hasAttachment,keywords:e.keywords,mailboxIds:e.mailboxIds,
  })),null,2)}\n`;
}
