// Admin › Integrations (§19). Outbound webhooks + MCP/API-key oversight (list +
// revoke), and whether this deployment has an LDAP directory and a Nextcloud
// bridge configured.
//
// The LDAP / Nextcloud rows render what `GET /admin/integrations` reports
// (`get_integrations`, `crates/mw-server/src/admin.rs`): `configured`,
// `not-configured` or `unknown`. Until that response arrives, and for any value
// this build does not recognise, the row says the status is unknown. It never
// says an integration is reachable — the server does not connect to either one to
// answer.

import { createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { useAdmin } from './context.ts';
import type { ApiKeyInfo, IntegrationsConfig, WebhookInfo } from '../../state/slices/admin.ts';
import { t } from '../../i18n';
import * as css from './admin.css.ts';

/** Before the server has answered, nothing is known about any of them. */
const UNKNOWN_INTEGRATIONS: IntegrationsConfig = {
  webhooks: 'unknown',
  apiKeyOversight: 'unknown',
  ldap: 'unknown',
  nextcloud: 'unknown',
};

/** The label for a status. An allowlist: anything unrecognised is "unknown". */
function statusLabel(status: string): string {
  switch (status) {
    case 'active':
      return t('admin-integrations-active');
    case 'configured':
      return t('admin-integrations-configured');
    case 'not-configured':
      return t('admin-integrations-not-configured');
    default:
      return t('admin-integrations-unknown');
  }
}

export function Integrations(): JSX.Element {
  const { api } = useAdmin();
  const [integrations, setIntegrations] = createSignal<IntegrationsConfig>(UNKNOWN_INTEGRATIONS);
  const [webhooks, setWebhooks] = createSignal<WebhookInfo[]>([]);
  const [keys, setKeys] = createSignal<ApiKeyInfo[]>([]);
  const [error, setError] = createSignal<string | null>(null);

  async function reload(): Promise<void> {
    try {
      const [i, w, k] = await Promise.all([api.getIntegrations(), api.listWebhooks(), api.listApiKeys()]);
      setIntegrations(i);
      setWebhooks(w);
      setKeys(k);
      setError(null);
    } catch {
      setError(t('admin-integrations-load-error'));
    }
  }
  onMount(() => void reload());

  async function onRevokeKey(id: string): Promise<void> {
    try {
      await api.revokeApiKey(id);
      await reload();
    } catch {
      setError(t('admin-integrations-revoke-error'));
    }
  }

  return (
    <section class={css.section} aria-label={t('admin-integrations-title')}>
      <h2 class={css.heading}>{t('admin-integrations-title')}</h2>
      <Show when={error()}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>

      <div class={css.card}>
        <div class={css.listRow}>
          <span>{t('admin-integrations-ldap')}</span>
          <span class={css.badge} data-testid="integration-ldap" data-status={integrations().ldap}>
            {statusLabel(integrations().ldap)}
          </span>
        </div>
        <div class={css.listRow}>
          <span>{t('admin-integrations-nextcloud')}</span>
          <span class={css.badge} data-testid="integration-nextcloud" data-status={integrations().nextcloud}>
            {statusLabel(integrations().nextcloud)}
          </span>
        </div>
        <p class={css.note}>{t('admin-integrations-config-note')}</p>
      </div>

      <div class={css.card}>
        <h3 class={css.heading}>
          {t('admin-integrations-webhooks')}{' '}
          <span class={css.badge}>{statusLabel(integrations().webhooks)}</span>
        </h3>
        <Show when={webhooks().length > 0} fallback={<p class={css.note}>{t('admin-integrations-webhooks-empty')}</p>}>
          <For each={webhooks()}>
            {(w) => (
              <div class={css.listRow}>
                <div>
                  <div class={css.mono} dir="auto">
                    {w.url}
                  </div>
                  <span class={css.note} dir="auto">
                    {w.accountId}
                  </span>
                </div>
              </div>
            )}
          </For>
        </Show>
      </div>

      <div class={css.card}>
        <h3 class={css.heading}>
          {t('admin-integrations-keys')}{' '}
          <span class={css.badge}>{statusLabel(integrations().apiKeyOversight)}</span>
        </h3>
        <Show when={keys().length > 0} fallback={<p class={css.note}>{t('admin-integrations-keys-empty')}</p>}>
          <div class={css.tableWrap}>
            <table class={css.table}>
              <thead>
                <tr>
                  <th>{t('admin-integrations-col-prefix')}</th>
                  <th>{t('admin-integrations-col-account')}</th>
                  <th>{t('admin-integrations-col-scopes')}</th>
                  <th>{t('admin-integrations-col-status')}</th>
                  <th />
                </tr>
              </thead>
              <tbody>
                <For each={keys()}>
                  {(k) => (
                    <tr>
                      <td class={css.mono} dir="auto">
                        {k.prefix}
                      </td>
                      <td dir="auto">{k.accountId}</td>
                      <td class={css.mono} dir="auto">
                        {k.scopesJson}
                      </td>
                      <td>
                        {k.revokedAt !== null
                          ? t('admin-integrations-status-revoked')
                          : t('admin-integrations-status-active')}
                      </td>
                      <td>
                        <Show when={k.revokedAt === null}>
                          <button
                            type="button"
                            class="btn btn--ghost"
                            aria-label={t('admin-integrations-revoke-key', { prefix: k.prefix })}
                            onClick={() => void onRevokeKey(k.id)}
                          >
                            {t('admin-revoke')}
                          </button>
                        </Show>
                      </td>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
          </div>
        </Show>
      </div>
    </section>
  );
}
