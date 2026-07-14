// Admin › Integrations (§19). Live: outbound webhooks + MCP/API-key oversight
// (list + revoke). Inert/deferred (config surface only, → V7): LDAP + Nextcloud,
// shown explicitly as "deferred" so the panel is honest about what is wired.

import { createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { useAdmin } from './context.ts';
import type { ApiKeyInfo, IntegrationsConfig, WebhookInfo } from '../../state/slices/admin.ts';
import * as css from './admin.css.ts';

const DEFAULT_INTEGRATIONS: IntegrationsConfig = {
  webhooks: 'active',
  apiKeyOversight: 'active',
  ldap: 'deferred',
  nextcloud: 'deferred',
};

export function Integrations(): JSX.Element {
  const { api } = useAdmin();
  const [integrations, setIntegrations] = createSignal<IntegrationsConfig>(DEFAULT_INTEGRATIONS);
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
      setError('Could not load integrations');
    }
  }
  onMount(() => void reload());

  async function onRevokeKey(id: string): Promise<void> {
    try {
      await api.revokeApiKey(id);
      await reload();
    } catch {
      setError('Could not revoke the key');
    }
  }

  return (
    <section class={css.section} aria-label="Integrations">
      <h2 class={css.heading}>Integrations</h2>
      <Show when={error()}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>

      <div class={css.card}>
        <div class={css.listRow}>
          <span>LDAP / GAL directory</span>
          <span class={`${css.badge} ${css.badgeDeferred}`}>Deferred</span>
        </div>
        <div class={css.listRow}>
          <span>Nextcloud bridge</span>
          <span class={`${css.badge} ${css.badgeDeferred}`}>Deferred</span>
        </div>
        <p class={css.note}>LDAP and Nextcloud are configuration surfaces only in this release; they are not yet wired.</p>
      </div>

      <div class={css.card}>
        <h3 class={css.heading}>
          Webhooks <span class={css.badge}>{integrations().webhooks === 'active' ? 'Active' : 'Deferred'}</span>
        </h3>
        <Show when={webhooks().length > 0} fallback={<p class={css.note}>No webhooks registered.</p>}>
          <For each={webhooks()}>
            {(w) => (
              <div class={css.listRow}>
                <div>
                  <div class={css.mono}>{w.url}</div>
                  <span class={css.note}>{w.accountId}</span>
                </div>
              </div>
            )}
          </For>
        </Show>
      </div>

      <div class={css.card}>
        <h3 class={css.heading}>
          API &amp; MCP keys{' '}
          <span class={css.badge}>{integrations().apiKeyOversight === 'active' ? 'Active' : 'Deferred'}</span>
        </h3>
        <Show when={keys().length > 0} fallback={<p class={css.note}>No keys issued.</p>}>
          <div class={css.tableWrap}>
            <table class={css.table}>
              <thead>
                <tr>
                  <th>Prefix</th>
                  <th>Account</th>
                  <th>Scopes</th>
                  <th>Status</th>
                  <th />
                </tr>
              </thead>
              <tbody>
                <For each={keys()}>
                  {(k) => (
                    <tr>
                      <td class={css.mono}>{k.prefix}</td>
                      <td>{k.accountId}</td>
                      <td class={css.mono}>{k.scopesJson}</td>
                      <td>{k.revokedAt !== null ? 'revoked' : 'active'}</td>
                      <td>
                        <Show when={k.revokedAt === null}>
                          <button
                            type="button"
                            class="btn btn--ghost"
                            aria-label={`Revoke key ${k.prefix}`}
                            onClick={() => void onRevokeKey(k.id)}
                          >
                            Revoke
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
