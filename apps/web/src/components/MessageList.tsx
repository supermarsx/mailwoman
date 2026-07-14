import { createMemo, createSignal, For, Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { t, isolate } from '../i18n/index.ts';
import { computeWindow } from './virtual.ts';
import { TagChips } from './TagChips.tsx';
import { MessageActions } from './MessageActions.tsx';
import * as a11y from './mailA11y.css.ts';
import type { Email, EmailAddress } from '../api/jmap-types.ts';

// The message list, virtualized for the §23 100k-row gate: only the rows inside
// the viewport (± overscan) are mounted, positioned inside a full-height spacer
// so the scrollbar still reflects the whole list. Rows keep the `.list__row`
// class + subject text the e2e/mock specs locate by. Pins float to the top and
// snoozed rows are hidden — both handled upstream in `app.listMessages()`.
//
// a11y (t8-e1): the list is a real `<ul>`/`<li>` list; each row carries
// `aria-posinset`/`aria-setsize` so its position in the FULL (virtualized) list
// is announced, `aria-current` marks the open message, and an off-screen
// "Unread" marker announces unread state. Arrow/Home/End move a roving focus
// between rows, scrolling the window as needed.

const ROW_HEIGHT = 72;

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
  index: number;
  total: number;
  focused: boolean;
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
      style={{ transform: `translateY(${props.top}px)`, height: `${ROW_HEIGHT}px` }}
    >
      <button
        type="button"
        ref={(el) => props.setRef(el)}
        class={`list__row ${a11y.focusable}`}
        classList={{
          'list__row--active': app.openEmail()?.id === email().id,
          'list__row--pinned': email().pinned === true,
          'list__row--unread': unread(),
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

export function MessageList(): JSX.Element {
  const app = useApp();
  const [scrollTop, setScrollTop] = createSignal(0);
  // jsdom reports 0 for clientHeight; fall back to a sane viewport so tests
  // (and first paint before layout) still mount a window of rows.
  const [viewportH, setViewportH] = createSignal(600);
  // Roving keyboard cursor over the FULL list (independent of which rows are
  // currently mounted); defaults to the first row.
  const [cursor, setCursor] = createSignal(0);

  const rows = () => app.listMessages();
  const win = createMemo(() => computeWindow(scrollTop(), viewportH(), ROW_HEIGHT, rows().length));

  let scroller: HTMLDivElement | undefined;
  const rowEls = new Map<number, HTMLButtonElement>();

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
      const rowTop = next * ROW_HEIGHT;
      const rowBottom = rowTop + ROW_HEIGHT;
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
    return rows().slice(w.startIndex, w.endIndex).map((email, i) => ({ email, index: w.startIndex + i }));
  });

  return (
    <section class="list" aria-label={t('mail-list-label')}>
      <Show when={!app.listLoading()} fallback={<p class="list__empty">{t('mail-loading')}</p>}>
        <Show when={rows().length > 0} fallback={<p class="list__empty">{t('mail-empty')}</p>}>
          <div
            class="list__scroll"
            ref={scroller}
            onScroll={onScroll}
            onKeyDown={onKeyDown}
            style={{ overflow: 'auto', height: '100%' }}
          >
            <ul
              class="list__items"
              role="list"
              aria-label={t('mail-list-label')}
              style={{ position: 'relative', height: `${win().totalHeight}px` }}
            >
              <For each={slice()}>
                {(entry) => (
                  <MessageRow
                    email={entry.email}
                    top={entry.index * ROW_HEIGHT}
                    index={entry.index}
                    total={rows().length}
                    focused={entry.index === cursor()}
                    setRef={(el) => {
                      if (el) rowEls.set(entry.index, el);
                      else rowEls.delete(entry.index);
                    }}
                  />
                )}
              </For>
            </ul>
          </div>
        </Show>
      </Show>
    </section>
  );
}
