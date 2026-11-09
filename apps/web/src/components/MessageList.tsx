import {hasFileAttachments} from './attachmentMetadata.ts';
import {mailPrefetchEvents} from './mailPrefetchEvents.ts';
import {ReplyDraftIndicator} from './ReplyState.tsx';
import {createReplyDraftIndex,type ReplyDraft} from './compose/replyDrafts.ts';
import {AsyncError,AsyncPending} from './AsyncState.tsx';
import {createInitialSync} from './initialSync.ts';
import * as loadingCss from './asyncState.css.ts';
import * as listLoadingCss from './messageListLoading.css.ts';
import {createMessageListMotion} from './messageListMotion.ts';
import {observePreferenceAccount,preferenceAccount} from '../api/accountPreferenceLifecycle.ts';
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
import { observeListViewport, viewportNeedsPage } from './listViewport.ts';
import { TagChips } from './TagChips.tsx';
import { EventHint } from './EventHint.tsx';
import { MessageActions } from './MessageActions.tsx';
import { OfflineQueueNotice } from './OfflineQueueNotice.tsx';
import { createBackgroundLoadToast } from './backgroundLoadToast.ts';
import * as a11y from './mailA11y.css.ts';
import * as thread from './threadList.css.ts';
import * as rowCss from './messageRow.css.ts';
import {rowExpired,visibleRows} from './listExpiry.ts';
import * as bulk from './bulkBar.css.ts';
import './messageFlash.css.ts';
import {focusRequest,clearFocusRequest,focusMessageInFolder} from './messageFocus.ts';
import { groupThreads, threadKey, type ThreadVisualRow } from './threads.ts';
import {ReadingPaneControl} from './ReadingPaneControl.tsx';
import {settings} from '../api/settings.ts';
import {createContextMenu} from './ContextMenu.tsx';
import {createMessageOrder,isMessageListOrder,MESSAGE_LIST_ORDERS,type MessageOrderController} from './messageOrder.ts';
import {MessageOrderMenu} from './MessageOrderMenu.tsx';
import {PriorityMark} from './PriorityMark.tsx';
import {isPropertiesShortcut,openMessageProperties} from './properties/host.tsx';
import {messageDialogEnabled} from './properties/policy.ts';
import {priorityOf} from './messagePriority.ts';
import {mailFeature} from '../state/mailFeatures.ts';
import { classifyRowClick, dblClickOpensTab } from './messageOpen.ts';
import { noteFolderSelected, noteMailboxList } from '../perf/clientTiming.ts';
import type { Density } from '../theme/contract.css.ts';
import type { Email, EmailAddress, Id } from '../api/jmap-types.ts';
import { EVENTS_VIEW_ID } from '../api/jmap.ts';

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

/** `thread` marks a conversation head: its menu then acts on the conversation's messages, not only the latest. */
type MessageMenu = {open:(event:MouseEvent,email:Email,thread?:boolean)=>void;keydown:(event:KeyboardEvent,email:Email,thread?:boolean)=>void;isOpen:(id:string)=>boolean};

/** Whether the open list mixes messages of several folders: search results, a saved-search view, an all-mail folder or the events view. */
export function listSpansFolders(app:{searchActive:()=>boolean;selectedMailboxId:()=>string|null;mailboxes:()=>{id:string;role?:string|null;mailwomanSearchQuery?:string|undefined}[]}): boolean {
  if (app.searchActive()) return true;
  const id = app.selectedMailboxId();
  if (id === EVENTS_VIEW_ID) return true;
  const box = app.mailboxes().find((value) => value.id === id);
  return box !== undefined && (box.mailwomanSearchQuery !== undefined || box.role === 'all');
}

/** In a list that spans folders (search, saved-search and all-mail views) the row's folder, as a real button that goes to the folder and flashes the message. */
function FolderChip(props:{email:Email}): JSX.Element {
  const app = useApp();
  const folder = () => listSpansFolders(app) ? Object.keys(props.email.mailboxIds ?? {}).map((id) => app.mailboxes().find((box) => box.id === id && props.email.mailboxIds[id])).find((box) => box !== undefined) : undefined;
  return (
    <Show when={folder()} keyed>
      {(box) => (
        <button type="button" class="list__folder-chip" data-testid="row-folder" aria-label={t('mail-focus-go-to', { folder: isolate(box.name) })} title={t('mail-focus-go-to', { folder: isolate(box.name) })}
          onClick={(e) => { e.stopPropagation(); void focusMessageInFolder(app, box.id, props.email.id); }}>
          <Icon name="folder" />{isolate(box.name)}
        </button>
      )}
    </Show>
  );
}

