// Mail slice: session/auth, mailbox list, message list + reader, compose+send,
// and — new in V2 (plan §3 e7, §1.5) — the modern-mail UX that mutates the
// message list: tags (keywords), pins, snooze, follow-up, archive/trash/move/
// spam, sweep, the shared 10-second undo primitive, and the focused-inbox
// derivation. The V1 session+mail behaviour is preserved verbatim; the
// V2 surface is additive.

import { createSignal, createMemo, type Accessor } from 'solid-js';
import { ApiError, NetworkError, PasswordChangeRequired, type LoginInput, type Me } from '../../api/client.ts';
import {
  cancelSubmission,
  emailGetFull,
  invalidRecipients,
  listMailbox,
  mailboxGet,
  moveEmail,
  parseRecipients,
  responseFor,
  searchEmails,
  sendEnvelope,
  setEmailKeyword,
  setEmailMeta,
  updateEmails,
  COUNTS_CALL_ID,
  type DraftInput,
} from '../../api/jmap.ts';
import { bindTrackingToken, withTrackingPixel } from '../../api/tracking.ts';
import {
  buildDownloadUrl,
  fetchObjectUrl,
  loadAttachments,
  type AttachmentItem,
} from '../../viewers/attachments.ts';
import {
  CAP_MAIL,
  type Email,
  type EmailGetResponse,
  type EmailQueryResponse,
  type EmailSetResponse,
  type EmailSubmissionSetResponse,
  type FilterCondition,
  type Id,
  type Identity,
  type JmapRequest,
  type JmapResponse,
  type Mailbox,
  type MailboxGetResponse,
  type SetError,
} from '../../api/jmap-types.ts';
import { t, isolate } from '../../i18n/index.ts';
import { clearDrafts } from '../../components/compose/drafts-store.ts';
import { purgeShellCaches } from '../../sw/purge.ts';
import { toast } from '../../components/toast/index.ts';
import type { SliceContext } from './context.ts';
import { currentMarkReadPolicy, decideMarkRead } from './readPolicy.ts';

/**
 * Rows fetched per page. The list used to request exactly this many ids and
 * nothing else, ever — so a folder's 51st message could not be reached from the
 * UI at all, however far the virtualized list was scrolled. It is now the page
 * size of an append-on-scroll cursor.
 */
export const PAGE_SIZE = 50;

/**
 * Ceiling on how much of the loaded extent an in-place refresh will refetch.
 *
 * `refreshCurrentMailbox` runs on every push tick and every peer-tab mutation.
 * Refetching only the first page would collapse a reader who has scrolled to row
 * 400 back to row 50; refetching everything loaded would put a 20 000-row
 * `Email/get` on the push path, which is the cost this tag exists to remove. So
 * a refresh renews the loaded extent up to this many rows and, past it, declines
 * — leaving the list exactly as it was rather than shrinking it. Refresh is
 * already best-effort here (it swallows transient failures for the same reason).
 */
const REFRESH_MAX_ROWS = 500;

/** Which query the list is currently showing, so a page N can repeat it. */
type PageSource =
  | { readonly kind: 'mailbox'; readonly mailboxId: Id }
  | { readonly kind: 'search'; readonly filter: FilterCondition };

/** The toast id of the pending undo; a new undo replaces the one on screen. */
const UNDO_TOAST_ID = 'mail-undo';

/** A dismissable, time-boxed reversible action (the 10-second undo, §1.5). */
export interface PendingUndo {
  label: string;
  /** Label for the action button (`Undo` by default; `Cancel` for undo-send). */
  actionLabel: string;
  /** Reverses the action; resolves once reversed. */
  run: () => Promise<void>;
  /** Epoch ms when the window would end if the toast is never paused. */
  expiresAt: number;
}

/** Which sweep to run against a sender's mail (Outlook-style, §1.5). */
export type SweepStrategy = 'all' | 'block' | 'keep-latest' | 'older-than';

/** The two-tab focused inbox destinations (§1.5). */
export type InboxTab = 'focused' | 'other';

/** The compose/send input (grown for identities + send-later + undo-send). */
export interface SendInput {
  /** The recipient field as typed: `addr` or `Name <addr>` entries, `,`/`;`-separated. */
  to: string;
  /** `Cc` and `Bcc` recipient fields, in the same syntax as `to`. */
  cc?: string;
  bcc?: string;
  /** Threading of a reply: the answered message's id, and the thread's ids
   *  oldest first — both without angle brackets. */
  inReplyTo?: string[];
  references?: string[];
  subject: string;
  /** The body, sent exactly as given. Nothing is appended here — the composer
   *  adds the identity's signature itself, because only it knows whether the
   *  body is ciphertext or a clear-signed block that must not be altered. */
  htmlBody: string;
  /** Send as this identity (its from-address); `null` = the account's own. */
  identity?: Identity | null;
  /** Send-later: ISO 8601 UTC time, which must be in the future; omitted/null =
   *  send now (with undo window). */
  sendAt?: string | null;
  /** Undo-send window in seconds; default 10. Ignored when `sendAt` is set. */
  holdSeconds?: number;
  /** V7 (§18.4): server-materialised blob attachments (e.g. from Nextcloud). */
  attachments?: { blobId: Id; name: string; type: string; size?: number }[];
  /** Images the body shows by `cid:` (t29-e12), already limited to the ones it
   *  refers to. */
  inline?: { blobId: Id; cid: string; disposition: 'inline'; type: string; name: string }[];
  /** Ask the recipient's mail program for a read receipt (t29-e12). */
  requestReadReceipt?: boolean;
  /** Record when the message is opened (t29-e12): a token is minted and its
   *  image appended to the body just before the send — at replay time for a
   *  send queued offline. If no token can be minted the send is refused, never
   *  sent untracked. The composer offers this only when `/api/policy` says
   *  `openTracking` and the message is neither encrypted nor signed. */
  openTracking?: boolean;
  /** The message this send answers or forwards. Once the server has accepted
   *  the submission it is given `keyword`; cancelling the send within the undo
   *  window takes the keyword off again. */
  source?: { emailId: Id; keyword: '$answered' | '$forwarded' };
}

/** The mail/session portion of `AppState` (accessors + actions). */
export interface MailSlice {
  me: Accessor<Me | null>;
  authChecked: Accessor<boolean>;
  mailboxes: Accessor<Mailbox[]>;
  selectedMailboxId: Accessor<Id | null>;
  messages: Accessor<Email[]>;
  listLoading: Accessor<boolean>;
  openEmail: Accessor<Email | null>;
  sanitizedHtml: Accessor<string | null>;
  readLoading: Accessor<boolean>;
  accountId: Accessor<string | null>;
  sentMailboxId: Accessor<Id | null>;
  draftsMailboxId: Accessor<Id | null>;
  /** JMAP `downloadUrl` template for blob/attachment/EML fetches (from session). */
  downloadUrl: Accessor<string | null>;

  // ── V2 integration (t4-e13): search, attachments, export, push refetch ──
  /** Current search query string (empty when browsing a mailbox). */
  search: Accessor<string>;
  /** True while the list shows search results rather than a mailbox. */
  searchActive: Accessor<boolean>;
  /** Run an `Email/query` search (engine → mw-search); offline uses the reduced
   *  cached-header search. Empty query clears back to the current mailbox. The
   *  optional `semantic` flag (V7 §14.3) requests embedding re-ranking; it is only
   *  ever set when the Assist semantic-search toggle is on. */
  searchMessages(query: string, opts?: { semantic?: boolean }): Promise<void>;
  /** Clear search and reload the selected mailbox. */
  clearSearch(): Promise<void>;
  /** Refetch the current mailbox list in place (push/peer-sync), preserving the
   *  open message + selection (unlike `selectMailbox`). No-op during search. */
  refreshCurrentMailbox(): Promise<void>;
  // ── Paging (t22-e4) ─────────────────────────────────────────────────────
  /**
   * How many messages the CURRENT query matches, or `null` when the server did
   * not or could not calculate one (see `EmailQueryResponse.total`). It is the
   * size of the query, NOT of `messages()` — with one page loaded out of a
   * 20 000-message folder this reads 20000 while `messages()` holds 50.
   *
   * `null` means unknown and must be carried through as unknown; substituting
   * the loaded row count is what made `aria-setsize` announce "1 of 50" in a
   * 20 000-message folder.
   */
  total: Accessor<number | null>;
  /**
   * Which slice of the query `messages()` currently holds, as half-open query
   * indices `[start, end)`. Value-compared, so a refresh that lands the same
   * window does not notify.
   *
   * Rows removed locally (archive, sweep, trash) shrink `end` without a refetch,
   * and `total` is not adjusted for them — both reflect the last thing the
   * server actually said.
   */
  loadedRange: Accessor<{ start: number; end: number }>;
  /** Whether any of the current query remains unfetched. */
  hasMore: Accessor<boolean>;
  /**
   * True while an APPEND page is in flight. Distinct from `listLoading`, which
   * means the list is being replaced and blanks it: a scroll must never blank
   * the rows the reader is looking at.
   */
  loadingMore: Accessor<boolean>;
  /**
   * Fetch the next page of the current query and APPEND it. Safe to call from a
   * scroll handler: it is a no-op — with no signal written, so no row rebuild —
   * when a fetch is already in flight or the query is exhausted.
   */
  loadMore(): Promise<void>;

