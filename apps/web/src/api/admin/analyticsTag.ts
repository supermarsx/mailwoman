import { adminRequest } from '../../screens/Admin/shell/adminApi.ts';

/** Browser analytics tag providers (t63). `custom` is an administrator-supplied
 *  script URL or snippet; every other value is a preset the server builds. */
export const TAG_PROVIDERS = ['plausible', 'matomo', 'umami', 'fathom', 'posthog', 'simple-analytics', 'cloudflare', 'custom'] as const;
export type TagProvider = typeof TAG_PROVIDERS[number];

export interface AnalyticsTagConfig {
  enabled: boolean;
  provider: TagProvider;
  siteId: string;
  host: string;
  region: string;
  customKind: 'url' | 'inline';
  scriptUrl: string;
  snippet: string;
  scriptOrigins: string[];
  connectOrigins: string[];
  imgOrigins: string[];
}
export type AnalyticsTagPreview =
  | { valid: true; csp: { scriptSrc: string[]; connectSrc: string[]; imgSrc: string[] }; path: string; sha256: string; bytes: number }
  | { valid: false; error: string };
export interface AnalyticsTagView { config: AnalyticsTagConfig; preview: AnalyticsTagPreview; maxSnippetBytes: number }
export interface AnalyticsTagTest {
  path: string; sha256: string; bytes: number; served: boolean;
  fetches: { url: string; ok: boolean; bytes?: number; contentType?: string | null; error?: string }[];
}
export interface AnalyticsTagApi {
  read(): Promise<{ data: AnalyticsTagView; etag: string }>;
  save(config: AnalyticsTagConfig & { acknowledgeFullAccess?: boolean }, etag: string): Promise<{ data: AnalyticsTagView; etag: string }>;
  test(config: AnalyticsTagConfig): Promise<AnalyticsTagTest>;
}
export const analyticsTagApi: AnalyticsTagApi = {
  async read() { const r = await adminRequest<AnalyticsTagView>('/analytics/tag'); return { data: r.data, etag: r.etag ?? '' }; },
  async save(config, etag) { const r = await adminRequest<AnalyticsTagView>('/analytics/tag', { method: 'PUT', body: config, ifMatch: etag }); return { data: r.data, etag: r.etag ?? '' }; },
  async test(config) { return (await adminRequest<AnalyticsTagTest>('/analytics/tag/test', { method: 'POST', body: config })).data; },
};