function MessageRow(props: {
  replyDraft:ReplyDraft|undefined;
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
  const prefetch=mailPrefetchEvents(trigger=>app.prefetchMessage?.(email().id,trigger)??(()=>undefined));
  const unread = () => email().keywords?.['$seen'] !== true;
  return (
    <li
      class={`list__slot ${rowCss.slot}`}
      data-motion-row={`m:${email().id}`}
      data-reply-draft={props.replyDraft?.id}
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
          [rowCss.expired]: rowExpired(email()),
        }}
        tabindex={props.focused ? 0 : -1}
        aria-current={app.openEmail()?.id === email().id ? 'true' : undefined}
        data-index={props.index}
        onClick={(e) => props.onClick(e)}
        onDblClick={(e) => props.onDblClick(e)}
        draggable={dragToFolderEnabled()&&!app.listLoading()} onDragStart={props.onDragStart} onDragEnd={clearMessageDrag}
        data-email-id={email().id} data-context-open={props.contextMenu.isOpen(email().id)?'true':undefined} {...prefetch} onContextMenu={e=>props.contextMenu.open(e,email())} onKeyDown={e=>props.contextMenu.keydown(e,email())}
      >
        <Show when={unread()}>
          <span class={a11y.srOnly}>{t('mail-unread')}</span>
        </Show>
        <span class={a11y.srOnly}>{t('mail-row-position', { pos: props.index + 1, total: props.total })}</span>
        <span class={`list__line1 ${rowCss.line1}`}>
          <SelectedMark selected={props.selected} />
          {/* isolate the sender: it shares a line with the date, so a spoofed
              display name must not reorder surrounding UI (SPEC §24). */}
          <span class={`list__sender ${rowCss.sender}`}><PersonAvatar name={email().from?.[0]?.name??undefined} address={email().from?.[0]?.email} messageSender auth={email().mailwomanAuth}/>{isolate(displayText(senderLabel(email().from)))}</span>
          <Show when={email().keywords?.['$flagged'] === true}>
            <span class={`list__flag ${bulk.flagMark}`} data-testid="row-flagged" role="img" aria-label={t('mail-flagged')}>
              <Icon name="flag" />
            </span>
          </Show>
          <Show when={email().pinned === true}>
            <span class="list__pin" aria-label={t('mail-pinned')}><Icon name="pin" /></span>
          </Show>
          <PriorityMark priority={priorityOf(email())}/>
          <Show when={hasFileAttachments(email())}>
            <span class="list__attach" aria-label={t('mail-has-attachment')}><Icon name="attachment" /></span>
          </Show>
          <Show when={email().keywords?.['$answered'] === true}>
            <span class="list__attach" data-testid="row-answered" role="img" aria-label={t('mail-answered')} title={t('mail-answered')}><Icon name="reply" /></span>
          </Show>
          <Show when={email().keywords?.['$forwarded'] === true}>
            <span class="list__attach" data-testid="row-forwarded" role="img" aria-label={t('mail-forwarded')} title={t('mail-forwarded')}><Icon name="forward" /></span>
          </Show>
          <Show when={props.replyDraft}><ReplyDraftIndicator/></Show>
          <span class={`list__date ${rowCss.date}`}>{formatDate(email().receivedAt)}</span>
        </span>
        <span class="list__subject">{displayText(email().subject ?? t('mail-no-subject'))}</span>
        <Show when={settings()['mail.messageListPreview']?.value!==false}><span class={`list__preview ${rowCss.preview}`}><EventHint hint={email().mailwomanCalendar} />{displayText(email().preview)}</span></Show>
        <TagChips email={email()} />
      </button>
      <FolderChip email={email()}/>
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
function PlaceholderRow(props: { top: number; height: number; index: number; total: number|null;table:boolean;preview:boolean }): JSX.Element {
  return (
    <li
      class={`list__slot list__slot--pending ${rowCss.slot}`}
      role={props.total===null?'presentation':'listitem'}
      aria-hidden={props.total===null?true:undefined}
      aria-busy="true"
      aria-posinset={props.total===null?undefined:props.index + 1}
      aria-setsize={props.total??undefined}
      style={{ transform: `translateY(${props.top}px)`, height: `${props.height}px` }}
    >
      <span class={a11y.srOnly}>{t('mail-loading')}</span>
      <div aria-hidden="true" data-skeleton="message-row" class={`${props.table?listLoadingCss.table:loadingCss.messageSkeleton} ${listLoadingCss.row}`}>
        <Show when={props.table} fallback={
          <>
            <div class={loadingCss.messageSkeletonHead}>
              <span class={`${loadingCss.messageSkeletonAvatar} ${loadingCss.skeletonPulse}`}/>
              <span class={`${loadingCss.skeletonLine} ${loadingCss.skeletonPulse} ${listLoadingCss.sender}`}/>
              <span class={`${loadingCss.messageSkeletonDate} ${loadingCss.skeletonPulse}`}/>
            </div>
            <span class={`${loadingCss.skeletonLine} ${loadingCss.skeletonPulse} ${listLoadingCss.subject}`}/>
            <Show when={props.preview}><span class={`${loadingCss.skeletonLine} ${loadingCss.messageSkeletonDim} ${loadingCss.skeletonPulse} ${listLoadingCss.preview}`}/></Show>
          </>
        }>
          <span class={`${loadingCss.skeletonLine} ${loadingCss.skeletonPulse} ${listLoadingCss.subject}`}/>
          <span class={`${loadingCss.skeletonLine} ${loadingCss.skeletonPulse} ${props.preview?listLoadingCss.preview:listLoadingCss.subjectWithoutPreview}`}/>
          <span class={`${loadingCss.messageSkeletonDate} ${loadingCss.skeletonPulse} ${listLoadingCss.date}`}/>
        </Show>
      </div>
    </li>
  );
}

/** A collapsed conversation head (W2): the disclosure toggle sits beside the
 *  primary row; the primary row opens the latest message (like any message row),
 *  the toggle expands/collapses the members in place. */
function ThreadHeadRow(props: {
  replyDraft:ReplyDraft|undefined;answered:boolean;forwarded:boolean;
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
  const prefetch=mailPrefetchEvents(trigger=>app.prefetchMessage?.(rep().id,trigger)??(()=>undefined));
  const active = () => app.openEmail()?.id === rep().id;
  return (
    <li
      class={`list__slot ${rowCss.slot} ${thread.headSlot}`}
      data-motion-row={`h:${props.row.key}`}
      data-reply-draft={props.replyDraft?.id}
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
          [rowCss.expired]: rowExpired(rep()),
        }}
        tabindex={props.focused ? 0 : -1}
        aria-current={active() ? 'true' : undefined}
        data-index={props.index}
        data-testid="thread-head"
        onClick={(e) => props.onClick(e)}
        onDblClick={(e) => props.onDblClick(e)}
        draggable={dragToFolderEnabled()&&!app.listLoading()} onDragStart={props.onDragStart} onDragEnd={clearMessageDrag}
        data-email-id={rep().id} data-context-open={props.contextMenu.isOpen(rep().id)?'true':undefined} {...prefetch} onContextMenu={e=>props.contextMenu.open(e,rep(),true)} onKeyDown={e=>props.contextMenu.keydown(e,rep(),true)}
      >
        <Show when={props.row.unread}>
          <span class={a11y.srOnly}>{t('mail-unread')}</span>
        </Show>
        <span class={a11y.srOnly}>{t('mail-row-position', { pos: props.index + 1, total: props.total })}</span>
        <span class={`list__line1 ${rowCss.line1}`}>
          <SelectedMark selected={props.selected} />
          <span class={`list__sender ${rowCss.sender}`}><PersonAvatar name={rep().from?.[0]?.name??undefined} address={rep().from?.[0]?.email} messageSender auth={rep().mailwomanAuth}/>{isolate(displayText(senderLabel(rep().from)))}</span>
          <span class={thread.count} aria-label={t('mail-thread-count', { count: props.row.count })}>
            {props.row.count}
          </span>
          <PriorityMark priority={props.row.priority ?? priorityOf(rep())}/>
          <Show when={hasFileAttachments(props.row)}>
            <span class="list__attach" aria-label={t('mail-has-attachment')}><Icon name="attachment" /></span>
          </Show>
          <Show when={props.answered}><span class="list__attach" data-testid="row-answered" role="img" aria-label={t('mail-answered')} title={t('mail-answered')}><Icon name="reply"/></span></Show>
          <Show when={props.forwarded}><span class="list__attach" data-testid="row-forwarded" role="img" aria-label={t('mail-forwarded')} title={t('mail-forwarded')}><Icon name="forward"/></span></Show>
          <Show when={props.replyDraft}><ReplyDraftIndicator/></Show>
          <span class={`list__date ${rowCss.date}`}>{formatDate(rep().receivedAt)}</span>
        </span>
        <span class="list__subject">{displayText(rep().subject ?? t('mail-no-subject'))}</span>
        <Show when={settings()['mail.messageListPreview']?.value!==false}><span class={`list__preview ${rowCss.preview}`}><EventHint hint={rep().mailwomanCalendar} />{displayText(rep().preview)}</span></Show>
      </button>
      <FolderChip email={rep()}/>
      <MessageActions email={rep()} onMore={e=>props.contextMenu.open(e,rep(),true)} contextOpen={props.contextMenu.isOpen(rep().id)}/>
    </li>
  );
}

