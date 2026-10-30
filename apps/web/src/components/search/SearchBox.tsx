// The search box above the message list (moved out of Mailbox.tsx, t88): the
// input, a magnifier icon button that runs the search, a × icon button that
// clears it, and, while a search is active, the filter chips (SearchFilterBar).
//
// The input text is the one source of the query. The chips are parsed from it
// as it is typed; a chip edit writes the canonical query text back into it and
// searches; a search started elsewhere (a "Search this sender" action) arrives
// through `app.search()` and lands in the input too.

import { createEffect, createSignal, on, onMount, Show, type JSX } from 'solid-js';
import { useApp } from '../../state/context.ts';
import { loadCatalog, t } from '../../i18n/index.ts';
import { Icon } from '../Icon.tsx';
import * as a11y from '../mailA11y.css.ts';
import { SemanticSearchToggle } from '../../modules/assist/index.ts';
import { SearchFilterBar } from './SearchFilterBar.tsx';
import * as css from './searchBar.css.ts';

/** Submits an `Email/query` (engine → mw-search online, reduced cached search
 *  offline). */
export function SearchBox(): JSX.Element {
  const app = useApp();
  onMount(() => void loadCatalog('search'));
  const [query, setQuery] = createSignal(app.search());
  // V7 semantic search (§14.3): off by default; the toggle only renders when the
  // `search-semantic` Assist capability is granted, and its state rides the query.
  const [semantic, setSemantic] = createSignal(false);
  // A search started or refined elsewhere, or cleared, shows in the input.
  createEffect(on(app.search, (value) => setQuery(value), { defer: true }));
  let input!: HTMLInputElement;

  const run = (text: string): void => {
    setQuery(text);
    void app.searchMessages(text, { semantic: semantic() });
  };
  const clear = (): void => {
    setQuery('');
    void app.clearSearch();
  };
  const showClear = (): boolean => app.searchActive() || query() !== '';

  return (
    <div class="mail-search-area">
      <form
        class={`mail-search ${css.form}`}
        role="search"
        onSubmit={(e) => {
          e.preventDefault();
          run(query());
        }}
      >
        <input
          ref={input}
          class="mail-search__input"
          type="search"
          aria-label={t('mail-search-label')}
          placeholder={t('mail-search-placeholder')}
          value={query()}
          onInput={(e) => setQuery(e.currentTarget.value)}
          onKeyDown={(e) => {
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
        <SemanticSearchToggle config={app.assist.config()} enabled={semantic()} onChange={setSemantic} />
      </form>
      <div class={css.reveal} data-open={app.searchActive()}>
        <div class={css.revealInner} inert={!app.searchActive()}>
          <Show when={app.searchActive()}>
            <SearchFilterBar
              text={query}
              onChange={run}
              mailboxes={app.mailboxes}
              currentMailboxId={app.selectedMailboxId}
            />
          </Show>
        </div>
      </div>
    </div>
  );
}
