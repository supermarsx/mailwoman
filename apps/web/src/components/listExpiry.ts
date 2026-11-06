// Expired messages in the message list (`mail.expiry.hideExpired`): dimmed, or
// left out of the list. The row only knows what the list fetch returned; the
// Expires headers are asked for only while the setting is on, so a deployment
// that leaves it off pays nothing.
import {settings} from '../api/settings.ts';
import {factsFromEmail,isExpired,isExpiryMode,type ExpiryMode} from './readerHeaders.ts';
import type {Email} from '../api/jmap-types.ts';

export function listExpiryMode(read:(key:string)=>unknown=key=>settings()[key]?.value):ExpiryMode {
 const value=read('mail.expiry.hideExpired');
 return isExpiryMode(value)?value:'off';
}

/** Whether a row for `email` is expired under the current mode. */
export function rowExpired(email:Pick<Email,'header:Expires:asText'|'header:Expiry-Date:asText'>,now:Date=new Date(),mode:ExpiryMode=listExpiryMode()):boolean {
 if(mode==='off')return false;
 return isExpired(factsFromEmail(email),now);
}

/** The rows to show: expired ones are dropped in `hide` mode, kept (and dimmed by the row) otherwise. */
export function visibleRows<T extends Pick<Email,'header:Expires:asText'|'header:Expiry-Date:asText'>>(rows:readonly T[],now:Date=new Date(),mode:ExpiryMode=listExpiryMode()):readonly T[] {
 return mode==='hide'?rows.filter(row=>!rowExpired(row,now,mode)):rows;
}