  /** Load every attachment across the account for the Attachments module. */
  listAttachments(): Promise<AttachmentItem[]>;
  /** Export the open message as an `.eml` and trigger a browser download. */
  exportMessage(): Promise<void>;

  // ── V2 derived views ──
  /** List rows to show: snoozed hidden, pinned floated to the top. */
  visibleMessages: Accessor<Email[]>;
  focusedMessages: Accessor<Email[]>;
  otherMessages: Accessor<Email[]>;
  snoozedMessages: Accessor<Email[]>;
  followUps: Accessor<Email[]>;
  /** The rows the message list renders: `visibleMessages` normally, or the
   *  focused/other split when the two-tab focused inbox is enabled. */
  listMessages: Accessor<Email[]>;
  inboxTab: Accessor<InboxTab>;
  setInboxTab(tab: InboxTab): void;
  /** Opt-in two-tab focused inbox (off by default so the list shows everything). */
  focusedInbox: Accessor<boolean>;
  setFocusedInbox(on: boolean): void;

  // ── V2 message mutations (each with a 10s undo) ──
  applyTag(id: Id, keyword: string): Promise<void>;
  removeTag(id: Id, keyword: string): Promise<void>;
  pinMessage(id: Id, pinned: boolean): Promise<void>;
  snoozeMessage(id: Id, untilIso: string): Promise<void>;
  unsnoozeMessage(id: Id): Promise<void>;
  setFollowUp(id: Id, atIso: string | null): Promise<void>;
  // The relocations and the read/flag marks take one id or many. Many ids are
  // ONE `Email/set` request (t28-e16), and one undo puts every message back.
  archiveMessage(ids: Id | readonly Id[]): Promise<void>;
  trashMessage(ids: Id | readonly Id[]): Promise<void>;
  moveMessage(ids: Id | readonly Id[], mailboxId: Id): Promise<void>;
  markSpam(ids: Id | readonly Id[]): Promise<void>;
  /** Set or clear `$seen` (read / unread). */
  markRead(ids: Id | readonly Id[], read: boolean): Promise<void>;
  /** Set or clear `$flagged`: the IMAP `\Flagged` flag, not a follow-up reminder. */
  flagMessage(ids: Id | readonly Id[], flagged: boolean): Promise<void>;

  // ── Selection (t28-e16): which list messages a bulk action applies to ──
  /** The selected messages, limited to those still in the list. */
  selectedIds: Accessor<ReadonlySet<Id>>;
  /** Replace the selection. */
  setSelection(ids: Iterable<Id>): void;
  /** Add `ids` to the selection, or remove them when all are already in it. */
  toggleSelection(ids: readonly Id[]): void;
  /** Select every message the list shows (`listMessages`). */
  selectAllInView(): void;
  clearSelection(): void;
  /** What a message action applies to: the selection when there is one, else
   *  the open message, else nothing. In list order. */
  actionTargets(): Id[];

  // ── V2 sweep ──
  sweepPreview(fromEmail: string, strategy: SweepStrategy, olderThanDays?: number): Email[];
  executeSweep(fromEmail: string, strategy: SweepStrategy, olderThanDays?: number): Promise<void>;
  blockedSenders: Accessor<string[]>;

  // ── V2 focused-inbox training ──
  trainSender(email: string, dest: InboxTab): void;

  // ── V2 undo primitive (shared by all reversible actions) ──
  pendingUndo: Accessor<PendingUndo | null>;
  undoNow(): Promise<void>;
  dismissUndo(): void;

  init(): Promise<void>;
  login(input: LoginInput): Promise<void>;
  logout(): Promise<void>;
  selectMailbox(id: Id): Promise<void>;
  openMessage(id: Id): Promise<void>;
  closeMessage(): void;
  sendMessage(input: SendInput): Promise<void>;
}

/** What a refused create says about itself: the server's description when it
 *  gave one (`to: "bob": no @`), otherwise the bare error type. */
function refusalReason(err: SetError): string {
  return typeof err.description === 'string' && err.description.length > 0 ? err.description : err.type;
}

function roleOf(mailboxes: Mailbox[], role: string): Id | null {
  return mailboxes.find((m) => m.role === role)?.id ?? null;
}

/** Bulk-sender heuristic for the focused/other split when there's no training. */
const BULK_RE = /(no-?reply|newsletter|notifications?|marketing|updates?|digest|mailer|automated|do-?not-?reply)/i;

function heuristicTab(email: Email): InboxTab {
  const addr = email.from?.[0];
  const hay = `${addr?.name ?? ''} ${addr?.email ?? ''}`;
  return BULK_RE.test(hay) ? 'other' : 'focused';
}

const FOCUS_KEY = 'mw.focused.v1';
const BLOCK_KEY = 'mw.blocked.v1';

function loadMap(key: string): Record<string, InboxTab> {
  try {
    const raw = globalThis.localStorage?.getItem(key);
    if (raw != null) return JSON.parse(raw) as Record<string, InboxTab>;
  } catch {
    /* ignore corrupt storage */
  }
  return {};
}

function loadList(key: string): string[] {
  try {
    const raw = globalThis.localStorage?.getItem(key);
    if (raw != null) return JSON.parse(raw) as string[];
  } catch {
    /* ignore */
  }
  return [];
}

