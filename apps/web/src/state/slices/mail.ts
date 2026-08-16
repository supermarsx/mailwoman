// Mail slice: session/auth, mailbox list, message list + reader, compose+send,
// and — new in V2 (plan §3 e7, §1.5) — the modern-mail UX that mutates the
// message list: tags (keywords), pins, snooze, follow-up, archive/trash/move/
// spam, sweep, the shared 10-second undo primitive, and the focused/unified
// inbox derivation. The V1 session+mail behaviour is preserved verbatim; the
// V2 surface is additive.

import { createSignal, createMemo, type Accessor } from 'solid-js';
import { ApiError, NetworkError, type LoginInput, type Me } from '../../api/client.ts';
import {
  cancelSubmission,
  emailGetFull,
  listMailbox,
  mailboxGet,
  moveEmail,
  responseFor,
  searchEmails,
  sendEnvelope,
  setEmailKeyword,
  setEmailMeta,
  type DraftInput,
} from '../../api/jmap.ts';
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
} from '../../api/jmap-types.ts';
import type { SliceContext } from './context.ts';

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

/**
 * The transport call, widened with an optional `{ signal }`.
 *
 * `Client.jmap` is declared `(body) => Promise<JmapResponse>`, and a function of
 * that type is assignable to this one (TypeScript lets an implementation ignore
 * trailing parameters), so this compiles against today's client and starts
 * cancelling for real the moment `api/client.ts` threads the signal into its
 * `fetch`. That one-line change is outside this lane's locks — see the lane log.
 *
 * Until it lands the signal is not wasted: it is what makes "this request has
 * been superseded" an observable fact rather than an internal counter, and it is
 * what the interleaving test asserts on. The property that actually closes the
 * stale-response race is the generation guard below, which does not depend on
 * the transport honouring anything.
 */
type JmapCall = (body: JmapRequest, opts?: { signal?: AbortSignal }) => Promise<JmapResponse>;

/** A dismissable, time-boxed reversible action (the 10-second undo, §1.5). */
export interface PendingUndo {
  label: string;
  /** Label for the action button (`Undo` by default; `Cancel` for undo-send). */
  actionLabel: string;
  /** Reverses the action; resolves once reversed. */
  run: () => Promise<void>;
  /** Epoch ms after which the toast auto-commits and disappears. */
  expiresAt: number;
}

/** Which sweep to run against a sender's mail (Outlook-style, §1.5). */
export type SweepStrategy = 'all' | 'block' | 'keep-latest' | 'older-than';

/** The two-tab focused inbox destinations (§1.5). */
export type InboxTab = 'focused' | 'other';

/** The compose/send input (grown for identities + send-later + undo-send). */
export interface SendInput {
  to: string;
  subject: string;
  htmlBody: string;
  /** Send as this identity (from-address + signature); `null` = default. */
  identity?: Identity | null;
  /** Send-later: ISO 8601 UTC time; omitted/null = send now (with undo window). */
  sendAt?: string | null;
  /** Undo-send window in seconds; default 10. Ignored when `sendAt` is set. */
  holdSeconds?: number;
  /** V7 (§18.4): server-materialised blob attachments (e.g. from Nextcloud). */
  attachments?: { blobId: Id; name: string; type: string; size?: number }[];
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
  unifiedInbox: Accessor<boolean>;
  setUnifiedInbox(on: boolean): void;
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
  archiveMessage(id: Id): Promise<void>;
  trashMessage(id: Id): Promise<void>;
  moveMessage(id: Id, mailboxId: Id): Promise<void>;
  markSpam(id: Id): Promise<void>;

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

  // `Client.jmap` takes no options today; see `JmapCall` for why passing one is
  // both type-safe and forward-compatible.
  const jmapCall: JmapCall = client.jmap.bind(client);

  const [inboxTab, setInboxTab] = createSignal<InboxTab>('focused');
  const [unifiedInbox, setUnifiedInbox] = createSignal(false);
  const [focusedInbox, setFocusedInbox] = createSignal(false);
  const [pendingUndo, setPendingUndo] = createSignal<PendingUndo | null>(null);
  const [training, setTraining] = createSignal<Record<string, InboxTab>>(loadMap(FOCUS_KEY));
  const [blocked, setBlocked] = createSignal<string[]>(loadList(BLOCK_KEY));

  let undoTimer: ReturnType<typeof setTimeout> | undefined;

  // ── undo primitive ──────────────────────────────────────────────────────
  function showUndo(label: string, run: () => Promise<void>, ttlMs = 10_000, actionLabel = 'Undo'): void {
    if (undoTimer !== undefined) clearTimeout(undoTimer);
    setPendingUndo({ label, actionLabel, run, expiresAt: Date.now() + ttlMs });
    undoTimer = setTimeout(() => setPendingUndo(null), ttlMs);
  }
  function dismissUndo(): void {
    if (undoTimer !== undefined) clearTimeout(undoTimer);
    setPendingUndo(null);
  }
  async function undoNow(): Promise<void> {
    const p = pendingUndo();
    dismissUndo();
    if (p === undefined || p === null) return;
    try {
      await p.run();
    } catch {
      showToast('error', 'Could not undo');
    }
  }

