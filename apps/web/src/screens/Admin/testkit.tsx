// Shared test helpers for the admin panel component tests (plan §3 e7). NOT a
// test suite itself (vitest only collects `*.{test,spec}`); it builds a fully-
// mocked `AdminApi` + a context-wrapped render so each section is tested against
// a mocked admin surface in isolation.

import { render } from '@solidjs/testing-library';
import { vi } from 'vitest';
import type { JSX } from 'solid-js';
import { AdminContext } from './context.ts';
import {
  createAdminSlice,
  type AdminApi,
  type AdminSlice,
  type IntegrationsConfig,
} from '../../state/slices/admin.ts';

type RenderResult = ReturnType<typeof render>;

/** A fully-stubbed `AdminApi`; pass `overrides` to shape a scenario. */
export function mockAdminApi(overrides: Partial<AdminApi> = {}): AdminApi {
  const base: AdminApi = {
    session: vi.fn(async () => ({ username: 'root' })),
    login: vi.fn(async () => ({ username: 'root' })),
    logout: vi.fn(async () => undefined),
    listDomains: vi.fn(async () => []),
    saveDomain: vi.fn(async () => undefined),
    deleteDomain: vi.fn(async () => undefined),
    listUsers: vi.fn(async () => []),
    provisionUser: vi.fn(async () => undefined),
    setQuota: vi.fn(async () => undefined),
    setFlags: vi.fn(async () => undefined),
    toggleZeroAccess: vi.fn(async () => undefined),
    revokeSessions: vi.fn(async () => 0),
    // `SecurityPolicyDto` (crates/mw-server/src/admin.rs:939): two fields, with
    // `mw_admin::SecurityPolicy::default()`'s values (crates/mw-admin/src/lib.rs).
    getSecurityPolicy: vi.fn(async () => ({
      dlpRulesJson: '[]',
      maxSecurityFloor: false,
    })),
    setSecurityPolicy: vi.fn(async () => undefined),
    // `get_integrations` (crates/mw-server/src/admin.rs:561) for a deployment with
    // no enabled `directory_config` row and no `MW_NEXTCLOUD_*`; the strings are
    // `IntegrationStatus::as_str` (crates/mw-admin/src/provisioning.rs).
    getIntegrations: vi.fn(
      async (): Promise<IntegrationsConfig> => ({
        webhooks: 'active',
        apiKeyOversight: 'active',
        ldap: 'not-configured',
        nextcloud: 'not-configured',
      }),
    ),
    listWebhooks: vi.fn(async () => []),
    listApiKeys: vi.fn(async () => []),
    revokeApiKey: vi.fn(async () => undefined),
    // `get_obs` (crates/mw-server/src/admin.rs). No screen calls this.
    getObservability: vi.fn(async () => ({
      logLevel: 'info',
      otlpDsn: null,
      metricsEnabled: false,
      sentryDsn: null,
    })),
    setObservability: vi.fn(async () => undefined),
    listAudit: vi.fn(async () => []),
    exportAudit: vi.fn(async () => ''),
    listBans: vi.fn(async () => []),
    addBan: vi.fn(async () => undefined),
    removeBan: vi.fn(async () => undefined),
    // `get_appearance` (crates/mw-server/src/admin.rs) with
    // `Appearance::default()` (crates/mw-admin/src/config.rs). No screen calls this.
    getAppearance: vi.fn(async () => ({ theme: 'grove-light', brandName: 'Mailwoman', accent: null })),
    setAppearance: vi.fn(async () => undefined),
  };
  return { ...base, ...overrides };
}

/** Render `ui` inside a provided admin slice built over `api`. */
export function renderWithAdmin(
  ui: () => JSX.Element,
  api: AdminApi = mockAdminApi(),
): RenderResult & { admin: AdminSlice; api: AdminApi } {
  const admin = createAdminSlice(api);
  const utils = render(() => <AdminContext.Provider value={admin}>{ui()}</AdminContext.Provider>);
  return { ...utils, admin, api };
}

/** Let queued microtasks (onMount async loads) settle. */
export async function flush(): Promise<void> {
  await Promise.resolve();
  await Promise.resolve();
}
