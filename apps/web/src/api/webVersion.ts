// `GET /api/version` (session required, `no-store`): which web build the server
// serves now. Used by the "new version available" notice (state/newVersionNotice.ts).

import { withBase } from './basePath.ts';
import { apiFetch } from './http.ts';

export interface WebVersion {
  /** The server binary's version. */
  version: string;
  /** Hash of the served web bundle's file names; changes on every redeploy. */
  webBuild: string;
}

/** The server's current build id, or null when signed out or the call failed. */
export async function fetchWebVersion(signal?: AbortSignal): Promise<WebVersion | null> {
  try {
    const res = await apiFetch(withBase('/api/version'), { cache: 'no-store', ...(signal ? { signal } : {}) });
    if (!res.ok) return null;
    const body: unknown = await res.json();
    if (typeof body !== 'object' || body === null) return null;
    const { version, webBuild } = body as Record<string, unknown>;
    return typeof version === 'string' && typeof webBuild === 'string' ? { version, webBuild } : null;
  } catch {
    return null;
  }
}

/** The id compared between the booted page and the server. */
export const buildId = (v: WebVersion): string => `${v.version}+${v.webBuild}`;
