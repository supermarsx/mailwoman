// API-key / MCP-key server I/O (plan §3 e8). Talks to the key endpoints e11 mounts;
// the transport is injectable so components unit-test without a live server. The mint
// response carries the shown-ONCE display token — it is never re-fetchable.

import { scopeToWire, type ApiKeyRecord, type ApiKeyScope, type MintedKey } from './types.ts';

export type Fetcher = (input: string, init?: RequestInit) => Promise<Response>;

const defaultFetcher: Fetcher = (input, init) => fetch(input, { credentials: 'same-origin', ...init });

async function jsonOrThrow<T>(res: Response): Promise<T> {
  if (!res.ok) throw new Error(`api-key request failed: ${res.status}`);
  return (await res.json()) as T;
}

/** The create-key request body (label + the wire scope). */
export interface CreateKeyRequest {
  readonly label: string;
  readonly accountId: string;
  readonly scope: ApiKeyScope;
}

/**
 * The API-key service backing the create/list/revoke UI.
 * Endpoints (e11 to satisfy):
 *   GET    /api/keys                    → ApiKeyRecord[]
 *   POST   /api/keys   (CreateKeyBody)  → MintedKey  (display token shown once)
 *   POST   /api/keys/:prefix/revoke     → { ok:true }
 */
export class ApiKeyService {
  constructor(private readonly fetcher: Fetcher = defaultFetcher) {}

  async list(): Promise<ApiKeyRecord[]> {
    const res = await this.fetcher('/api/keys');
    return jsonOrThrow<ApiKeyRecord[]>(res);
  }

  async create(req: CreateKeyRequest): Promise<MintedKey> {
    const body = { label: req.label, accountId: req.accountId, scope: scopeToWire(req.scope) };
    const res = await this.fetcher('/api/keys', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(body),
    });
    return jsonOrThrow<MintedKey>(res);
  }

  async revoke(prefix: string): Promise<void> {
    const res = await this.fetcher(`/api/keys/${encodeURIComponent(prefix)}/revoke`, { method: 'POST' });
    if (!res.ok) throw new Error(`revoke failed: ${res.status}`);
  }
}
