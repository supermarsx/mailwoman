// Signature templates CRUD (t16 e15 — W12). Backed by the `mw-store` 0017
// `signatures` rows via the account-signatures contract. One default may be set;
// a signature carries an optional selection rule (JSON) the composer honours.

import { createResource, createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { t, loadCatalog } from '../../i18n';
import { SettingsService } from './service.ts';
import type { Signature } from './types.ts';
import * as css from './styles.css.ts';

export interface SignaturesProps {
  service?: SettingsService;
}

const EMPTY: Signature = { name: '', body: '', isDefault: false };

export function Signatures(props: SignaturesProps): JSX.Element {
  const service = props.service ?? new SettingsService();
  onMount(() => void loadCatalog('settings'));

  const [signatures, { refetch }] = createResource<Signature[]>(() => service.listSignatures());
  const [draft, setDraft] = createSignal<Signature>({ ...EMPTY });
  const [editing, setEditing] = createSignal(false);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');

  function edit(sig: Signature): void {
    setDraft({ ...sig });
    setEditing(true);
    setError('');
  }

  function startNew(): void {
    setDraft({ ...EMPTY });
    setEditing(true);
    setError('');
  }

  async function save(): Promise<void> {
    const sig = draft();
    if (sig.name.trim() === '') {
      setError(t('settings-sig-error-name'));
      return;
    }
    setError('');
    setBusy(true);
    try {
      await service.upsertSignature({ ...sig, name: sig.name.trim() });
      setEditing(false);
      setDraft({ ...EMPTY });
      await refetch();
    } catch (e) {
      setError(e instanceof Error ? e.message : t('settings-sig-error-generic'));
    } finally {
      setBusy(false);
    }
  }

  async function remove(name: string): Promise<void> {
    setError('');
    setBusy(true);
    try {
      await service.deleteSignature(name);
      await refetch();
    } catch (e) {
      setError(e instanceof Error ? e.message : t('settings-sig-error-generic'));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section class={css.section} aria-label={t('settings-sig-title')}>
      <h2 class={css.heading}>{t('settings-sig-title')}</h2>
      <p class={css.prose}>{t('settings-sig-intro')}</p>

      <ul class={css.list} data-testid="signature-list">
        <For each={signatures() ?? []}>
          {(sig) => (
            <li class={css.item}>
              <div class={css.itemMain}>
                <span class={css.itemName}>
                  {sig.name}
                  <Show when={sig.isDefault}>
                    {' '}
                    <span class={css.badge}>{t('settings-sig-default')}</span>
                  </Show>
                </span>
                <span class={css.meta}>{sig.body.slice(0, 80)}</span>
              </div>
              <div class={css.actions}>
                <button type="button" class={css.ghost} disabled={busy()} onClick={() => edit(sig)}>
                  {t('settings-edit')}
                </button>
                <button type="button" class={css.danger} disabled={busy()} onClick={() => void remove(sig.name)}>
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
              {t('settings-sig-new')}
            </button>
          </div>
        }
      >
        <div class={css.field} data-testid="signature-editor">
          <label class={css.field}>
            <span class={css.label}>{t('settings-sig-name-label')}</span>
            <input
              class={css.input}
              aria-label={t('settings-sig-name-label')}
              value={draft().name}
              onInput={(e) => setDraft({ ...draft(), name: e.currentTarget.value })}
            />
          </label>
          <label class={css.field}>
            <span class={css.label}>{t('settings-sig-body-label')}</span>
            <textarea
              class={css.textarea}
              aria-label={t('settings-sig-body-label')}
              value={draft().body}
              onInput={(e) => setDraft({ ...draft(), body: e.currentTarget.value })}
            />
          </label>
          <label class={css.check}>
            <input
              class={css.checkbox}
              type="checkbox"
              aria-label={t('settings-sig-default-label')}
              checked={draft().isDefault}
              onChange={(e) => setDraft({ ...draft(), isDefault: e.currentTarget.checked })}
            />
            <span>{t('settings-sig-default-label')}</span>
          </label>
          <div class={css.actions}>
            <button type="button" class={css.button} disabled={busy()} onClick={() => void save()} data-testid="signature-save">
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

export default Signatures;
