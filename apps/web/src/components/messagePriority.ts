import {settings} from '../api/settings.ts';
import type {Email} from '../api/jmap-types.ts';

export type MessagePriority='high'|'normal'|'low';
export type PriorityMarksMode='both'|'high'|'off';
export type PriorityMarkStyle='icon'|'iconText'|'bar';

/** The server's reading of the sender's priority headers; absent means normal. */
export function priorityOf(email:Pick<Email,'mailwomanPriority'>&Partial<Pick<Email,'mailwomanAuth'>>|null|undefined):MessagePriority {
 const p=email?.mailwomanPriority;
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
