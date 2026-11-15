// Pure wire helpers; safe in a worker without DOM or settings side effects.
import {CAP_CORE,CAP_MAIL,type Id,type Invocation,type JmapRequest,type JmapResponse} from './jmap-types.ts';
const MAIL_USING=[CAP_CORE,CAP_MAIL];
export const COUNTS_CALL_ID='mb';
export function request(using: string[], methodCalls: Invocation[]): JmapRequest {
  return { using, methodCalls };
}

export function updateEmails(accountId: Id, patches: Record<Id, Record<string, unknown>>): JmapRequest {
  return request(MAIL_USING, [
    ['Email/set', { accountId, update: patches }, 'set'],
    ['Mailbox/get', { accountId, ids: null }, COUNTS_CALL_ID],
  ]);
}
export function responseFor<T>(res: JmapResponse, callId: string): T {
  const found = res.methodResponses.find((m) => m[2] === callId);
  if (found === undefined) {
    throw new Error(`no method response for call "${callId}"`);
  }
  if (found[0] === 'error') {
    const err = found[1] as { type?: string; description?: string };
    throw new Error(`JMAP method error: ${err.type ?? 'unknown'} ${err.description ?? ''}`.trim());
  }
  return found[1] as T;
}
