// Admin maintenance client (t14 26.14, plan §Workstream-3 — JWZ backfill).
//
// The JWZ historical backfill is an admin-gated, explicit ONE-SHOT: re-running the
// shipped full `thread::thread` set algorithm over an account's stored mail and
// re-keying its `thread_id`s (E5 engine driver, exposed by E-mount). Because it
// re-keys conversation grouping it is NEVER automatic — a person selects an account
// and confirms it in the admin panel (see `screens/Admin/RethreadMaintenance.tsx`).
//
// This client mirrors the admin REST shape in `state/slices/admin.ts`: same-origin,
// cookie-authed against the SEPARATE `mw_admin_session` domain (it shares nothing
// with the JMAP client, so the mailbox path is byte-unchanged). It lives in its own
// file rather than `acl-types.ts` (E4) — the endpoint is an admin REST POST, not a
// JMAP method call, so it has no `AclClient`/`jmap` surface in common.
//
// Contract (E-mount, Wave B): `POST /admin/maintenance/rethread`, admin-session
// cookie gated, body `{ "accountId": "<id>" }` → 200 JSON RethreadSummary.

/**
 * The summary the backfill returns: how much of the corpus it touched and how many
 * messages ended up in a DIFFERENT thread than before (`reassigned` — the visible
 * effect the admin is warned about).
 */
export interface RethreadSummary {
  /** Accounts processed (1 for the single-account admin action). */
  accounts: number;
  /** Messages considered across those accounts. */
  messages: number;
  /** Distinct threads the full-set algorithm produced. */
  threads: number;
  /** Messages whose `thread_id` changed (moved to a different conversation). */
  reassigned: number;
}

/** Raised when the maintenance endpoint returns a non-2xx response. */
export class MaintenanceApiError extends Error {
  readonly status: number;
  constructor(status: number, message: string) {
    super(message);
    this.name = 'MaintenanceApiError';
    this.status = status;
  }
}

/**
 * The admin maintenance client. Component tests inject a mock; `createHttpMaintenanceApi`
 * is the production `fetch` impl. `base` lets a native shell point at a remote server
 * (as the admin/JMAP clients do), defaulting to same-origin in the browser.
 */
export interface MaintenanceApi {
  /**
   * `POST /admin/maintenance/rethread` → run the one-shot JWZ backfill for `accountId`
   * and resolve with the {@link RethreadSummary}. Throws {@link MaintenanceApiError}
   * on a non-2xx response (surfaced as the component's honest error state).
   */
  rethread(accountId: string): Promise<RethreadSummary>;
}

/** The production HTTP client (same-origin, admin-session cookie). */
export function createHttpMaintenanceApi(base = ''): MaintenanceApi {
  return {
    async rethread(accountId) {
      const res = await fetch(`${base}/admin/maintenance/rethread`, {
        method: 'POST',
        credentials: 'same-origin',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ accountId }),
      });
      if (!res.ok) {
        throw new MaintenanceApiError(res.status, `rethread failed (${res.status})`);
      }
      return (await res.json()) as RethreadSummary;
    },
  };
}

// ── Search index (t28-e13) ────────────────────────────────────────────────────
//
// `GET /admin/maintenance/search-index` and `POST /admin/maintenance/reindex`,
// both behind the admin session and both engine-mode only (501 in proxy mode).
// Shapes follow `crates/mw-server/src/admin_maintenance.rs` (`search_status_json`
// and the `reindex` handler).

/** What the search index holds, measured against the store. */
export interface SearchIndexStatus {
  /** Documents in the index, all accounts. */
  documents: number;
  /** Stored messages that belong in the index (zero-access accounts are not indexed). */
  messages: number;
  /** Documents held for zero-access accounts; a correct index holds none. */
  zeroAccessDocuments: number;
  /** `true`: the index is on disk. `false`: in memory, rebuilt at every start. */
  persistent: boolean;
  /** Whether a rebuild is running now. */
  rebuilding: boolean;
  /** Messages the running (or last) rebuild has processed. */
  rebuildDone: number;
  /** Messages the running (or last) rebuild set out to process. */
  rebuildTotal: number;
}

/** What one rebuild did. */
export interface ReindexSummary {
  accounts: number;
  /** Stored messages read. */
  messages: number;
  /** Index documents written. */
  indexed: number;
  /** Index documents removed (message gone, or account is zero-access). */
  removed: number;
  /** Zero-access accounts, which are left unindexed. */
  zeroAccess: number;
  /** Stored messages that could not be read back. */
  failed: number;
}

export interface SearchIndexApi {
  /** `GET /admin/maintenance/search-index`. Throws {@link MaintenanceApiError} on non-2xx. */
  status(): Promise<SearchIndexStatus>;
  /**
   * `POST /admin/maintenance/reindex` for every account; resolves when the rebuild
   * has finished. Throws {@link MaintenanceApiError}: 409 while another rebuild runs.
   */
  reindexAll(): Promise<ReindexSummary>;
}

/** The production HTTP client (same-origin, admin-session cookie). */
export function createHttpSearchIndexApi(base = ''): SearchIndexApi {
  return {
    async status() {
      const res = await fetch(`${base}/admin/maintenance/search-index`, {
        credentials: 'same-origin',
      });
      if (!res.ok) {
        throw new MaintenanceApiError(res.status, `search index status failed (${res.status})`);
      }
      return (await res.json()) as SearchIndexStatus;
    },
    async reindexAll() {
      const res = await fetch(`${base}/admin/maintenance/reindex`, {
        method: 'POST',
        credentials: 'same-origin',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({}),
      });
      if (!res.ok) {
        throw new MaintenanceApiError(res.status, `reindex failed (${res.status})`);
      }
      return (await res.json()) as ReindexSummary;
    },
  };
}
