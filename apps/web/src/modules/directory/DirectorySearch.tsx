// GAL search / autocomplete for recipient fields (SPEC §13, plan §3 e7).
//
// Drop into every recipient field: as the user types, it debounces a paged GAL
// query against `/api/directory/*` and surfaces matches (people + distribution
// groups). Picking a group hands it up as a group entry so the composer can offer
// expand-before-send (see GroupExpand). EXPORTED for e14 to wire into the composer;
// this file does not touch the router or the compose screen.

import { createSignal, For, Show, createMemo, onMount, type JSX } from 'solid-js';
import { DirectoryService, type Fetcher } from './service.ts';
import type { GalEntry } from './index.ts';
import { t, loadCatalog } from '../../i18n';
import * as css from './styles.css.ts';

export interface DirectorySearchProps {
  /** Controlled query text (the recipient-field input). */
  query: string;
  /** The account/composer picked a GAL entry. */
  onPick: (entry: GalEntry) => void;
  fetcher?: Fetcher;
  /** Tests inject a service double; production constructs one over `fetcher`. */
  service?: DirectoryService;
  /** Debounce window in ms (default 180; tests pass 0 for determinism). */
  debounceMs?: number;
}

/** Debounce that resolves after `ms`; a subsequent call cancels the pending one. */
function makeDebouncer(): (fn: () => void, ms: number) => void {
  let timer: ReturnType<typeof setTimeout> | undefined;
  return (fn, ms) => {
    if (timer !== undefined) clearTimeout(timer);
    if (ms <= 0) {
      fn();
      return;
    }
    timer = setTimeout(fn, ms);
  };
}

export function DirectorySearch(props: DirectorySearchProps): JSX.Element {
  onMount(() => void loadCatalog('directory'));
  const service = createMemo(() => props.service ?? new DirectoryService(props.fetcher));
  const [results, setResults] = createSignal<GalEntry[]>([]);
  const [hasMore, setHasMore] = createSignal(false);
  const [page, setPage] = createSignal(0);
  const [error, setError] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const debounce = makeDebouncer();

  async function run(query: string, pageIndex: number, append: boolean): Promise<void> {
    setError('');
    setBusy(true);
    try {
      const res = await service().searchGal(query, pageIndex);
      setResults((cur) => (append ? [...cur, ...res.entries] : res.entries));
      setHasMore(res.hasMore);
      setPage(res.page);
    } catch (e) {
      setError(e instanceof Error ? e.message : t('directory-search-error'));
      if (!append) setResults([]);
    } finally {
      setBusy(false);
    }
  }

  // React to the controlled query (debounced).
  createMemo(() => {
    const q = props.query;
    debounce(() => {
      if (q.trim() === '') {
        setResults([]);
        setHasMore(false);
        return;
      }
      void run(q, 0, false);
    }, props.debounceMs ?? 180);
  });

  return (
    <div class={css.wrap} data-module="directory">
      <Show when={error() !== ''}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>
      <Show when={results().length > 0}>
        <ul class={css.listbox} role="listbox" aria-label={t('directory-matches-label')}>
          <For each={results()}>
            {(entry) => (
              <li
                class={css.option}
                role="option"
                aria-selected="false"
                data-gal-dn={entry.dn}
                onClick={() => props.onPick(entry)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') props.onPick(entry);
                }}
                tabindex={0}
              >
                <span>
                  <span class={css.optName}>{entry.displayName}</span>{' '}
                  <span class={css.optMail}>{entry.mail}</span>
                </span>
                <Show when={entry.isGroup}>
                  <span class={css.badge} data-testid="group-badge">
                    {t('directory-group-badge')}
                  </span>
                </Show>
              </li>
            )}
          </For>
        </ul>
        <Show when={hasMore()}>
          <button
            type="button"
            class={css.button}
            disabled={busy()}
            onClick={() => void run(props.query, page() + 1, true)}
          >
            {t('directory-load-more')}
          </button>
        </Show>
      </Show>
    </div>
  );
}

export default DirectorySearch;
