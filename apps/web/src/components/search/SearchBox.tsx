// The search box above the message list (moved out of Mailbox.tsx, t88): the
// input, a magnifier icon button that runs the search, a × icon button that
// clears it, and, while a search is active, the filter chips (SearchFilterBar).
//
// The input text is the one source of the query. The chips are parsed from it
// as it is typed; a chip edit writes the canonical query text back into it and
// searches; a search started elsewhere (a "Search this sender" action) arrives
// through `app.search()` and lands in the input too.

import { createEffect, createMemo, createSignal, createUniqueId, on, onCleanup, onMount, Show, type JSX } from 'solid-js';
import { useApp } from '../../state/context.ts';
import { loadCatalog, t } from '../../i18n/index.ts';
import { Icon } from '../Icon.tsx';
import * as a11y from '../mailA11y.css.ts';
import { SemanticSearchToggle } from '../../modules/assist/index.ts';
import { SearchFilterBar } from './SearchFilterBar.tsx';
import * as css from './searchBar.css.ts';
import { applyDefaultSearchScope, normalSearchRequest } from '../../search/defaultScope.ts';
import {hasFolderScope} from '../../search/searchQuery.ts';

export const LIVE_SEARCH_DELAY_MS = 250;

/** Submits an `Email/query` (engine → mw-search online, reduced cached search
 *  offline). */
export function SearchBox(): JSX.Element {
  const app = useApp();
  onMount(() => void loadCatalog('search'));
  const [query, setQuery] = createSignal(app.search());
  const [filtersOpen, setFiltersOpen] = createSignal(false);
  const filtersId = createUniqueId();
  // V7 semantic search (§14.3): off by default; the toggle only renders when the
  // `search-semantic` Assist capability is granted, and its state rides the query.
  const [semantic, setSemantic] = createSignal(false);
  const [explicitScope, setExplicitScope] = createSignal(app.searchActive());
  let ownSearch: string | undefined;
  let implicitScope: string | undefined;
  let pending: ReturnType<typeof setTimeout> | undefined;
  let composing = false;
  const cancelPending = (): void => { clearTimeout(pending); pending = undefined; };
  onCleanup(cancelPending);
  // A search started or refined elsewhere, or cleared, shows in the input.
  createEffect(on(app.search, (value) => { cancelPending(); setQuery(value); if (value !== ownSearch) {setExplicitScope(value.trim() !== '');implicitScope=undefined;} ownSearch = undefined; }, { defer: true }));
  // A delayed query belongs to the mailbox where it was typed. Never carry it
  // into a different account or folder while the user is navigating.
  const scope = createMemo(() => JSON.stringify([app.accountId(), app.linkedMailboxId?.(), app.me()?.username,
    app.selectedMailboxId(), app.switchingMailbox?.()]));
  createEffect(on(scope, () => { cancelPending(); composing = false; implicitScope=undefined;setExplicitScope(false); setQuery(app.search()); }, { defer: true }));
  let input!: HTMLInputElement;
  let filtersButton!: HTMLButtonElement;
  const closeFilters = (): void => { setFiltersOpen(false); filtersButton.focus(); };

  const run = (text: string, fromFilters=false): void => {
    cancelPending();
    const request=normalSearchRequest(text,app.selectedMailboxId(),app.mailboxes(),explicitScope(),implicitScope,fromFilters&&!hasFolderScope(query()));
    implicitScope=request.scopeMailboxId;
    ownSearch = request.query;
    setQuery(request.query);
    void app.searchMessages(request.query, { semantic: semantic(),...(request.scopeMailboxId?{scopeMailboxId:request.scopeMailboxId,baseQuery:request.baseQuery??text}:{}) });
  };
  const clear = (): void => {
    cancelPending();
    setExplicitScope(false); ownSearch = undefined;implicitScope=undefined;
    setQuery('');
    void app.clearSearch();
  };
  const typed = (text: string): void => {
    cancelPending();
    setQuery(text);
    if (composing || app.switchingMailbox?.()) return;
    if (!text.trim()) { if (app.searchActive()) clear(); return; }
    const owner = scope();
    pending = setTimeout(() => {
      pending = undefined;
      if (composing || owner !== scope()) return;
      run(text);
    }, LIVE_SEARCH_DELAY_MS);
  };
  const showClear = (): boolean => app.searchActive() || query() !== '';

  return (
    <div class="mail-search-area">
      <form
        class={`mail-search ${css.form}`}
        role="search"
        onSubmit={(e) => {
          e.preventDefault();
          if (!composing) run(query());
        }}
      >
        <input
          ref={input}
          class="mail-search__input"
          type="search"
          aria-label={t('mail-search-label')}
          placeholder={t('mail-search-placeholder')}
          value={query()}
          onInput={(e) => typed(e.currentTarget.value)}
          onCompositionStart={() => { composing = true; cancelPending(); }}
          onCompositionEnd={(e) => { composing = false; typed(e.currentTarget.value); }}
          onKeyDown={(e) => {
            if (e.key === 'Escape' && filtersOpen()) {
              e.preventDefault(); e.stopPropagation(); closeFilters(); return;
            }
            // Esc clears (and restores the folder) instead of the browser's own
            // clear, which would leave the results of the old query showing.
            if (e.key === 'Escape' && showClear()) {
              e.preventDefault();
              e.stopPropagation();
              clear();
            }
          }}
        />
        <Show when={showClear()}>
          <button
            type="button"
            class={`btn btn--ghost mail-search__clear ${a11y.iconButton}`}
            aria-label={t('mail-search-clear')}
            title={t('search-clear-help')}
            aria-keyshortcuts="Escape"
            data-testid="search-clear"
            onClick={() => {
              clear();
              input.focus();
            }}
          >
            <Icon name="close" />
          </button>
        </Show>
        <button
          type="submit"
          class={`btn btn--ghost mail-search__submit ${a11y.iconButton}`}
          aria-label={t('mail-search')}
          title={t('search-submit-help')}
          aria-keyshortcuts="Enter"
          data-testid="search-submit"
        >
          <Icon name="search" />
        </button>
        <button
          ref={filtersButton} type="button" class={`btn btn--ghost ${a11y.iconButton} ${css.filtersToggle}`}
          aria-label={t('search-guided-button')} aria-expanded={filtersOpen()} aria-controls={filtersId}
          title={t('search-guided-help')} data-testid="search-guided-toggle"
          onClick={() => setFiltersOpen(value => !value)}
        ><Icon name="filter" /></button>
        <SemanticSearchToggle config={app.assist.config()} enabled={semantic()} onChange={setSemantic} />
      </form>
      <div class={css.reveal} data-open={app.searchActive() || filtersOpen()}>
        <div id={filtersId} class={css.revealInner} inert={!app.searchActive() && !filtersOpen()}
          onKeyDown={event => { if(event.key === 'Escape' && filtersOpen()){event.preventDefault();event.stopPropagation();closeFilters();} }}>
          <Show when={app.searchActive() || filtersOpen()}>
            <SearchFilterBar
              guided={filtersOpen()}
              text={() => applyDefaultSearchScope(query(), app.selectedMailboxId(), app.mailboxes(), explicitScope(), filtersOpen())}
              onChange={text=>run(text,true)}
              onScopeChange={() => {setExplicitScope(true);implicitScope=undefined;}}
              mailboxes={app.mailboxes}
              currentMailboxId={app.selectedMailboxId}
            />
          </Show>
        </div>
      </div>
    </div>
  );
}
