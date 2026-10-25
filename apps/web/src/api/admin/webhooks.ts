// Admin client of the webhooks routes (t28-e18; worker f34). Server:
// `crates/mw-server/src/webhooks.rs`, registered through the admin registry, every
// URL through `adminUrl()` (via the UX seam's `adminRequest`).
//
//   GET    /admin/webhooks       → { webhooks: AdminWebhook[], eventKinds, actionKinds, etag }
//   POST   /admin/webhooks       ← NewWebhook (+ dryRun) → 201 { webhook, secret, etag }
//   DELETE /admin/webhooks/<id>  → 204
//
// Writes send `If-Match` with the list's tag; refusals arrive as
// `AdminProblemError` (`422 {code, detail, field}`, `412 stale`).

import { adminRead, adminRequest, type AdminReadResult } from '../../screens/Admin/shell/adminApi.ts';

/** Outbound (Mailwoman calls a URL) or inbound (a system calls Mailwoman). */
export type WebhookDirection = 'outbound' | 'inbound';

/** One webhook as the admin list shows it. Secrets are never listed. */
export interface AdminWebhook {
  id: string;
  direction: WebhookDirection;
  accountId: string;
  label: string;
  /** Outbound: where deliveries go. */
  url?: string;
  /** Outbound: the event types it receives (`message.received`, …). */
  events?: string[];
  /** Inbound: the action kinds it may run (`run_rules`, `sieve`). */
  actions?: string[];
  createdAt: string;
  createdBy: string;
  /** Inbound: the last accepted call, if any. */
  lastUsedAt?: string | null;
}

export interface WebhookList {
  webhooks: AdminWebhook[];
  eventKinds: string[];
  actionKinds: string[];
  etag: string;
}

export interface NewWebhook {
  direction: WebhookDirection;
  accountId: string;
  label: string;
  url?: string;
  events?: string[];
  actions?: string[];
}

export interface CreatedWebhook {
  webhook: AdminWebhook;
  /** Shown once: the HMAC secret both sides sign with. */
  secret: string;
  etag: string | null;
}

/** `GET /admin/webhooks`; `unavailable` on a server without the route. */
export function listWebhooks(): Promise<AdminReadResult<WebhookList>> {
  return adminRead<WebhookList>('/webhooks');
}

/** `POST /admin/webhooks`. */
export async function createWebhook(body: NewWebhook, ifMatch: string | null): Promise<CreatedWebhook> {
  const r = await adminRequest<{ webhook: AdminWebhook; secret: string }>('/webhooks', {
    method: 'POST',
    body,
    ifMatch,
  });
  return { webhook: r.data.webhook, secret: r.data.secret, etag: r.etag };
}

/** `DELETE /admin/webhooks/<id>`; resolves to the list's new tag. */
export async function deleteWebhook(id: string, ifMatch: string | null): Promise<string | null> {
  const r = await adminRequest<null>(`/webhooks/${encodeURIComponent(id)}`, { method: 'DELETE', ifMatch });
  return r.etag;
}
