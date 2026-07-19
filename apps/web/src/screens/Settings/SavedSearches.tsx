// Saved searches → search folders (t16 e15 — W13). Reuses the FROZEN `mw-store`
// 0003 `saved_searches` rows (id, name, query_json, as_folder) — NOT a new table.
// A saved search whose `as_folder` is on surfaces as a virtual folder in the
// mailbox list (JMAP `Mailbox/get` already emits these with `mailwomanSearchQuery`);
// this screen toggles that promotion and manages the set.

import { createResource, createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { t, loadCatalog } from '../../i18n';
import { SettingsService } from './service.ts';
import type { SavedSearch } from './types.ts';
import * as css from './styles.css.ts';

export interface SavedSearchesProps {
  service?: SettingsService;
}

export function SavedSearches(props: SavedSearchesProps): JSX.Element {
  const service = props.service ?? new SettingsService();
  onMount(() => void loadCatalog('settings'));

  const [searches, { refetch }] = createResource<SavedSearch[]>(() => service.listSavedSearches());
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');

  function fail(e: unknown): void {
    setError(e instanceof Error ? e.message : t('settings-search-error'));
  }

  async function toggleFolder(search: SavedSearch, asFolder: boolean): Promise<void> {
    setError('');
    setBusy(true);
    try {
      await service.upsertSavedSearch({ ...search, asFolder });
      await refetch();
    } catch (e) {
      fail(e);
    } finally {
      setBusy(false);
    }
  }

  async function remove(id: string): Promise<void> {
    setError('');
    setBusy(true);
    try {
      await service.deleteSavedSearch(id);
      await refetch();
    } catch (e) {
      fail(e);
    } finally {
      setBusy(false);
    }
  }

  return (
    <section class={css.section} aria-label={t('settings-search-title')}>
      <h2 class={css.heading}>{t('settings-search-title')}</h2>
      <p class={css.prose}>{t('settings-search-intro')}</p>

      <Show
        when={(searches() ?? []).length > 0}
        fallback={<p class={css.meta}>{t('settings-search-empty')}</p>}
      >
        <ul class={css.list} data-testid="saved-search-list">
          <For each={searches() ?? []}>
            {(search) => (
              <li class={css.item}>
                <div class={css.itemMain}>
                  <span class={css.itemName}>{search.name}</span>
                </div>
                <div class={css.actions}>
                  <label class={css.check}>
                    <input
                      class={css.checkbox}
                      type="checkbox"
                      aria-label={t('settings-search-as-folder', { name: search.name })}
                      checked={search.asFolder}
                      disabled={busy()}
                      onChange={(e) => void toggleFolder(search, e.currentTarget.checked)}
                    />
                    <span>{t('settings-search-as-folder-label')}</span>
                  </label>
                  <button type="button" class={css.danger} disabled={busy()} onClick={() => void remove(search.id)}>
                    {t('settings-delete')}
                  </button>
                </div>
              </li>
            )}
          </For>
        </ul>
      </Show>

      <Show when={error() !== ''}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>
    </section>
  );
}

export default SavedSearches;
