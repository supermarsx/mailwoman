import { CAP_CORE, CAP_MAIL } from './jmap-types.ts';
import { responseFor } from './jmap.ts';
import { createConfiguredClient, transportBase } from './transport.ts';
import { apiFetch } from './http.ts';
import { fetchMailHeaders, mailHeaderValues } from './mailHeaders.ts';

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

export const MAX_DELIVERY_BYTES = 8 * 1024 * 1024;
/** Root headers only: ordinary mail never causes a whole-message download. */
export function isDeliveryReportHeader(raw: string): boolean {
  const values = mailHeaderValues(raw).get('content-type');
  if (values?.length !== 1) return false;
  return /^multipart\/report\s*(?:;|$)/i.test(values[0]!)
    && /;\s*report-type\s*=\s*(?:"(?:global-)?delivery-status"|(?:global-)?delivery-status)(?:\s*;|\s*$)/i.test(values[0]!);
}
/** Uses the existing authorized blob URL, never a message-supplied remote URL. */
export async function fetchProxyDeliveryReport(url: string, signal: AbortSignal): Promise<DeliveryReport | null> {
  if (!isDeliveryReportHeader(await fetchMailHeaders(url,signal))) return null;
  const source = await apiFetch(url,{signal});
  if (!source.ok || !source.body) throw new Error('Delivery report unavailable');
  const reader = source.body.getReader();
  const chunks: Uint8Array[] = []; let total = 0;
  try {
    const length = Number(source.headers.get('content-length'));
    if (Number.isFinite(length) && length > MAX_DELIVERY_BYTES) throw new Error('Delivery report exceeds limit');
    for (;;) {
      const {done,value} = await reader.read();
      if (value) {
        total += value.byteLength;
        if (total > MAX_DELIVERY_BYTES) throw new Error('Delivery report exceeds limit');
        chunks.push(value);
      }
      if (done) break;
    }
  } finally { await reader.cancel().catch(()=>undefined); }
  if (!total) return null;
  let binary = '';
  for (const chunk of chunks) for (let offset=0;offset<chunk.length;offset+=8192) binary+=String.fromCharCode(...chunk.subarray(offset,offset+8192));
  const response = await apiFetch(`${transportBase()}/api/mail/delivery-status`,{
    method:'POST',signal,headers:{'Content-Type':'application/json'},body:JSON.stringify({bytesBase64:btoa(binary)}),
  });
  if (!response.ok) throw new Error('Delivery report unavailable');
  return deliveryReport((await response.json() as {report:unknown}).report);
}
