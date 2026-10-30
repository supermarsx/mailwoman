// t109: a reload puts the browser tab back where it was, including in a linked
// mailbox. What is kept, per browser tab (sessionStorage, so two tabs keep their
// own state), for the person signed in:
//   - which mailbox was active (the sign-in account or a link id);
//   - per mailbox: the folder, the reading-pane message, the list's scroll
//     anchor (the top row's message id, not pixels) and the search query.
// The workspace tabs (message, folder, compose, Settings, module tabs) are kept
// per mailbox by realtime/workspaceStorage.ts.
//
// The address bar carries the mailbox (a link id, never an address), the folder
// and the message (`?mailbox=&folder=&message=`), so a reload or a shared link
// opens the same place. Every id read back is a reference only: the mailbox
// goes through the full linked-mailbox authorization, a folder or message that
// no longer exists is skipped, and nothing recorded for one sign-in identity is
// read by another (the record names the sign-in account's storage authority).
// Sign-out forgets all of it.
//
// No localStorage fallback for a new browser tab: a new tab opens the mailbox
// the server marks to open first (t40), which is the person's stated choice for
// a fresh start; restoring another tab's linked mailbox there would override it.

export const SESSION_RESTORE_KEY = 'mw.session.v1';
const MAX_VIEWS = 16;
const MAX_SEARCH = 2000;
const URL_KEYS = ['mailbox', 'folder', 'message'] as const;

/** One mailbox's view: every field optional, each an opaque server id. */
export interface MailboxView {
  readonly authority: string;
  readonly folder?: string;
  readonly message?: string;
  readonly anchor?: string;
  readonly search?: string;
}
export interface SessionRecord {
  readonly version: 1;
  /** The sign-in account's storage authority: the record belongs to it alone. */
  readonly user: string;
  /** The active linked mailbox, or null for the sign-in account. */
  readonly active: string | null;
  readonly views: Readonly<Record<string, MailboxView>>;
}
/** What the address bar names. `mailbox` null is the sign-in account. */
export interface UrlTarget {
  readonly mailbox: string | null;
  readonly folder?: string;
  readonly message?: string;
}
/** The mailbox a view belongs to: its account id and storage authority. */
export interface MailboxKey { readonly id: string; readonly storageAuthority?: string }

export const isIdentifier = (value: unknown): value is string =>
  typeof value === 'string' && value.length > 0 && value.length <= 512 &&
  ![...value].some((c) => c.charCodeAt(0) < 32 || c.charCodeAt(0) === 127);
const isAuthority = (value: unknown): value is string => typeof value === 'string' && /^[A-Za-z0-9_-]{16,256}$/.test(value);

function storage(): Storage | null {
  try { return globalThis.sessionStorage ?? null; } catch { return null; }
}

function parseView(value: unknown): MailboxView | null {
  if (!value || typeof value !== 'object') return null;
  const v = value as Record<string, unknown>;
  if (!isAuthority(v.authority)) return null;
  const out: { authority: string; folder?: string; message?: string; anchor?: string; search?: string } = { authority: v.authority };
  if (isIdentifier(v.folder)) out.folder = v.folder;
  if (isIdentifier(v.message)) out.message = v.message;
  if (isIdentifier(v.anchor)) out.anchor = v.anchor;
  if (typeof v.search === 'string' && v.search.trim() !== '' && v.search.length <= MAX_SEARCH) out.search = v.search;
  return out;
}

/** The record for `user`, or null when it is missing, malformed or someone else's. */
export function parseSession(raw: string, user: string): SessionRecord | null {
  if (!isAuthority(user) || raw.length > 65536) return null;
  try {
    const value: unknown = JSON.parse(raw);
    if (!value || typeof value !== 'object') return null;
    const r = value as Record<string, unknown>;
    if (r.version !== 1 || r.user !== user) return null;
    if (r.active !== null && !isIdentifier(r.active)) return null;
    const views: Record<string, MailboxView> = {};
    if (r.views && typeof r.views === 'object') {
      for (const [id, raw] of Object.entries(r.views as Record<string, unknown>).slice(0, MAX_VIEWS)) {
        const view = isIdentifier(id) ? parseView(raw) : null;
        if (view) views[id] = view;
      }
    }
    return { version: 1, user, active: r.active as string | null, views };
  } catch { return null; }
}

/** This tab's record for `user`. A record left by another sign-in is removed. */
export function readSession(user: string | undefined): SessionRecord | null {
  if (!user) return null;
  const store = storage();
  try {
    const raw = store?.getItem(SESSION_RESTORE_KEY);
    if (raw === null || raw === undefined) return null;
    const record = parseSession(raw, user);
    if (!record) store?.removeItem(SESSION_RESTORE_KEY);
    return record;
  } catch { return null; }
}

function write(record: SessionRecord): void {
  try { storage()?.setItem(SESSION_RESTORE_KEY, JSON.stringify(record)); } catch { /* storage denied: nothing is kept */ }
}

