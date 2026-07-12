// The global Attachments module (plan §0.10 / §2.4): a grid/list of every
// attachment across the account, filtered by type / sender / size / date and by
// the shared search operators (`filename:` / `type:` / `larger:` / `from:` …).
// Data comes from `Email/query{filter:{hasAttachment:true}}` via `loadAttachments`
// (online → engine `mw-search` backs the operators; e9). The component takes its
// data as a `load` callback or preloaded `items`, so it is wireable AND testable.

import { createMemo, createResource, createSignal, For, Show, type JSX } from 'solid-js';
import '../viewers/viewers.css';
import {
  categoryOf,
  filterAttachments,
  formatSize,
  parseAttachmentQuery,
  type AttachmentItem,
  type TypeCategory,
} from '../viewers/attachments.ts';

export interface AttachmentsProps {
  /** Live loader (e.g. `() => loadAttachments(client, accountId)`). */
  load?: () => Promise<AttachmentItem[]>;
  /** Preloaded rows (tests / offline slice); used when `load` is absent. */
  items?: AttachmentItem[];
  onOpen?: (item: AttachmentItem) => void;
}

const CATEGORIES: Array<TypeCategory | 'all'> = [
  'all',
  'image',
  'pdf',
  'text',
  'audio',
  'video',
  'other',
];

export function Attachments(props: AttachmentsProps): JSX.Element {
  const [data] = createResource(async () =>
    props.load !== undefined ? await props.load() : (props.items ?? []),
  );
  const [query, setQuery] = createSignal('');
  const [cat, setCat] = createSignal<TypeCategory | 'all'>('all');

  const all = (): AttachmentItem[] => data() ?? props.items ?? [];

  const filtered = createMemo<AttachmentItem[]>(() => {
    const base = parseAttachmentQuery(query());
    const filters = cat() === 'all' ? base : { ...base, category: cat() };
    return filterAttachments(all(), filters);
  });

  return (
    <section class="mw-attach" aria-label="Attachments">
      <header class="mw-attach__bar">
        <h2 class="mw-attach__title">Attachments</h2>
        <input
          class="mw-attach__search"
          type="search"
          placeholder="filename:report type:pdf larger:1mb from:alice"
          aria-label="Search attachments"
          value={query()}
          onInput={(e) => setQuery(e.currentTarget.value)}
        />
        <select
          class="mw-attach__cat"
          aria-label="Filter by type"
          value={cat()}
          onChange={(e) => setCat(e.currentTarget.value as TypeCategory | 'all')}
        >
          <For each={CATEGORIES}>{(c) => <option value={c}>{c}</option>}</For>
        </select>
      </header>

      <Show
        when={!data.loading}
        fallback={<p class="mw-attach__status">Loading attachments…</p>}
      >
        <Show
          when={filtered().length > 0}
          fallback={<p class="mw-attach__status">No attachments match.</p>}
        >
          <ul class="mw-attach__grid">
            <For each={filtered()}>
              {(item) => (
                <li>
                  <button
                    type="button"
                    class="mw-attach__card"
                    data-category={categoryOf(item.mime)}
                    onClick={() => props.onOpen?.(item)}
                  >
                    <span class="mw-attach__kind">{categoryOf(item.mime).toUpperCase()}</span>
                    <span class="mw-attach__name">{item.name}</span>
                    <span class="mw-attach__meta">
                      {formatSize(item.size)} · {item.from}
                    </span>
                    <span class="mw-attach__subject">{item.subject}</span>
                  </button>
                </li>
              )}
            </For>
          </ul>
        </Show>
      </Show>
    </section>
  );
}

export default Attachments;
