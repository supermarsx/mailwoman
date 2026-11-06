// What the Scoring tab adds beside the spam scores and authentication results:
// how many trackers the reader's sanitiser blocks in this message's HTML, and
// whether the message was used to train the spam filter.
import type {Email} from '../../api/jmap-types.ts';
import {emailGetFull,responseFor} from '../../api/jmap.ts';
import type {EmailGetResponse} from '../../api/jmap-types.ts';
import {analyzeBlockedContent} from '../../api/remote-images.ts';
import {sanitizeBody} from '../../sanitize/index.ts';
import {extractHtmlBody} from '../../state/slices/mail.ts';
import type {Client} from '../../api/client.ts';

/** Trackers the sanitiser marks in `html` (the same count the reader's badge shows); 0 for no HTML. */
export async function trackerCountOfHtml(html:string,sanitize:typeof sanitizeBody=sanitizeBody):Promise<number> {
 if(html==='')return 0;
 const safe=await sanitize({op:'reader',html,blockTrackers:true,cleanLinks:true});
 return analyzeBlockedContent(safe).trackerCount;
}

/** Fetch the message body and count its trackers; `null` when the body cannot be read. */
export async function trackerCountFor(client:Pick<Client,'jmap'>,accountId:string,id:string,sanitize:typeof sanitizeBody=sanitizeBody):Promise<number|null> {
 try{
  const email=responseFor<EmailGetResponse>(await client.jmap(emailGetFull(accountId,id)),'g').list[0];
  return email?await trackerCountOfHtml(extractHtmlBody(email),sanitize):null;
 }catch{return null;}
}

export type TrainingState='spam'|'ham'|'untrained';
/** Whether the message was marked as spam or not spam: the `$junk`/`$notjunk` keywords, then the filter's own autolearn note in `X-Spam-Status`. */
export function trainingState(email:Pick<Email,'keywords'>,spamStatus:string|null):TrainingState {
 const keywords=Object.entries(email.keywords??{}).filter(([,on])=>on).map(([key])=>key.toLowerCase());
 if(keywords.includes('$junk'))return 'spam';
 if(keywords.includes('$notjunk'))return 'ham';
 const learn=/autolearn=(\w+)/i.exec(spamStatus??'')?.[1]?.toLowerCase();
 if(learn==='spam')return 'spam';
 if(learn==='ham')return 'ham';
 return 'untrained';
}
