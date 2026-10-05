// Admin → UI plugins client. The wire shapes are the server's
// (crates/mw-server/src/ui_plugins.rs): `list_admin` (the plugin rows), `RegisterReq`
// and `register` (the upload and its `201` answer), `GrantReq` and `grant`,
// `approve` (which also enables), `enable`, `disable`, `remove`. Every refusal is
// `{ "error": string }`.
//
// Same-origin, cookie-authed against the admin session. The client is an interface
// so the screen is tested without a server.

import { withBase } from '../../../api/basePath.ts';

/** One row of `GET /admin/ui-plugins` (`list_admin`). */
export interface UiPluginInfo {
  readonly id: string;
  readonly name: string;
  readonly version: string;
  readonly enabled: boolean;
  readonly approved: boolean;
  /** The stored row carries a signature. */
  readonly signed: boolean;
  /** Capabilities the manifest declares. The list does not say which are granted. */
  readonly capabilities: string[];
  readonly extensionPoints: string[];
}

/** The `201` answer of `POST /admin/ui-plugins` (`register`). */
export interface UiPluginRegistered {
  readonly id: string;
  readonly signed: boolean;
  /** An unsigned plugin was admitted under `allowUnsigned`. */
  readonly bannerSignal: boolean;
}

/** The capability whose grant carries a host list (`host_allowed`: `{ "hosts": [...] }`). */
export const NET_HOST_ALLOWLIST = 'net:host-allowlist';

/** A refused or failed `/admin/ui-plugins` request; `message` is the server's `error`. */
export class UiPluginsApiError extends Error {
  readonly status: number;
  constructor(status: number, message: string) {
    super(message);
    this.name = 'UiPluginsApiError';
    this.status = status;
  }
}

export interface UiPluginsApi {
  list(): Promise<UiPluginInfo[]>;
  /** `bundle` is the base64 of the bundle file, or null when none was chosen. */
  register(manifest: unknown, bundle: string | null, allowUnsigned: boolean): Promise<UiPluginRegistered>;
  /** Approves and enables. */
  approve(id: string): Promise<void>;
  enable(id: string): Promise<void>;
  disable(id: string): Promise<void>;
  grant(id: string, capability: string, params: Record<string, unknown>): Promise<void>;
  remove(id: string): Promise<void>;
}

export function createHttpUiPluginsApi(): UiPluginsApi {
  async function call(path: string, method: string, body?: unknown): Promise<unknown> {
    const init: RequestInit = { method, credentials: 'same-origin' };
    if (body !== undefined) {
      init.headers = { 'content-type': 'application/json' };
      init.body = JSON.stringify(body);
    }
    const res = await fetch(withBase(`/admin/ui-plugins${path}`), init);
    const answer: unknown = res.status === 204 ? null : await res.json().catch(() => null);
    if (!res.ok) {
      const error = (answer as { error?: unknown } | null)?.error;
      throw new UiPluginsApiError(res.status, typeof error === 'string' ? error : `${method} ${path} (${res.status})`);
    }
    return answer;
  }
  const at = (id: string, action: string): string => `/${encodeURIComponent(id)}/${action}`;
  return {
    list: async () => ((await call('', 'GET')) as { plugins: UiPluginInfo[] }).plugins,
    register: async (manifest, bundle, allowUnsigned) =>
      (await call('', 'POST', bundle === null ? { manifest, allowUnsigned } : { manifest, bundle, allowUnsigned })) as UiPluginRegistered,
    approve: async (id) => {
      await call(at(id, 'approve'), 'POST');
    },
    enable: async (id) => {
      await call(at(id, 'enable'), 'POST');
    },
    disable: async (id) => {
      await call(at(id, 'disable'), 'POST');
    },
    grant: async (id, capability, params) => {
      await call(at(id, 'grant'), 'POST', { capability, params });
    },
    remove: async (id) => {
      await call(`/${encodeURIComponent(id)}`, 'DELETE');
    },
  };
}

/** Base64 of a file's bytes, as `RegisterReq.bundle` expects. */
export async function fileToBase64(file: Blob): Promise<string> {
  const buffer = await new Promise<ArrayBuffer>((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as ArrayBuffer);
    reader.onerror = () => reject(reader.error ?? new Error('file read failed'));
    reader.readAsArrayBuffer(file);
  });
  const bytes = new Uint8Array(buffer);
  let binary = '';
  for (let i = 0; i < bytes.length; i += 0x8000) {
    binary += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  }
  return btoa(binary);
}
