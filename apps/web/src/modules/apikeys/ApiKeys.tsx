// Scoped API-key management (SPEC §20.1, plan §3 e8): create (with the scope builder),
// list, and revoke keys. The freshly minted secret is SHOWN ONCE — it is displayed
// inline and never re-fetchable; the list holds only the non-secret record.
//
// EXPORTED for e11 to mount; this file does not touch the router or Settings.tsx.

import { createSignal, createResource, For, Show, type JSX } from 'solid-js';
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
  const service = new ApiKeyService(props.fetcher);
  const [keys, { refetch }] = createResource<ApiKeyRecord[]>(() => props.initialKeys ?? service.list());

  const [label, setLabel] = createSignal('');
  const [scope, setScope] = createSignal<ApiKeyScope>(readOnlyScope(props.accountId));
  const [minted, setMinted] = createSignal<MintedKey | null>(null);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');

  async function onCreate(): Promise<void> {
    setError('');
    setMinted(null);
    if (label().trim() === '') {
      setError('give the key a label so you can recognise it later');
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
      setError(e instanceof Error ? e.message : 'could not create the key');
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
      setError(e instanceof Error ? e.message : 'could not revoke the key');
    }
  }

  return (
    <div class={css.panel} aria-label="API keys">
      <section class={css.section}>
        <h2 class={css.heading}>API keys</h2>
        <p class={css.prose}>
          Create scoped keys for scripts and integrations. Each key is hashed at rest, shown once,
          and individually revocable. Grant the least scope that works.
        </p>

        <label class={css.field}>
          <span class={css.subHeading}>Label</span>
          <input
            class={css.input}
            value={label()}
            placeholder="e.g. backup script"
            aria-label="Key label"
            onInput={(e) => setLabel(e.currentTarget.value)}
          />
        </label>

        <ScopeBuilder scope={scope()} onChange={setScope} />

        <button type="button" class={css.button} disabled={busy()} onClick={() => void onCreate()}>
          Create key
        </button>

        <Show when={minted()}>
          {(m) => (
            <div class={css.field} data-testid="minted-key">
              <p class={css.warn}>
                Copy this secret now — it is shown once and cannot be retrieved again.
              </p>
              <code class={css.token} data-testid="minted-token">
                {m().displayToken}
              </code>
              <button type="button" class={css.ghost} onClick={() => setMinted(null)}>
                I have saved it
              </button>
            </div>
          )}
        </Show>

        <Show when={error() !== ''}>
          <p class={css.error} role="alert">
            {error()}
          </p>
        </Show>
      </section>

      <section class={css.section} aria-label="Existing keys">
        <span class={css.subHeading}>Existing keys</span>
        <Show when={(keys() ?? []).length > 0} fallback={<p class={css.prose}>No keys yet.</p>}>
          <ul class={css.keyList}>
            <For each={keys()}>
              {(k) => (
                <li class={k.revokedAt ? `${css.keyItem} ${css.revoked}` : css.keyItem}>
                  <div class={css.field}>
                    <strong>{k.label}</strong>
                    <span class={css.meta}>
                      mwk_{k.prefix} · {summarizeScope(k.scope).join(' · ')}
                    </span>
                    <span class={css.meta}>
                      created {k.createdAt}
                      {k.lastUsedAt ? ` · last used ${k.lastUsedAt}` : ' · never used'}
                      {k.revokedAt ? ` · revoked ${k.revokedAt}` : ''}
                    </span>
                  </div>
                  <Show when={!k.revokedAt}>
                    <button type="button" class={css.danger} onClick={() => void onRevoke(k.prefix)}>
                      Revoke
                    </button>
                  </Show>
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
