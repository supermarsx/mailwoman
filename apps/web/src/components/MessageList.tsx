import {AsyncError,AsyncPending} from './AsyncState.tsx';
import {createInitialSync} from './initialSync.ts';
import * as loadingCss from './asyncState.css.ts';
import {createMessageContextMenu} from './MessageContextMenu.tsx';
import {PersonAvatar} from './PersonAvatar.tsx';
import {EmptyTrash} from './EmptyTrash.tsx';
import { Icon } from './Icon.tsx';
import {MailSelectionBar} from './MailSelectionBar.tsx';
import * as barCss from './selectionBar.css.ts';
import type {ComposeInitial} from './compose/reply.ts';
import { createMemo, createSignal, createEffect, on, For, onCleanup, Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { t, isolate } from '../i18n/index.ts';
import {displayText} from '../i18n/presentation.ts';
import { formatDateWith } from '../i18n/format.ts';
import { messageTimeZone } from './TimeHelpers.tsx';
import { useActions } from '../keymap/index.ts';
import {modifierSelectionEnabled,dragToFolderEnabled} from './mailInteractions.ts';
import {beginMessageDrag,clearMessageDrag} from './messageDrag.ts';
import { computeWindow, projectedRowCount, sameWindow, splitWindow } from './virtual.ts';
import { TagChips } from './TagChips.tsx';
import { EventHint } from './EventHint.tsx';
import { MessageActions } from './MessageActions.tsx';
import { OfflineQueueNotice } from './OfflineQueueNotice.tsx';
import { createBackgroundLoadToast } from './backgroundLoadToast.ts';
import * as a11y from './mailA11y.css.ts';
import * as thread from './threadList.css.ts';
import * as rowCss from './messageRow.css.ts';
import * as bulk from './bulkBar.css.ts';
import { groupThreads, threadKey, type ThreadVisualRow } from './threads.ts';
import { readingPane, setReadingPane, READING_PANE_OPTIONS, type ReadingPane } from './readingPane.ts';
import { classifyRowClick, dblClickOpensTab } from './messageOpen.ts';
import type { Density } from '../theme/contract.css.ts';
import type { Email, EmailAddress, Id } from '../api/jmap-types.ts';

// The message list, virtualized for the §23 100k-row gate: only the rows inside
// the viewport (± overscan) are mounted, positioned inside a full-height spacer
// so the scrollbar still reflects the whole list. Each slot is absolutely
// positioned at the spacer's top (`rowCss.slot`) and moved to its place by
// `translateY(index × rowHeight)` alone. Rows keep the `.list__row`
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
//
// Selection (t28-e16): Ctrl/Cmd+click toggles a row, Shift+click selects the
// range from the last clicked row, a plain click opens the message and clears
// the selection. From the keyboard: Space toggles the focused row, Shift+Arrow
// extends the range, Ctrl/Cmd+A selects every row in view, Escape clears. A
// conversation head stands for all its messages. While anything is selected
// the floating selection bar (t58, MailSelectionBar.tsx) sits over the list,
// whose actions each send ONE request for the whole selection. The `next`/`previous` shortcuts are registered here with the
// one keymap dispatcher (src/keymap/); the keys above stay element-local.

/** Virtualized row height per density. Cozy is the historical default (72px);
 *  the message row is three lines, so these sit above the single-line density
 *  tokens in the theme contract. */
const ROW_HEIGHTS: Record<Density, number> = { compact: 56, cozy: 72, relaxed: 88 };

/**
 * How close the mounted window must come to the last LOADED row before the next
 * page is requested. Measured against loaded rows, not against the spacer's end:
 * the spacer now spans the whole folder, most of which is placeholders.
 */
const LOAD_MORE_SLACK = 10;

/** Anything in a display name that reads as an email address. */
const ADDRESS_IN_NAME = /[^\s<>"'(),;:]+@[^\s<>"'(),;:]+/g;

/**
 * The sender as the list shows it. The address is always part of the label: a
 * display name is whatever the sender typed, so a row that showed the name
 * alone let `"Your Bank" <x@evil.example>` read as "Your Bank".
 *
 *  - no name, or a name that is just the address: the address;
 *  - a name: `Name <address>`;
 *  - a name that itself contains an address other than the real one
 *    (`"ceo@bank.example" <x@evil.example>`): the real address first, then the
 *    name in parentheses, so the claimed address is not what the eye lands on.
 */
export function senderLabel(from: EmailAddress[] | null): string {
  const first = from?.[0];
  if (first === undefined) return t('mail-unknown-sender');
  const address = first.email;
  const name = (first.name ?? '').trim();
  if (name.length === 0 || name.toLowerCase() === address.toLowerCase()) return address;
  const claimed = name.match(ADDRESS_IN_NAME) ?? [];
  if (claimed.some((a) => a.toLowerCase() !== address.toLowerCase())) return `${address} (${name})`;
  return `${name} <${address}>`;
}

function formatDate(iso: string): string {
  return formatDateWith(iso, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit', timeZone: messageTimeZone() });
}

/** What a click did to the row, for the list to act on. */
type RowClick = (e: MouseEvent) => void;
/** A double-click on the row (t74: with the reading pane on it opens a tab). */
type RowDblClick = (e: MouseEvent) => void;

/** The check mark and screen-reader text a selected row carries. */
function SelectedMark(props: { selected: boolean }): JSX.Element {
  return (
    <Show when={props.selected}>
      <span class={bulk.check} aria-hidden="true">
        <Icon name="check" />
      </span>
      <span class={a11y.srOnly}>{t('mail-selected')}</span>
    </Show>
  );
}

type MessageMenu = {open:(event:MouseEvent,email:Email)=>void;keydown:(event:KeyboardEvent,email:Email)=>void;isOpen:(id:string)=>boolean};

function MessageRow(props: {
  contextMenu:MessageMenu;
  onDragStart:(event:DragEvent)=>void;
  email: Email;
  top: number;
  height: number;
  index: number;
  total: number;
  focused: boolean;
  selected: boolean;
  threadChild?: boolean;
  setRef: (el: HTMLButtonElement | undefined) => void;
  onClick: RowClick;
  onDblClick: RowDblClick;
}): JSX.Element {
  const app = useApp();
  const email = () => props.email;
  const unread = () => email().keywords?.['$seen'] !== true;
  return (
    <li
      class={`list__slot ${rowCss.slot}`}
      role="listitem"
      aria-posinset={props.index + 1}
      aria-setsize={props.total}
      style={{ transform: `translateY(${props.top}px)`, height: `${props.height}px` }}
    >
      <button
        type="button"
        ref={(el) => props.setRef(el)}
        aria-disabled={app.listLoading()}
        class={`list__row ${rowCss.row} ${a11y.focusable}`}
        classList={{
          'list__row--active': app.openEmail()?.id === email().id,
          'list__row--pinned': email().pinned === true,
          'list__row--unread': unread(),
          'list__row--selected': props.selected,
          [bulk.selectedRow]: props.selected,
          [thread.childRow]: props.threadChild === true,
        }}
        tabindex={props.focused ? 0 : -1}
        aria-current={app.openEmail()?.id === email().id ? 'true' : undefined}
        data-index={props.index}
        onClick={(e) => props.onClick(e)}
        onDblClick={(e) => props.onDblClick(e)}
        draggable={dragToFolderEnabled()&&!app.listLoading()} onDragStart={props.onDragStart} onDragEnd={clearMessageDrag}
        data-email-id={email().id} onContextMenu={e=>props.contextMenu.open(e,email())} onKeyDown={e=>props.contextMenu.keydown(e,email())}
      >
        <Show when={unread()}>
          <span class={a11y.srOnly}>{t('mail-unread')}</span>
        </Show>
        <span class={a11y.srOnly}>{t('mail-row-position', { pos: props.index + 1, total: props.total })}</span>
        <span class={`list__line1 ${rowCss.line1}`}>
          <SelectedMark selected={props.selected} />
          {/* isolate the sender: it shares a line with the date, so a spoofed
              display name must not reorder surrounding UI (SPEC §24). */}
          <span class={`list__sender ${rowCss.sender}`}><PersonAvatar name={email().from?.[0]?.name??undefined} address={email().from?.[0]?.email}/>{isolate(displayText(senderLabel(email().from)))}</span>
          <Show when={email().keywords?.['$flagged'] === true}>
            <span class={`list__flag ${bulk.flagMark}`} data-testid="row-flagged" role="img" aria-label={t('mail-flagged')}>
              <Icon name="flag" />
            </span>
          </Show>
          <Show when={email().pinned === true}>
            <span class="list__pin" aria-label={t('mail-pinned')}><Icon name="pin" /></span>
          </Show>
          <Show when={email().hasAttachment === true}>
            <span class="list__attach" aria-label={t('mail-has-attachment')}><Icon name="attachment" /></span>
          </Show>
          <Show when={email().keywords?.['$answered'] === true}>
            <span class="list__attach" data-testid="row-answered" aria-label={t('mail-answered')}><Icon name="reply" /></span>
          </Show>
          <Show when={email().keywords?.['$forwarded'] === true}>
            <span class="list__attach" data-testid="row-forwarded" aria-label={t('mail-forwarded')}><Icon name="forward" /></span>
          </Show>
          <span class={`list__date ${rowCss.date}`}>{formatDate(email().receivedAt)}</span>
        </span>
        <span class="list__subject">{displayText(email().subject ?? t('mail-no-subject'))}</span>
        <span class={`list__preview ${rowCss.preview}`}><EventHint hint={email().mailwomanCalendar} />{displayText(email().preview)}</span>
        <TagChips email={email()} />
      </button>
      <MessageActions email={email()} onMore={e=>props.contextMenu.open(e,email())} contextOpen={props.contextMenu.isOpen(email().id)}/>
    </li>
  );
}

/**
 * A slot for a row inside the query but not yet fetched (L4).
 *
 * It is the same height as a real row, so the scrollbar does not jump when the
 * page arrives, and it carries `aria-busy` so a screen reader announces the slot
 * as pending rather than as an empty message. It is NOT focusable: the roving
 * cursor stays inside loaded rows, because focusing a slot with no message would
 * announce nothing and strand the cursor.
 */
function PlaceholderRow(props: { top: number; height: number; index: number; total: number }): JSX.Element {
  return (
    <li
      class={`list__slot list__slot--pending ${rowCss.slot}`}
      role="listitem"
      aria-busy="true"
      aria-posinset={props.index + 1}
      aria-setsize={props.total}
      style={{ transform: `translateY(${props.top}px)`, height: `${props.height}px` }}
    >
      <span class={a11y.srOnly}>{t('mail-loading')}</span>
    </li>
  );
}

/** A collapsed conversation head (W2): the disclosure toggle sits beside the
 *  primary row; the primary row opens the latest message (like any message row),
 *  the toggle expands/collapses the members in place. */
function ThreadHeadRow(props: {
  contextMenu:MessageMenu;
  onDragStart:(event:DragEvent)=>void;
  row: ThreadVisualRow;
  top: number;
  height: number;
  index: number;
  total: number;
  focused: boolean;
  selected: boolean;
  setRef: (el: HTMLButtonElement | undefined) => void;
  onToggle: () => void;
  onClick: RowClick;
  onDblClick: RowDblClick;
}): JSX.Element {
  const app = useApp();
  const rep = () => props.row.email;
  const active = () => app.openEmail()?.id === rep().id;
  return (
    <li
      class={`list__slot ${rowCss.slot} ${thread.headSlot}`}
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
        <Icon name={props.row.expanded ? 'chevronDown' : 'chevron'} />
      </button>
      <button
        type="button"
        ref={(el) => props.setRef(el)}
        aria-disabled={app.listLoading()}
        class={`list__row ${rowCss.row} ${thread.headRow} ${a11y.focusable}`}
        classList={{
          'list__row--active': active(),
          'list__row--unread': props.row.unread,
          'list__row--selected': props.selected,
          [bulk.selectedRow]: props.selected,
        }}
        tabindex={props.focused ? 0 : -1}
        aria-current={active() ? 'true' : undefined}
        data-index={props.index}
        data-testid="thread-head"
        onClick={(e) => props.onClick(e)}
        onDblClick={(e) => props.onDblClick(e)}
        draggable={dragToFolderEnabled()&&!app.listLoading()} onDragStart={props.onDragStart} onDragEnd={clearMessageDrag}
        data-email-id={rep().id} onContextMenu={e=>props.contextMenu.open(e,rep())} onKeyDown={e=>props.contextMenu.keydown(e,rep())}
      >
        <Show when={props.row.unread}>
          <span class={a11y.srOnly}>{t('mail-unread')}</span>
        </Show>
        <span class={a11y.srOnly}>{t('mail-row-position', { pos: props.index + 1, total: props.total })}</span>
        <span class={`list__line1 ${rowCss.line1}`}>
          <SelectedMark selected={props.selected} />
          <span class={`list__sender ${rowCss.sender}`}><PersonAvatar name={rep().from?.[0]?.name??undefined} address={rep().from?.[0]?.email}/>{isolate(displayText(senderLabel(rep().from)))}</span>
          <span class={thread.count} aria-label={t('mail-thread-count', { count: props.row.count })}>
            {props.row.count}
          </span>
          <Show when={props.row.hasAttachment}>
            <span class="list__attach" aria-label={t('mail-has-attachment')}><Icon name="attachment" /></span>
          </Show>
          <span class={`list__date ${rowCss.date}`}>{formatDate(rep().receivedAt)}</span>
        </span>
        <span class="list__subject">{displayText(rep().subject ?? t('mail-no-subject'))}</span>
        <span class={`list__preview ${rowCss.preview}`}><EventHint hint={rep().mailwomanCalendar} />{displayText(rep().preview)}</span>
      </button>
      <MessageActions email={rep()} onMore={e=>props.contextMenu.open(e,rep())} contextOpen={props.contextMenu.isOpen(rep().id)}/>
    </li>
  );
}

/** The always-visible view toolbar: the reading-pane position control (W3). */
function ListToolbar(props:{selecting:boolean;toggleSelection:()=>void}): JSX.Element {
  const label = (opt: ReadingPane): string => t(`mail-reading-pane-${opt}`);
  return (
    <div class={thread.toolbar} role="group" aria-label={t('mail-view-options')}>
      <button type="button" class={`${thread.segBtn} ${a11y.focusable}`} aria-label={t('mail-select-messages')} aria-pressed={props.selecting} onClick={props.toggleSelection}><Icon name="bulkSelect"/></button>
      <EmptyTrash/><div class="list-reading-pane-controls"><span class={thread.toolbarLabel}>{t('mail-reading-pane')}</span>
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
    </div>
  );
}

// Test seam: a weak handle on the most recently mounted list's row-ref map, so
// a spec can assert the map stays bounded by the mounted window instead of
// growing to one entry per row ever scrolled past. Weak so the seam itself
// retains nothing once the list unmounts. Not read by application code.
let liveRowEls: WeakRef<Map<string, HTMLButtonElement>> | undefined;

/** Test-only: how many row refs the mounted list holds (-1 when none is up). */
export function rowRefCount(): number {
  return liveRowEls?.deref()?.size ?? -1;
}

/** A visual row's identity across list rebuilds: a conversation head by its
 *  thread key, any other row by its message. */
function rowId(row: ThreadVisualRow): string {
  return row.kind === 'head' ? `h:${row.key}` : `m:${row.email.id}`;
}

/**
 * `onOpenMessage`: a plain activation (click, tap, Enter, `next`/`previous`).
 * `onOpenMessageTab`: an explicit ask for a tab (double-click, the `openInTab`
 * shortcut); without it those fall back to `onOpenMessage`. Where each goes is
 * the host's decision (t74: Mailbox routes by the reading pane).
 */
export function MessageList(props: { onOpenMessage?: (id: string) => void; onOpenMessageTab?: (id: string) => void; onCompose?: (initial:ComposeInitial)=>void;
  /** t109: a message to bring to the top once the list has loaded (a reload's scroll anchor); one that is not loaded is skipped. */
  restoreAnchor?: () => string | null;
  /** t109: the message in the top row as the user scrolls (null at the top). */
  onAnchorChange?: (id: string | null) => void } = {}): JSX.Element {
  const app = useApp();
  const [contextEmail,setContextEmail]=createSignal<Email>();
  const contextMenu=createMessageContextMenu(contextEmail);
  const messageMenu:MessageMenu={open:(event,email)=>{setContextEmail(email);contextMenu.open(event);},keydown:(event,email)=>{if(event.key==='ContextMenu'||(event.shiftKey&&event.key==='F10')){setContextEmail(email);contextMenu.keydown(event);}},isOpen:id=>contextMenu.isOpen()&&contextEmail()?.id===id};
  const [selecting,setSelecting]=createSignal(false);
  createEffect(()=>{app.accountId();app.selectedMailboxId();setSelecting(false);});
  const [scrollTop, setScrollTop] = createSignal(0);
  // jsdom reports 0 for clientHeight; fall back to a sane viewport so tests
  // (and first paint before layout) still mount a window of rows.
  const [viewportH, setViewportH] = createSignal(600);
  // Roving keyboard cursor over the FULL list (independent of which rows are
  // currently mounted); defaults to the first row.
  const [cursor, setCursor] = createSignal(0);
  // Expanded conversation keys (W2). Toggling re-derives the visual rows.
  const [expanded, setExpanded] = createSignal<ReadonlySet<string>>(new Set());
  // Where a Shift+click or Shift+Arrow range starts: the row last clicked or
  // toggled (a visual-row index), or `null` before any.
  const [anchor, setAnchor] = createSignal<number | null>(null);

  const rowHeight = (): number => ROW_HEIGHTS[app.density()];
  // The FLAT list folded into visual rows (singletons + conversation heads/members).
  const rows = createMemo<ThreadVisualRow[]>(() => groupThreads(app.listMessages(), expanded()));
  // How many slots the list spans: the loaded rows plus the query's unfetched
  // tail. This is what makes the scrollbar and `aria-setsize` describe the FOLDER
  // rather than the page — before paging they described `rows().length`, so a
  // 20 000-message folder announced "1 of 50" behind a 3 600px scrollbar.
  const spanRows = createMemo(() => projectedRowCount(rows().length, app.total(), app.loadedRange().end));
  // `computeWindow` returns a fresh object every call, so without a comparator
  // this memo notifies on every scroll EVENT — including the many that leave
  // the mounted slice unchanged — and rebuilt the whole window each time.
  const win = createMemo(
    () => computeWindow(scrollTop(), viewportH(), rowHeight(), spanRows()),
    undefined,
    { equals: sameWindow },
  );
  // Which of the mounted slots have rows and which are still pending.
  const split = createMemo(() => splitWindow(win(), rows().length));

  // ── selection over visual rows (t28-e16) ────────────────────────────────
  // A conversation's messages, by thread key, so a head row (collapsed or not)
  // stands for every message in it.
  const membersByKey = createMemo(() => {
    const out = new Map<string, Id[]>();
    for (const m of app.listMessages()) {
      const key = threadKey(m);
      const ids = out.get(key);
      if (ids === undefined) out.set(key, [m.id]);
      else ids.push(m.id);
    }
    return out;
  });
  function idsOf(row: ThreadVisualRow): Id[] {
    return row.kind === 'head' ? (membersByKey().get(row.key) ?? [row.email.id]) : [row.email.id];
  }
  function isSelected(row: ThreadVisualRow): boolean {
    const sel = app.selectedIds();
    if (sel.size === 0) return false;
    return idsOf(row).every((id) => sel.has(id));
  }
  /** Select the rows from `from` to `to` (inclusive); `add` keeps what was selected. */
  function selectRange(from: number, to: number, add: boolean): void {
    const all = rows();
    const lo = Math.max(0, Math.min(from, to));
    const hi = Math.min(all.length - 1, Math.max(from, to));
    const ids = all.slice(lo, hi + 1).flatMap(idsOf);
    app.setSelection(add ? [...app.selectedIds(), ...ids] : ids);
  }
  const open = (id: Id): void => void (props.onOpenMessage ? props.onOpenMessage(id) : app.openMessage(id));
  const openTab = (id: Id): void => { if (props.onOpenMessageTab) props.onOpenMessageTab(id); else open(id); };
  function onRowClick(e: MouseEvent, index: number, row: ThreadVisualRow): void {
    if (app.listLoading()) return;
    setCursor(index);
    const action = classifyRowClick(e, { modifierSelection: modifierSelectionEnabled(), selecting: selecting() });
    if (action === 'range') {
      selectRange(anchor() ?? index, index, e.ctrlKey || e.metaKey);
      return;
    }
    setAnchor(index);
    if (action === 'toggle') {
      app.toggleSelection(idsOf(row));
      return;
    }
    // The second click of a double-click: the first one opened the message.
    if (action === 'ignore') return;
    app.clearSelection();
    open(row.email.id);
  }
  function onRowDblClick(e: MouseEvent, row: ThreadVisualRow): void {
    if (app.listLoading() || !dblClickOpensTab(e, { selecting: selecting() })) return;
    openTab(row.email.id);
  }

  function dragRow(event:DragEvent,row:ThreadVisualRow):void {
    const selected=app.selectedIds();
    const ids=idsOf(row);
    const targets=ids.some(id=>selected.has(id))?[...selected]:ids;
    if(targets.length>1000){event.preventDefault();app.showToast('error',t('mail-drag-too-many',{count:1000}));return;}
    beginMessageDrag(event,app.accountId(),targets);
  }

  let scroller: HTMLDivElement | undefined;
  createEffect(on(app.listLoading, (busy, previous) => {
    if (!busy && previous) {
      setScrollTop(0); setCursor(0);
      if (scroller) scroller.scrollTop = 0;
    }
  }));
  // After the reset above: a restored anchor lands once its rows are in.
  createEffect(on(() => props.restoreAnchor?.() ?? null, (id) => { if (id !== null) { pendingAnchor = id; queueMicrotask(applyAnchor); } }));
  createEffect(on([rows, app.listLoading], () => { if (pendingAnchor !== null) queueMicrotask(applyAnchor); }));
  // Keyed by ROW IDENTITY, not index. Two reasons: `<For>` is reference-keyed,
  // so a reused row object keeps its DOM node and its entry; and once paging
  // prepends rows every index shifts under an index-keyed map, resolving focus
  // to the wrong row or a detached node. The row `key` is the *thread* key —
  // shared by a head and its children — so it is not a usable identity here.
  // Entries are dropped in each row's `onCleanup`: Solid does not invoke a
  // `ref` callback on dispose, so nothing else would ever remove them.
  const rowEls = new Map<string, HTMLButtonElement>();
  liveRowEls = new WeakRef(rowEls);
  // The visual rows by their stable id (t44). `rows()` is rebuilt from scratch on
  // every list change (a mark-read, a sync, a flag), so the window below is keyed
  // by these ids, not by row objects: a keyed-by-object `<For>` remounted every
  // row on each change, focus fell to <body> and the arrow keys went dead.
  const rowById = createMemo(() => new Map(rows().map((row) => [rowId(row), row] as const)));
  // The id of the row that last held focus while focus is still in the list, so
  // a row removed under the focus (trashed, moved) hands it to its neighbour.
  let focusedRowId: string | null = null;
  const focusedRowEl = (): [string, HTMLButtonElement] | undefined => {
    const active = document.activeElement;
    for (const entry of rowEls) if (entry[1] === active) return entry;
    return undefined;
  };
  createEffect(on(rows, (all) => {
    const held = focusedRowEl();
    if (held !== undefined) {
      // Keep the roving cursor on the focused message when rows above it come or go.
      const at = all.findIndex((row) => rowId(row) === held[0]);
      if (at >= 0 && at !== cursor()) setCursor(at);
      return;
    }
    const lost = focusedRowId;
    const active = document.activeElement;
    if (lost === null || rowById().has(lost) || (active !== null && active !== document.body)) return;
    focusedRowId = null;
    if (all.length > 0) moveCursor(Math.min(cursor(), all.length - 1));
  }, { defer: true }));
  function onListFocusIn(e: FocusEvent): void {
    const entry = focusedRowEl();
    if (entry !== undefined && entry[1] === e.target) focusedRowId = entry[0];
  }
  function onListFocusOut(e: FocusEvent): void {
    const next = e.relatedTarget;
    if (next instanceof Node && scroller?.contains(next)) return;
    const left = e.target;
    // Focus that moves elsewhere, or a blur that keeps the row in the document,
    // is the user leaving the list; a row removed from the document is not.
    if (next !== null) focusedRowId = null;
    else setTimeout(() => { if (left instanceof Node && left.isConnected && !scroller?.contains(document.activeElement)) focusedRowId = null; }, 0);
  }

  function toggleThread(key: string): void {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  }

  /**
   * Request the next page once the mounted window nears the end of what is
   * loaded. This is the ONLY thing that turns paging into a user-reachable
   * feature: every server- and state-side piece landed in `3a8fefb`, but until
   * a DOM event calls `loadMore()` the list stops at the first page and the app
   * looks exactly as it did before paging existed.
   *
   * Gated on `loadingMore()` because scroll fires continuously — a handler
   * without it issues a request per scroll event, dozens per gesture — and on
   * `listLoading()` because a whole-list replace is already in flight and an
   * append onto a list about to be discarded is wasted.
   *
   * Nothing here guards against re-requesting a page that came back holding only
   * ids already loaded. It does not need to: `mail.ts` ends the query when a page
   * adds nothing, so `hasMore()` is already false by the next scroll event. That
   * guard lived here briefly and was moved once the state layer could express it —
   * two copies of the same rule is how they drift apart.
   */
  function maybeLoadMore(): void {
    if (!app.hasMore() || app.loadingMore() || app.listLoading()) return;
    if (win().endIndex >= rows().length - LOAD_MORE_SLACK) void app.loadMore();
  }

  function onScroll(): void {
    if (scroller !== undefined) {
      setScrollTop(scroller.scrollTop);
      if (scroller.clientHeight > 0) setViewportH(scroller.clientHeight);
      reportAnchor();
    }
    maybeLoadMore();
  }
  // t109: the scroll position is kept as the top row's message, not pixels, so
  // it survives a density change and rows arriving above it.
  let pendingAnchor: string | null = null;
  let reportedAnchor: string | null | undefined;
  function reportAnchor(): void {
    if (!props.onAnchorChange || scroller === undefined || pendingAnchor !== null || app.listLoading()) return;
    const top = scroller.scrollTop < rowHeight() / 2 ? undefined : rows()[Math.round(scroller.scrollTop / rowHeight())];
    const id = top === undefined ? null : top.email.id;
    if (id === reportedAnchor) return;
    reportedAnchor = id;
    props.onAnchorChange(id);
  }
  function applyAnchor(): void {
    const id = pendingAnchor;
    if (id === null || app.listLoading() || scroller === undefined) return;
    const all = rows();
    if (all.length === 0) return;
    pendingAnchor = null;
    const index = all.findIndex((row) => row.email.id === id || (row.kind === 'head' && idsOf(row).includes(id)));
    if (index < 0) return; // Gone, or past the loaded pages: the list stays at the top.
    reportedAnchor = id;
    scroller.scrollTop = index * rowHeight();
    setScrollTop(scroller.scrollTop);
    setCursor(index);
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
      // The selection bar covers the scroller's bottom padding (t58): a row is
      // in view only above it.
      const visible = scroller.clientHeight - (parseFloat(getComputedStyle(scroller).paddingBottom) || 0);
      if (rowTop < scroller.scrollTop) scroller.scrollTop = rowTop;
      else if (rowBottom > scroller.scrollTop + visible) {
        scroller.scrollTop = rowBottom - visible;
      }
      onScroll();
    }
    // Focus after the window re-renders the target row. Resolve the row object
    // inside the microtask so the lookup uses the post-render list. A key moved
    // it, so the ring shows: Firefox otherwise keeps a click's no-ring state
    // for focus a script moves (the option is not in TypeScript's DOM lib yet).
    // A list hidden behind the reader (reading pane Off, phone width) does not
    // take focus: `next`/`previous` pressed in the reader keep it there instead
    // of dropping it on <body> through a hidden row (t51).
    queueMicrotask(() => {
      const row = rows()[next];
      const el = row === undefined ? undefined : rowEls.get(rowId(row));
      if (el === undefined || el.closest('[hidden], [inert]') !== null) return;
      el.focus({ focusVisible: true } as FocusOptions);
    });
  }

  /** Shift+Arrow: move the cursor and select from the anchor to it. */
  function extendTo(to: number): void {
    const count = rows().length;
    const next = Math.max(0, Math.min(count - 1, to));
    const from = anchor() ?? cursor();
    if (anchor() === null) setAnchor(from);
    moveCursor(next);
    selectRange(from, next, false);
  }

  function onKeyDown(e: KeyboardEvent): void {
    if (app.listLoading()) return;
    const count = rows().length;
    if (count === 0) return;
    // Selection keys act on a row, not on the thread toggle or a row's action
    // buttons, which have their own meaning for Space.
    const onRow = e.target instanceof Element && e.target.classList.contains('list__row');
    const mod = e.ctrlKey || e.metaKey;
    if (e.key === 'ArrowDown' && e.shiftKey && modifierSelectionEnabled()) extendTo(cursor() + 1);
    else if (e.key === 'ArrowUp' && e.shiftKey && modifierSelectionEnabled()) extendTo(cursor() - 1);
    else if (e.key === 'ArrowDown') moveCursor(cursor() + 1);
    else if (e.key === 'ArrowUp') moveCursor(cursor() - 1);
    else if (e.key === 'Home') moveCursor(0);
    else if (e.key === 'End') moveCursor(count - 1);
    else if (e.key === 'PageDown' || e.key === 'PageUp') {
      const page = Math.max(1, Math.floor((scroller?.clientHeight || viewportH()) / rowHeight()) - 1);
      moveCursor(cursor() + (e.key === 'PageDown' ? page : -page));
    }
    else if (e.key === ' ' && onRow && !mod) {
      const row = rows()[cursor()];
      if (row === undefined) return;
      setAnchor(cursor());
      app.toggleSelection(idsOf(row));
    } else if ((e.key === 'a' || e.key === 'A') && mod && onRow && modifierSelectionEnabled()) app.selectAllInView();
    else if (e.key === 'Escape' && app.selectedIds().size > 0) app.clearSelection();
    else return;
    e.preventDefault();
  }

  /** `next` / `previous` shortcuts: move to the neighbouring row and open it.
   *  Counted from the open message's row when there is one, else the cursor. */
  function step(delta: number): boolean {
    const all = rows();
    if (all.length === 0) return false;
    const openId = app.openEmail()?.id;
    const at = openId === undefined ? -1 : all.findIndex((r) => r.email.id === openId);
    const from = at >= 0 ? at : cursor();
    const to = Math.max(0, Math.min(all.length - 1, from + delta));
    moveCursor(to);
    const row = all[to];
    if (row !== undefined && row.email.id !== openId) open(row.email.id);
    return true;
  }
  useActions({
    next: () => step(1), previous: () => step(-1),
    // Shift+Enter by default (t74): the focused row's message in its own tab.
    openInTab: () => {
      const focused = document.activeElement;
      const row = rows().find(item => rowEls.get(rowId(item)) === focused);
      if (!row) return false;
      if (app.listLoading()) return true;
      openTab(row.email.id);
      return true;
    },
    trash: () => {
      const focused = document.activeElement;
      const row = rows().find(item => rowEls.get(rowId(item)) === focused);
      if (!row) return false;
      if (app.listLoading()) return true;
      const selected = app.selectedIds();
      const targets = selected.size > 0 ? [...selected] : idsOf(row);
      void app.trashMessage(targets);
      return true;
    },
  });

  // No wrapper objects: `<For>` keys on reference, and a fresh `{row, index}`
  // wrapper per render made every mounted row unmount and remount on each
  // window change. The visual rows themselves are stable while `rows()` holds,
  // so the slice alone lets `<For>` reuse them; the absolute position comes
  // from `<For>`'s index accessor plus the window start.
  const slice = createMemo(() => {
    const s = split();
    return rows().slice(s.loadedStart, s.loadedEnd).map(rowId);
  });
  // The mounted slots with no row yet, as absolute indices. Plain numbers so
  // `<For>` reconciles them by value and reuses the slot elements a scroll keeps.
  const pendingSlots = createMemo(() => {
    const s = split();
    const out: number[] = [];
    for (let i = s.pendingStart; i < s.pendingEnd; i += 1) out.push(i);
    return out;
  });

  // First sync (dcd88d3): an empty folder that the engine has not indexed yet
  // is "still loading", not "No messages". The list reloads once it is indexed.
  const sync = createInitialSync(() => (app.searchActive() ? null : app.selectedMailboxId()), () => app.refreshCurrentMailbox());
  const showSkeleton = (): boolean => rows().length === 0 && (app.listLoading() || app.switchingMailbox() || sync.pending());
  const skeletonRows = (): number => Math.max(4, Math.min(16, Math.ceil((globalThis.innerHeight || 800) / rowHeight())));

  // One delayed spinner covers initial loads, folder changes and linked/primary
  // account switches. Updating the destination replaces this toast in place.
  const loadToast = createBackgroundLoadToast({ id: 'mail-list-loading', testId: 'mail-loading-toast' });
  createEffect(() => {
    const switching = app.switchingMailbox();
    const loading = app.listLoading() || sync.pending();
    const destination = switching ? app.switchingMailboxName() : app.mailboxes().find(box => box.id === app.selectedMailboxId())?.name;
    if (switching || loading) loadToast.start(destination ? t('mail-loading-mailbox', {name: destination}) : t('mail-loading'));
    else if (app.loadingMore()) loadToast.start(t('mail-loading-more'));
    else loadToast.finish();
  });
  onCleanup(() => loadToast.cancel());

  return (
    <section class="list" aria-label={t('mail-list-label')} aria-busy={app.listLoading() || showSkeleton()}>
      <ListToolbar selecting={selecting()} toggleSelection={()=>setSelecting(value=>!value)}/>
      <OfflineQueueNotice />
      <Show when={sync.running() && rows().length > 0}><div class={loadingCss.syncProgress} aria-hidden="true" data-testid="sync-progress" /></Show>
        <Show when={rows().length > 0} fallback={
          <Show when={showSkeleton()} fallback={
            <Show when={sync.failed()} fallback={<p class="list__empty">{t('mail-empty')}</p>}>
              <AsyncError message={t('mail-initial-sync-error')} detail={t('mail-initial-sync-error-help')} onRetry={() => sync.retry()} />
            </Show>
          }>
            <AsyncPending variant="messages" quiet bounded={!sync.pending()} rowHeight={rowHeight()} rows={skeletonRows()} message={t('mail-loading')} onRetry={()=>void app.refreshCurrentMailbox()}/>
          </Show>
        }>
          <div
            class="list__scroll"
            classList={{ [barCss.listInset]: selecting() || app.selectedIds().size > 0 }}
            inert={app.listLoading()}
            ref={scroller}
            onScroll={onScroll}
            onKeyDown={onKeyDown}
            onFocusIn={onListFocusIn}
            onFocusOut={onListFocusOut}
          >
            <ul
              class="list__items"
              role="list"
              aria-label={t('mail-list-label')}
              style={{ height: `${win().totalHeight}px` }}
            >
              <For each={pendingSlots()}>
                {(index) => (
                  <PlaceholderRow
                    top={index * rowHeight()}
                    height={rowHeight()}
                    index={index}
                    total={spanRows()}
                  />
                )}
              </For>
              <For each={slice()}>
                {(id, i) => {
                  const index = (): number => split().loadedStart + i();
                  // The current row for this id. The last one is kept for the
                  // moment between the id leaving the list and this slot's disposal.
                  let last = rowById().get(id)!;
                  const row = (): ThreadVisualRow => (last = rowById().get(id) ?? last);
                  const setRef = (el: HTMLButtonElement | undefined): void => {
                    if (el) rowEls.set(id, el);
                    else rowEls.delete(id);
                  };
                  // The only path that removes the ref: Solid never calls a
                  // `ref` callback back on dispose.
                  onCleanup(() => rowEls.delete(id));
                  return (
                    <Show
                      when={row().kind === 'head'}
                      fallback={
                        <MessageRow contextMenu={messageMenu}
                          email={row().email}
                          top={index() * rowHeight()}
                          height={rowHeight()}
                          index={index()}
                          total={spanRows()}
                          focused={index() === cursor()}
                          selected={isSelected(row())}
                          threadChild={row().kind === 'child'}
                          setRef={setRef}
                          onClick={(e) => onRowClick(e, index(), row())}
                          onDblClick={(e) => onRowDblClick(e, row())}
                          onDragStart={event=>dragRow(event,row())}
                        />
                      }
                    >
                      <ThreadHeadRow contextMenu={messageMenu}
                        row={row()}
                        top={index() * rowHeight()}
                        height={rowHeight()}
                        index={index()}
                        total={spanRows()}
                        focused={index() === cursor()}
                        selected={isSelected(row())}
                        onToggle={() => toggleThread(row().key)}
                        setRef={setRef}
                        onClick={(e) => onRowClick(e, index(), row())}
                        onDblClick={(e) => onRowDblClick(e, row())}
                        onDragStart={event=>dragRow(event,row())}
                      />
                    </Show>
                  );
                }}
              </For>
            </ul>
          </div>
        </Show>
      {/* The floating selection bar (t58): over the list, after it in tab order.
          It never takes focus itself; Esc/F6 in it come back to the cursor row. */}
      <MailSelectionBar selecting={selecting()} onCompose={props.onCompose} onClear={()=>{app.clearSelection();setSelecting(false);}} returnFocus={()=>moveCursor(cursor())}/>
      {contextMenu.menu()}
    </section>
  );
}
