import {createSignal} from 'solid-js';
import {settings} from '../api/settings.ts';
import type {Email} from '../api/jmap-types.ts';

export type MessagePriority='high'|'normal'|'low';
export type PriorityMarksMode='both'|'high'|'off';
export type PriorityMarkStyle='icon'|'iconText'|'bar';

// Messages stored before the server read priority headers carry no mark. When such a
// message is opened its raw headers are read here once and the answer is kept for the
// session (and, bounded, in sessionStorage), so the list shows the mark from then on.
const BACKFILL_KEY='mw.priority.backfill';
const BACKFILL_MAX=500;
const backfilled=new Map<string,'high'|'low'|'normal'>();
const [backfillVersion,setBackfillVersion]=createSignal(0);
try{const stored=JSON.parse(sessionStorage.getItem(BACKFILL_KEY)??'{}') as Record<string,unknown>;for(const [id,value] of Object.entries(stored))if(value==='high'||value==='low'||value==='normal')backfilled.set(id,value);}catch{/* storage unavailable: memory only */}

function levelOfWord(value:string):'high'|'low'|'normal'|null {
 const word=value.toLowerCase().replace(/[\d()]/g,' ').trim();
 if(/^(high|highest|urgent|important|emergency)\b/.test(word))return 'high';
 if(/^(low|lowest|non-urgent|nonurgent|not urgent)\b/.test(word))return 'low';
 if(/^(normal|medium|routine)\b/.test(word))return 'normal';
 return null;
}
/** Priority from a raw header block, in the order the server reads it: X-Priority, Importance, X-MSMail-Priority, Priority. `normal` when none says anything. */
export function priorityFromHeaders(raw:string):'high'|'low'|'normal' {
 const head=raw.split(/\r?\n\r?\n/,1)[0]!.replace(/\r?\n[ \t]+/g,' ');
 const get=(name:string)=>new RegExp(`^${name}:[ \\t]*(.*)$`,'im').exec(head)?.[1]?.trim();
 const x=get('X-Priority');
 if(x){const first=x[0];if(first==='1'||first==='2')return 'high';if(first==='4'||first==='5')return 'low';if(first==='3')return 'normal';const word=levelOfWord(x);if(word)return word;}
 for(const name of ['Importance','X-MSMail-Priority','Priority']){const value=get(name);const level=value?levelOfWord(value):null;if(level)return level;}
 return 'normal';
}
/** Remember the priority of an opened message whose list row had none. */
export function backfillPriority(emailId:string,raw:string):void {
 const level=priorityFromHeaders(raw);
 if(backfilled.get(emailId)===level)return;
 if(backfilled.size>=BACKFILL_MAX)backfilled.delete(backfilled.keys().next().value as string);
 backfilled.set(emailId,level);
 try{sessionStorage.setItem(BACKFILL_KEY,JSON.stringify(Object.fromEntries(backfilled)));}catch{/* memory only */}
 setBackfillVersion(value=>value+1);
}
export function forgetBackfilledPriorities():void {backfilled.clear();try{sessionStorage.removeItem(BACKFILL_KEY);}catch{/* ignore */}setBackfillVersion(value=>value+1);}

/** The server's reading of the sender's priority headers; absent means normal. A message the server never read is filled in once it has been opened. */
export function priorityOf(email:Pick<Email,'mailwomanPriority'>&Partial<Pick<Email,'mailwomanAuth'|'id'>>|null|undefined):MessagePriority {
 void backfillVersion();
 const p=email?.mailwomanPriority??(email?.id?backfilled.get(email.id):undefined);
 if(p!=='high'&&p!=='low')return 'normal';
 // `mail.priority.trustSenderHeaders` (admin): a sender can claim any priority, so it can be ignored, or honoured only for mail that passed an aligned DMARC check.
 const trust=settings()['mail.priority.trustSenderHeaders']?.value;
 if(trust==='never')return 'normal';
 if(trust==='trusted'&&!(email?.mailwomanAuth?.dmarc.result==='pass'&&email.mailwomanAuth.dmarc.aligned))return 'normal';
 return p;
}
/** The highest priority among several messages (a conversation row). */
export function highestPriority(emails:ReadonlyArray<Pick<Email,'mailwomanPriority'>&Partial<Pick<Email,'mailwomanAuth'>>>):MessagePriority {
 let out:MessagePriority='normal';
 for(const e of emails){const p=priorityOf(e);if(p==='high')return 'high';if(p==='low')out='low';}
 return out;
}
export function priorityMarksMode():PriorityMarksMode {
 const v=settings()['mail.priorityMarks']?.value;
 return v==='high'||v==='off'?v:'both';
}
export function priorityMarkStyle():PriorityMarkStyle {
 const v=settings()['mail.priorityMarks.style']?.value;
 return v==='iconText'||v==='bar'?v:'icon';
}
/** Whether a mark is drawn for `priority` under the current `mail.priorityMarks`. */
export function showsPriorityMark(priority:MessagePriority,mode:PriorityMarksMode=priorityMarksMode()):boolean {
 if(priority==='normal'||mode==='off')return false;
 return priority==='high'||mode==='both';
}
