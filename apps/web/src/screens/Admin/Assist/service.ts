// Admin → Assist client (SPEC §14/§19). Reads and writes the deployment Assist
// configuration and the kill switch.
//
// The wire shape is the server's, and this file follows it; it is described and
// produced in `crates/mw-server/src/v7_mount.rs` (the comment above `snake_to_camel`,
// `assist_admin_wire`, `AssistAdminReq`, `assist_status`). `GET /admin/assist`
// returns the object `PUT /admin/assist` accepts, every key present, camelCase
// throughout. The server refuses a body with a missing, extra or misnamed key, so
// nothing here may rename or drop one.
//
// Same-origin, cookie-authed against the admin session (like the rest of
// `/admin/*`). The transport is injectable so the screen is tested without a server.

import { withBase } from '../../../api/basePath.ts';
import { ASSIST_CAPABILITIES, type AssistCapability } from '../../../modules/assist/types.ts';

/** `mw_assist::AdapterConfig`, keys camelCased. Fields the server defaults are optional on write. */
export type AssistAdapter =
  | {
      readonly kind: 'open-ai-compatible';
      readonly baseUrl: string;
      readonly apiKey?: string;
      readonly chatModel?: string;
      readonly embedModel?: string;
      readonly sttModel?: string;
    }
  | {
      readonly kind: 'anthropic';
      readonly baseUrl?: string;
      readonly apiKey?: string;
      readonly model?: string;
      readonly anthropicVersion?: string;
      readonly maxTokens?: number;
    }
  | { readonly kind: 'local-process'; readonly program: string; readonly args?: readonly string[] };

export type AssistAdapterKind = AssistAdapter['kind'];

/** `mw_assist::DataScope` as the deployment ceiling, keys camelCased. */
export interface DataCeilings {
  /** Accounts whose mail may be sent. Empty means none. */
  readonly accounts: readonly string[];
  /** Folders sending is limited to. Empty means every folder of those accounts. */
  readonly folders: readonly string[];
  readonly includeE2ee: boolean;
  readonly includeAttachments: boolean;
}

/** The body of `GET /admin/assist` and of `PUT /admin/assist`. */
export interface AdminAssistConfig {
  readonly enabled: boolean;
  readonly adapter: AssistAdapter | null;
  readonly capabilityGrants: readonly AssistCapability[];
  readonly dataCeilings: DataCeilings;
}

/** What the running gateway is doing (`GET /admin/assist/status`; also the answer to PUT and kill). */
export interface AdminAssistStatus {
  /** The stored setting. */
  readonly enabled: boolean;
  /** The gateway in the running server answers Assist requests. */
  readonly running: boolean;
  readonly endpointHost: string | null;
  /** The stored configuration is not the one in effect until the server restarts. */
  readonly restartPending: boolean;
}

/** What the server returns when nothing is stored. Used only as the form's starting state. */
export const DEFAULT_ADMIN_ASSIST_CONFIG: AdminAssistConfig = {
  enabled: false,
  adapter: null,
  capabilityGrants: [],
  dataCeilings: { accounts: [], folders: [], includeE2ee: false, includeAttachments: false },
};

type Json = Record<string, unknown>;

function isObject(v: unknown): v is Json {
  return typeof v === 'object' && v !== null && !Array.isArray(v);
}

function stringList(v: unknown, what: string): string[] {
  if (!Array.isArray(v) || !v.every((x) => typeof x === 'string')) {
    throw new Error(`admin assist: ${what} is not a list of strings`);
  }
  return [...(v as string[])];
}

function flag(v: unknown, what: string): boolean {
  if (typeof v !== 'boolean') throw new Error(`admin assist: ${what} is not a boolean`);
  return v;
}

const ADAPTER_KINDS: readonly AssistAdapterKind[] = ['open-ai-compatible', 'anthropic', 'local-process'];

/**
 * Read a `GET /admin/assist` body. Throws on anything that is not the server's
 * shape instead of filling gaps with defaults: a config assembled from defaults
 * and then saved is how a stored configuration gets overwritten.
 */
