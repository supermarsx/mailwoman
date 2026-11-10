import type {Email} from '../api/jmap-types.ts';
import {t} from '../i18n';
import {formatLocale} from '../i18n/format.ts';
import {groupThreads,type ThreadVisualRow} from './threads.ts';
import {priorityOf} from './messagePriority.ts';
import type {MessageListGroup} from './messageOrder.ts';
import {computeWindow,type Window} from './virtual.ts';

export const GROUP_HEADER_HEIGHT=32;
export interface MessageGroupHeader {key:string;label:string;count:number;index:number;top:number}
export interface GroupedMessageLayout {rows:ThreadVisualRow[];headers:MessageGroupHeader[];tops:number[];height:number}

function category(email:Email,group:MessageListGroup,dates:{key:Intl.DateTimeFormat;label:Intl.DateTimeFormat}|undefined):{key:string;label:string} {
 if(group==='sender'){
  const sender=email.from?.[0],address=sender?.email??'';
  return {key:address.trim().toLowerCase(),label:sender?.name?`${sender.name} <${address}>`:address||t('mail-unknown-sender')};
 }
 if(group==='subject'){
  const subject=(email.subject??'').trim().replace(/^(?:(?:re|fw|fwd)\s*:\s*)+/i,'').trim();
  return {key:subject.toLowerCase(),label:subject||t('mail-list-group-no-subject')};
 }
 if(group==='importance') {const key=priorityOf(email);return {key,label:t(`mail-list-group-priority-${key}`)};}
 if(group==='size'){
  const bytes=email.size??0,key=bytes<25*1024?'tiny':bytes<100*1024?'small':bytes<1024*1024?'medium':bytes<10*1024*1024?'large':'huge';
  return {key,label:t(`mail-list-group-size-${key}`)};
 }
 const date=new Date((group==='sent'?email.sentAt:null)||email.receivedAt);
 if(!Number.isFinite(date.valueOf()))return {key:'unknown',label:t('mail-list-group-date-unknown')};
 const key=dates!.key.format(date);
 return {key,label:dates!.label.format(date)};
}

/** Categorize whole conversations by their representative message. Expanded
 * children stay with their head; selection and thread actions keep their scope.
 * Server comparators put the grouping field before the chosen order. */
export function groupMessageRows(emails:Email[],expanded:ReadonlySet<string>,group:MessageListGroup,rowHeight:number,timeZone='UTC',conversations=true):GroupedMessageLayout {
 if(group==='none') {const rows=groupThreads(emails,expanded,conversations);return {rows,headers:[],tops:rows.map((_,i)=>i*rowHeight),height:rows.length*rowHeight};}
 const dates=group==='received'||group==='sent'?{key:new Intl.DateTimeFormat('en-CA',{timeZone,year:'numeric',month:'2-digit',day:'2-digit'}),label:new Intl.DateTimeFormat(formatLocale(),{timeZone,dateStyle:'full'})}:undefined;
 const buckets=new Map<string,{key:string;label:string;rows:ThreadVisualRow[];count:number}>();
 let current:{key:string;label:string}|undefined;
 for(const row of groupThreads(emails,expanded,conversations)){
  if(row.kind!=='child'||!current)current=category(row.email,group,dates);
  let bucket=buckets.get(current.key);if(!bucket){bucket={...current,rows:[],count:0};buckets.set(current.key,bucket);}
  bucket.rows.push(row);if(row.kind!=='child')bucket.count+=row.count;
 }
 const rows:ThreadVisualRow[]=[],headers:MessageGroupHeader[]=[],tops:number[]=[];let top=0;
 for(const bucket of buckets.values()){
  headers.push({key:bucket.key,label:bucket.label,count:bucket.count,index:rows.length,top});top+=GROUP_HEADER_HEIGHT;
  for(const row of bucket.rows){rows.push(row);tops.push(top);top+=rowHeight;}
 }
 return {rows,headers,tops,height:top};
}

/** Locate message indices without turning decorative headings into keyboard or
 * selection targets. The unfetched tail retains the normal fixed-height slots. */
export function groupedIndexAt(layout:GroupedMessageLayout,top:number,rowHeight:number):number {
 if(top>=layout.height)return layout.rows.length+Math.floor((top-layout.height)/rowHeight);
 let low=0,high=layout.tops.length;
 while(low<high){const middle=(low+high)>>>1;if(layout.tops[middle]!+rowHeight<=top)low=middle+1;else high=middle;}
 return low;
}
export function groupedRowTop(layout:GroupedMessageLayout,index:number,rowHeight:number):number {
 return layout.tops[index]??layout.height+Math.max(0,index-layout.rows.length)*rowHeight;
}
export function groupedWindow(layout:GroupedMessageLayout,top:number,height:number,rowHeight:number,count:number):Window {
 if(layout.headers.length===0)return computeWindow(top,height,rowHeight,count);
 const totalHeight=layout.height+Math.max(0,count-layout.rows.length)*rowHeight;
 const clamped=Math.max(0,Math.min(top,Math.max(0,totalHeight-height)));
 const startIndex=Math.max(0,groupedIndexAt(layout,clamped,rowHeight)-6);
 const endIndex=Math.min(count,groupedIndexAt(layout,clamped+height,rowHeight)+7);
 return {startIndex,endIndex,offsetY:groupedRowTop(layout,startIndex,rowHeight),totalHeight};
}
