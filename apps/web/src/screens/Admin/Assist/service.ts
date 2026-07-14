// V7 Admin → Assist governance client (SPEC §14/§19, plan §3 e6). Drives the
// tenant-wide Assist policy: the endpoint allowlist, per-capability locks, the
// data-class ceilings (E2EE / attachments — default DENY), and the kill switch.
//
// Same-origin, cookie-authed against the admin session domain (like the rest of
// `/admin/*`); it shares nothing with the JMAP client, so the mailbox path is
// unchanged. The transport is injectable so the screen unit-tests without a server.

import type { AssistCapability } from '../../../modules/assist/types.ts';

/** A capability is either usable ('allowed') or disabled tenant-wide ('locked'). */
export type CapabilityLock = 'allowed' | 'locked';

/** The data-class ceiling: what the admin permits to leave the deployment at all. */
export interface DataCeilings {
  /** Permit E2EE-decrypted content to be forwarded. DEFAULT false (safety). */
  readonly includeE2ee: boolean;
  /** Permit attachments to be forwarded. DEFAULT false (safety). */
  readonly includeAttachments: boolean;
}

/** The full governance config (`GET /admin/assist`). */
export interface AdminAssistConfig {
  /** Master kill switch (§19). When false, the gateway reports Disabled to every user. */
  readonly enabled: boolean;
  /** Allowed endpoint hosts; a request to any other host is refused by the gateway. */
  readonly endpointAllowlist: readonly string[];
  /** Per-capability locks. A locked capability is never offered, regardless of grants. */
  readonly capabilityLocks: Readonly<Record<AssistCapability, CapabilityLock>>;
  readonly dataCeilings: DataCeilings;
}

// ── Wire DTOs (snake_case; e9 satisfies under /admin/assist) ───────────────────

interface WireAdminAssistConfig {
  enabled: boolean;
  endpoint_allowlist: string[];
  capability_locks: Record<string, CapabilityLock>;
  data_ceilings: { include_e2ee: boolean; include_attachments: boolean };
}

const ALL_CAPS: readonly AssistCapability[] = [
  'summarize',
  'draft',
  'grammar',
  'dictation',
  'search-semantic',
  'auto-tag',
  'recap',
  'assistant',
];

function locksFromWire(raw: Record<string, CapabilityLock>): Record<AssistCapability, CapabilityLock> {
  const out = {} as Record<AssistCapability, CapabilityLock>;
  for (const cap of ALL_CAPS) out[cap] = raw[cap] ?? 'allowed';
  return out;
}

function configFromWire(w: WireAdminAssistConfig): AdminAssistConfig {
  return {
    enabled: w.enabled,
    endpointAllowlist: [...w.endpoint_allowlist],
    capabilityLocks: locksFromWire(w.capability_locks),
    dataCeilings: {
      includeE2ee: w.data_ceilings.include_e2ee,
      includeAttachments: w.data_ceilings.include_attachments,
    },
  };
}

function configToWire(c: AdminAssistConfig): WireAdminAssistConfig {
  return {
    enabled: c.enabled,
    endpoint_allowlist: [...c.endpointAllowlist],
    capability_locks: { ...c.capabilityLocks },
    data_ceilings: {
      include_e2ee: c.dataCeilings.includeE2ee,
      include_attachments: c.dataCeilings.includeAttachments,
    },
  };
}

/** The safe default: OFF, empty allowlist, everything allowed-but-gated, DENY ceilings. */
export const DEFAULT_ADMIN_ASSIST_CONFIG: AdminAssistConfig = {
  enabled: false,
  endpointAllowlist: [],
  capabilityLocks: locksFromWire({}),
  dataCeilings: { includeE2ee: false, includeAttachments: false },
};

export type Fetcher = (input: string, init?: RequestInit) => Promise<Response>;
const defaultFetcher: Fetcher = (input, init) => fetch(input, { credentials: 'same-origin', ...init });

/**
 * The Assist governance client.
 * Endpoints (e9 fills, e14 mounts):
 *   GET  /admin/assist          → WireAdminAssistConfig
 *   PUT  /admin/assist          → (save)
 *   POST /admin/assist/kill     → { on }   (the §19 kill switch)
 */
export class AdminAssistApi {
  constructor(private readonly fetcher: Fetcher = defaultFetcher) {}

  async get(): Promise<AdminAssistConfig> {
    const res = await this.fetcher('/admin/assist');
    if (!res.ok) throw new Error(`admin assist config failed (${res.status})`);
    return configFromWire((await res.json()) as WireAdminAssistConfig);
  }

  async save(config: AdminAssistConfig): Promise<void> {
    const res = await this.fetcher('/admin/assist', {
      method: 'PUT',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(configToWire(config)),
    });
    if (!res.ok) throw new Error(`admin assist save failed (${res.status})`);
  }

  async setKillSwitch(on: boolean): Promise<void> {
    const res = await this.fetcher('/admin/assist/kill', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ on }),
    });
    if (!res.ok) throw new Error(`admin assist kill switch failed (${res.status})`);
  }
}
