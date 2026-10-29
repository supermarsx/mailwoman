import { storageAuthorityHeaders } from './storageAuthority.ts';
import { apiFetch } from './http.ts';
import { withBase } from './basePath.ts';

/** The browser analytics tag the administrator enabled (t63), as the consent
 *  control needs it. `consentKey` binds the provider to the exact loader, so a
 *  provider change or a new script asks the user again. */
export interface BrowserTagStatus { provider: string; consentKey: string; consent: boolean }

/** Product names, the same in every language. */
export const TAG_PROVIDER_NAMES: Record<string, string> = {
  plausible: 'Plausible', matomo: 'Matomo', umami: 'Umami', fathom: 'Fathom', posthog: 'PostHog',
  'simple-analytics': 'Simple Analytics', cloudflare: 'Cloudflare Web Analytics',
};

export async function loadBrowserTagStatus(signal?: AbortSignal): Promise<BrowserTagStatus | null> {
  try {
    const response = await apiFetch(withBase('/api/analytics/status'), { ...(signal ? { signal } : {}), headers: storageAuthorityHeaders() });
    if (!response.ok) return null;
    const body = await response.json() as { browserTag?: unknown };
    const tag = body.browserTag as Partial<BrowserTagStatus> | null | undefined;
    if (!tag || typeof tag.provider !== 'string' || typeof tag.consentKey !== 'string') return null;
    if (!/^[a-z-]+:[0-9a-f]{64}$/.test(tag.consentKey)) return null;
    return { provider: tag.provider, consentKey: tag.consentKey, consent: tag.consent === true };
  } catch {
    return null;
  }
}
