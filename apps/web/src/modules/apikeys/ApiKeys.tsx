// Scoped API-key management (SPEC §20.1, plan §3 e8): create (with the scope builder),
// list, and revoke keys. The freshly minted secret is SHOWN ONCE — it is displayed
// inline and never re-fetchable; the list holds only the non-secret record.
//
// EXPORTED for e11 to mount; this file does not touch the router or Settings.tsx.

import { createSignal, createResource, For, Show, onMount, type JSX } from 'solid-js';
import { t, isolate, loadCatalog } from '../../i18n';
import { ApiKeyService, type Fetcher } from './service.ts';
import { ScopeBuilder, summarizeScope } from './ScopeBuilder.tsx';
import { readOnlyScope, type ApiKeyRecord, type ApiKeyScope, type MintedKey } from './types.ts';
import * as css from './styles.css.ts';

export interface ApiKeysProps {
  accountId: string;
  fetcher?: Fetcher;
  /** Tests inject an initial list; production fetches. */
  initialKeys?: ApiKeyRecord[];
}

export function ApiKeys(props: ApiKeysProps): JSX.Element {
  onMount(() => void loadCatalog('apikeys'));
  const service = new ApiKeyService(props.fetcher);
  const [keys, { refetch }] = createResource<ApiKeyRecord[]>(() => props.initialKeys ?? service.list());

  const [label, setLabel] = createSignal('');
  const [scope, setScope] = createSignal<ApiKeyScope>(readOnlyScope(props.accountId));
  const [minted, setMinted] = createSignal<MintedKey | null>(null);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');
  const [copied, setCopied] = createSignal(false);

  async function copySecret(secret: string): Promise<void> {
    try {
      await navigator.clipboard?.writeText(secret);
      setCopied(true);
    } catch {
      setCopied(false);
    }
  }

  async function onCreate(): Promise<void> {
    setError('');
    setMinted(null);
    setCopied(false);
    if (label().trim() === '') {
      setError(t('apikeys-error-need-label'));
      return;
    }
    setBusy(true);
    try {
      const result = await service.create({ label: label().trim(), accountId: props.accountId, scope: scope() });
      setMinted(result);
      setLabel('');
      setScope(readOnlyScope(props.accountId));
      await refetch();
    } catch (e) {
      setError(e instanceof Error ? e.message : t('apikeys-error-create'));
    } finally {
      setBusy(false);
    }
  }

  async function onRevoke(prefix: string): Promise<void> {
    setError('');
    try {
      await service.revoke(prefix);
      await refetch();
    } catch (e) {
      setError(e instanceof Error ? e.message : t('apikeys-error-revoke'));
    }
  }

  return (
    <div class={css.panel} aria-label={t('apikeys-panel-label')}>
      <section class={css.section}>
        <h2 class={css.heading}>{t('apikeys-heading')}</h2>
        <p class={css.prose}>{t('apikeys-intro')}</p>

        <label class={css.field}>
          <span class={css.subHeading}>{t('apikeys-label')}</span>
          <input
            class={css.input}
            value={label()}
            placeholder={t('apikeys-label-placeholder')}
            aria-label={t('apikeys-label-aria')}
            onInput={(e) => setLabel(e.currentTarget.value)}
          />
        </label>

        <ScopeBuilder scope={scope()} onChange={setScope} />

        <button type="button" class={css.button} disabled={busy()} onClick={() => void onCreate()}>
          {t('apikeys-create')}
        </button>

        <Show when={minted()}>
          {(m) => (
            <div class={css.field} data-testid="minted-key">
              <p class={css.warn}>{t('apikeys-reveal-warning')}</p>
              <code class={css.token} data-testid="minted-token">
                {m().displayToken}
              </code>
              <div class={css.row}>
                <button
                  type="button"
                  class={css.ghost}
                  aria-label={t('apikeys-copy-aria')}
                  onClick={() => void copySecret(m().displayToken)}
                >
                  {t('apikeys-copy')}
                </button>
                <button type="button" class={css.ghost} onClick={() => setMinted(null)}>
                  {t('apikeys-saved')}
                </button>
              </div>
              <p class={css.copiedNote} role="status" aria-live="polite">
                {copied() ? t('apikeys-copied') : ''}
              </p>
            </div>
          )}
        </Show>

        <Show when={error() !== ''}>
          <p class={css.error} role="alert">
            {error()}
          </p>
        </Show>
      </section>

      <section class={css.section} aria-label={t('apikeys-existing-label')}>
        <span class={css.subHeading}>{t('apikeys-existing')}</span>
        <Show when={(keys() ?? []).length > 0} fallback={<p class={css.prose}>{t('apikeys-none')}</p>}>
          <ul class={css.keyList}>
            <For each={keys()}>
              {(k) => (
                <li class={k.revokedAt ? `${css.keyItem} ${css.revoked}` : css.keyItem}>
                  <div class={css.field}>
                    <strong>{isolate(k.label)}</strong>
                    <span class={css.meta}>
                      mwk_{k.prefix} · {summarizeScope(k.scope).join(' · ')}
                    </span>
                    <span class={css.meta}>
                      {t('apikeys-created', { date: isolate(k.createdAt) })}
                      {k.lastUsedAt
                        ? ` · ${t('apikeys-last-used', { date: isolate(k.lastUsedAt) })}`
                        : ` · ${t('apikeys-never-used')}`}
                      {k.revokedAt ? ` · ${t('apikeys-revoked-at', { date: isolate(k.revokedAt) })}` : ''}
                    </span>
                  </div>
                  <div class={css.row}>
                    <span class={k.revokedAt ? css.statusRevoked : css.statusActive}>
                      {k.revokedAt ? t('apikeys-status-revoked') : t('apikeys-status-active')}
                    </span>
                    <Show when={!k.revokedAt}>
                      <button type="button" class={css.danger} onClick={() => void onRevoke(k.prefix)}>
                        {t('apikeys-revoke')}
                      </button>
                    </Show>
                  </div>
                </li>
              )}
            </For>
          </ul>
        </Show>
      </section>
    </div>
  );
}

export default ApiKeys;
