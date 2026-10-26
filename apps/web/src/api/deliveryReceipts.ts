import type {JmapSession} from './jmap-types.ts';
import {CAP_SUBMISSION} from './jmap-types.ts';
/** Capability belongs to this exact account. Missing/unknown never means supported. */
export function supportsDeliveryReceipts(session:JmapSession,accountId:string):boolean {
 const cap=session.accounts?.[accountId]?.accountCapabilities?.[CAP_SUBMISSION];
 if(!cap||typeof cap!=='object'||!('submissionExtensions' in cap))return false;
 const extensions=cap.submissionExtensions;
 return !!extensions&&typeof extensions==='object'&&'DSN' in extensions&&Array.isArray(extensions.DSN);
}
