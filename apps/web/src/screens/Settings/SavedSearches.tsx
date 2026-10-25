// Saved searches → search folders (t16 e15 — W13). Reuses the FROZEN `mw-store`
// 0003 `saved_searches` rows (id, name, query_json, as_folder) — NOT a new table.
// A saved search whose `as_folder` is on surfaces as a virtual folder in the
// mailbox list (JMAP `Mailbox/get` already emits these with `mailwomanSearchQuery`);
// this screen toggles that promotion and manages the set.

import { createResource, createSignal, For, Show, onMount, type JSX } from 'solid-js';
import { t, loadCatalog } from '../../i18n';
import { SettingsService } from './service.ts';
import type { SavedSearch } from './types.ts';
import { toast } from '../../components/toast/index.ts';
import { SearchFolderEditor } from './SearchFolderEditor.tsx';
import { useEntityNavigation } from './entityEditor.tsx';
import * as css from './styles.css.ts';

export interface SavedSearchesProps {
  service?: SettingsService;
  onChanged?: () => void | Promise<void>;
}

export function SavedSearches(props: SavedSearchesProps): JSX.Element {
  const service = props.service ?? new SettingsService();
  onMount(() => void loadCatalog('settings'));

  const [searches, { refetch }] = createResource<SavedSearch[]>(() => service.listSavedSearches());
  const navigation = useEntityNavigation('searches');
  const editor = () => navigation.route().section === 'searches' ? navigation.route().editor : undefined;
  const back = async (): Promise<void> => { await navigation.navigate({ section: 'searches' }); };
  const open = (id?: string) => navigation.navigate({ section: 'searches', editor: id ? { kind: 'edit', id } : { kind: 'new' } });
  const changed = async (): Promise<void> => { await refetch(); await props.onChanged?.(); };
  const [busy, setBusy] = createSignal(false);

  // A failed action is a toast (t32 f39).
  function fail(e: unknown): void {
    toast.error(e instanceof Error ? e.message : t('settings-search-error'));
  }

  async function toggleFolder(search: SavedSearch, asFolder: boolean): Promise<void> {
    setBusy(true);
    try {
      await service.upsertSavedSearch({ ...search, asFolder });
      await changed();
    } catch (e) {
      fail(e);
    } finally {
      setBusy(false);
    }
  }

  async function remove(id: string): Promise<void> {
    setBusy(true);
    try {
      await service.deleteSavedSearch(id);
      await changed();
    } catch (e) {
      fail(e);
    } finally {
      setBusy(false);
    }
  }

  return (
    <Show when={editor()} keyed fallback={<section class={css.section} aria-label={t('settings-search-title')}>
      <h2 class={css.heading}>{t('settings-search-title')}</h2>
      <p class={css.prose}>{t('settings-search-intro')}</p>
      <div class={css.actions}><button class={css.button} type="button" onClick={() => void open()}>{t('settings-search-new')}</button></div>

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
                  <button type="button" class={css.ghost} disabled={busy()} onClick={() => void open(search.id)}>{t('settings-edit')}</button>
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
    </section>}>{route => <Show when={route.kind === 'new'} fallback={<Show when={searches()?.find(item => item.id === route.id)} keyed fallback={<p>{t('settings-editor-missing')}</p>}>{item => <SearchFolderEditor initial={item} service={service} onSaved={async () => { await changed(); await back(); }} onCancel={back} />}</Show>}><SearchFolderEditor service={service} onSaved={async () => { await changed(); await back(); }} onCancel={back} /></Show>}</Show>
  );
}

export default SavedSearches;