export function parseAdminAssistConfig(body: unknown): AdminAssistConfig {
  if (!isObject(body)) throw new Error('admin assist: config is not an object');
  const ceilings = body['dataCeilings'];
  if (!isObject(ceilings)) throw new Error('admin assist: dataCeilings is not an object');
  const adapter = body['adapter'];
  if (adapter !== null) {
    if (!isObject(adapter) || !ADAPTER_KINDS.includes(adapter['kind'] as AssistAdapterKind)) {
      throw new Error('admin assist: adapter is neither null nor a known adapter');
    }
  }
  const grants = stringList(body['capabilityGrants'], 'capabilityGrants');
  const unknown = grants.find((g) => !ASSIST_CAPABILITIES.includes(g as AssistCapability));
  if (unknown !== undefined) throw new Error(`admin assist: unknown capability ${unknown}`);
  return {
    enabled: flag(body['enabled'], 'enabled'),
    adapter: adapter === null ? null : ({ ...adapter } as unknown as AssistAdapter),
    capabilityGrants: grants as AssistCapability[],
    dataCeilings: {
      accounts: stringList(ceilings['accounts'], 'dataCeilings.accounts'),
      folders: stringList(ceilings['folders'], 'dataCeilings.folders'),
      includeE2ee: flag(ceilings['includeE2ee'], 'dataCeilings.includeE2ee'),
      includeAttachments: flag(ceilings['includeAttachments'], 'dataCeilings.includeAttachments'),
    },
  };
}

function parseStatus(body: unknown): AdminAssistStatus {
  if (!isObject(body)) throw new Error('admin assist: status is not an object');
  const host = body['endpointHost'];
  if (host !== null && typeof host !== 'string') {
    throw new Error('admin assist: endpointHost is neither null nor a string');
  }
  return {
    enabled: flag(body['enabled'], 'enabled'),
    running: flag(body['running'], 'running'),
    endpointHost: host,
    restartPending: flag(body['restartPending'], 'restartPending'),
  };
}

export type Fetcher = (input: string, init?: RequestInit) => Promise<Response>;
const defaultFetcher: Fetcher = (input, init) => fetch(input, { credentials: 'same-origin', ...init });

/**
 * The Assist admin client.
 *   GET  /admin/assist          → AdminAssistConfig
 *   PUT  /admin/assist          AdminAssistConfig → AdminAssistStatus
 *   GET  /admin/assist/status   → AdminAssistStatus
 *   POST /admin/assist/kill     { on } → AdminAssistStatus   (on: true turns Assist off)
 */
export class AdminAssistApi {
  constructor(private readonly fetcher: Fetcher = defaultFetcher) {}

  async get(): Promise<AdminAssistConfig> {
    const res = await this.fetcher(withBase('/admin/assist'));
    if (!res.ok) throw new Error(`admin assist config failed (${res.status})`);
    return parseAdminAssistConfig(await res.json());
  }

  async status(): Promise<AdminAssistStatus> {
    const res = await this.fetcher(withBase('/admin/assist/status'));
    if (!res.ok) throw new Error(`admin assist status failed (${res.status})`);
    return parseStatus(await res.json());
  }

  async save(config: AdminAssistConfig): Promise<AdminAssistStatus> {
    const res = await this.fetcher(withBase('/admin/assist'), {
      method: 'PUT',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(config),
    });
    if (!res.ok) throw new Error(`admin assist save failed (${res.status})`);
    return parseStatus(await res.json());
  }

  /** `on: true` stops Assist on the running server; `on: false` turns it back on. */
  async setKillSwitch(on: boolean): Promise<AdminAssistStatus> {
    const res = await this.fetcher(withBase('/admin/assist/kill'), {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ on }),
    });
    if (!res.ok) throw new Error(`admin assist kill switch failed (${res.status})`);
    return parseStatus(await res.json());
  }
}
