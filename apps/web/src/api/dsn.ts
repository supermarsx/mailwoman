import { CAP_CORE, CAP_MAIL } from './jmap-types.ts';
import { responseFor } from './jmap.ts';
import { createConfiguredClient } from './transport.ts';

export type DeliveryAction = 'failed' | 'delayed' | 'delivered' | 'relayed' | 'expanded';
export interface DeliveryRecipient {
  finalRecipient: string; addressType: string; originalRecipient: string | null;
  action: DeliveryAction; status: string; diagnostic: string | null;
  remoteMta: string | null; lastAttempt: string | null; retryUntil: string | null;
}
export interface DeliveryReport {
  reportingMta: string; envelopeId: string | null; arrivalDate: string | null;
  recipients: DeliveryRecipient[];
}
const text = (value: unknown): value is string => typeof value === 'string' && value.length <= 4096;
const optional = (value: unknown): value is string | null => value === null || text(value);
export function deliveryReport(value: unknown): DeliveryReport | null {
  if (!value || typeof value !== 'object') return null;
  const r = value as DeliveryReport;
  if (!text(r.reportingMta) || !optional(r.envelopeId) || !optional(r.arrivalDate) || !Array.isArray(r.recipients) || !r.recipients.length || r.recipients.length > 64) return null;
  for (const row of r.recipients) {
    if (!row || !text(row.finalRecipient) || !text(row.addressType) || !['failed','delayed','delivered','relayed','expanded'].includes(row.action)
      || !text(row.status) || !/^[245]\.\d{1,3}\.\d{1,3}$/.test(row.status)
      || ![row.originalRecipient,row.diagnostic,row.remoteMta,row.lastAttempt,row.retryUntil].every(optional)) return null;
  }
  return r;
}
/** Engine-only explicit property; callers must not assume an upstream supports it. */
export async function fetchDeliveryReport(accountId: string, emailId: string): Promise<DeliveryReport | null> {
  const response = await createConfiguredClient().jmap({ using:[CAP_CORE,CAP_MAIL], methodCalls:[['Email/get',{accountId,ids:[emailId],properties:['id','mailwomanDsnReport']},'dsn']] });
  const email = responseFor<{list:Record<string,unknown>[]}>(response,'dsn').list.find(row=>row['id']===emailId);
  return deliveryReport(email?.['mailwomanDsnReport']);
}