  // ── message-list helpers ─────────────────────────────────────────────────
  function patchMessage(id: Id, patch: Partial<Email>): void {
    setMessages((msgs) => msgs.map((m) => (m.id === id ? { ...m, ...patch } : m)));
    const open = openEmail();
    if (open !== null && open.id === id) setOpenEmail({ ...open, ...patch });
  }

  /** Remove a message from the current list, returning it + its index for undo. */
  function takeFromList(id: Id): { email: Email; index: number } | null {
    const msgs = messages();
    const index = msgs.findIndex((m) => m.id === id);
    if (index < 0) return null;
    const email = msgs[index]!;
    setMessages([...msgs.slice(0, index), ...msgs.slice(index + 1)]);
    if (openEmail()?.id === id) closeMessage();
    return { email, index };
  }
  function restoreToList(taken: { email: Email; index: number }): void {
    setMessages((msgs) => {
      const at = Math.min(taken.index, msgs.length);
      return [...msgs.slice(0, at), taken.email, ...msgs.slice(at)];
    });
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
    showUndo('Label added', () => rawKeyword(id, keyword, false));
  }
  async function removeTag(id: Id, keyword: string): Promise<void> {
    await rawKeyword(id, keyword, false);
    showUndo('Label removed', () => rawKeyword(id, keyword, true));
  }

  // ── pin / snooze / follow-up ──────────────────────────────────────────────
  async function pinMessage(id: Id, pinned: boolean): Promise<void> {
    const prev = messages().find((m) => m.id === id)?.pinned ?? false;
    await rawMeta(id, { pinned });
    showUndo(pinned ? 'Pinned' : 'Unpinned', () => rawMeta(id, { pinned: prev }));
  }
  async function snoozeMessage(id: Id, untilIso: string): Promise<void> {
    const prev = messages().find((m) => m.id === id)?.snoozedUntil ?? null;
    await rawMeta(id, { snoozedUntil: untilIso });
    showUndo('Snoozed', () => rawMeta(id, { snoozedUntil: prev }));
  }
  async function unsnoozeMessage(id: Id): Promise<void> {
    await rawMeta(id, { snoozedUntil: null });
  }
  async function setFollowUp(id: Id, atIso: string | null): Promise<void> {
    const prev = messages().find((m) => m.id === id)?.followUpAt ?? null;
    await rawMeta(id, { followUpAt: atIso });
    showUndo(atIso ? 'Follow-up set' : 'Follow-up cleared', () => rawMeta(id, { followUpAt: prev }));
  }

  // ── archive / trash / move / spam ─────────────────────────────────────────
  async function relocateWithUndo(id: Id, targetMailboxId: Id, label: string): Promise<void> {
    const acct = accountId();
    if (acct === null) return;
    const taken = takeFromList(id);
    if (taken === null) return;
    const priorMailboxIds = taken.email.mailboxIds;
    if (isOffline() && ctx.enqueueOffline) {
      await ctx.enqueueOffline('move', {
        accountId: acct,
        emailId: id,
        mailboxIds: { [targetMailboxId]: true },
      });
    } else {
      await client.jmap(moveEmail(acct, id, { [targetMailboxId]: true }));
      ctx.broadcastChange?.();
    }
    showUndo(label, async () => {
      await client.jmap(moveEmail(acct, id, priorMailboxIds));
      restoreToList(taken);
    });
  }
  async function archiveMessage(id: Id): Promise<void> {
    const target = roleOf(mailboxes(), 'archive');
    if (target === null) {
      showToast('error', 'No Archive folder');
      return;
    }
    await relocateWithUndo(id, target, 'Archived');
  }
  async function trashMessage(id: Id): Promise<void> {
    const target = roleOf(mailboxes(), 'trash');
    if (target === null) {
      showToast('error', 'No Trash folder');
      return;
    }
    await relocateWithUndo(id, target, 'Moved to Trash');
  }
  async function markSpam(id: Id): Promise<void> {
    const target = roleOf(mailboxes(), 'junk');
    if (target === null) {
      showToast('error', 'No Spam folder');
      return;
    }
    await relocateWithUndo(id, target, 'Marked as spam');
  }
  async function moveMessage(id: Id, mailboxId: Id): Promise<void> {
    const box = mailboxes().find((m) => m.id === mailboxId);
    await relocateWithUndo(id, mailboxId, `Moved to ${box?.name ?? 'folder'}`);
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
      showToast('error', 'No Trash folder');
      return;
    }
    const victims = sweepMatches(fromEmail, strategy, olderThanDays);
    if (victims.length === 0) {
      showToast('info', 'Nothing to sweep');
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
    showUndo(`Swept ${victims.length} message${victims.length === 1 ? '' : 's'}`, async () => {
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
    const boxes = responseFor<MailboxGetResponse>(res, 'c0').list;
    boxes.sort((a, b) => {
      if (a.role === 'inbox') return -1;
      if (b.role === 'inbox') return 1;
      if (a.sortOrder !== b.sortOrder) return a.sortOrder - b.sortOrder;
      return a.name.localeCompare(b.name);
    });
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
      setMessages((prev) => appendPage(prev, page));
      // Continuations do not ask for a total; keep the one the query reported.
      const t = pageTotal(res);
      if (t !== null) setTotal(t);
      setExhausted(page.length < PAGE_SIZE);
    } catch {
      // A failed page leaves the query un-exhausted, so the next scroll retries.
      // It must not throw: this runs from a scroll handler.
      if (isCurrent(req)) showToast('error', 'Could not load more messages');
    } finally {
      if (isCurrent(req)) setLoadingMore(false);
      endListRequest(req);
    }
  }

  async function selectMailbox(id: Id): Promise<void> {
    setSelectedMailboxId(id);
    setSearchActive(false);
    setSearch('');
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
  }

  async function openMessage(id: Id): Promise<void> {
    const acct = accountId();
    if (acct === null) return;
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
    setSearch('');
    setSearchActive(false);
    const cur = selectedMailboxId();
    if (cur !== null) await selectMailbox(cur);
  }

  async function searchMessages(query: string, opts?: { semantic?: boolean }): Promise<void> {
    const acct = accountId();
    if (acct === null) return;
    setSearch(query);
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
      showToast('error', 'Nothing to export');
      return;
    }
    const base = (email.subject ?? 'message').replace(/[^\w.-]+/g, '_').slice(0, 80);
    const name = `${base.length > 0 ? base : 'message'}.eml`;
    try {
      const dl = buildDownloadUrl(url, { accountId: acct, blobId, name, mime: 'message/rfc822' });
      const objectUrl = await fetchObjectUrl(dl);
      triggerDownload(objectUrl, name);
      showToast('success', 'Exported .eml');
    } catch {
      showToast('error', 'Export failed');
    }
  }