/** The always-visible view toolbar: the reading-pane position control (W3). */
function ListToolbar(props:{order:MessageOrderController;scope:()=>string;selecting:boolean;toggleSelection:()=>void;hideReadingPane?:boolean;hideSelectionToggle?:boolean;hideOrderControl?:boolean;goToTop?:(()=>void)|undefined}): JSX.Element {
  return (
    <div class={thread.toolbar} role="group" aria-label={t('mail-view-options')}>
      <Show when={!props.hideSelectionToggle}><button type="button" class={`${thread.segBtn} ${a11y.focusable}`} aria-label={t('mail-select-messages')} aria-pressed={props.selecting} onClick={props.toggleSelection}><Icon name="bulkSelect"/></button></Show>
      <Show when={props.goToTop}>{goToTop=><button type="button" class={`${thread.segBtn} ${a11y.focusable}`} data-testid="list-go-to-top" aria-label={t('mail-go-to-top')} title={t('mail-go-to-top')} onClick={()=>goToTop()()}><Icon name="chevronUp"/></button>}</Show>
      <Show when={!props.hideOrderControl}><MessageOrderMenu order={props.order} scope={props.scope}/></Show><EmptyTrash/><Show when={!props.hideReadingPane}><ReadingPaneControl/></Show>
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
export function MessageList(props: { onOpenReplyDraft?:(id:string)=>void;hideReadingPane?:boolean; hideSelectionToggle?:boolean; hideOrderControl?:boolean; selectionMode?:()=>boolean; onSelectionModeChange?:(enabled:boolean)=>void; selectionReset?:()=>number; onOpenMessage?: (id: string) => void; onOpenMessageTab?: (id: string) => void; onCompose?: (initial:ComposeInitial)=>void;
  /** t109: a message to bring to the top once the list has loaded (a reload's scroll anchor); one that is not loaded is skipped. */
  restoreAnchor?: () => string | null;
  /** t109: the message in the top row as the user scrolls (null at the top). */
  onAnchorChange?: (id: string | null) => void } = {}): JSX.Element {
  const app = useApp();
  const order=createMessageOrder(()=>app.showToast('error',t('mail-list-order-error')));
  const orderScope=()=>`${app.accountId()}:${preferenceAccount()?.generation}:${app.selectedMailboxId()}`;
  const orderContext=createContextMenu({label:()=>t('mail-view-options'),scope:orderScope,actions:()=>[order.contextAction()]});
  function backgroundContext(event:MouseEvent):void {
    const target=event.target as Element|null;
    if(event.defaultPrevented||!target||target.closest('button,input,select,textarea,[contenteditable="true"],.list__row,.msg-more__toggle'))return;
    orderContext.open(event);
  }

  const replyDrafts=createReplyDraftIndex(app.accountId);
  const messageThreads=createMemo(()=>new Map(app.listMessages().map(email=>[email.id,threadKey(email)])));
  const threadDrafts=createMemo(()=>{const result=new Map<string,ReplyDraft>();for(const draft of replyDrafts().values()){const key=messageThreads().get(draft.emailId);if(key&&(!result.has(key)||result.get(key)!.savedAt<draft.savedAt))result.set(key,draft);}return result;});
  const answeredThreads=createMemo(()=>new Set(app.listMessages().filter(email=>email.keywords?.['$answered']===true).map(threadKey)));
  const forwardedThreads=createMemo(()=>new Set(app.listMessages().filter(email=>email.keywords?.['$forwarded']===true).map(threadKey)));
  const [contextEmail,setContextEmail]=createSignal<Email>();
  const [contextThread,setContextThread]=createSignal(false);
  const contextMenu=createMessageContextMenu(contextEmail,{replyDraft:id=>replyDrafts().get(id)??(()=>{const key=messageThreads().get(id);return key?threadDrafts().get(key):undefined;})(),...(props.onOpenReplyDraft?{onOpenReplyDraft:props.onOpenReplyDraft}:{})},undefined,{threadMode:contextThread,threadMembers:email=>app.listMessages().filter(member=>threadKey(member)===threadKey(email))},{onOpenMessage:id=>open(id),...(props.onOpenMessageTab?{onOpenMessageTab:(id:string)=>openTab(id)}:{})});
  const messageMenu:MessageMenu={open:(event,email,thread=false)=>{setContextThread(thread);setContextEmail(email);contextMenu.open(event);},keydown:(event,email,thread=false)=>{if(event.key==='ContextMenu'||(event.shiftKey&&event.key==='F10')){setContextThread(thread);setContextEmail(email);contextMenu.keydown(event);}},isOpen:id=>contextMenu.isOpen()&&contextEmail()?.id===id};
  const [localSelecting,setLocalSelecting]=createSignal(false);
  const selecting=()=>props.selectionMode?.()??localSelecting();
  const setSelecting=(value:boolean|((previous:boolean)=>boolean)):boolean=>{const next=typeof value==='function'?value(selecting()):value;setLocalSelecting(next);props.onSelectionModeChange?.(next);return next;};
  createEffect(on(()=>props.selectionReset?.()??0,()=>setSelecting(false),{defer:true}));
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

  const tableMode=()=>settings()['mail.messageListStyle']?.value==='table';
  const rowHeight = (): number => tableMode() ? ({compact:36,cozy:44,relaxed:52}[app.density()]) : ROW_HEIGHTS[app.density()];
  // The FLAT list folded into visual rows (singletons + conversation heads/members).
  const rows = createMemo<ThreadVisualRow[]>(() => groupThreads(visibleRows(app.listMessages()) as Email[], expanded()));
  // How many slots the list spans: the loaded rows plus the query's unfetched
  // tail. This is what makes the scrollbar and `aria-setsize` describe the FOLDER
  // rather than the page — before paging they described `rows().length`, so a
  // 20 000-message folder announced "1 of 50" behind a 3 600px scrollbar.
  const spanRows = createMemo(() => app.loadMoreFailed?.() === true ? rows().length : projectedRowCount(rows().length, app.total(), app.loadedRange().end));
  // An unknown query total stays unknown. Reserve only one viewport of
  // decorative loading space while its next page is actually in flight.
  const loadingTail=()=>app.total()===null&&app.loadingMore()?Math.ceil(viewportH()/rowHeight()):0;
  // `computeWindow` returns a fresh object every call, so without a comparator
  // this memo notifies on every scroll EVENT — including the many that leave
  // the mounted slice unchanged — and rebuilt the whole window each time.
  const win = createMemo(
    () => computeWindow(scrollTop(), viewportH(), rowHeight(), spanRows()+loadingTail()),
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
  let listHost: HTMLElement | undefined;
  const [hostVisible, setHostVisible] = createSignal(false);
  const [viewportVisible, setViewportVisible] = createSignal(false);
  function attachListHost(element: HTMLElement): void {
    listHost = element;
    const stop = observeListViewport(element, height => setHostVisible(height > 0));
    onCleanup(stop);
  }
  function attachScroller(element: HTMLDivElement): void {
    scroller = element;
    const stop = observeListViewport(element, height => { setViewportVisible(height > 0); if (height > 0) setViewportH(height); });
    onCleanup(() => { stop(); if (scroller === element) scroller = undefined; });
  }
  /** `mail.showGoToTop`: back to the first message of the list as loaded (no reload); instant under reduced motion. */
  function goToTop(): void {
    if (!scroller) return;
    const still = (typeof matchMedia === 'function' && matchMedia('(prefers-reduced-motion: reduce)').matches) || document.documentElement.hasAttribute('data-reduced-motion');
    if (typeof scroller.scrollTo === 'function') scroller.scrollTo({ top: 0, behavior: still ? 'auto' : 'smooth' });
    else scroller.scrollTop = 0;
    setCursor(0);
  }
  // Load-time measurement (perf/clientTiming.ts): folder open starts here, the list reports its state.
  createEffect(on(() => app.selectedMailboxId(), () => noteFolderSelected(), { defer: true }));
  createEffect(() => noteMailboxList(app.listLoading(), rows().length));
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
    if (entry !== undefined && entry[1] === e.target) {
      focusedRowId = entry[0];
      // Context-menu restoration and keyboard focus need not pass through a
      // row click. A removed focused row must hand focus to its real neighbour,
      // rather than scroll back to an unrelated old roving cursor.
      const index=Number(entry[1].dataset['index']);
      if(Number.isInteger(index)&&index>=0&&index<rows().length)setCursor(index);
    }
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

  // Fill the measured viewport on arrival/resize, then prefetch near the loaded
  // edge on a scroll gesture. The state slice retains page-size and exhaustion
  // policy; this layer only decides when the visible list needs another page.
  const pagingScope = (): string => JSON.stringify([app.accountId(),preferenceAccount()?.generation,app.selectedMailboxId(),app.searchActive(),app.search?.(),settings()['mail.messageListOrder']?.value]);
  let lastPageExtent: string | undefined;
  let pagingDisposed = false;
  const [autoFillPaused, setAutoFillPaused] = createSignal(false);
  let noVisualProgress = 0;
  let autoRequest: { scope: string } | undefined;
  // A pause must survive layout changes caused by its own continuation button.
  createEffect(on([pagingScope, rowHeight], () => { noVisualProgress = 0; setAutoFillPaused(false); }));
  onCleanup(() => { pagingDisposed = true; });
  function maybeLoadMore(prefetch = false): void {
    const scope = pagingScope();
    if (pagingDisposed || autoFillPaused() || autoRequest?.scope === scope || !app.online() || !app.hasMore() || app.loadingMore() || app.listLoading() || app.loadMoreFailed?.() === true) return;
    const host = scroller ?? listHost;
    // Hidden retained lists have no viewport to fill, including scroll events
    // delivered by a layout change while the list is hidden.
    if (!host?.isConnected || host.clientHeight <= 0) return;
    // A replacement folder can mount before ResizeObserver delivers its first
    // notification; never fill it using the previous folder's container height.
    const height = scroller?.clientHeight || viewportH();
    if (!(prefetch ? win().endIndex >= rows().length - LOAD_MORE_SLACK : viewportNeedsPage(scrollTop(), height, rowHeight(), rows().length))) return;
    const extent = JSON.stringify([pagingScope(),app.loadedRange(),rows().length]);
    // State normally exhausts a no-progress page. Also avoid reissuing the same
    // automatic continuation if a transport refuses it without changing state.
    if (extent === lastPageExtent) return;
    lastPageExtent = extent;
    const request = { scope }, before = rows().length;
    autoRequest = request;
    void app.loadMore().finally(() => {
      if (pagingDisposed || pagingScope() !== scope || autoRequest !== request) return;
      // New messages can fold into the same collapsed conversation or remain
      // hidden. Four pages without visual growth are enough for one automatic
      // fill; leave the next continuation to an explicit user action.
      noVisualProgress = rows().length > before ? 0 : noVisualProgress + 1;
      if (noVisualProgress >= 4) setAutoFillPaused(true);
      autoRequest = undefined;
      queueMicrotask(() => { if (!pagingDisposed && pagingScope() === scope) maybeLoadMore(); });
    });
  }
  function resumeViewportFill(): void {
    noVisualProgress = 0; lastPageExtent = undefined; setAutoFillPaused(false); maybeLoadMore();
  }

  // Re-evaluate after every state change that can leave the viewport at the
  // end with nothing in flight (a page landed, a refresh superseded the append,
  // the window grew). Scroll events alone miss those, which left the skeleton
  // tail up with no request behind it.
  createEffect(() => {
    const scope = pagingScope();
    void [rows().length, scrollTop(), viewportH(), hostVisible(), viewportVisible(), rowHeight(), app.loadingMore(), app.online(), app.hasMore()];
    if (app.listLoading()) { lastPageExtent = undefined; noVisualProgress = 0; setAutoFillPaused(false); return; }
    if (app.loadMoreFailed?.() === true) return;
    queueMicrotask(() => { if (!pagingDisposed && pagingScope() === scope) maybeLoadMore(); });
  });

  function onScroll(): void {
    if (scroller !== undefined) {
      const focused=focusedRowEl();
      if(focused){const index=rows().findIndex(row=>rowId(row)===focused[0]);const top=index*rowHeight();if(top+rowHeight()<scroller.scrollTop||top>scroller.scrollTop+(scroller.clientHeight||viewportH())){focusedRowId=null;focused[1].blur();}}
      setScrollTop(scroller.scrollTop);
      if (scroller.clientHeight > 0) setViewportH(scroller.clientHeight);
      reportAnchor();
    }
    maybeLoadMore(true);
  }
  // t109: the scroll position is kept as the top row's message, not pixels, so
  // it survives a density change and rows arriving above it.
  let pendingAnchor: string | null = null;
  let reportedAnchor: string | null | undefined;
  // Go to reply / forward / folder indicator: centre the requested row, move the cursor to it and flash it.
  let flashTimer:ReturnType<typeof setTimeout>|undefined;
  onCleanup(()=>clearTimeout(flashTimer));
  function applyFocusRequest(): void {
    const request = focusRequest();
    if (request === null || app.listLoading() || scroller === undefined) return;
    let index = rows().findIndex((row) => row.email.id === request.emailId && row.kind !== 'head');
    if (index < 0) {
      index = rows().findIndex((row) => row.kind === 'head' && idsOf(row).includes(request.emailId));
      const head = index < 0 ? undefined : rows()[index]!;
      // The reply is in this conversation: open the conversation; the rows change and the request is applied again.
      if (head !== undefined && request.thread === true && !head.expanded) { toggleThread(head.key); return; }
    }
    if (index < 0) return;
    clearFocusRequest();
    const view = scroller.clientHeight || viewportH();
    scroller.scrollTop = Math.max(0, index * rowHeight() - (view - rowHeight()) / 2);
    setScrollTop(scroller.scrollTop);
    setCursor(index);
    const host = scroller;
    const flash = (attempt = 0): void => {
      const row = Array.from(host.querySelectorAll<HTMLElement>('.list__row')).find((value) => value.dataset['emailId'] === request.emailId) ?? host.querySelector<HTMLElement>(`.list__row[data-index="${index}"]`);
      if (!row) { if (attempt < 5) setTimeout(() => flash(attempt + 1), 30); return; }
      row.focus({ preventScroll: true });
      row.removeAttribute('data-flash'); void row.offsetWidth; row.setAttribute('data-flash', 'true');
      clearTimeout(flashTimer);
      flashTimer = setTimeout(() => row.removeAttribute('data-flash'), 1600);
    };
    queueMicrotask(() => flash());
  }
  createEffect(on([focusRequest, rows, app.listLoading], () => { if (focusRequest() !== null) queueMicrotask(applyFocusRequest); }));
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
      el.focus({ preventScroll:true, focusVisible: true } as FocusOptions);
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
    if (onRow && isPropertiesShortcut(e) && messageDialogEnabled()) {
      const email = app.messages().find((m) => m.id === (e.target as HTMLElement).dataset['emailId']);
      if (email) { e.preventDefault(); openMessageProperties(email); return; }
    }
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
  const motionIds=createMemo(()=>rows().map(rowId));
  const listMotion=createMessageListMotion();
  const stopMotionAuthority=observePreferenceAccount(()=>listMotion.clear());
  onCleanup(()=>{stopMotionAuthority();listMotion.dispose();});
  createEffect(()=>{
    // The window/scroll reads refresh only the bounded visible geometry. The
    // complete immutable ID list is rebuilt only when the visual query changes.
    slice();scrollTop();
    listMotion.update({host:scroller,ids:motionIds(),busy:app.listLoading()||app.loadingMore(),scope:JSON.stringify([
      app.accountId(),preferenceAccount()?.generation,app.selectedMailboxId(),app.searchActive(),app.search?.(),
      rowHeight(),settings()['mail.messageListOrder']?.value,[...expanded()].sort(),
    ])});
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
  // Rows can all be hidden client-side (expired mail in `hide` mode) while more pages exist: that is "still loading", not an empty folder.
  const showSkeleton = (): boolean => rows().length === 0 && (app.listLoading() || app.switchingMailbox() || sync.pending() || (!autoFillPaused() && app.messages().length > 0 && app.hasMore() && app.loadMoreFailed?.() !== true));
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
    <section ref={attachListHost} class="list" onContextMenu={backgroundContext} data-list-style={tableMode()?'table':'cards'} aria-label={t('mail-list-label')} aria-busy={app.listLoading() || app.loadingMore() || showSkeleton()}>
      <ListToolbar order={order} scope={orderScope} hideOrderControl={props.hideOrderControl===true} goToTop={mailFeature('showGoToTop')&&rows().length>0?goToTop:undefined} hideSelectionToggle={props.hideSelectionToggle===true} hideReadingPane={props.hideReadingPane===true} selecting={selecting()} toggleSelection={()=>setSelecting(value=>!value)}/>
      <Show when={tableMode()}><div class="list-column-headers"><span>{t('mail-from')}</span><span>{t('mail-subject')}</span><select aria-label={t('mail-list-order')} value={order.value()} disabled={order.locked()||order.saving()} onChange={event=>{const select=event.currentTarget,value=select.value;if(isMessageListOrder(value))void order.set(value).then(()=>{if(select.isConnected)select.value=order.value();});}}><For each={MESSAGE_LIST_ORDERS}>{order=><option value={order}>{t(`mail-list-order-${order}`)}</option>}</For></select></div></Show>
      <OfflineQueueNotice />
      <Show when={sync.running() && rows().length > 0}><div class={loadingCss.syncProgress} aria-hidden="true" data-testid="sync-progress" /></Show>
        <Show when={rows().length > 0} fallback={
          <Show when={!autoFillPaused()}><Show when={showSkeleton()} fallback={
            <Show when={sync.failed()} fallback={<p class="list__empty">{t('mail-empty')}</p>}>
              <AsyncError message={t('mail-initial-sync-error')} detail={t('mail-initial-sync-error-help')} onRetry={() => sync.retry()} />
            </Show>
          }>
            <AsyncPending variant="messages" quiet bounded={!sync.pending()} rowHeight={rowHeight()} rows={skeletonRows()} message={t('mail-loading')} onRetry={()=>void app.refreshCurrentMailbox()}/>
          </Show></Show>
        }>
          <div
            class="list__scroll"
            classList={{ [barCss.listInset]: selecting() || app.selectedIds().size > 0 }}
            inert={app.listLoading()}
            ref={attachScroller}
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
                    total={app.total()===null?null:spanRows()}
                    table={tableMode()}
                    preview={settings()['mail.messageListPreview']?.value!==false}
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
                          replyDraft={replyDrafts().get(row().email.id)}
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
                        replyDraft={threadDrafts().get(row().key)} answered={answeredThreads().has(row().key)} forwarded={forwardedThreads().has(row().key)}
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
            <Show when={app.loadMoreFailed?.() === true}>
              <div class="list__load-failed" role="alert" data-testid="load-more-failed" style={{position:'sticky',bottom:'0',padding:'8px 12px',display:'flex','align-items':'center',gap:'8px'}}>
                <span>{t('mail-toast-load-more-failed')}</span>
                <button type="button" onClick={()=>void app.loadMore()}>{t('common-retry')}</button>
              </div>
            </Show>
          </div>
        </Show>
      <Show when={autoFillPaused() && app.hasMore() && app.loadMoreFailed?.() !== true}>
        <button type="button" disabled={app.loadingMore() || app.listLoading() || !app.online()} onClick={resumeViewportFill}>{t('mail-conversation-more')}</button>
      </Show>
      {/* The floating selection bar (t58): over the list, after it in tab order.
          It never takes focus itself; Esc/F6 in it come back to the cursor row. */}
      <MailSelectionBar selecting={selecting()} onCompose={props.onCompose} onClear={()=>{app.clearSelection();setSelecting(false);}} returnFocus={()=>moveCursor(cursor())}/>
      {contextMenu.menu()}
      {orderContext.menu()}
    </section>
  );
}
