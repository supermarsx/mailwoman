import { createConfiguredClient } from './transport.ts';
import { CAP_CORE, CAP_MAIL, type Email } from './jmap-types.ts';
import { responseFor } from './jmap.ts';
import { deliveryReport, fetchProxyReceiptReports } from './dsn.ts';
import { fetchMailHeaders, mailHeaderValues } from './mailHeaders.ts';
import { buildDownloadUrl } from '../viewers/attachments.ts';

export const RECEIPT_PAGE_SIZE = 40;
const REPORT_SIZE_LIMIT = 512 * 1024;
type Kind = 'read' | 'delivery';
export interface RecipientReport {
  address: string;
  kind: Kind;
  status: string;
  at: number;
  code?: string;
}
export interface ReceiptTarget {
  messageId: string | null;
  recipients: string[];
  after: string | null;
  readRequested: boolean;
  deliveryRequested: boolean;
}
interface ReportMessage {
  id: string;
  blobId?: string;
  size?: number;
  receivedAt: string;
  'header:Content-Type:asText'?: unknown;
  mailwomanMdnReport?: unknown;
  mailwomanDsnReport?: unknown;
}
export interface ReceiptSource {
  accountId: string;
  engine: boolean;
  downloadUrl: string | null;
}
function messageId(value: unknown): string | null {
  if (typeof value !== 'string') return null;
  const id = value.trim().replace(/^<([^<>]+)>$/, '$1');
  return id.length <= 998 && /^[^<>\s@]+@[^<>\s@]+$/.test(id) ? id : null;
}
function mailbox(value: unknown): string | null {
  if (typeof value !== 'string' || value.length > 1000) return null;
  const addr = value.trim();
  const at = addr.lastIndexOf('@');
  if (at <= 0 || at === addr.length - 1) return null;
  return addr.slice(0, at + 1) + addr.slice(at + 1).toLowerCase();
}

export async function receiptTarget(source: ReceiptSource, emailId: string, signal: AbortSignal): Promise<ReceiptTarget> {
  const { accountId, engine, downloadUrl } = source;
  const properties = ['id', 'blobId', 'messageId', 'to', 'cc', 'bcc', 'sentAt', 'receivedAt', 'keywords'];
  properties.push(engine ? 'mailwomanMdn' : 'header:Disposition-Notification-To:asText');
  const response = await createConfiguredClient().jmap({ using: [CAP_CORE, CAP_MAIL],
    methodCalls: [['Email/get', { accountId, ids: [emailId], properties }, 'original']] }, { signal });
  const result = responseFor<{ accountId: string; list: (Email & Record<string, unknown>)[] }>(response, 'original');
  const email = result.list.find(row => row.id === emailId);
  if (result.accountId !== accountId || !email) throw new Error('Sent message unavailable');
  let readRequested = email.keywords?.['$mailwomanReadReceiptRequested'] === true
    || (engine ? !!email['mailwomanMdn'] : !!email['header:Disposition-Notification-To:asText']);
  if (!engine && !readRequested && downloadUrl && email.blobId) {
    try {
      const url = buildDownloadUrl(downloadUrl, { accountId, blobId: email.blobId, name: 'message.eml', mime: 'message/rfc822' });
      const headers = mailHeaderValues(await fetchMailHeaders(url, signal));
      readRequested = headers.get('disposition-notification-to')?.length === 1;
    } catch { signal.throwIfAborted(); }
  }
  const addresses = [...(email.to ?? []), ...(email.cc ?? []), ...(email.bcc ?? [])].flatMap(row => {
    const address = mailbox(row.email);
    return address ? [address] : [];
  });
  const sentAt = Date.parse(email.sentAt ?? email.receivedAt);
  return {
    messageId: email.messageId?.length === 1 ? messageId(email.messageId[0]) : null,
    recipients: [...new Set(addresses)],
    after: Number.isFinite(sentAt) ? new Date(sentAt - 86_400_000).toISOString() : null,
    readRequested,
    deliveryRequested: email.keywords?.['$mailwomanDeliveryReceiptRequested'] === true,
  };
}