export function createMailSlice(ctx: SliceContext): MailSlice {
  const { client, showToast } = ctx;

  const [me, setMe] = createSignal<Me | null>(null);
  const [authChecked, setAuthChecked] = createSignal(false);
  const [accountId, setAccountId] = createSignal<string | null>(null);
  const [mailboxes, setMailboxes] = createSignal<Mailbox[]>([]);
  const [selectedMailboxId, setSelectedMailboxId] = createSignal<Id | null>(null);
  const [messages, setMessages] = createSignal<Email[]>([]);
  const [listLoading, setListLoading] = createSignal(false);
  const [openEmail, setOpenEmail] = createSignal<Email | null>(null);
  const [sanitizedHtml, setSanitizedHtml] = createSignal<string | null>(null);
  const [readLoading, setReadLoading] = createSignal(false);
  const [downloadUrl, setDownloadUrl] = createSignal<string | null>(null);
  const [search, setSearch] = createSignal('');
  const [searchActive, setSearchActive] = createSignal(false);

  const isOffline = (): boolean => ctx.online?.() === false;

  // ── paging state (t22-e4) ────────────────────────────────────────────────
  const [total, setTotal] = createSignal<number | null>(null);
  const [loadedStart, setLoadedStart] = createSignal(0);
  const [loadingMore, setLoadingMore] = createSignal(false);
  /** Set once a page comes back short: the query has no more rows to give. */
  const [exhausted, setExhausted] = createSignal(true);
  /** The query `loadMore` repeats. `null` before the first list load. */
  const [pageSource, setPageSource] = createSignal<PageSource | null>(null);

  // Value-compared for the same reason `virtual.ts`'s `sameWindow` exists: this
  // is read by the virtualizer, and an equal-but-fresh object every tick would
  // re-notify it. `createMemo`'s default `equals` is reference identity, which a
  // freshly built object never satisfies.
  const loadedRange = createMemo(
    () => ({ start: loadedStart(), end: loadedStart() + messages().length }),
    undefined,
    { equals: (a, b) => a.start === b.start && a.end === b.end },
  );

  const hasMore = (): boolean => {
    if (pageSource() === null || exhausted()) return false;
    const t = total();
    // Unknown total: keep going until a short page proves otherwise.
    return t === null || loadedRange().end < t;
  };

  // ── fetch generation guard + abort (t22-e4, L5) ──────────────────────────
  // Every list fetch takes a monotonically increasing generation, and ONLY the
  // newest generation may write `messages` / `total` / `listLoading`. A response
  // from an older generation is dropped, including its `finally` and including
  // its failure.
  //
  // Without this, two list fetches resolving out of order leave the previous
  // folder's rows under the newly selected one with `listLoading` already false
  // — no spinner, no error, wrong mail on screen. It needed two mailbox switches
  // in flight to reproduce before paging; with append-on-scroll every scroll is
  // another racing fetch, so the guard is a precondition of the feature and not
  // a hardening pass over it.
  let listGeneration = 0;
  let inFlight: AbortController | undefined;

  interface ListRequest {
    readonly gen: number;
    readonly signal: AbortSignal;
  }

  function beginListRequest(): ListRequest {
    // Supersede whatever was running: its signal aborts, so a transport that
    // honours it drops the socket, and its generation can no longer win.
    inFlight?.abort();
    const controller = new AbortController();
    inFlight = controller;
    listGeneration += 1;
    // The superseded request's `finally` will not run its own cleanup (it is no
    // longer current), so clear the append flag here or a replace that lands
    // during an append leaves it stuck true. `listLoading` needs no equivalent:
    // only `loadFirstPage` supersedes a `loadFirstPage`, and it re-sets it.
    setLoadingMore(false);
    return { gen: listGeneration, signal: controller.signal };
  }
  const isCurrent = (req: ListRequest): boolean => req.gen === listGeneration;
  function endListRequest(req: ListRequest): void {
    if (isCurrent(req)) inFlight = undefined;
  }

  // `client.jmap(body, { signal })` cancels for real: the transport threads the
  // signal into `fetch` and rethrows the abort reason rather than a
  // `NetworkError`, so a superseded query does not register as going offline.
  const jmapCall = client.jmap.bind(client);

  const [inboxTab, setInboxTab] = createSignal<InboxTab>('focused');
  const [focusedInbox, setFocusedInbox] = createSignal(false);
  const [pendingUndo, setPendingUndo] = createSignal<PendingUndo | null>(null);
  const [training, setTraining] = createSignal<Record<string, InboxTab>>(loadMap(FOCUS_KEY));
  const [blocked, setBlocked] = createSignal<string[]>(loadList(BLOCK_KEY));

  // ── undo primitive ──────────────────────────────────────────────────────
  // The pending action is shown as a toast with the action button and the
  // seconds left (components/toast). The toast's clock is the undo window:
  // it stops while the toast is hovered or focused, except for undo-send,
  // whose window is the engine's hold and ends regardless (`hardDeadline`).
  async function runUndo(p: PendingUndo): Promise<void> {
    try {
      await p.run();
    } catch {
      showToast('error', t('mail-undo-failed'));
    }
  }
  function showUndo(
    label: string,
    run: () => Promise<void>,
    ttlMs = 10_000,
    actionLabel = t('mail-undo-action'),
    hardDeadline = false,
  ): void {
    const entry: PendingUndo = { label, actionLabel, run, expiresAt: Date.now() + ttlMs };
    setPendingUndo(entry);
    toast.info(label, {
      id: UNDO_TOAST_ID,
      duration: ttlMs,
      countdown: true,
      pausable: !hardDeadline,
      action: { label: actionLabel, run: () => runUndo(entry) },
      onClose: () => {
        if (pendingUndo() === entry) setPendingUndo(null);
      },
    });
  }
  function dismissUndo(): void {
    setPendingUndo(null);
    toast.dismiss(UNDO_TOAST_ID);
  }
  async function undoNow(): Promise<void> {
    const p = pendingUndo();
    dismissUndo();
    if (p === undefined || p === null) return;
    await runUndo(p);
  }

  // ── message-list helpers ─────────────────────────────────────────────────
  function patchMessage(id: Id, patch: Partial<Email>): void {
    setMessages((msgs) => msgs.map((m) => (m.id === id ? { ...m, ...patch } : m)));
    const open = openEmail();
    if (open !== null && open.id === id) setOpenEmail({ ...open, ...patch });
  }

  /** The message with `id` as the list (or, failing that, the reader) holds it. */
  function findEmail(id: Id): Email | null {
    const listed = messages().find((m) => m.id === id);
    if (listed !== undefined) return listed;
    const open = openEmail();
    return open !== null && open.id === id ? open : null;
  }

  /** One id or many, as a duplicate-free array. */
  function asIds(ids: Id | readonly Id[]): Id[] {
    return typeof ids === 'string' ? [ids] : [...new Set(ids)];
  }

  interface Taken {
    email: Email;
    index: number;
  }

  /** Remove messages from the current list, returning each with its index for undo. */
  function takeFromList(ids: readonly Id[]): Taken[] {
    const want = new Set(ids);
    const msgs = messages();
    const taken: Taken[] = [];
    msgs.forEach((email, index) => {
      if (want.has(email.id)) taken.push({ email, index });
    });
    if (taken.length === 0) return taken;
    setMessages(msgs.filter((m) => !want.has(m.id)));
    const open = openEmail();
    if (open !== null && want.has(open.id)) closeMessage();
    return taken;
  }
  /** Put taken messages back at their original indices (lowest first, so each
   *  index means what it meant when the message was taken). */
  function restoreToList(taken: readonly Taken[]): void {
    if (taken.length === 0) return;
    setMessages((msgs) => {
      const out = [...msgs];
      for (const item of [...taken].sort((a, b) => a.index - b.index)) {
        out.splice(Math.min(item.index, out.length), 0, item.email);
      }
      return out;
    });
  }

  // ── folder counts (t28-e16) ──────────────────────────────────────────────
  function sortMailboxes(boxes: Mailbox[]): Mailbox[] {
    return [...boxes].sort((a, b) => {
      if (a.role === 'inbox') return -1;
      if (b.role === 'inbox') return 1;
      if (a.sortOrder !== b.sortOrder) return a.sortOrder - b.sortOrder;
      return a.name.localeCompare(b.name);
    });
  }

  /** Take the folder counts from the `Mailbox/get` an `updateEmails` request
   *  carries. Only the counts: the folder list itself is what was loaded. A
   *  response without that call (an old server, a test double) changes nothing. */
  function applyCounts(res: JmapResponse): void {
    const found = res.methodResponses.find((m) => m[0] === 'Mailbox/get' && m[2] === COUNTS_CALL_ID);
    const list = (found?.[1] as Partial<MailboxGetResponse> | undefined)?.list;
    if (!Array.isArray(list)) return;
    const byId = new Map(list.map((b) => [b.id, b]));
    setMailboxes((boxes) =>
      boxes.map((b) => {
        const fresh = byId.get(b.id);
        return fresh === undefined ? b : { ...b, unreadEmails: fresh.unreadEmails, totalEmails: fresh.totalEmails };
      }),
    );
  }

  /** Move the unread badges as if each message in `emails` became `seen`, before
   *  the server answers (and offline, where it does not). `applyCounts` replaces
   *  the guess with the server's numbers when they come. */
  function shiftUnread(emails: readonly Email[], seen: boolean): void {
    const delta = new Map<Id, number>();
    for (const e of emails) {
      if ((e.keywords?.['$seen'] === true) === seen) continue;
      for (const [box, inBox] of Object.entries(e.mailboxIds ?? {})) {
        if (inBox) delta.set(box, (delta.get(box) ?? 0) + (seen ? -1 : 1));
      }
    }
    if (delta.size === 0) return;
    setMailboxes((boxes) =>
      boxes.map((b) => {
        const d = delta.get(b.id);
        return d === undefined ? b : { ...b, unreadEmails: Math.max(0, b.unreadEmails + d) };
      }),
    );
  }

  /** The ids of `ids` that an `updateEmails` response did not update: those in
   *  `notUpdated`, or all of them when the `Email/set` call itself failed. */
  function refusedUpdates(res: JmapResponse, ids: readonly Id[]): Set<Id> {
    const found = res.methodResponses.find((m) => m[2] === 'set');
    if (found === undefined) return new Set();
    if (found[0] === 'error') return new Set(ids);
    const notUpdated = (found[1] as Partial<EmailSetResponse>).notUpdated;
    return new Set(notUpdated != null ? Object.keys(notUpdated).filter((id) => ids.includes(id)) : []);
  }

  // ── selection (t28-e16) ──────────────────────────────────────────────────
  const [selection, setSelectionRaw] = createSignal<ReadonlySet<Id>>(new Set());
  // Limited to what the list holds, so a message archived elsewhere, or dropped
  // by a refresh, is neither counted in the bulk bar nor acted on.
  const selectedIds = createMemo<ReadonlySet<Id>>(() => {
    const raw = selection();
    if (raw.size === 0) return raw;
    const present = new Set<Id>();
    for (const m of messages()) if (raw.has(m.id)) present.add(m.id);
    return present;
  });
  function setSelection(ids: Iterable<Id>): void {
    setSelectionRaw(new Set(ids));
  }
  function toggleSelection(ids: readonly Id[]): void {
    if (ids.length === 0) return;
    const current = selectedIds();
    const next = new Set(current);
    if (ids.every((id) => current.has(id))) for (const id of ids) next.delete(id);
    else for (const id of ids) next.add(id);
    setSelectionRaw(next);
  }
  function clearSelection(): void {
    if (selection().size > 0) setSelectionRaw(new Set<Id>());
  }
  function selectAllInView(): void {
    setSelection(listMessages().map((m) => m.id));
  }
  function actionTargets(): Id[] {
    const sel = selectedIds();
    if (sel.size > 0) return messages().filter((m) => sel.has(m.id)).map((m) => m.id);
    const open = openEmail();
    return open !== null ? [open.id] : [];
  }

  // ── read / flagged marks, batched (t28-e16) ──────────────────────────────
  /**
   * Write one keyword for many messages in ONE request. `changes` maps each id
   * to the value wanted; ids already in that state, or not loaded, are skipped.
   * The list and the unread badges change first; ids the server refuses (or all
   * of them, when the request fails) are put back. Resolves to the number of
   * messages changed: 0 when there was nothing to do or the write failed.
   *
   * `quiet` is for writes the user did not ask for (mark-read on open): a
   * failure is put back without a toast.
   */
  async function writeKeyword(
    keyword: string,
    changes: ReadonlyMap<Id, boolean>,
    opts: { quiet?: boolean } = {},
  ): Promise<number> {
    const acct = accountId();
    if (acct === null) return 0;
    const touched: { before: Email; on: boolean }[] = [];
    for (const [id, on] of changes) {
      const before = findEmail(id);
      if (before === null || (before.keywords?.[keyword] === true) === on) continue;
      touched.push({ before, on });
    }
    if (touched.length === 0) return 0;

    const withKeyword = (email: Email, on: boolean): Record<string, boolean> => {
      const keywords = { ...(email.keywords ?? {}) };
      if (on) keywords[keyword] = true;
      else delete keywords[keyword];
      return keywords;
    };
    for (const { before, on } of touched) {
      if (keyword === '$seen') shiftUnread([before], on);
      patchMessage(before.id, { keywords: withKeyword(before, on) });
    }
    /** Undo the local change for `items` (refused or failed). */
    const putBack = (items: readonly { before: Email; on: boolean }[]): void => {
      for (const { before, on } of items) {
        patchMessage(before.id, { keywords: before.keywords ?? {} });
        if (keyword === '$seen') shiftUnread([{ ...before, keywords: withKeyword(before, on) }], !on);
      }
    };

    if (isOffline() && ctx.enqueueOffline) {
      for (const { before, on } of touched) {
        await ctx.enqueueOffline('flag', { accountId: acct, emailId: before.id, keyword, value: on });
      }
      return touched.length;
    }

    const ids = touched.map((x) => x.before.id);
    const patches: Record<Id, Record<string, unknown>> = {};
    for (const { before, on } of touched) patches[before.id] = { [`keywords/${keyword}`]: on ? true : null };
    let res: JmapResponse;
    try {
      res = await client.jmap(updateEmails(acct, patches));
    } catch {
      putBack(touched);
      if (opts.quiet !== true) showToast('error', t('mail-toast-update-failed'));
      return 0;
    }
    const refused = refusedUpdates(res, ids);
    if (refused.size > 0) {
      putBack(touched.filter((x) => refused.has(x.before.id)));
      if (opts.quiet !== true) showToast('error', t('mail-toast-update-refused', { count: refused.size }));
    }
    applyCounts(res);
    ctx.broadcastChange?.();
    return touched.length - refused.size;
  }

  /** Each id's current state of `keyword`, for an undo to restore. */
  function keywordSnapshot(ids: readonly Id[], keyword: string): Map<Id, boolean> {
    const out = new Map<Id, boolean>();
    for (const id of ids) {
      const email = findEmail(id);
      if (email !== null) out.set(id, email.keywords?.[keyword] === true);
    }
    return out;
  }

  /** A keyword change the user asked for over `ids`, with the shared undo. */
  async function markWithUndo(
    ids: readonly Id[],
    keyword: string,
    on: boolean,
    label: (count: number) => string,
  ): Promise<void> {
    const prior = keywordSnapshot(ids, keyword);
    const written = await writeKeyword(keyword, new Map(ids.map((id) => [id, on])));
    if (written === 0) return;
    showUndo(label(written), async () => {
      await writeKeyword(keyword, prior);
    });
  }

  async function markRead(ids: Id | readonly Id[], read: boolean): Promise<void> {
    const list = asIds(ids);
    // Marking the open message unread must not be undone a moment later by the
    // mark-read-on-open timer still running for it.
    const open = openEmail();
    if (!read && open !== null && list.includes(open.id)) cancelMarkRead();
    await markWithUndo(list, '$seen', read, (count) =>
      read ? t('mail-toast-marked-read', { count }) : t('mail-toast-marked-unread', { count }),
    );
  }

  async function flagMessage(ids: Id | readonly Id[], flagged: boolean): Promise<void> {
    await markWithUndo(asIds(ids), '$flagged', flagged, (count) =>
      flagged ? t('mail-toast-flagged', { count }) : t('mail-toast-unflagged', { count }),
    );
  }

  // ── mark read on open (t28-e16, D-16) ────────────────────────────────────
  // One pending timer at most: opening another message, closing the reader,
  // switching folder or signing out cancels it, so only a message that stayed
  // on screen for the whole delay is marked. The decision is `readPolicy.ts`'s.
  let markReadTimer: ReturnType<typeof setTimeout> | undefined;
  function cancelMarkRead(): void {
    if (markReadTimer !== undefined) clearTimeout(markReadTimer);
    markReadTimer = undefined;
  }
  function scheduleMarkRead(email: Email): void {
    cancelMarkRead();
    const decision = decideMarkRead(currentMarkReadPolicy(), {
      kind: 'open',
      email,
      mailboxId: searchActive() ? null : selectedMailboxId(),
    });
    if (!decision.mark) return;
    const fire = (): void => {
      markReadTimer = undefined;
      if (openEmail()?.id !== email.id) return;
      void writeKeyword('$seen', new Map([[email.id, true]]), { quiet: true });
    };
    if (decision.delayMs === 0) fire();
    else markReadTimer = setTimeout(fire, decision.delayMs);
  }

  // ── raw mutators (no undo) ───────────────────────────────────────────────
  // Route a mutation to the offline replay queue when the network is down;
  // otherwise send it directly and notify peer tabs to refetch (multi-window).
  async function rawKeyword(id: Id, keyword: string, on: boolean): Promise<void> {
    const acct = accountId();
    if (acct === null) return;
    const current = messages().find((m) => m.id === id) ?? openEmail();
    const keywords = { ...(current?.keywords ?? {}) };
    if (on) keywords[keyword] = true;
    else delete keywords[keyword];
    patchMessage(id, { keywords });
    if (isOffline() && ctx.enqueueOffline) {
      await ctx.enqueueOffline('flag', { accountId: acct, emailId: id, keyword, value: on });
      return;
    }
    await client.jmap(setEmailKeyword(acct, id, keyword, on));
    ctx.broadcastChange?.();
  }
  async function rawMeta(id: Id, patch: Partial<Email>): Promise<void> {
    const acct = accountId();
    if (acct === null) return;
    patchMessage(id, patch);
    // Engine-local meta (pin/snooze/follow-up) has no offline-replay opcode in
    // the frozen mw-outbox contract, so offline it stays an optimistic local patch.
    if (isOffline()) return;
    await client.jmap(setEmailMeta(acct, id, patch));
    ctx.broadcastChange?.();
  }

  // ── tags ─────────────────────────────────────────────────────────────────
  async function applyTag(id: Id, keyword: string): Promise<void> {
    await rawKeyword(id, keyword, true);
    showUndo(t('mail-toast-label-added'), () => rawKeyword(id, keyword, false));
  }
  async function removeTag(id: Id, keyword: string): Promise<void> {
    await rawKeyword(id, keyword, false);
    showUndo(t('mail-toast-label-removed'), () => rawKeyword(id, keyword, true));
  }

  // ── pin / snooze / follow-up ──────────────────────────────────────────────
  async function pinMessage(id: Id, pinned: boolean): Promise<void> {
    const prev = messages().find((m) => m.id === id)?.pinned ?? false;
    await rawMeta(id, { pinned });
    showUndo(pinned ? t('mail-toast-pinned') : t('mail-toast-unpinned'), () => rawMeta(id, { pinned: prev }));
  }
  async function snoozeMessage(id: Id, untilIso: string): Promise<void> {
    const prev = messages().find((m) => m.id === id)?.snoozedUntil ?? null;
    await rawMeta(id, { snoozedUntil: untilIso });
    showUndo(t('mail-toast-snoozed'), () => rawMeta(id, { snoozedUntil: prev }));
  }
  async function unsnoozeMessage(id: Id): Promise<void> {
    await rawMeta(id, { snoozedUntil: null });
  }
  async function setFollowUp(id: Id, atIso: string | null): Promise<void> {
    const prev = messages().find((m) => m.id === id)?.followUpAt ?? null;
    await rawMeta(id, { followUpAt: atIso });
    showUndo(atIso ? t('mail-toast-follow-up-set') : t('mail-toast-follow-up-cleared'), () =>
      rawMeta(id, { followUpAt: prev }),
    );
  }

  // ── archive / trash / move / spam ─────────────────────────────────────────
  /**
   * Move `ids` to `targetMailboxId` with the shared undo. Online, every id goes
   * in ONE `Email/set` (t28-e16); offline, each is queued as its own move (the
   * replay queue's unit). Rows leave the list at once; ids the server refuses,
   * or all of them when the request fails, come back.
   */
  async function relocateWithUndo(
    ids: readonly Id[],
    targetMailboxId: Id,
    label: (count: number) => string,
  ): Promise<void> {
    const acct = accountId();
    if (acct === null) return;
    let taken = takeFromList(ids);
    if (taken.length === 0) return;
    const queue = ctx;
    // The id of each queued move, when the move was queued and the queue says
    // which item it became (a context without the offline slice says nothing).
    const queuedIds = new Map<Id, string>();
    if (isOffline() && queue.enqueueOffline) {
      for (const { email } of taken) {
        const queued = await queue.enqueueOffline('move', {
          accountId: acct,
          emailId: email.id,
          mailboxIds: { [targetMailboxId]: true },
        });
        if (typeof queued === 'string') queuedIds.set(email.id, queued);
      }
    } else {
      const patches: Record<Id, Record<string, unknown>> = {};
      for (const { email } of taken) patches[email.id] = { mailboxIds: { [targetMailboxId]: true } };
      let res: JmapResponse;
      try {
        res = await client.jmap(updateEmails(acct, patches));
      } catch {
        restoreToList(taken);
        showToast('error', t('mail-toast-move-failed'));
        return;
      }
      const refused = refusedUpdates(res, Object.keys(patches));
      if (refused.size > 0) {
        restoreToList(taken.filter((x) => refused.has(x.email.id)));
        taken = taken.filter((x) => !refused.has(x.email.id));
        showToast('error', t('mail-toast-update-refused', { count: refused.size }));
      }
      applyCounts(res);
      ctx.broadcastChange?.();
      if (taken.length === 0) return;
    }
    const moved = taken;
    showUndo(label(moved.length), async () => {
      // Still waiting in the offline queue: take it out, and the server never
      // hears of the move.
      const back: Taken[] = [];
      const reverse: Taken[] = [];
      for (const item of moved) {
        const queuedId = queuedIds.get(item.email.id);
        if (queuedId !== undefined && queue.dequeueOffline && (await queue.dequeueOffline(queuedId))) back.push(item);
        else reverse.push(item);
      }
      // The rest reached the server, or are queued where they cannot be taken
      // back: move each message to where it was. Offline, that goes into the
      // queue too, behind the original, so the pair nets out on reconnect.
      if (reverse.length > 0) {
        if (isOffline() && queue.enqueueOffline) {
          for (const { email } of reverse) {
            await queue.enqueueOffline('move', { accountId: acct, emailId: email.id, mailboxIds: email.mailboxIds });
          }
        } else {
          const patches: Record<Id, Record<string, unknown>> = {};
          for (const { email } of reverse) patches[email.id] = { mailboxIds: email.mailboxIds };
          applyCounts(await client.jmap(updateEmails(acct, patches)));
          ctx.broadcastChange?.();
        }
      }
      restoreToList([...back, ...reverse]);
    });
  }
  async function archiveMessage(ids: Id | readonly Id[]): Promise<void> {
    const target = roleOf(mailboxes(), 'archive');
    if (target === null) {
      showToast('error', t('mail-toast-no-archive-folder'));
      return;
    }
    await relocateWithUndo(asIds(ids), target, (count) =>
      count === 1 ? t('mail-toast-archived') : t('mail-toast-archived-count', { count }),
    );
  }
  async function trashMessage(ids: Id | readonly Id[]): Promise<void> {
    const target = roleOf(mailboxes(), 'trash');
    if (target === null) {
      showToast('error', t('mail-toast-no-trash-folder'));
      return;
    }
    await relocateWithUndo(asIds(ids), target, (count) =>
      count === 1 ? t('mail-toast-trashed') : t('mail-toast-trashed-count', { count }),
    );
  }
  async function markSpam(ids: Id | readonly Id[]): Promise<void> {
    const target = roleOf(mailboxes(), 'junk');
    if (target === null) {
      showToast('error', t('mail-toast-no-spam-folder'));
      return;
    }
    await relocateWithUndo(asIds(ids), target, (count) =>
      count === 1 ? t('mail-toast-marked-spam') : t('mail-toast-marked-spam-count', { count }),
    );
  }
  async function moveMessage(ids: Id | readonly Id[], mailboxId: Id): Promise<void> {
    const box = mailboxes().find((m) => m.id === mailboxId);
    await relocateWithUndo(asIds(ids), mailboxId, (count) => {
      if (box === undefined) return count === 1 ? t('mail-toast-moved') : t('mail-toast-moved-count', { count });
      const folder = isolate(box.name);
      return count === 1 ? t('mail-toast-moved-to', { folder }) : t('mail-toast-moved-to-count', { folder, count });
    });
  }

  // ── sweep ─────────────────────────────────────────────────────────────────
  function sweepMatches(fromEmail: string, strategy: SweepStrategy, olderThanDays?: number): Email[] {
    const target = fromEmail.trim().toLowerCase();
    const fromSender = messages().filter((m) => (m.from?.[0]?.email ?? '').toLowerCase() === target);
    // Newest-first (the list is already receivedAt-desc, but be explicit).
    const byDate = [...fromSender].sort((a, b) => b.receivedAt.localeCompare(a.receivedAt));
    switch (strategy) {
      case 'keep-latest':
        return byDate.slice(1);
      case 'older-than': {
        const days = olderThanDays ?? 30;
        const cutoff = Date.now() - days * 86_400_000;
        return byDate.filter((m) => new Date(m.receivedAt).getTime() < cutoff);
      }
      case 'all':
      case 'block':
      default:
        return byDate;
    }
  }
  function sweepPreview(fromEmail: string, strategy: SweepStrategy, olderThanDays?: number): Email[] {
    return sweepMatches(fromEmail, strategy, olderThanDays);
  }
  async function executeSweep(fromEmail: string, strategy: SweepStrategy, olderThanDays?: number): Promise<void> {
    const acct = accountId();
    if (acct === null) return;
    const trash = roleOf(mailboxes(), 'trash');
    if (trash === null) {
      showToast('error', t('mail-toast-no-trash-folder'));
      return;
    }
    const victims = sweepMatches(fromEmail, strategy, olderThanDays);
    if (victims.length === 0) {
      showToast('info', t('mail-toast-sweep-nothing'));
      return;
    }
    // Snapshot for undo, then move each victim to Trash.
    const taken = victims
      .map((v) => {
        const idx = messages().findIndex((m) => m.id === v.id);
        return idx < 0 ? null : { email: v, index: idx };
      })
      .filter((t): t is { email: Email; index: number } => t !== null);
    const remaining = messages().filter((m) => !victims.some((v) => v.id === m.id));
    setMessages(remaining);
    for (const v of victims) await client.jmap(moveEmail(acct, v.id, { [trash]: true }));
    if (strategy === 'block') {
      const target = fromEmail.trim().toLowerCase();
      if (!blocked().includes(target)) {
        const next = [...blocked(), target];
        setBlocked(next);
        try {
          globalThis.localStorage?.setItem(BLOCK_KEY, JSON.stringify(next));
        } catch {
          /* ignore */
        }
      }
    }
    showUndo(t('mail-toast-swept', { count: victims.length }), async () => {
      for (const t of taken) {
        await client.jmap(moveEmail(acct, t.email.id, t.email.mailboxIds));
      }
      // Restore original ordering.
      const restored = [...messages()];
      for (const t of [...taken].sort((a, b) => a.index - b.index)) {
        const at = Math.min(t.index, restored.length);
        restored.splice(at, 0, t.email);
      }
      setMessages(restored);
    });
  }

  // ── focused-inbox training ────────────────────────────────────────────────
  function trainSender(email: string, dest: InboxTab): void {
    const next = { ...training(), [email.toLowerCase()]: dest };
    setTraining(next);
    try {
      globalThis.localStorage?.setItem(FOCUS_KEY, JSON.stringify(next));
    } catch {
      /* ignore */
    }
  }
  function classify(email: Email): InboxTab {
    const from = (email.from?.[0]?.email ?? '').toLowerCase();
    return training()[from] ?? heuristicTab(email);
  }

  // ── derived views ─────────────────────────────────────────────────────────
  const visibleMessages = createMemo<Email[]>(() => {
    const now = Date.now();
    const shown = messages().filter((m) => {
      const s = m.snoozedUntil;
      return s == null || new Date(s).getTime() <= now;
    });
    // Stable partition: pinned first, order otherwise preserved.
    const pinned = shown.filter((m) => m.pinned === true);
    const rest = shown.filter((m) => m.pinned !== true);
    return [...pinned, ...rest];
  });
  const focusedMessages = createMemo<Email[]>(() => visibleMessages().filter((m) => classify(m) === 'focused'));
  const otherMessages = createMemo<Email[]>(() => visibleMessages().filter((m) => classify(m) === 'other'));
  const snoozedMessages = createMemo<Email[]>(() => {
    const now = Date.now();
    return messages().filter((m) => m.snoozedUntil != null && new Date(m.snoozedUntil).getTime() > now);
  });
  const followUps = createMemo<Email[]>(() =>
    messages()
      .filter((m) => m.followUpAt != null)
      .sort((a, b) => (a.followUpAt ?? '').localeCompare(b.followUpAt ?? '')),
  );
  const listMessages = createMemo<Email[]>(() => {
    if (!focusedInbox()) return visibleMessages();
    return inboxTab() === 'focused' ? focusedMessages() : otherMessages();
  });

  // ── V1 session + mail behaviour (unchanged) ───────────────────────────────
  async function loadMailboxes(): Promise<void> {
    const session = await client.session();
    setDownloadUrl(session.downloadUrl);
    const primary = session.primaryAccounts[CAP_MAIL];
    const acct = primary ?? Object.keys(session.accounts)[0] ?? null;
    setAccountId(acct);
    if (acct === null) {
      setMailboxes([]);
      return;
    }
    const res = await client.jmap(mailboxGet(acct));
    const boxes = sortMailboxes(responseFor<MailboxGetResponse>(res, 'c0').list);
    setMailboxes(boxes);
    const inbox = boxes.find((m) => m.role === 'inbox') ?? boxes[0];
    if (inbox !== undefined) {
      await selectMailbox(inbox.id);
    }
  }

  // ── list fetching + paging (t22-e4) ──────────────────────────────────────
  // Every entry point below has the same shape: a SYNCHRONOUS prelude that
  // records intent (which mailbox, search or not) in call order, then an async
  // payload behind the generation guard. Intent must not be guarded — the user's
  // last click is the current selection regardless of which fetch resolves — and
  // the payload must be, because the network does not preserve call order.

  /** The one-round-trip request for a window of `src`. */
  function pageRequest(
    acct: string,
    src: PageSource,
    position: number,
    limit: number,
    calculateTotal: boolean,
  ): JmapRequest {
    const page = { position, calculateTotal };
    return src.kind === 'mailbox'
      ? listMailbox(acct, src.mailboxId, limit, page)
      : searchEmails(acct, src.filter, limit, page);
  }

  /** The query total a list response reported, or `null` when it reported none. */
  function pageTotal(res: JmapResponse): number | null {
    const q = responseFor<EmailQueryResponse>(res, 'q');
    return typeof q.total === 'number' ? q.total : null;
  }

  /**
   * Append `page` to `prev`, skipping ids already held — a page can overlap what
   * is loaded when a message arrives above the window between requests.
   *
   * Returns `prev` ITSELF, not a copy, when nothing is new. A fresh array here
   * would notify every downstream memo and rebuild every mounted row for a page
   * that added nothing, which is precisely the churn `57046c7` removed from the
   * scroll path.
   */
  function appendPage(prev: Email[], page: Email[]): Email[] {
    if (page.length === 0) return prev;
    const held = new Set(prev.map((m) => m.id));
    const fresh = page.filter((m) => !held.has(m.id));
    if (fresh.length === 0) return prev;
    return [...prev, ...fresh];
  }

  /**
   * Replace the list with the first page of `src`.
   *
   * If this request is superseded before it resolves, its response is dropped
   * whole: no rows written, no spinner cleared, no error raised. That is the
   * whole of the stale-response fix — the newer selection's fetch is the only
   * one that can write, whichever order the two come back in.
   */
  async function loadFirstPage(acct: string, src: PageSource): Promise<void> {
    setPageSource(src);
    const req = beginListRequest();
    setListLoading(true);
    try {
      const res = await jmapCall(pageRequest(acct, src, 0, PAGE_SIZE, true), { signal: req.signal });
      if (!isCurrent(req)) return;
      const page = responseFor<EmailGetResponse>(res, 'g').list;
      setMessages(page);
      setLoadedStart(0);
      setTotal(pageTotal(res));
      setExhausted(page.length < PAGE_SIZE);
    } catch (err) {
      // A superseded request's failure is not the user's problem: the request
      // that replaced it owns the outcome, including the error surface.
      if (!isCurrent(req)) return;
      throw err;
    } finally {
      if (isCurrent(req)) setListLoading(false);
      endListRequest(req);
    }
  }

  async function loadMore(): Promise<void> {
    const acct = accountId();
    const src = pageSource();
    // Every refusal below returns WITHOUT writing a signal, so a scroll that
    // cannot page costs nothing and rebuilds nothing.
    if (acct === null || src === null) return;
    if (listLoading() || loadingMore() || !hasMore()) return;
    const position = loadedRange().end;
    const req = beginListRequest();
    setLoadingMore(true);
    try {
      const res = await jmapCall(pageRequest(acct, src, position, PAGE_SIZE, false), { signal: req.signal });
      if (!isCurrent(req)) return;
      const page = responseFor<EmailGetResponse>(res, 'g').list;
      const before = messages();
      const next = appendPage(before, page);
      setMessages(next);
      // Continuations do not ask for a total; keep the one the query reported.
      const t = pageTotal(res);
      if (t !== null) setTotal(t);
      // A page that ADDS NOTHING ends the query, not just a short one.
      //
      // A continuation can come back full-size and yet hold only ids already
      // loaded — the query's window shifted under the reader between requests.
      // Nothing is appended, so `loadedRange().end` does not move, so the next
      // request is byte-identical to the one that just returned. Asking again
      // cannot make a page that added nothing add something, and with `hasMore`
      // left true an append-on-scroll caller re-issues that request on every
      // scroll event for as long as the user keeps scrolling — an unbounded
      // request loop whose only symptom is "the list feels slow".
      //
      // The test is `next === before` — did the MESSAGE array move — and it must
      // stay that. The tempting rewrite is to compare a rendered row count, and
      // that version is wrong on a shape most fixtures do not contain: a page
      // that folds entirely into conversations already on screen adds messages
      // while leaving the visible row count unchanged, so a rows-based test ends
      // the query on a folder that is one large thread. That is a worse bug than
      // the loop this guards, and it is the version written first.
      setExhausted(page.length < PAGE_SIZE || next === before);
    } catch {
      // A failed page leaves the query un-exhausted, so the next scroll retries.
      // It must not throw: this runs from a scroll handler.
      if (isCurrent(req)) showToast('error', t('mail-toast-load-more-failed'));
    } finally {
      if (isCurrent(req)) setLoadingMore(false);
      endListRequest(req);
    }
  }

  async function selectMailbox(id: Id): Promise<void> {
    setSelectedMailboxId(id);
    setSearchActive(false);
    setSearch('');
    cancelMarkRead();
    clearSelection();
    setOpenEmail(null);
    setSanitizedHtml(null);
    const acct = accountId();
    if (acct === null) return;
    await loadFirstPage(acct, { kind: 'mailbox', mailboxId: id });
  }

  function readFromCache(id: Id): void {
    const cached = messages().find((m) => m.id === id) ?? null;
    setOpenEmail(cached);
    setSanitizedHtml(cached !== null ? extractHtmlBody(cached) : null);
    if (cached !== null) scheduleMarkRead(cached);
  }

  async function openMessage(id: Id): Promise<void> {
    const acct = accountId();
    if (acct === null) return;
    // Whatever was pending for the previous message is void: it is no longer open.
    cancelMarkRead();
    // Offline: render the cached header's escaped preview (a reduced read).
    if (isOffline()) {
      readFromCache(id);
      return;
    }
    setReadLoading(true);
    setSanitizedHtml(null);
    try {
      const res = await client.jmap(emailGetFull(acct, id));
      const email = responseFor<EmailGetResponse>(res, 'g').list[0] ?? null;
      setOpenEmail(email);
      if (email !== null) {
        scheduleMarkRead(email);
        const raw = extractHtmlBody(email);
        const clean = await client.sanitize(raw);
        setSanitizedHtml(clean);
      }
    } catch (err) {
      // A mid-read network drop degrades to the cached header, not an error.
      if (err instanceof NetworkError) readFromCache(id);
      else throw err;
    } finally {
      setReadLoading(false);
    }
  }

  function closeMessage(): void {
    cancelMarkRead();
    setOpenEmail(null);
    setSanitizedHtml(null);
  }

  // ── V2 integration: push/peer refetch, search, attachments, export ────────
  async function refreshCurrentMailbox(): Promise<void> {
    const acct = accountId();
    const src = pageSource();
    if (acct === null || searchActive()) return;
    if (src === null || src.kind !== 'mailbox') return;
    // A replace or an append already in flight is strictly better than this
    // refresh; superseding it would abandon a spinner nothing else clears.
    if (listLoading() || loadingMore()) return;
    const range = loadedRange();
    const loaded = Math.max(PAGE_SIZE, range.end - range.start);
    if (loaded > REFRESH_MAX_ROWS) return; // see REFRESH_MAX_ROWS
    const req = beginListRequest();
    try {
      const res = await jmapCall(pageRequest(acct, src, range.start, loaded, true), { signal: req.signal });
      if (!isCurrent(req)) return;
      const page = responseFor<EmailGetResponse>(res, 'g').list;
      setMessages(page);
      setLoadedStart(range.start);
      setTotal(pageTotal(res));
      setExhausted(page.length < loaded);
    } catch {
      // Transient/offline refetch — keep the current list.
    } finally {
      endListRequest(req);
    }
  }

  async function clearSearch(): Promise<void> {
    clearSelection();
    setSearch('');
    setSearchActive(false);
    const cur = selectedMailboxId();
    if (cur !== null) await selectMailbox(cur);
  }

  async function searchMessages(query: string, opts?: { semantic?: boolean }): Promise<void> {
    const acct = accountId();
    if (acct === null) return;
    setSearch(query);
    clearSelection();
    if (query.trim() === '') {
      await clearSearch();
      return;
    }
    // Offline: the reduced substring search over the cached header slice. It is
    // not a server query, so there is nothing to page: the cached slice IS the
    // whole result, which is why `total` may honestly be the row count here and
    // nowhere else.
    if (isOffline() && ctx.searchOffline) {
      const hits = ctx.searchOffline({ text: query });
      setPageSource(null);
      setMessages(hits);
      setLoadedStart(0);
      setTotal(hits.length);
      setExhausted(true);
      setSearchActive(true);
      return;
    }
    // Intent before payload: the list is showing search results from the moment
    // the user asks, not from the moment the network agrees.
    setSearchActive(true);
    // The whole operator string rides `filter.text`; the engine routes it to
    // mw-search, which parses `from:`/`subject:`/`larger:`/… itself (§2.1). The
    // semantic flag (V7 §14.3) is added only when the Assist toggle is on.
    const filter: FilterCondition = {
      text: query,
      ...(opts?.semantic === true ? { semantic: true } : {}),
    };
    await loadFirstPage(acct, { kind: 'search', filter });
  }

  async function listAttachments(): Promise<AttachmentItem[]> {
    const acct = accountId();
    if (acct === null) return [];
    return loadAttachments(client, acct);
  }

  function triggerDownload(objectUrl: string, name: string): void {
    if (typeof document === 'undefined') return;
    const a = document.createElement('a');
    a.href = objectUrl;
    a.download = name;
    a.rel = 'noopener';
    document.body.appendChild(a);
    a.click();
    a.remove();
    setTimeout(() => URL.revokeObjectURL(objectUrl), 10_000);
  }

  async function exportMessage(): Promise<void> {
    const email = openEmail();
    const url = downloadUrl();
    const acct = accountId();
    if (email === null || acct === null || url === null) return;
    const blobId = email.blobId;
    if (blobId === undefined || blobId === '') {
      showToast('error', t('mail-toast-export-nothing'));
      return;
    }
    const base = (email.subject ?? 'message').replace(/[^\w.-]+/g, '_').slice(0, 80);
    const name = `${base.length > 0 ? base : 'message'}.eml`;
    try {
      const dl = buildDownloadUrl(url, { accountId: acct, blobId, name, mime: 'message/rfc822' });
      const objectUrl = await fetchObjectUrl(dl);
      triggerDownload(objectUrl, name);
      showToast('success', t('mail-toast-exported'));
    } catch {
      showToast('error', t('mail-toast-export-failed'));
    }
  }

  async function sendMessage(input: SendInput): Promise<void> {
    const acct = accountId();
    const user = me();
    if (acct === null || user === null) throw new Error(t('mail-send-not-signed-in'));
    const drafts = roleOf(mailboxes(), 'drafts') ?? selectedMailboxId();
    if (drafts === null) throw new Error(t('mail-send-no-draft-folder'));
    const sent = roleOf(mailboxes(), 'sent');

    // Refuse what the server would refuse (`Email/set` rejects a draft whose
    // `to`, `cc` or `bcc` holds anything but mailboxes, and a submission with
    // no recipient at all), naming the entries, before a request is made or —
    // offline — before the send is queued to fail on reconnect.
    const cc = input.cc ?? '';
    const bcc = input.bcc ?? '';
    const everyone = [input.to, cc, bcc].join(',');
    if (parseRecipients(everyone).length === 0) throw new Error(t('mail-send-no-recipient'));
    const bad = invalidRecipients(everyone);
    if (bad.length > 0) {
      throw new Error(t('mail-send-bad-recipient', { count: bad.length, addresses: isolate(bad.join(', ')) }));
    }

    const identity = input.identity ?? null;
    const fromEmail = identity?.email ?? user.username;
    const fromName = identity?.name ?? null;

    // A scheduled send carries no undo hold, so a `sendAt` that is unreadable
    // or already past would leave at once and could not be cancelled. It is
    // refused here; only a time in the future zeroes the hold.
    let scheduled = false;
    if (input.sendAt != null && input.sendAt.length > 0) {
      const at = new Date(input.sendAt).getTime();
      if (Number.isNaN(at) || at <= Date.now()) throw new Error(t('mail-send-later-past'));
      scheduled = true;
    }
    const holdSeconds = scheduled ? 0 : (input.holdSeconds ?? 10);

    const draft: DraftInput = {
      from: { name: fromName, email: fromEmail },
      draftMailboxId: drafts,
      to: input.to,
      ...(cc.trim() !== '' ? { cc } : {}),
      ...(bcc.trim() !== '' ? { bcc } : {}),
      ...(input.inReplyTo !== undefined && input.inReplyTo.length > 0 ? { inReplyTo: input.inReplyTo } : {}),
      ...(input.references !== undefined && input.references.length > 0 ? { references: input.references } : {}),
      subject: input.subject,
      htmlBody: input.htmlBody,
      holdSeconds,
      ...(sent !== null ? { sentMailboxId: sent } : {}),
      ...(identity !== null ? { identityId: identity.id } : {}),
      ...(scheduled ? { sendAt: input.sendAt! } : {}),
      ...(input.attachments !== undefined && input.attachments.length > 0
        ? { attachments: input.attachments }
        : {}),
      ...(input.inline !== undefined && input.inline.length > 0 ? { inline: input.inline } : {}),
      ...(input.requestReadReceipt === true ? { requestReadReceipt: true } : {}),
    };
    const track = input.openTracking === true;

    // Offline: queue the send for replay on reconnect (drainOutbox → sendEnvelope).
    // A tracked send is queued without its image; the replay mints the token.
    if (isOffline() && ctx.enqueueOffline) {
      await ctx.enqueueOffline('send', { accountId: acct, draft, ...(track ? { track: true } : {}) });
      showToast('info', t('mail-toast-send-queued'));
      return;
    }

    let outgoing = draft;
    let trackingToken: string | null = null;
    if (track) {
      try {
        const tracked = await withTrackingPixel(draft);
        outgoing = tracked.draft;
        trackingToken = tracked.token;
      } catch {
        throw new Error(t('mail-send-tracking-failed'));
      }
    }

    const res = await client.jmap(sendEnvelope(acct, outgoing));
    ctx.broadcastChange?.();
    const setRes = responseFor<EmailSetResponse>(res, 'set');
    // A refused create throws, so the composer stays open and shows why.
    const draftRefused = setRes.notCreated?.['draft'];
    if (draftRefused !== undefined) {
      throw new Error(t('mail-send-draft-refused', { reason: isolate(refusalReason(draftRefused)) }));
    }
    const subRes = responseFor<EmailSubmissionSetResponse>(res, 'submit');
    const sendRefused = subRes.notCreated?.['send'];
    if (sendRefused !== undefined) {
      throw new Error(t('mail-send-submission-refused', { reason: isolate(refusalReason(sendRefused)) }));
    }
    const submissionId = subRes.created?.['send']?.id ?? null;

    // Bind the tracking token to the Email the send created. The Email keeps
    // its id when it is filed into Sent, which is where the count is read.
    if (trackingToken !== null) {
      const emailId = setRes.created?.['draft']?.id;
      const bound = emailId !== undefined && (await bindTrackingToken(trackingToken, emailId));
      if (!bound) showToast('error', t('mail-toast-tracking-not-bound'));
    }

    // Mark the original answered / forwarded. This is done when the submission
    // is accepted, not when SMTP has taken the message: the engine holds every
    // send for the undo window (or until its scheduled time), and nothing tells
    // the client when it leaves. A send cancelled from the toast below clears
    // the mark; a scheduled send cancelled later from the Outbox does not.
    // The send itself has succeeded, so a failure here is not reported as one.
    const source = input.source;
    if (source !== undefined) {
      try {
        await rawKeyword(source.emailId, source.keyword, true);
      } catch {
        // The message went out; only the mark is missing.
      }
    }

    if (scheduled) {
      showToast('success', t('mail-toast-send-scheduled'));
    } else if (submissionId !== null) {
      // Undo-send: the engine holds the submission for `holdSeconds`; the toast
      // Cancel path flips it to `canceled` before it dials SMTP (plan §1.3).
      showUndo(
        t('mail-toast-sent'),
        async () => {
          await client.jmap(cancelSubmission(acct, submissionId));
          if (source !== undefined) await rawKeyword(source.emailId, source.keyword, false).catch(() => undefined);
          showToast('info', t('mail-toast-send-canceled'));
        },
        holdSeconds * 1000,
        t('mail-undo-send-cancel'),
        true,
      );
    } else {
      showToast('success', t('mail-toast-sent'));
    }

    const current = selectedMailboxId();
    if (current !== null) await selectMailbox(current);
  }

  // Forced password change (t27, OH-1). While the account's admin flag is set
  // the server answers every JMAP request with 403, so a flagged session is
  // recorded (`me()` carries `passwordChangeRequired`, which the shell renders
  // as the change screen) and no mail is loaded for it.
  const mustChangePassword = (user: Me): boolean => user.passwordChangeRequired === true;
  function holdForPasswordChange(): void {
    setMe((user) => (user === null || mustChangePassword(user) ? user : { ...user, passwordChangeRequired: true }));
  }

  // The flag can also be set while a session is open: the next request is then
  // refused, and the client reports it here.
  client.onPasswordChangeRequired?.(holdForPasswordChange);

  /** Load mail for a new session, unless the server holds it at the change screen. */
  async function loadMailFor(user: Me): Promise<void> {
    if (mustChangePassword(user)) return;
    try {
      await loadMailboxes();
    } catch (err) {
      // `/api/me` did not carry the flag but the first mail request was refused
      // for it. That is the hold, not a boot or sign-in failure.
      if (!(err instanceof PasswordChangeRequired)) throw err;
      holdForPasswordChange();
    }
  }

  async function init(): Promise<void> {
    try {
      const user = await client.me();
      setMe(user);
      setAccountId(user.accountId);
      await loadMailFor(user);
    } catch (err) {
      if (err instanceof ApiError && err.status === 401) {
        setMe(null);
      } else {
        throw err;
      }
    } finally {
      setAuthChecked(true);
    }
  }

  async function login(input: LoginInput): Promise<void> {
    const user = await client.login(input);
    setMe(user);
    setAccountId(user.accountId);
    await loadMailFor(user);
  }

  async function logout(): Promise<void> {
    try {
      await client.logout();
    } finally {
      setMe(null);
      setAccountId(null);
      setMailboxes([]);
      setMessages([]);
      setSelectedMailboxId(null);
      cancelMarkRead();
      setSelectionRaw(new Set<Id>());
      setOpenEmail(null);
      setSanitizedHtml(null);
      // Drop the paging cursor too, and supersede anything in flight so a fetch
      // issued as the previous user cannot land rows after the session ended.
      beginListRequest();
      setPageSource(null);
      setTotal(null);
      setLoadedStart(0);
      setExhausted(true);
      setListLoading(false);
      dismissUndo();
      // What this browser holds for the account outside the store: the
      // composer's auto-saved drafts (plaintext in localStorage) and the
      // service worker's caches. The worker deletes its caches itself when it
      // sees the logout request, but only for a page it controls; this covers
      // the page it does not (see sw/purge.ts). Not awaited: it cannot fail and
      // the signed-out screen should not wait on Cache Storage.
      clearDrafts();
      void purgeShellCaches();
    }
  }

  return {
    me,
    authChecked,
    mailboxes,
    selectedMailboxId,
    messages,
    listLoading,
    openEmail,
    sanitizedHtml,
    readLoading,
    accountId,
    sentMailboxId: () => roleOf(mailboxes(), 'sent'),
    draftsMailboxId: () => roleOf(mailboxes(), 'drafts'),
    downloadUrl,
    search,
    searchActive,
    searchMessages,
    clearSearch,
    refreshCurrentMailbox,

    total,
    loadedRange,
    hasMore,
    loadingMore,
    loadMore,

    listAttachments,
    exportMessage,

    visibleMessages,
    focusedMessages,
    otherMessages,
    snoozedMessages,
    followUps,
    listMessages,
    inboxTab,
    setInboxTab,
    focusedInbox,
    setFocusedInbox,

    applyTag,
    removeTag,
    pinMessage,
    snoozeMessage,
    unsnoozeMessage,
    setFollowUp,
    archiveMessage,
    trashMessage,
    moveMessage,
    markSpam,
    markRead,
    flagMessage,

    selectedIds,
    setSelection,
    toggleSelection,
    selectAllInView,
    clearSelection,
    actionTargets,

    sweepPreview,
    executeSweep,
    blockedSenders: blocked,

    trainSender,

    pendingUndo,
    undoNow,
    dismissUndo,

    init,
    login,
    logout,
    selectMailbox,
    openMessage,
    closeMessage,
    sendMessage,
  };
}

/** Pick the HTML body value to sanitize, falling back to escaped text/preview. */
export function extractHtmlBody(email: Email): string {
  const parts = email.htmlBody ?? [];
  const values = email.bodyValues ?? {};
  for (const part of parts) {
    if (part.partId !== null) {
      const v = values[part.partId];
      if (v !== undefined) return v.value;
    }
  }
  const text = email.textBody
    ?.map((p) => (p.partId !== null ? (values[p.partId]?.value ?? '') : ''))
    .join('\n');
  const fallback = text && text.length > 0 ? text : email.preview;
  return `<pre>${escapeHtml(fallback)}</pre>`;
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
}
