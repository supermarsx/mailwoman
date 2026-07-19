// Send-identity management (t16 e15). An identity is the From/Reply-To/signature
// tuple the composer sends as (maps to JMAP `Identity` server-side). CRUD over the
// account-identities contract; a signature name links to a `Signatures` template.

import { createResource, createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { t, loadCatalog, isolate } from '../../i18n';
import { SettingsService } from './service.ts';
import type { Identity, Signature } from './types.ts';
import * as css from './styles.css.ts';

export interface IdentitiesProps {
  service?: SettingsService;
}

function emptyIdentity(): Identity {
  return { id: '', name: '', email: '' };
}

export function Identities(props: IdentitiesProps): JSX.Element {
  const service = props.service ?? new SettingsService();
  onMount(() => void loadCatalog('settings'));

  const [identities, { refetch }] = createResource<Identity[]>(() => service.listIdentities());
  // Signatures feed the per-identity default-signature picker (best-effort).
  const [signatures] = createResource<Signature[]>(() => service.listSignatures().catch(() => []));
  const [draft, setDraft] = createSignal<Identity>(emptyIdentity());
  const [editing, setEditing] = createSignal(false);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');

  function edit(identity: Identity): void {
    setDraft({ ...identity });
    setEditing(true);
    setError('');
  }

  function startNew(): void {
    setDraft(emptyIdentity());
    setEditing(true);
    setError('');
  }

  function validEmail(email: string): boolean {
    return /^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(email);
  }

  async function save(): Promise<void> {
    const identity = draft();
    if (identity.name.trim() === '' || !validEmail(identity.email.trim())) {
      setError(t('settings-ident-error-fields'));
      return;
    }
    setError('');
    setBusy(true);
    try {
      await service.upsertIdentity({
        ...identity,
        name: identity.name.trim(),
        email: identity.email.trim(),
      });
      setEditing(false);
      setDraft(emptyIdentity());
      await refetch();
    } catch (e) {
      setError(e instanceof Error ? e.message : t('settings-ident-error-generic'));
    } finally {
      setBusy(false);
    }
  }

  async function remove(id: string): Promise<void> {
    setError('');
    setBusy(true);
    try {
      await service.deleteIdentity(id);
      await refetch();
    } catch (e) {
      setError(e instanceof Error ? e.message : t('settings-ident-error-generic'));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section class={css.section} aria-label={t('settings-ident-title')}>
      <h2 class={css.heading}>{t('settings-ident-title')}</h2>
      <p class={css.prose}>{t('settings-ident-intro')}</p>

      <ul class={css.list} data-testid="identity-list">
        <For each={identities() ?? []}>
          {(identity) => (
            <li class={css.item}>
              <div class={css.itemMain}>
                <span class={css.itemName}>{isolate(identity.name)}</span>
                <span class={css.meta}>{isolate(identity.email)}</span>
              </div>
              <div class={css.actions}>
                <button type="button" class={css.ghost} disabled={busy()} onClick={() => edit(identity)}>
                  {t('settings-edit')}
                </button>
                <button type="button" class={css.danger} disabled={busy()} onClick={() => void remove(identity.id)}>
                  {t('settings-delete')}
                </button>
              </div>
            </li>
          )}
        </For>
      </ul>

      <Show
        when={editing()}
        fallback={
          <div class={css.actions}>
            <button type="button" class={css.button} onClick={startNew}>
              {t('settings-ident-new')}
            </button>
          </div>
        }
      >
        <div class={css.field} data-testid="identity-editor">
          <label class={css.field}>
            <span class={css.label}>{t('settings-ident-name-label')}</span>
            <input
              class={css.input}
              aria-label={t('settings-ident-name-label')}
              value={draft().name}
              onInput={(e) => setDraft({ ...draft(), name: e.currentTarget.value })}
            />
          </label>
          <label class={css.field}>
            <span class={css.label}>{t('settings-ident-email-label')}</span>
            <input
              class={css.input}
              type="email"
              autocomplete="email"
              aria-label={t('settings-ident-email-label')}
              value={draft().email}
              onInput={(e) => setDraft({ ...draft(), email: e.currentTarget.value })}
            />
          </label>
          <label class={css.field}>
            <span class={css.label}>{t('settings-ident-replyto-label')}</span>
            <input
              class={css.input}
              type="email"
              aria-label={t('settings-ident-replyto-label')}
              value={draft().replyTo ?? ''}
              onInput={(e) => setDraft({ ...draft(), replyTo: e.currentTarget.value })}
            />
          </label>
          <label class={css.field}>
            <span class={css.label}>{t('settings-ident-signature-label')}</span>
            <select
              class={css.select}
              aria-label={t('settings-ident-signature-label')}
              value={draft().signatureName ?? ''}
              onChange={(e) => {
                const value = e.currentTarget.value;
                const { signatureName: _drop, ...rest } = draft();
                setDraft(value === '' ? rest : { ...rest, signatureName: value });
              }}
            >
              <option value="">{t('settings-ident-signature-none')}</option>
              <For each={signatures() ?? []}>{(sig) => <option value={sig.name}>{sig.name}</option>}</For>
            </select>
          </label>
          <div class={css.actions}>
            <button type="button" class={css.button} disabled={busy()} onClick={() => void save()} data-testid="identity-save">
              {t('settings-save')}
            </button>
            <button type="button" class={css.ghost} disabled={busy()} onClick={() => setEditing(false)}>
              {t('settings-cancel')}
            </button>
          </div>
        </div>
      </Show>

      <Show when={error() !== ''}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>
    </section>
  );
}

export default Identities;
