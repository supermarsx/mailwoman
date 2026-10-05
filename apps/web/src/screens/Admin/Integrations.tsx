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
//
// Unattended send: a key's owner can ask, when minting it, that mail sent with it
// through MCP skips the Outbox. The request does nothing until an admin approves
// it here (`PUT /admin/api-keys/{id}/unattended-send`, `set_key_unattended_send`
// in `crates/mw-server/src/oauth.rs`). Approving and withdrawing each go through
// a confirmation dialog that names the key, its owner and its scope.

import { createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { useAdmin } from './context.ts';
import {
  AdminApiError,
  type ApiKeyInfo,
  type IntegrationsConfig,
  type WebhookInfo,
} from '../../state/slices/admin.ts';
import { createFocusTrap } from '../../components/a11y';
import { modalOverlay } from '../../components/modalOverlay.css.ts';
import { vars } from '../../theme/contract.css.ts';
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

/** The label for a key's unattended-send state. */
function unattendedLabel(k: ApiKeyInfo): string {
  if (k.unattendedSendApproved) return t('admin-integrations-unattended-approved');
  if (k.unattendedSendRequested) return t('admin-integrations-unattended-requested');
  return t('admin-integrations-unattended-not-requested');
}

/**
 * One sentence naming what a key's scope grants, from the `mw-oauth` `Scope`
 * JSON (`read`/`send`/`delete`, `mail`/`pim`, `mcp_tools`). A scope that does not
 * parse is reported as unreadable, never summarised as empty.
 */
function scopeSummary(scopesJson: string): string {
  let scope: Record<string, unknown>;
  try {
    const parsed: unknown = JSON.parse(scopesJson);
    if (typeof parsed !== 'object' || parsed === null) throw new Error('not an object');
    scope = parsed as Record<string, unknown>;
  } catch {
    return t('admin-integrations-unattended-scope-unreadable');
  }
  const labels: [string, () => string][] = [
    ['read', () => t('admin-integrations-scope-read')],
    ['send', () => t('admin-integrations-scope-send')],
    ['delete', () => t('admin-integrations-scope-delete')],
    ['mail', () => t('admin-integrations-scope-mail')],
    ['pim', () => t('admin-integrations-scope-pim')],
  ];
  const permissions = labels.filter(([name]) => scope[name] === true).map(([, label]) => label());
  const tools = Array.isArray(scope.mcp_tools)
    ? scope.mcp_tools.filter((tool): tool is string => typeof tool === 'string')
    : [];
  const none = t('admin-integrations-scope-none');
  return t('admin-integrations-unattended-scope', {
    permissions: permissions.length > 0 ? permissions.join(', ') : none,
    tools: tools.length > 0 ? tools.join(', ') : none,
  });
}

/** The change an admin is being asked to confirm. */
interface PendingUnattended {
  key: ApiKeyInfo;
  approve: boolean;
}

export function Integrations(): JSX.Element {
  const { api } = useAdmin();
  const [integrations, setIntegrations] = createSignal<IntegrationsConfig>(UNKNOWN_INTEGRATIONS);
  const [webhooks, setWebhooks] = createSignal<WebhookInfo[]>([]);
  const [keys, setKeys] = createSignal<ApiKeyInfo[]>([]);
  const [error, setError] = createSignal<string | null>(null);
  const [notice, setNotice] = createSignal<string | null>(null);
  const [pending, setPending] = createSignal<PendingUnattended | null>(null);
  const [saving, setSaving] = createSignal(false);

  // Focus-trapped confirmation dialog; Esc and Cancel close it without a request.
  const [dialogEl, setDialogEl] = createSignal<HTMLDivElement>();
  createFocusTrap(dialogEl, { active: () => pending() !== null, onEscape: () => setPending(null) });

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

  function askUnattended(key: ApiKeyInfo, approve: boolean): void {
    setNotice(null);
    setError(null);
    setPending({ key, approve });
  }

  async function confirmUnattended(): Promise<void> {
    const change = pending();
    if (change === null) return;
    const { key, approve } = change;
    setSaving(true);
    let failure: string | null = null;
    try {
      await api.setApiKeyUnattendedSend(key.id, approve);
    } catch (e) {
      // The route's refusals (`set_key_unattended_send`): 404 for a key that is
      // gone or was revoked meanwhile, 409 for approving a key that is revoked or
      // did not ask.
      const status = e instanceof AdminApiError ? e.status : 0;
      if (status === 404) failure = t('admin-integrations-unattended-error-gone', { prefix: key.prefix });
      else if (status === 409) failure = t('admin-integrations-unattended-error-conflict', { prefix: key.prefix });
      else failure = t('admin-integrations-unattended-error', { prefix: key.prefix });
    }
    setSaving(false);
    setPending(null);
    // Either way the list is read again, so the row shows what the server holds.
    await reload();
    if (failure !== null) {
      setError(failure);
    } else if (error() === null) {
      setNotice(
        approve
          ? t('admin-integrations-unattended-approved-done', { prefix: key.prefix })
          : t('admin-integrations-unattended-withdrawn-done', { prefix: key.prefix }),
      );
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
      <Show when={notice()}>
        <p class={css.note} role="status" data-testid="integration-unattended-notice">
          {notice()}
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
                  <th>{t('admin-integrations-col-unattended')}</th>
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
                        <span data-testid={`unattended-state-${k.id}`}>{unattendedLabel(k)}</span>{' '}
                        <Show when={k.revokedAt === null && k.unattendedSendApproved}>
                          <button
                            type="button"
                            class="btn btn--ghost"
                            aria-label={t('admin-integrations-unattended-withdraw-key', { prefix: k.prefix })}
                            onClick={() => askUnattended(k, false)}
                          >
                            {t('admin-integrations-unattended-withdraw')}
                          </button>
                        </Show>
                        <Show
                          when={k.revokedAt === null && k.unattendedSendRequested && !k.unattendedSendApproved}
                        >
                          <button
                            type="button"
                            class="btn btn--ghost"
                            aria-label={t('admin-integrations-unattended-approve-key', { prefix: k.prefix })}
                            onClick={() => askUnattended(k, true)}
                          >
                            {t('admin-integrations-unattended-approve')}
                          </button>
                        </Show>
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

      <Show when={pending()}>
        {(change) => (
          <div class={modalOverlay}>
            <div
              ref={setDialogEl}
              role="dialog"
              aria-modal="true"
              aria-labelledby="admin-unattended-title"
              aria-describedby="admin-unattended-meaning"
              tabindex="-1"
              data-testid="admin-unattended-dialog"
              style={{
                display: 'flex',
                'flex-direction': 'column',
                gap: vars.space[4],
                'max-width': '480px',
                width: '100%',
                padding: vars.space[5],
                background: vars.color.surface,
                color: vars.color.text,
                border: `1px solid ${vars.color.border}`,
                'border-radius': vars.radius.lg,
              }}
            >
              <h3 id="admin-unattended-title" class={css.heading}>
                {change().approve
                  ? t('admin-integrations-unattended-approve-title')
                  : t('admin-integrations-unattended-withdraw-title')}
              </h3>
              <div>
                <div dir="auto">{t('admin-integrations-unattended-owner', { account: change().key.accountId })}</div>
                <div class={css.mono} dir="auto">
                  {t('admin-integrations-unattended-key', { prefix: change().key.prefix })}
                </div>
                <div dir="auto">{scopeSummary(change().key.scopesJson)}</div>
              </div>
              <p id="admin-unattended-meaning">
                {change().approve
                  ? t('admin-integrations-unattended-approve-meaning')
                  : t('admin-integrations-unattended-withdraw-meaning')}
              </p>
              <p class={css.note}>
                {change().approve
                  ? t('admin-integrations-unattended-approve-effect')
                  : t('admin-integrations-unattended-withdraw-effect')}
              </p>
              <div style={{ display: 'flex', gap: vars.space[3], 'justify-content': 'flex-end' }}>
                <button
                  type="button"
                  class="btn btn--ghost"
                  data-testid="admin-unattended-cancel"
                  onClick={() => setPending(null)}
                >
                  {t('admin-integrations-unattended-cancel')}
                </button>
                <button
                  type="button"
                  class="btn btn--primary"
                  data-testid="admin-unattended-confirm"
                  disabled={saving()}
                  onClick={() => void confirmUnattended()}
                >
                  {saving()
                    ? t('admin-integrations-unattended-saving')
                    : change().approve
                      ? t('admin-integrations-unattended-approve')
                      : t('admin-integrations-unattended-withdraw')}
                </button>
              </div>
            </div>
          </div>
        )}
      </Show>
    </section>
  );
}
