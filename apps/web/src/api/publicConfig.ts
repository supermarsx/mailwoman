import { apiFetch } from './http.ts';
import { transportBase } from './transport.ts';
import { createSwrCache, swrCachesEnabled } from './swrCache.ts';

export interface PublicConfig {
  brandName: string;
  maintenance?: {enabled:boolean;message:string;retryAfterSecs:number};
  rememberSignins?: boolean;
  showRememberedHint?: boolean;
  showAccountHelpNotice?: boolean;
  logoUrl?: string | null;
  faviconUrl?: string | null;
  loginBackgroundUrl?: string | null;
  theme: string;
  accent: string;
  loginNotice: string | null;
  showDomainHint?: boolean;
  retryCooldownSecs?: number;
  antiBot?: { mailbox: boolean; admin: boolean };
  usernameDomainHint: string | null;
}

async function fetchPublicConfig(base: string): Promise<PublicConfig> {
  const response = await apiFetch(`${base}/api/public-config`, { cache: 'no-store' });
  if (!response.ok) throw new Error('Public configuration is unavailable');
  return response.json() as Promise<PublicConfig>;
}

// t99: the sign-in page, the branding, Settings › Linked mailboxes and the renew
// dialog each read this; one boot asked it twice. It is public (the same for
// every visitor) and changes only when an administrator edits branding or the
// sign-in options, so a read is shared for 30 s and served stale (refreshed in
// the background) for 5 min.
const cache = createSwrCache<PublicConfig>({ freshMs: 30_000, maxAgeMs: 5 * 60_000 });

/** Public hints never contain the domain inventory or an upstream address. */
export async function publicConfig(): Promise<PublicConfig> {
  const base = transportBase();
  if (!swrCachesEnabled()) return fetchPublicConfig(base);
  return cache.get(base, () => fetchPublicConfig(base));
}

/** Read it from the server now (and keep the answer). */
export function refreshPublicConfig(): Promise<PublicConfig> {
  const base = transportBase();
  if (!swrCachesEnabled()) return fetchPublicConfig(base);
  return cache.refresh(base, () => fetchPublicConfig(base));
}

/** Forget the kept copy (tests). */
export function clearPublicConfigCache(): void {
  cache.clear();
}