/** Match the original Message-ID and a real recipient. Report contents remain claims. */
export function correlateReports(target: ReceiptTarget, receivedAt: string, mdn: unknown, dsn: unknown): RecipientReport[] {
  if (!target.messageId) return [];
  const received = Date.parse(receivedAt);
  if (!Number.isFinite(received)) return [];
  const reports: RecipientReport[] = [];
  const read = mdn as { originalMessageId?: unknown; finalRecipient?: unknown; originalRecipient?: unknown; disposition?: unknown } | null;
  if (read && messageId(read.originalMessageId) === target.messageId) {
    const address = mailbox(read.originalRecipient ?? read.finalRecipient);
    if (address && target.recipients.includes(address) && typeof read.disposition === 'string'
      && ['displayed', 'deleted', 'dispatched', 'processed', 'denied', 'failed'].includes(read.disposition)) {
      reports.push({ address, kind: 'read', status: read.disposition, at: received });
    }
  }
  const delivery = deliveryReport(dsn);
  if (delivery && messageId(delivery.originalMessageId) === target.messageId) {
    for (const row of delivery.recipients) {
      const address = mailbox(row.originalRecipient ?? row.finalRecipient);
      if (!address || !target.recipients.includes(address)) continue;
      const attempt = row.lastAttempt ? Date.parse(row.lastAttempt) : NaN;
      reports.push({ address, kind: 'delivery', status: row.action,
        at: Number.isFinite(attempt) ? attempt : received, code: row.status });
    }
  }
  return reports;
}

export async function receiptPage(source: ReceiptSource, target: ReceiptTarget, position: number, signal: AbortSignal):
  Promise<{ reports: RecipientReport[]; next: number | null; incomplete: boolean }> {
  if (!target.messageId || !target.recipients.length) return { reports: [], next: null, incomplete: false };
  const { accountId, engine, downloadUrl } = source;
  const client = createConfiguredClient();
  const limit = engine ? RECEIPT_PAGE_SIZE : 12;
  const properties = ['id', 'receivedAt', 'blobId', 'size'];
  if (engine) properties.push('mailwomanMdnReport', 'mailwomanDsnReport');
  else properties.push('header:Content-Type:asText');
  const common = { ...(target.after ? { after: target.after } : {}), maxSize: REPORT_SIZE_LIMIT };
  async function query(filter: Record<string, unknown>) {
    const response = await client.jmap({ using: [CAP_CORE, CAP_MAIL], methodCalls: [
      ['Email/query', { accountId, filter, position, limit, sort: [{ property: 'receivedAt', isAscending: false }] }, 'q'],
      ['Email/get', { accountId, '#ids': { resultOf: 'q', name: 'Email/query', path: '/ids' }, properties }, 'reports'],
    ] }, { signal });
    return { ids: responseFor<{ accountId: string; ids: string[] }>(response, 'q'),
      messages: responseFor<{ accountId: string; list: ReportMessage[] }>(response, 'reports') };
  }
  let page;
  if (engine) page = await query(common);
  else {
    try { page = await query({ ...common, header: ['Content-Type', 'multipart/report'] }); }
    catch { signal.throwIfAborted(); page = await query(common); }
  }
  if (page.ids.accountId !== accountId || page.messages.accountId !== accountId
    || page.ids.ids.length > limit || page.messages.list.length > limit) throw new Error('Report query out of scope');
  const reports: RecipientReport[] = [];
  let incomplete = false;
  const candidates = page.messages.list.filter(row => page.ids.ids.includes(row.id) && (row.size ?? Infinity) <= REPORT_SIZE_LIMIT);
  async function collect(row: ReportMessage): Promise<void> {
    if (engine) reports.push(...correlateReports(target, row.receivedAt, row.mailwomanMdnReport, row.mailwomanDsnReport));
    else if (downloadUrl && row.blobId && /^multipart\/report\b/i.test(String(row['header:Content-Type:asText'] ?? ''))) {
      const url = buildDownloadUrl(downloadUrl, { accountId, blobId: row.blobId, name: 'receipt.eml', mime: 'message/rfc822' });
      try {
        const parsed = await fetchProxyReceiptReports(url, AbortSignal.any([signal, AbortSignal.timeout(3_500)]));
        if (parsed) reports.push(...correlateReports(target, row.receivedAt, parsed.mdn, parsed.report));
      } catch { signal.throwIfAborted(); incomplete = true; }
    }
  }
  // Proxy report parsing has its own server-side work limit. Keep the fan-out small.
  for (let offset = 0; offset < candidates.length; offset += 3) {
    await Promise.all(candidates.slice(offset, offset + 3).map(collect));
  }
  return { reports, next: page.ids.ids.length === limit ? position + limit : null, incomplete };
}