  async function sendMessage(input: SendInput): Promise<void> {
    const acct = accountId();
    const user = me();
    if (acct === null || user === null) throw new Error('not authenticated');
    const drafts = roleOf(mailboxes(), 'drafts') ?? selectedMailboxId();
    if (drafts === null) throw new Error('no mailbox to hold the draft');
    const sent = roleOf(mailboxes(), 'sent');

    const identity = input.identity ?? null;
    const fromEmail = identity?.email ?? user.username;
    const fromName = identity?.name ?? null;
    let html = input.htmlBody;
    if (identity?.signatureHtml) html += `<br><br>${identity.signatureHtml}`;

    const scheduled = input.sendAt != null && input.sendAt.length > 0;
    const holdSeconds = scheduled ? 0 : (input.holdSeconds ?? 10);

    const draft: DraftInput = {
      from: { name: fromName, email: fromEmail },
      draftMailboxId: drafts,
      to: input.to,
      subject: input.subject,
      htmlBody: html,
      holdSeconds,
      ...(sent !== null ? { sentMailboxId: sent } : {}),
      ...(identity !== null ? { identityId: identity.id } : {}),
      ...(scheduled ? { sendAt: input.sendAt! } : {}),
      ...(input.attachments !== undefined && input.attachments.length > 0
        ? { attachments: input.attachments }
        : {}),
    };

    // Offline: queue the send for replay on reconnect (drainOutbox → sendEnvelope).
    if (isOffline() && ctx.enqueueOffline) {
      await ctx.enqueueOffline('send', { accountId: acct, draft });
      showToast('info', 'Queued — will send when back online');
      return;
    }

    const res = await client.jmap(sendEnvelope(acct, draft));
    ctx.broadcastChange?.();
    const setRes = responseFor<EmailSetResponse>(res, 'set');
    if (setRes.notCreated?.['draft'] !== undefined) {
      throw new Error(`draft rejected: ${setRes.notCreated['draft'].type}`);
    }
    const subRes = responseFor<EmailSubmissionSetResponse>(res, 'submit');
    if (subRes.notCreated?.['send'] !== undefined) {
      throw new Error(`send rejected: ${subRes.notCreated['send'].type}`);
    }
    const submissionId = subRes.created?.['send']?.id ?? null;

    if (scheduled) {
      showToast('success', 'Scheduled to send');
    } else if (submissionId !== null) {
      // Undo-send: the engine holds the submission for `holdSeconds`; the toast
      // Cancel path flips it to `canceled` before it dials SMTP (plan §1.3).
      showUndo(
        'Message sent',
        async () => {
          await client.jmap(cancelSubmission(acct, submissionId));
          showToast('info', 'Send canceled');
        },
        holdSeconds * 1000,
        'Cancel',
      );
    } else {
      showToast('success', 'Message sent');
    }

    const current = selectedMailboxId();
    if (current !== null) await selectMailbox(current);
  }

  async function init(): Promise<void> {
    try {
      const user = await client.me();
      setMe(user);
      setAccountId(user.accountId);
      await loadMailboxes();
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
    await loadMailboxes();
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
    unifiedInbox,
    setUnifiedInbox,
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
