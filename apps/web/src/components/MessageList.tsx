import { createMemo, createSignal, For, Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { computeWindow } from './virtual.ts';
import { TagChips } from './TagChips.tsx';
import { MessageActions } from './MessageActions.tsx';
import type { Email, EmailAddress } from '../api/jmap-types.ts';

// The message list, virtualized for the §23 100k-row gate: only the rows inside
// the viewport (± overscan) are mounted, positioned inside a full-height spacer
// so the scrollbar still reflects the whole list. Rows keep the `.list__row`
// class + subject text the e2e/mock specs locate by. Pins float to the top and
// snoozed rows are hidden — both handled upstream in `app.listMessages()`.

const ROW_HEIGHT = 72;

function senderLabel(from: EmailAddress[] | null): string {
  const first = from?.[0];
  if (first === undefined) return '(unknown sender)';
  return first.name && first.name.length > 0 ? first.name : first.email;
}

function formatDate(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return '';
  return d.toLocaleString(undefined, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
}

function MessageRow(props: { email: Email; top: number }): JSX.Element {
  const app = useApp();
  const email = () => props.email;
  return (
    <li class="list__slot" style={{ transform: `translateY(${props.top}px)`, height: `${ROW_HEIGHT}px` }}>
      <button
        type="button"
        class="list__row"
        classList={{
          'list__row--active': app.openEmail()?.id === email().id,
          'list__row--pinned': email().pinned === true,
          'list__row--unread': email().keywords?.['$seen'] !== true,
        }}
        onClick={() => void app.openMessage(email().id)}
      >
        <span class="list__line1">
          <span class="list__sender">{senderLabel(email().from)}</span>
          <Show when={email().pinned === true}>
            <span class="list__pin" aria-label="Pinned">📌</span>
          </Show>
          <Show when={email().hasAttachment === true}>
            <span class="list__attach" aria-label="Has attachment">📎</span>
          </Show>
          <span class="list__date">{formatDate(email().receivedAt)}</span>
        </span>
        <span class="list__subject">{email().subject ?? '(no subject)'}</span>
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

  const rows = () => app.listMessages();
  const win = createMemo(() => computeWindow(scrollTop(), viewportH(), ROW_HEIGHT, rows().length));

  let scroller: HTMLDivElement | undefined;
  function onScroll(): void {
    if (scroller !== undefined) {
      setScrollTop(scroller.scrollTop);
      if (scroller.clientHeight > 0) setViewportH(scroller.clientHeight);
    }
  }

  const slice = createMemo(() => {
    const w = win();
    return rows().slice(w.startIndex, w.endIndex).map((email, i) => ({ email, index: w.startIndex + i }));
  });

  return (
    <section class="list" aria-label="Messages">
      <Show when={!app.listLoading()} fallback={<p class="list__empty">Loading messages…</p>}>
        <Show when={rows().length > 0} fallback={<p class="list__empty">No messages</p>}>
          <div
            class="list__scroll"
            ref={scroller}
            onScroll={onScroll}
            style={{ overflow: 'auto', height: '100%' }}
          >
            <ul class="list__items" style={{ position: 'relative', height: `${win().totalHeight}px` }}>
              <For each={slice()}>
                {(entry) => <MessageRow email={entry.email} top={entry.index * ROW_HEIGHT} />}
              </For>
            </ul>
          </div>
        </Show>
      </Show>
    </section>
  );
}