/** Record the active mailbox (null: the sign-in account) for `user`. */
export function rememberActiveMailbox(user: string | undefined, linkId: string | null): void {
  if (!isAuthority(user) || (linkId !== null && !isIdentifier(linkId))) return;
  const current = readSession(user);
  write({ version: 1, user, active: linkId, views: current?.views ?? {} });
}

/** Merge `patch` into the view of `mailbox`; `undefined` fields are cleared. */
export function rememberView(user: string | undefined, mailbox: MailboxKey, patch: Omit<MailboxView, 'authority'>): void {
  if (!isAuthority(user) || !isIdentifier(mailbox.id) || !isAuthority(mailbox.storageAuthority)) return;
  const current = readSession(user);
  const view = parseView({ ...patch, authority: mailbox.storageAuthority });
  if (!view) return;
  const views: Record<string, MailboxView> = {};
  // The mailbox just written goes last, so the oldest views are the ones dropped.
  for (const [id, value] of Object.entries(current?.views ?? {})) if (id !== mailbox.id) views[id] = value;
  views[mailbox.id] = view;
  const ids = Object.keys(views);
  for (const id of ids.slice(0, Math.max(0, ids.length - MAX_VIEWS))) delete views[id];
  write({ version: 1, user, active: current?.active ?? null, views });
}

/** The saved view of `mailbox`, when it was saved under the same authority. */
export function savedView(user: string | undefined, mailbox: MailboxKey): MailboxView | null {
  const view = readSession(user)?.views[mailbox.id];
  return view && view.authority === mailbox.storageAuthority ? view : null;
}

/** Forget everything this tab kept: the record and the address-bar target. */
export function forgetSession(): void {
  try { storage()?.removeItem(SESSION_RESTORE_KEY); } catch { /* storage unavailable */ }
  writeUrlTarget(null);
}

// ── the address bar ─────────────────────────────────────────────────────────

/** The target the address bar names, or null when it names none. */
export function readUrlTarget(search: string = typeof location === 'undefined' ? '' : location.search): UrlTarget | null {
  let params: URLSearchParams;
  try { params = new URLSearchParams(search); } catch { return null; }
  if (!URL_KEYS.some((key) => params.has(key))) return null;
  const mailbox = params.get('mailbox');
  const folder = params.get('folder');
  const message = params.get('message');
  return {
    mailbox: isIdentifier(mailbox) ? mailbox : null,
    ...(isIdentifier(folder) ? { folder } : {}),
    ...(isIdentifier(message) ? { message } : {}),
  };
}

/** Put `target` in the address bar (in place: no history entry). Null removes it. */
export function writeUrlTarget(target: UrlTarget | null): void {
  if (typeof location === 'undefined' || typeof history === 'undefined') return;
  try {
    const params = new URLSearchParams(location.search);
    for (const key of URL_KEYS) params.delete(key);
    if (target) {
      if (target.mailbox) params.set('mailbox', target.mailbox);
      if (target.folder) params.set('folder', target.folder);
      if (target.message) params.set('message', target.message);
    }
    const query = params.toString();
    const next = `${location.pathname}${query ? `?${query}` : ''}${location.hash}`;
    if (next !== `${location.pathname}${location.search}${location.hash}`) history.replaceState(history.state, '', next);
  } catch { /* a sandboxed frame may refuse replaceState */ }
}

/**
 * The mailbox to open after a reload for `user`: the address bar's, else this
 * tab's record. `undefined` when neither names one (the normal first-mailbox
 * choice applies).
 */
export function restoreMailbox(user: string | undefined, url: UrlTarget | null = readUrlTarget()): { linkId: string | null; source: 'url' | 'session' } | undefined {
  if (url) return { linkId: url.mailbox, source: 'url' };
  const record = readSession(user);
  return record ? { linkId: record.active, source: 'session' } : undefined;
}

/**
 * What to restore in `mailbox` (the active one, `linkId` its link or null):
 * the session's view, with the address bar's folder and message on top when
 * the address bar names this mailbox.
 */
export function restoreView(user: string | undefined, mailbox: MailboxKey, linkId: string | null, url: UrlTarget | null = readUrlTarget()): Omit<MailboxView, 'authority'> | null {
  const saved = savedView(user, mailbox);
  const fromUrl = url && url.mailbox === linkId ? url : null;
  if (!saved && !fromUrl) return null;
  const view = { ...(saved ?? {}) } as { folder?: string; message?: string; anchor?: string; search?: string; authority?: string };
  delete view.authority;
  if (fromUrl?.folder && fromUrl.folder !== view.folder) {
    // Another folder than the session's: its anchor and search were for that one.
    view.folder = fromUrl.folder; delete view.anchor; delete view.search;
  }
  // The address bar is the latest word on the message: none there is none.
  if (fromUrl) { if (fromUrl.message) view.message = fromUrl.message; else delete view.message; }
  return view;
}
