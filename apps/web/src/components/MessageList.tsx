import { createMemo, createSignal, For, Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { t, isolate } from '../i18n/index.ts';
import { computeWindow } from './virtual.ts';
import { TagChips } from './TagChips.tsx';
import { MessageActions } from './MessageActions.tsx';
import * as a11y from './mailA11y.css.ts';
import * as thread from './threadList.css.ts';
import { groupThreads, type ThreadVisualRow } from './threads.ts';
import { readingPane, setReadingPane, READING_PANE_OPTIONS, type ReadingPane } from './readingPane.ts';
import type { Density } from '../theme/contract.css.ts';
import type { Email, EmailAddress } from '../api/jmap-types.ts';

// The message list, virtualized for the §23 100k-row gate: only the rows inside
// the viewport (± overscan) are mounted, positioned inside a full-height spacer
// so the scrollbar still reflects the whole list. Rows keep the `.list__row`
// class + subject text the e2e/mock specs locate by. Pins float to the top and
// snoozed rows are hidden — both handled upstream in `app.listMessages()`.
//
// W2 threading: the flat, already-sorted list is folded into VISUAL ROWS by
// `groupThreads()` — a lone message stays one row; a conversation (shared
// engine `threadId`) collapses to one head row that expands to its members in
// place. The virtualizer windows over that uniform-height visual array, so a
// list with no repeated threadId renders exactly as the pre-threading flat list.
//
// W17 density: the virtualized row height tracks the `data-density` preference
// (`app.density()`); cozy keeps the historical 72px so the default is unchanged.
//
// W3 reading pane: the always-visible list toolbar hosts the reading-pane
// position control (right / bottom / off); the layout switch itself is CSS keyed
// on `:root[data-reading-pane]` (readingPane.ts + readerPane.css.ts).
//
// a11y (t8-e1): the list is a real `<ul>`/`<li>` list; each row carries
// `aria-posinset`/`aria-setsize` so its position in the FULL (virtualized) list
// is announced, `aria-current` marks the open message, and an off-screen
// "Unread" marker announces unread state. Arrow/Home/End move a roving focus
// between rows, scrolling the window as needed.

/** Virtualized row height per density. Cozy is the historical default (72px);
 *  the message row is three lines, so these sit above the single-line density
 *  tokens in the theme contract. */
const ROW_HEIGHTS: Record<Density, number> = { compact: 56, cozy: 72, relaxed: 88 };

function senderLabel(from: EmailAddress[] | null): string {
  const first = from?.[0];
  if (first === undefined) return t('mail-unknown-sender');
  return first.name && first.name.length > 0 ? first.name : first.email;
}

function formatDate(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return '';
  return d.toLocaleString(undefined, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
}

function MessageRow(props: {
  email: Email;
  top: number;
  height: number;
  index: number;
  total: number;
  focused: boolean;
  threadChild?: boolean;
  setRef: (el: HTMLButtonElement | undefined) => void;
}): JSX.Element {
  const app = useApp();
  const email = () => props.email;
  const unread = () => email().keywords?.['$seen'] !== true;
  return (
    <li
      class="list__slot"
      role="listitem"
      aria-posinset={props.index + 1}
      aria-setsize={props.total}
      style={{ transform: `translateY(${props.top}px)`, height: `${props.height}px` }}
    >
      <button
        type="button"
        ref={(el) => props.setRef(el)}
        class={`list__row ${a11y.focusable}`}
        classList={{
          'list__row--active': app.openEmail()?.id === email().id,
          'list__row--pinned': email().pinned === true,
          'list__row--unread': unread(),
          [thread.childRow]: props.threadChild === true,
        }}
        tabindex={props.focused ? 0 : -1}
        aria-current={app.openEmail()?.id === email().id ? 'true' : undefined}
        data-index={props.index}
        onClick={() => void app.openMessage(email().id)}
      >
        <Show when={unread()}>
          <span class={a11y.srOnly}>{t('mail-unread')}</span>
        </Show>
        <span class={a11y.srOnly}>{t('mail-row-position', { pos: props.index + 1, total: props.total })}</span>
        <span class="list__line1">
          {/* isolate the sender: it shares a line with the date, so a spoofed
              display name must not reorder surrounding UI (SPEC §24). */}
          <span class="list__sender">{isolate(senderLabel(email().from))}</span>
          <Show when={email().pinned === true}>
            <span class="list__pin" aria-label={t('mail-pinned')}>📌</span>
          </Show>
          <Show when={email().hasAttachment === true}>
            <span class="list__attach" aria-label={t('mail-has-attachment')}>📎</span>
          </Show>
          <span class="list__date">{formatDate(email().receivedAt)}</span>
        </span>
        <span class="list__subject">{email().subject ?? t('mail-no-subject')}</span>
        <span class="list__preview">{email().preview}</span>
        <TagChips email={email()} />
      </button>
      <MessageActions email={email()} />
    </li>
  );
}

/** A collapsed conversation head (W2): the disclosure toggle sits beside the
 *  primary row; the primary row opens the latest message (like any message row),
 *  the toggle expands/collapses the members in place. */
function ThreadHeadRow(props: {
  row: ThreadVisualRow;
  top: number;
  height: number;
  index: number;
  total: number;
  focused: boolean;
  setRef: (el: HTMLButtonElement | undefined) => void;
  onToggle: () => void;
}): JSX.Element {
  const app = useApp();
  const rep = () => props.row.email;
  const active = () => app.openEmail()?.id === rep().id;
  return (
    <li
      class={`list__slot ${thread.headSlot}`}
      role="listitem"
      aria-posinset={props.index + 1}
      aria-setsize={props.total}
      style={{ transform: `translateY(${props.top}px)`, height: `${props.height}px` }}
    >
      <button
        type="button"
        class={`${thread.toggle} ${a11y.focusable}`}
        aria-expanded={props.row.expanded}
        aria-label={
          props.row.expanded
            ? t('mail-thread-collapse', { count: props.row.count })
            : t('mail-thread-expand', { count: props.row.count })
        }
        data-testid="thread-toggle"
        onClick={props.onToggle}
      >
        {props.row.expanded ? '▾' : '▸'}
      </button>
      <button
        type="button"
        ref={(el) => props.setRef(el)}
        class={`list__row ${thread.headRow} ${a11y.focusable}`}
        classList={{
          'list__row--active': active(),
          'list__row--unread': props.row.unread,
        }}
        tabindex={props.focused ? 0 : -1}
        aria-current={active() ? 'true' : undefined}
        data-index={props.index}
        data-testid="thread-head"
        onClick={() => void app.openMessage(rep().id)}
      >
        <Show when={props.row.unread}>
          <span class={a11y.srOnly}>{t('mail-unread')}</span>
        </Show>
        <span class={a11y.srOnly}>{t('mail-row-position', { pos: props.index + 1, total: props.total })}</span>
        <span class="list__line1">
          <span class="list__sender">{isolate(senderLabel(rep().from))}</span>
          <span class={thread.count} aria-label={t('mail-thread-count', { count: props.row.count })}>
            {props.row.count}
          </span>
          <Show when={props.row.hasAttachment}>
            <span class="list__attach" aria-label={t('mail-has-attachment')}>📎</span>
          </Show>
          <span class="list__date">{formatDate(rep().receivedAt)}</span>
        </span>
        <span class="list__subject">{rep().subject ?? t('mail-no-subject')}</span>
        <span class="list__preview">{rep().preview}</span>
      </button>
    </li>
  );
}

/** The always-visible view toolbar: the reading-pane position control (W3). */
function ListToolbar(): JSX.Element {
  const label = (opt: ReadingPane): string => t(`mail-reading-pane-${opt}`);
  return (
    <div class={thread.toolbar} role="group" aria-label={t('mail-view-options')}>
      <span class={thread.toolbarLabel}>{t('mail-reading-pane')}</span>
      <For each={READING_PANE_OPTIONS}>
        {(opt) => (
          <button
            type="button"
            class={`${thread.segBtn} ${a11y.focusable}`}
            aria-pressed={readingPane() === opt}
            data-testid={`reading-pane-${opt}`}
            onClick={() => setReadingPane(opt)}
          >
            {label(opt)}
          </button>
        )}
      </For>
    </div>
  );
}

export function MessageList(): JSX.Element {
  const app = useApp();
  const [scrollTop, setScrollTop] = createSignal(0);
  // jsdom reports 0 for clientHeight; fall back to a sane viewport so tests
  // (and first paint before layout) still mount a window of rows.
  const [viewportH, setViewportH] = createSignal(600);
  // Roving keyboard cursor over the FULL list (independent of which rows are
  // currently mounted); defaults to the first row.
  const [cursor, setCursor] = createSignal(0);
  // Expanded conversation keys (W2). Toggling re-derives the visual rows.
  const [expanded, setExpanded] = createSignal<ReadonlySet<string>>(new Set());

  const rowHeight = (): number => ROW_HEIGHTS[app.density()];
  // The FLAT list folded into visual rows (singletons + conversation heads/members).
  const rows = createMemo<ThreadVisualRow[]>(() => groupThreads(app.listMessages(), expanded()));
  const win = createMemo(() => computeWindow(scrollTop(), viewportH(), rowHeight(), rows().length));

  let scroller: HTMLDivElement | undefined;
  const rowEls = new Map<number, HTMLButtonElement>();

  function toggleThread(key: string): void {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  }

  function onScroll(): void {
    if (scroller !== undefined) {
      setScrollTop(scroller.scrollTop);
      if (scroller.clientHeight > 0) setViewportH(scroller.clientHeight);
    }
  }

  /** Move the roving cursor, scroll the target into the window, then focus it. */
  function moveCursor(to: number): void {
    const count = rows().length;
    if (count === 0) return;
    const next = Math.max(0, Math.min(count - 1, to));
    setCursor(next);
    if (scroller !== undefined) {
      const h = rowHeight();
      const rowTop = next * h;
      const rowBottom = rowTop + h;
      if (rowTop < scroller.scrollTop) scroller.scrollTop = rowTop;
      else if (rowBottom > scroller.scrollTop + scroller.clientHeight) {
        scroller.scrollTop = rowBottom - scroller.clientHeight;
      }
      onScroll();
    }
    // Focus after the window re-renders the target row.
    queueMicrotask(() => rowEls.get(next)?.focus());
  }

  function onKeyDown(e: KeyboardEvent): void {
    const count = rows().length;
    if (count === 0) return;
    if (e.key === 'ArrowDown') moveCursor(cursor() + 1);
    else if (e.key === 'ArrowUp') moveCursor(cursor() - 1);
    else if (e.key === 'Home') moveCursor(0);
    else if (e.key === 'End') moveCursor(count - 1);
    else return;
    e.preventDefault();
  }

  const slice = createMemo(() => {
    const w = win();
    return rows().slice(w.startIndex, w.endIndex).map((row, i) => ({ row, index: w.startIndex + i }));
  });

  return (
    <section class="list" aria-label={t('mail-list-label')} style={{ display: 'flex', 'flex-direction': 'column' }}>
      <ListToolbar />
      <Show when={!app.listLoading()} fallback={<p class="list__empty">{t('mail-loading')}</p>}>
        <Show when={rows().length > 0} fallback={<p class="list__empty">{t('mail-empty')}</p>}>
          <div
            class="list__scroll"
            ref={scroller}
            onScroll={onScroll}
            onKeyDown={onKeyDown}
            style={{ overflow: 'auto', flex: '1 1 auto', 'min-height': '0' }}
          >
            <ul
              class="list__items"
              role="list"
              aria-label={t('mail-list-label')}
              style={{ position: 'relative', height: `${win().totalHeight}px` }}
            >
              <For each={slice()}>
                {(entry) => (
                  <Show
                    when={entry.row.kind === 'head'}
                    fallback={
                      <MessageRow
                        email={entry.row.email}
                        top={entry.index * rowHeight()}
                        height={rowHeight()}
                        index={entry.index}
                        total={rows().length}
                        focused={entry.index === cursor()}
                        threadChild={entry.row.kind === 'child'}
                        setRef={(el) => {
                          if (el) rowEls.set(entry.index, el);
                          else rowEls.delete(entry.index);
                        }}
                      />
                    }
                  >
                    <ThreadHeadRow
                      row={entry.row}
                      top={entry.index * rowHeight()}
                      height={rowHeight()}
                      index={entry.index}
                      total={rows().length}
                      focused={entry.index === cursor()}
                      onToggle={() => toggleThread(entry.row.key)}
                      setRef={(el) => {
                        if (el) rowEls.set(entry.index, el);
                        else rowEls.delete(entry.index);
                      }}
                    />
                  </Show>
                )}
              </For>
            </ul>
          </div>
        </Show>
      </Show>
    </section>
  );
}
