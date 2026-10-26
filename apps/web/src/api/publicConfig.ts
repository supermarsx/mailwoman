import { apiFetch } from './http.ts';
import { transportBase } from './transport.ts';

export interface PublicConfig {
  brandName: string;
  maintenance?: {enabled:boolean;message:string;retryAfterSecs:number};
  rememberSignins?: boolean;
  showRememberedHint?: boolean;
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

/** Public hints never contain the domain inventory or an upstream address. */
export async function publicConfig(): Promise<PublicConfig> {
  const response = await apiFetch(`${transportBase()}/api/public-config`, { cache: 'no-store' });
  if (!response.ok) throw new Error('Public configuration is unavailable');
  return response.json() as Promise<PublicConfig>;
}
