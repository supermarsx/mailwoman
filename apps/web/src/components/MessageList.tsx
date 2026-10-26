import {AsyncPending} from './AsyncState.tsx';
import {createMessageContextMenu} from './MessageContextMenu.tsx';
import {PersonAvatar} from './PersonAvatar.tsx';
import {EmptyTrash} from './EmptyTrash.tsx';
import {InitialSyncNotice} from './InitialSyncNotice.tsx';
import { Icon } from './Icon.tsx';
import {downloadMessages,forwardMessages} from '../api/mailBulk.ts';
import {preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import type {ComposeInitial} from './compose/reply.ts';
import { createMemo, createSignal, createEffect, on, For, onCleanup, Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { t, isolate } from '../i18n/index.ts';
import { formatDateWith } from '../i18n/format.ts';
import { messageTimeZone } from './TimeHelpers.tsx';
import { useActions } from '../keymap/index.ts';
import { computeWindow, projectedRowCount, sameWindow, splitWindow } from './virtual.ts';
import { TagChips } from './TagChips.tsx';
import { MessageActions } from './MessageActions.tsx';
import { OfflineQueueNotice } from './OfflineQueueNotice.tsx';
import * as a11y from './mailA11y.css.ts';
import * as thread from './threadList.css.ts';
import * as rowCss from './messageRow.css.ts';
import * as bulk from './bulkBar.css.ts';
import { groupThreads, threadKey, type ThreadVisualRow } from './threads.ts';
import { readingPane, setReadingPane, READING_PANE_OPTIONS, type ReadingPane } from './readingPane.ts';
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
// the toolbar becomes the bulk bar, whose actions each send ONE request for the
// whole selection. The `next`/`previous` shortcuts are registered here with the
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
          <span class={`list__sender ${rowCss.sender}`}><PersonAvatar name={email().from?.[0]?.name??undefined} address={email().from?.[0]?.email}/>{isolate(senderLabel(email().from))}</span>
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
        <span class="list__subject">{email().subject ?? t('mail-no-subject')}</span>
        <span class={`list__preview ${rowCss.preview}`}>{email().preview}</span>
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
        data-email-id={rep().id} onContextMenu={e=>props.contextMenu.open(e,rep())} onKeyDown={e=>props.contextMenu.keydown(e,rep())}
      >
        <Show when={props.row.unread}>
          <span class={a11y.srOnly}>{t('mail-unread')}</span>
        </Show>
        <span class={a11y.srOnly}>{t('mail-row-position', { pos: props.index + 1, total: props.total })}</span>
        <span class={`list__line1 ${rowCss.line1}`}>
          <SelectedMark selected={props.selected} />
          <span class={`list__sender ${rowCss.sender}`}><PersonAvatar name={rep().from?.[0]?.name??undefined} address={rep().from?.[0]?.email}/>{isolate(senderLabel(rep().from))}</span>
          <span class={thread.count} aria-label={t('mail-thread-count', { count: props.row.count })}>
            {props.row.count}
          </span>
          <Show when={props.row.hasAttachment}>
            <span class="list__attach" aria-label={t('mail-has-attachment')}><Icon name="attachment" /></span>
          </Show>
          <span class={`list__date ${rowCss.date}`}>{formatDate(rep().receivedAt)}</span>
        </span>
        <span class="list__subject">{rep().subject ?? t('mail-no-subject')}</span>
        <span class={`list__preview ${rowCss.preview}`}>{rep().preview}</span>
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
      <button type="button" class={`${thread.segBtn} ${a11y.focusable}`} aria-label={t('mail-select-messages')} aria-pressed={props.selecting} onClick={props.toggleSelection}><Icon name="check"/></button>
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

/**
 * The bulk bar (t28-e16): the toolbar while messages are selected. Every action
 * applies to the whole selection in ONE request (see `mail.ts` `updateEmails`)
 * and raises the shared undo. Flag reads "Remove flag" when every selected
 * message is already flagged.
 */
function BulkBar(props:{onCompose?:((initial:ComposeInitial)=>void)|undefined;onClear:()=>void}): JSX.Element {
  const app = useApp();
  const [busy,setBusy]=createSignal(false);const [error,setError]=createSignal('');
  const controller=new AbortController();onCleanup(()=>controller.abort());
  const targets = (): Id[] => app.actionTargets();
  function forward():void {
    setError('');try {const ids=new Set(targets());props.onCompose?.(forwardMessages(app.messages().filter(message=>ids.has(message.id)),t('mail-bulk-forward-subject',{count:ids.size})));}catch{setError(t('mail-bulk-transfer-failed'));}
  }
  async function download():Promise<void>{
    if(busy())return;const owner=preferenceAccount();if(!owner)return;setBusy(true);setError('');
    try {const blob=await downloadMessages(targets(),()=>preferenceAccountIsCurrent(owner),controller.signal);if(!preferenceAccountIsCurrent(owner))return;const url=URL.createObjectURL(blob);const anchor=document.createElement('a');anchor.href=url;anchor.download='messages.mbox';document.body.append(anchor);anchor.click();anchor.remove();setTimeout(()=>URL.revokeObjectURL(url),10000);}
    catch {if(!controller.signal.aborted&&preferenceAccountIsCurrent(owner))setError(t('mail-bulk-transfer-failed'));}finally{setBusy(false);}
  }
  const allFlagged = (): boolean => {
    const sel = app.selectedIds();
    const chosen = app.messages().filter((m) => sel.has(m.id));
    return chosen.length > 0 && chosen.every((m) => m.keywords?.['$flagged'] === true);
  };
  const moveTargets = () => app.mailboxes().filter((b) => b.id !== app.selectedMailboxId());
  const btn = `${thread.segBtn} ${a11y.focusable}`;
  return (
    <div
      class={`${thread.toolbar} ${bulk.bar}`}
      role="toolbar"
      aria-label={t('mail-bulk-label')}
      data-testid="bulk-bar"
    >
      <span class={bulk.count} aria-live="polite" data-testid="bulk-count">
        {t('mail-bulk-count', { count: app.selectedIds().size })}
      </span>
      <button type="button" class={btn} onClick={() => void app.markRead(targets(), true)}>
        {t('mail-mark-read')}
      </button>
      <button type="button" class={btn} onClick={() => void app.markRead(targets(), false)}>
        {t('mail-mark-unread')}
      </button>
      <button type="button" class={btn} onClick={() => void app.flagMessage(targets(), !allFlagged())}>
        {allFlagged() ? t('mail-unflag-message') : t('mail-flag-message')}
      </button>
      <button type="button" class={btn} onClick={() => void app.archiveMessage(targets())}>
        {t('mail-archive')}
      </button>
      <select
        class={`${bulk.select} ${a11y.focusable}`}
        aria-label={t('mail-move-to')}
        data-testid="bulk-move"
        onChange={(e) => {
          const box = e.currentTarget.value;
          e.currentTarget.value = '';
          if (box !== '') void app.moveMessage(targets(), box);
        }}
      >
        <option value="">{t('mail-move-to-placeholder')}</option>
        <For each={moveTargets()}>{(box) => <option value={box.id}>{box.name}</option>}</For>
      </select>
      <button type="button" class={btn} onClick={() => void app.trashMessage(targets())}>
        {t('mail-delete')}
      </button>
      <button type="button" class={btn} aria-label={t('mail-mark-spam')} onClick={()=>void app.markSpam(targets())}><Icon name="shield"/></button>
      <select class={`${bulk.select} ${a11y.focusable}`} aria-label={t('mail-bulk-add-label')} onChange={event=>{const keyword=event.currentTarget.value;event.currentTarget.value='';if(keyword)void app.tagMessages(targets(),keyword,true,app.tagByKeyword(keyword)?.name);}}><option value="">{t('mail-bulk-add-label')}</option><For each={app.tags()}>{tag=><option value={tag.id}>{tag.name}</option>}</For></select>
      <select class={`${bulk.select} ${a11y.focusable}`} aria-label={t('mail-bulk-remove-label')} onChange={event=>{const keyword=event.currentTarget.value;event.currentTarget.value='';if(keyword)void app.tagMessages(targets(),keyword,false,app.tagByKeyword(keyword)?.name);}}><option value="">{t('mail-bulk-remove-label')}</option><For each={app.tags()}>{tag=><option value={tag.id}>{tag.name}</option>}</For></select>
      <Show when={props.onCompose}><button type="button" class={btn} aria-label={t('mail-bulk-forward')} onClick={forward}><Icon name="forward"/></button></Show>
      <button type="button" class={btn} disabled={busy()} aria-label={t('mail-bulk-download')} onClick={()=>void download()}><Icon name="download"/></button>
      <Show when={error()}><span role="alert">{error()}</span></Show>
      <button type="button" class={btn} onClick={() => app.selectAllInView()}>
        {t('mail-select-all')}
      </button>
      <button type="button" class={btn} aria-label={t('mail-clear-selection')} onClick={props.onClear}>
        <Icon name="close" />
      </button>
    </div>
  );
}

// Test seam: a weak handle on the most recently mounted list's row-ref map, so
// a spec can assert the map stays bounded by the mounted window instead of
// growing to one entry per row ever scrolled past. Weak so the seam itself
// retains nothing once the list unmounts. Not read by application code.
let liveRowEls: WeakRef<Map<ThreadVisualRow, HTMLButtonElement>> | undefined;

/** Test-only: how many row refs the mounted list holds (-1 when none is up). */
export function rowRefCount(): number {
  return liveRowEls?.deref()?.size ?? -1;
}

export function MessageList(props: { onOpenMessage?: (id: string) => void; onCompose?: (initial:ComposeInitial)=>void } = {}): JSX.Element {
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
  function onRowClick(e: MouseEvent, index: number, row: ThreadVisualRow): void {
    if (app.listLoading()) return;
    setCursor(index);
    if (e.shiftKey) {
      selectRange(anchor() ?? index, index, e.ctrlKey || e.metaKey);
      return;
    }
    setAnchor(index);
    if (e.ctrlKey || e.metaKey || selecting()) {
      app.toggleSelection(idsOf(row));
      return;
    }
    app.clearSelection();
    void (props.onOpenMessage ? props.onOpenMessage(row.email.id) : app.openMessage(row.email.id));
  }

  let scroller: HTMLDivElement | undefined;
  createEffect(on(app.listLoading, (busy, previous) => {
    if (!busy && previous) {
      setScrollTop(0); setCursor(0);
      if (scroller) scroller.scrollTop = 0;
    }
  }));
  // Keyed by ROW IDENTITY, not index. Two reasons: `<For>` is reference-keyed,
  // so a reused row object keeps its DOM node and its entry; and once paging
  // prepends rows every index shifts under an index-keyed map, resolving focus
  // to the wrong row or a detached node. The row `key` is the *thread* key —
  // shared by a head and its children — so it is not a usable identity here.
  // Entries are dropped in each row's `onCleanup`: Solid does not invoke a
  // `ref` callback on dispose, so nothing else would ever remove them.
  const rowEls = new Map<ThreadVisualRow, HTMLButtonElement>();
  liveRowEls = new WeakRef(rowEls);

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
    }
    maybeLoadMore();
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
    // Focus after the window re-renders the target row. Resolve the row object
    // inside the microtask so the lookup uses the post-render list.
    queueMicrotask(() => {
      const row = rows()[next];
      if (row !== undefined) rowEls.get(row)?.focus();
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
    if (e.key === 'ArrowDown' && e.shiftKey) extendTo(cursor() + 1);
    else if (e.key === 'ArrowUp' && e.shiftKey) extendTo(cursor() - 1);
    else if (e.key === 'ArrowDown') moveCursor(cursor() + 1);
    else if (e.key === 'ArrowUp') moveCursor(cursor() - 1);
    else if (e.key === 'Home') moveCursor(0);
    else if (e.key === 'End') moveCursor(count - 1);
    else if (e.key === ' ' && onRow && !mod) {
      const row = rows()[cursor()];
      if (row === undefined) return;
      setAnchor(cursor());
      app.toggleSelection(idsOf(row));
    } else if ((e.key === 'a' || e.key === 'A') && mod && onRow) app.selectAllInView();
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
    if (row !== undefined && row.email.id !== openId) void (props.onOpenMessage ? props.onOpenMessage(row.email.id) : app.openMessage(row.email.id));
    return true;
  }
  useActions({ next: () => step(1), previous: () => step(-1) });

  // No wrapper objects: `<For>` keys on reference, and a fresh `{row, index}`
  // wrapper per render made every mounted row unmount and remount on each
  // window change. The visual rows themselves are stable while `rows()` holds,
  // so the slice alone lets `<For>` reuse them; the absolute position comes
  // from `<For>`'s index accessor plus the window start.
  const slice = createMemo(() => {
    const s = split();
    return rows().slice(s.loadedStart, s.loadedEnd);
  });
  // The mounted slots with no row yet, as absolute indices. Plain numbers so
  // `<For>` reconciles them by value and reuses the slot elements a scroll keeps.
  const pendingSlots = createMemo(() => {
    const s = split();
    const out: number[] = [];
    for (let i = s.pendingStart; i < s.pendingEnd; i += 1) out.push(i);
    return out;
  });

  return (
    <section class="list" aria-label={t('mail-list-label')} aria-busy={app.listLoading()}>
      <Show when={app.selectedIds().size > 0} fallback={<ListToolbar selecting={selecting()} toggleSelection={()=>setSelecting(value=>!value)}/>}>
        <BulkBar onCompose={props.onCompose} onClear={()=>{app.clearSelection();setSelecting(false);}}/>
      </Show>
      <OfflineQueueNotice />
      <InitialSyncNotice />
      <Show when={app.listLoading() && rows().length > 0}><div class="list__loading" role="status">{t('mail-loading')}</div></Show>
        <Show when={rows().length > 0} fallback={<Show when={app.listLoading()} fallback={<p class="list__empty">{t('mail-empty')}</p>}><AsyncPending variant="list" message={t('mail-loading')} onRetry={()=>void app.refreshCurrentMailbox()}/></Show>}>
          <div
            class="list__scroll"
            inert={app.listLoading()}
            ref={scroller}
            onScroll={onScroll}
            onKeyDown={onKeyDown}
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
                {(row, i) => {
                  const index = (): number => split().loadedStart + i();
                  const setRef = (el: HTMLButtonElement | undefined): void => {
                    if (el) rowEls.set(row, el);
                    else rowEls.delete(row);
                  };
                  // The only path that removes the ref: Solid never calls a
                  // `ref` callback back on dispose.
                  onCleanup(() => rowEls.delete(row));
                  return (
                    <Show
                      when={row.kind === 'head'}
                      fallback={
                        <MessageRow contextMenu={messageMenu}
                          email={row.email}
                          top={index() * rowHeight()}
                          height={rowHeight()}
                          index={index()}
                          total={spanRows()}
                          focused={index() === cursor()}
                          selected={isSelected(row)}
                          threadChild={row.kind === 'child'}
                          setRef={setRef}
                          onClick={(e) => onRowClick(e, index(), row)}
                        />
                      }
                    >
                      <ThreadHeadRow contextMenu={messageMenu}
                        row={row}
                        top={index() * rowHeight()}
                        height={rowHeight()}
                        index={index()}
                        total={spanRows()}
                        focused={index() === cursor()}
                        selected={isSelected(row)}
                        onToggle={() => toggleThread(row.key)}
                        setRef={setRef}
                        onClick={(e) => onRowClick(e, index(), row)}
                      />
                    </Show>
                  );
                }}
              </For>
            </ul>
          </div>
        </Show>
      {contextMenu.menu()}
    </section>
  );
}
