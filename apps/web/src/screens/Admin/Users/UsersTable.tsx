// Admin › Users list (t107). The server searches, filters, sorts and pages
// (`GET /admin/users/table`); the view lives in the URL fragment
// (`#/users?q=…&status=blocked&sort=lastSignin&dir=desc`), so a view is
// shareable and survives a reload. Column visibility, density and saved views
// are the admin's own preferences (`/admin/me/preferences`, key `usersTable`).
//
// A reload keeps the rows on screen (t75): only the first load shows skeleton
// rows, in place, inside the table frame. Rows form one Tab stop: Up/Down/Home/
// End move between them, Space selects, Enter opens the user's page. At phone
// width the table becomes stacked cards with the key fields.

import { batch, createEffect, createMemo, createSignal, For, on, onCleanup, onMount, Show, useContext, type JSX } from 'solid-js';
import { t, formatDate, formatDateTime, formatNumber, formatRelative, formatBytes } from '../../../i18n/index.ts';
import { Icon, type IconName } from '../../../components/Icon.tsx';
import { createContextMenu, type ContextAction } from '../../../components/ContextMenu.tsx';
import { ShellContext } from '../shell/context.ts';
import { hrefFor, sectionPath } from '../shell/router.ts';
import { ConfirmDialog } from '../shell/ConfirmDialog.tsx';
import { Feedback, describeAdminError } from '../shell/Feedback.tsx';
import { useOptionalPersonalPreferences } from '../shell/personalPreferences.tsx';
import { srOnly } from '../shell/shell.css.ts';
import * as ux from '../shell/ux.css.ts';
import { UserFilterBar, type Preset } from './UserFilterBar.tsx';
import {
  PAGE_SIZES, activeFilterCount, assignProfile, defaultView, deleteAccess, loadUserTable, setLoginBlocked, signOutUser,
  viewFromQuery, viewToQuery, type SortKey, type UserFilters, type UserRow, type UserTablePage, type UserView,
} from '../../../api/admin/usersTable.ts';
import * as css from './usersTable.css.ts';

export type ColumnId = 'user' | 'domain' | 'status' | 'twoFactor' | 'role' | 'lastSeen' | 'lastSignin' | 'created' | 'quota' | 'profile' | 'linked';
interface ColumnDef { id: ColumnId; sort: SortKey; label: string }
const COLUMNS: readonly ColumnDef[] = [
  { id: 'user', sort: 'name', label: 'admin-userlist-col-user' },
  { id: 'domain', sort: 'domain', label: 'admin-userlist-col-domain' },
  { id: 'status', sort: 'status', label: 'admin-userlist-col-status' },
  { id: 'twoFactor', sort: 'twoFactor', label: 'admin-userlist-col-2fa' },
  { id: 'role', sort: 'role', label: 'admin-userlist-col-role' },
  { id: 'lastSeen', sort: 'lastSeen', label: 'admin-userlist-col-last-seen' },
  { id: 'lastSignin', sort: 'lastSignin', label: 'admin-userlist-col-last-signin' },
  { id: 'created', sort: 'created', label: 'admin-userlist-col-created' },
  { id: 'quota', sort: 'quota', label: 'admin-userlist-col-quota' },
  { id: 'profile', sort: 'profile', label: 'admin-userlist-col-profile' },
  { id: 'linked', sort: 'linked', label: 'admin-userlist-col-linked' },
];
/** Sorts that read best newest first. */
const DESC_FIRST: ReadonlySet<SortKey> = new Set(['lastSeen', 'lastSignin', 'created', 'linked', 'quota']);
const DEFAULT_HIDDEN: readonly ColumnId[] = ['quota'];
const STATE_ICON: Record<UserRow['state'], IconName> = { active: 'check', pending: 'clock', blocked: 'lock', disabled: 'lock', deleted: 'delete' };
const FACTOR_ICON: Record<UserRow['twoFactor'], IconName> = { passkey: 'key', totp: 'twofactor', none: 'minus' };
const PHONE = '(max-width: 640px)';
const MAX_PRESETS = 6;

export interface TablePrefs { hidden: ColumnId[]; density: 'comfortable' | 'compact'; presets: Preset[] }
const DEFAULT_PREFS: TablePrefs = { hidden: [...DEFAULT_HIDDEN], density: 'comfortable', presets: [] };

export function parsePrefs(value: unknown): TablePrefs | null {
  if (typeof value !== 'object' || value === null) return null;
  const v = value as Record<string, unknown>;
  const ids = COLUMNS.map((c) => c.id) as string[];
  const hidden = Array.isArray(v['hidden']) ? (v['hidden'] as unknown[]).filter((x): x is ColumnId => typeof x === 'string' && ids.includes(x) && x !== 'user') : [...DEFAULT_HIDDEN];
  const density = v['density'] === 'compact' ? 'compact' : 'comfortable';
  const presets = Array.isArray(v['presets'])
    ? (v['presets'] as unknown[]).flatMap((p): Preset[] => {
      if (typeof p !== 'object' || p === null) return [];
      const { name, query } = p as { name?: unknown; query?: unknown };
      if (typeof name !== 'string' || typeof query !== 'object' || query === null) return [];
      const entries = Object.entries(query as Record<string, unknown>).filter((e): e is [string, string] => typeof e[1] === 'string');
      return [{ name: name.slice(0, 40), query: Object.fromEntries(entries) }];
    }).slice(0, MAX_PRESETS)
    : [];
  return { hidden, density, presets };
}

function storageKey(user: string): string { return `mw-admin-userlist-table:${user}`; }
function readStored(user: string): unknown {
  try { const raw = localStorage.getItem(storageKey(user)); return raw ? JSON.parse(raw) : null; } catch { return null; }
}
function writeStored(user: string, prefs: TablePrefs): void {
  try { localStorage.setItem(storageKey(user), JSON.stringify(prefs)); } catch { /* storage unavailable: the server copy still holds it */ }
}

/** Initials for the avatar: two words of the name, or the address's local part. */
export function initials(row: Pick<UserRow, 'displayName' | 'username'>): string {
  const source = (row.displayName ?? row.username).trim();
  const words = source.split(/[\s._-]+/u).filter((w) => w !== '');
  const letters = words.length > 1 ? [words[0]!, words[1]!].map((w) => [...w][0] ?? '') : [...(words[0] ?? '?')].slice(0, 2);
  return letters.join('').toLocaleUpperCase();
}

/** `text` with every case-insensitive occurrence of `needle` wrapped in <mark>. */
export function Highlight(props: { text: string; needle: string }): JSX.Element {
  const parts = createMemo(() => {
    const needle = props.needle.trim().toLocaleLowerCase();
    if (needle === '') return [{ text: props.text, hit: false }];
    const lower = props.text.toLocaleLowerCase();
    const out: { text: string; hit: boolean }[] = [];
    let at = 0;
    for (let i = lower.indexOf(needle); i !== -1; i = lower.indexOf(needle, at)) {
      if (i > at) out.push({ text: props.text.slice(at, i), hit: false });
      out.push({ text: props.text.slice(i, i + needle.length), hit: true });
      at = i + needle.length;
    }
    if (at < props.text.length) out.push({ text: props.text.slice(at), hit: false });
    return out;
  });
  return <For each={parts()}>{(p) => (p.hit ? <mark class={css.mark}>{p.text}</mark> : p.text)}</For>;
}

export interface UsersTableActions {
  load: (view: UserView, signal?: AbortSignal) => Promise<UserTablePage>;
  setLoginBlocked: (accountId: string, blocked: boolean) => Promise<void>;
  signOut: (accountId: string) => Promise<void>;
  deleteAccess: (accountId: string) => Promise<void>;
  assignProfile: (accountId: string, profileId: string | null, current: string | null) => Promise<void>;
}
const HTTP_ACTIONS: UsersTableActions = { load: loadUserTable, setLoginBlocked, signOut: signOutUser, deleteAccess, assignProfile };

type BulkAction = 'block' | 'unblock' | 'signout' | 'delete' | { profile: string | null };
interface Pending { action: BulkAction; rows: UserRow[] }

export function UsersTable(props: { actions?: Partial<UsersTableActions> } = {}): JSX.Element {
  const actions: UsersTableActions = { ...HTTP_ACTIONS, ...props.actions };
  const shell = useContext(ShellContext);
  const prefsStore = useOptionalPersonalPreferences();
  const can = (cap: Parameters<NonNullable<typeof shell>['can']['can']>[0]): boolean => shell?.can.can(cap) ?? true;
  const admin = (): string => shell?.username() ?? '';

  // ── The view: the URL when the console shell is there, a signal otherwise ──
  const [localView, setLocalView] = createSignal<UserView>(defaultView());
  const view = createMemo<UserView>(() => (shell ? viewFromQuery(shell.router.route().query) : localView()));
  function setView(next: UserView): void {
    if (!shell) { setLocalView(next); return; }
    void shell.router.navigate(sectionPath('users', [], viewToQuery(next)), { replace: true });
  }
  const setFilters = (filters: UserFilters): void => setView({ ...view(), filters, page: 0 });

  // ── Preferences (per admin) ──────────────────────────────────────────────
  const [localPrefs, setLocalPrefs] = createSignal<TablePrefs | null>(null);
  const prefs = createMemo<TablePrefs>(() => localPrefs()
    ?? parsePrefs(prefsStore?.data()['usersTable'])
    ?? parsePrefs(readStored(admin()))
    ?? DEFAULT_PREFS);
  function savePrefs(next: TablePrefs): void {
    setLocalPrefs(next);
    writeStored(admin(), next);
    prefsStore?.patch({ usersTable: next });
  }
  const visible = (id: ColumnId): boolean => id === 'user' || !prefs().hidden.includes(id);
  const columns = createMemo(() => COLUMNS.filter((c) => visible(c.id)));

  // ── Data ────────────────────────────────────────────────────────────────
  const [page, setPage] = createSignal<UserTablePage | null>(null);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [failure, setFailure] = createSignal<string | null>(null);
  const [notice, setNotice] = createSignal<string | null>(null);
  let generation = 0;
  let inFlight: AbortController | undefined;
  async function reload(): Promise<void> {
    const mine = ++generation;
    inFlight?.abort();
    const controller = new AbortController();
    inFlight = controller;
    setLoading(true);
    try {
      const result = await actions.load(view(), controller.signal);
      if (mine !== generation) return;
      batch(() => {
        setPage(result);
        setError(null);
        // Keep only the selected rows still shown.
        const shown = new Set(result.rows.map((r) => r.accountId));
        setSelected((now) => new Set([...now].filter((id) => shown.has(id))));
      });
    } catch (cause) {
      if (mine !== generation || controller.signal.aborted) return;
      setError(cause instanceof Error && cause.name === 'AbortError' ? null : t('admin-users-load-error'));
    } finally {
      if (mine === generation) setLoading(false);
    }
  }
  createEffect(on(() => JSON.stringify(view()), () => void reload()));
  onCleanup(() => { generation++; inFlight?.abort(); });

  // A minute clock for the relative times.
  const [now, setNow] = createSignal(Date.now());
  const clock = setInterval(() => setNow(Date.now()), 60_000);
  onCleanup(() => clearInterval(clock));

  // Phone width: cards instead of the table.
  const [phone, setPhone] = createSignal(false);
  onMount(() => {
    if (typeof window.matchMedia !== 'function') return;
    const query = window.matchMedia(PHONE);
    setPhone(query.matches);
    const change = (e: MediaQueryListEvent): void => { setPhone(e.matches); };
    query.addEventListener('change', change);
    onCleanup(() => query.removeEventListener('change', change));
  });

  // ── Search (debounced) ───────────────────────────────────────────────────
  const [text, setText] = createSignal(view().filters.q);
  let searchBox: HTMLInputElement | undefined;
  createEffect(on(() => view().filters.q, (q) => { if (document.activeElement !== searchBox) setText(q); }, { defer: true }));
  let timer: ReturnType<typeof setTimeout> | undefined;
  onCleanup(() => clearTimeout(timer));
  function onSearch(value: string): void {
    setText(value);
    clearTimeout(timer);
    timer = setTimeout(() => setFilters({ ...view().filters, q: value }), 250);
  }

  // ── Sort ────────────────────────────────────────────────────────────────
  function sortBy(key: SortKey): void {
    const v = view();
    const dir = v.sort === key ? (v.dir === 'asc' ? 'desc' : 'asc') : DESC_FIRST.has(key) ? 'desc' : 'asc';
    setView({ ...v, sort: key, dir, page: 0 });
  }
  const ariaSort = (key: SortKey): 'ascending' | 'descending' | 'none' => (view().sort !== key ? 'none' : view().dir === 'asc' ? 'ascending' : 'descending');

  // ── Selection ───────────────────────────────────────────────────────────
  const [selected, setSelected] = createSignal<Set<string>>(new Set());
  const rows = (): UserRow[] => page()?.rows ?? [];
  const selectedRows = (): UserRow[] => rows().filter((r) => selected().has(r.accountId));
  const toggle = (id: string, on?: boolean): void => {
    setSelected((now) => {
      const next = new Set(now);
      if (on ?? !next.has(id)) next.add(id); else next.delete(id);
      return next;
    });
  };
  const allSelected = (): boolean => rows().length > 0 && rows().every((r) => selected().has(r.accountId));
  const someSelected = (): boolean => selected().size > 0 && !allSelected();

  // ── Actions ─────────────────────────────────────────────────────────────
  const [pending, setPending] = createSignal<Pending | null>(null);
  const eligible = (action: BulkAction, list: UserRow[]): UserRow[] => list.filter((r) => {
    if (r.state === 'deleted') return false;
    if (action === 'block') return r.status !== 'disabled';
    if (action === 'unblock') return r.status === 'disabled';
    return true;
  });
  const actionWord = (action: BulkAction): string => typeof action === 'object' ? 'profile' : action;
  async function run(change: Pending): Promise<void> {
    const targets = eligible(change.action, change.rows);
    let failed = 0;
    let lastError: unknown = null;
    for (const row of targets) {
      try {
        const a = change.action;
        if (a === 'block' || a === 'unblock') await actions.setLoginBlocked(row.accountId, a === 'block');
        else if (a === 'signout') await actions.signOut(row.accountId);
        else if (a === 'delete') await actions.deleteAccess(row.accountId);
        else await actions.assignProfile(row.accountId, a.profile, row.profile?.via === 'user' ? row.profile.id : null);
      } catch (cause) {
        failed++;
        lastError = cause;
      }
    }
    setPending(null);
    setSelected(new Set<string>());
    await reload();
    const done = targets.length - failed;
    if (failed > 0) {
      setFailure(null);
      setFailure(t('admin-userlist-bulk-failed', { done, failed, reason: describeAdminError(lastError) }));
    } else {
      setNotice(null);
      setNotice(t(`admin-userlist-bulk-done-${actionWord(change.action)}`, { count: done }));
    }
  }

  const rowMenuTarget = createSignal<UserRow | null>(null);
  const [menuRow, setMenuRow] = rowMenuTarget;
  const rowMenu = createContextMenu({
    label: () => t('admin-userlist-more-for', { account: menuRow()?.loginAddress ?? '' }),
    scope: () => `users-row:${menuRow()?.accountId ?? ''}`,
    actions: (): ContextAction[] => {
      const row = menuRow();
      if (!row) return [];
      const gone = row.state === 'deleted';
      const blocked = row.status === 'disabled';
      return [
        { id: 'block', label: t(blocked ? 'admin-userlist-unblock' : 'admin-userlist-block'), icon: blocked ? 'unlock' : 'lock', disabled: gone || row.flags.disabled || !can('users.flags'), run: () => { setPending({ action: blocked ? 'unblock' : 'block', rows: [row] }); } },
        { id: 'signout', label: t('admin-userlist-sign-out'), icon: 'signOut', disabled: gone || !can('users.revoke_sessions'), run: () => { setPending({ action: 'signout', rows: [row] }); } },
        { id: 'delete', label: t('admin-userlist-delete'), icon: 'delete', separatorBefore: true, disabled: gone || !can('users.delete'), run: () => { setPending({ action: 'delete', rows: [row] }); } },
      ];
    },
  });
  const profileMenu = createContextMenu({
    label: () => t('admin-userlist-bulk-profile'),
    scope: () => 'users-bulk-profile',
    actions: (): ContextAction[] => [
      ...(page()?.facets.profiles ?? []).map((p): ContextAction => ({ id: p.id, label: p.name, icon: 'policy', run: () => { setPending({ action: { profile: p.id }, rows: selectedRows() }); } })),
      { id: '__none', label: t('admin-userlist-bulk-profile-clear'), icon: 'close', separatorBefore: true, run: () => { setPending({ action: { profile: null }, rows: selectedRows() }); } },
    ],
  });

  // ── Columns menu ────────────────────────────────────────────────────────
  const [columnsOpen, setColumnsOpen] = createSignal(false);
  let columnsButton: HTMLButtonElement | undefined;
  let columnsPanel: HTMLDivElement | undefined;
  createEffect(() => {
    if (!columnsOpen()) return;
    const outside = (e: PointerEvent): void => {
      if (e.target instanceof Node && !columnsPanel?.contains(e.target) && !columnsButton?.contains(e.target)) setColumnsOpen(false);
    };
    document.addEventListener('pointerdown', outside, true);
    onCleanup(() => document.removeEventListener('pointerdown', outside, true));
  });
  function setColumn(id: ColumnId, show: boolean): void {
    const hidden = prefs().hidden.filter((h) => h !== id);
    if (!show) hidden.push(id);
    savePrefs({ ...prefs(), hidden });
  }

  // ── Presets ─────────────────────────────────────────────────────────────
  const savePreset = (name: string): void => {
    const query = viewToQuery({ ...view(), page: 0 });
    const presets = [...prefs().presets.filter((p) => p.name !== name), { name, query }].slice(-MAX_PRESETS);
    savePrefs({ ...prefs(), presets });
  };
  const deletePreset = (name: string): void => savePrefs({ ...prefs(), presets: prefs().presets.filter((p) => p.name !== name) });
  const applyPreset = (preset: Preset): void => { const next = viewFromQuery(preset.query); setText(next.filters.q); setView(next); };

  // ── Keyboard row navigation ─────────────────────────────────────────────
  const [activeRow, setActiveRow] = createSignal(0);
  let tbody: HTMLTableSectionElement | undefined;
  function onRowKey(e: KeyboardEvent, index: number, row: UserRow): void {
    if (e.target !== e.currentTarget) return;
    const last = rows().length - 1;
    let next = index;
    if (e.key === 'ArrowDown') next = Math.min(last, index + 1);
    else if (e.key === 'ArrowUp') next = Math.max(0, index - 1);
    else if (e.key === 'Home') next = 0;
    else if (e.key === 'End') next = last;
    else if (e.key === ' ') { e.preventDefault(); toggle(row.accountId); return; }
    else if (e.key === 'Enter') { e.preventDefault(); void shell?.router.navigate(sectionPath('users', [row.accountId])); return; }
    else return;
    e.preventDefault();
    setActiveRow(next);
    tbody?.querySelectorAll<HTMLElement>(':scope > tr[data-row]')[next]?.focus();
  }

  // ── Cells ───────────────────────────────────────────────────────────────
  const needle = (): string => view().filters.q;
  const configureHref = (row: UserRow): string => hrefFor(sectionPath('users', [row.accountId]));
  const Person = (p: { row: UserRow }): JSX.Element => (
    <div class={css.person}>
      <span class={css.avatar} aria-hidden="true">{initials(p.row)}</span>
      <span class={css.personText}>
        <a class={css.personName} href={configureHref(p.row)} dir="auto"><Highlight text={p.row.displayName ?? p.row.loginAddress} needle={needle()} /></a>
        <Show when={p.row.displayName}><span class={css.personAddress} dir="auto"><Highlight text={p.row.loginAddress} needle={needle()} /></span></Show>
      </span>
    </div>
  );
  const Status = (p: { row: UserRow }): JSX.Element => (
    <span class={css.status} data-state={p.row.state}><Icon name={STATE_ICON[p.row.state]} />{t(`admin-userlist-state-${p.row.state}`)}</span>
  );
  const LastSignin = (p: { row: UserRow }): JSX.Element => (
    <Show when={p.row.lastSigninAt} fallback={<span class={css.dim}>{t('admin-userlist-signin-never')}</span>}>
      {(at) => {
        const ms = (): number => at() * 1000;
        const exact = (): string => formatDateTime(ms(), { dateStyle: 'full', timeStyle: 'long' });
        return <time class={css.nowrap} datetime={new Date(ms()).toISOString()} title={exact()} aria-label={t('admin-userlist-signin-at', { relative: formatRelative(ms(), now()), exact: exact() })}>{formatRelative(ms(), now())}</time>;
      }}
    </Show>
  );
  const LastSeen = (p: { row: UserRow }): JSX.Element => (
    <Show when={p.row.lastSeenAt} fallback={<span class={css.dim}>{t('admin-userlist-seen-never')}</span>}>
      {(at) => {
        const ms = (): number => at() * 1000;
        const exact = (): string => formatDateTime(ms(), { dateStyle: 'full', timeStyle: 'long' });
        return <time class={css.nowrap} datetime={new Date(ms()).toISOString()} title={exact()} aria-label={t('admin-userlist-seen-at', { relative: formatRelative(ms(), now()), exact: exact() })}>{formatRelative(ms(), now())}</time>;
      }}
    </Show>
  );
  const cell = (id: ColumnId, row: UserRow): JSX.Element => {
    switch (id) {
      case 'user': return <Person row={row} />;
      case 'domain': return <span dir="auto"><Highlight text={row.domain || '—'} needle={needle()} /></span>;
      case 'status': return <Status row={row} />;
      case 'twoFactor': return (
        <span class={css.badge} data-factor={row.twoFactor}>
          <Icon name={FACTOR_ICON[row.twoFactor]} />
          {row.twoFactorMethods.length > 0 ? row.twoFactorMethods.map((m) => t(`admin-userlist-2fa-${m}`)).join(' + ') : t('admin-userlist-2fa-none')}
        </span>
      );
      case 'role': return <Show when={row.adminRole} fallback={<span class={css.dim}>—</span>}>{(role) => <span class={css.badge}><Icon name="admins" />{role()}</span>}</Show>;
      case 'lastSignin': return <LastSignin row={row} />;
      case 'lastSeen': return <LastSeen row={row} />;
      case 'created': return <Show when={row.createdAt} fallback={<span class={css.dim}>—</span>}>{(at) => <time class={css.nowrap} datetime={new Date(at() * 1000).toISOString()} title={formatDateTime(at() * 1000)}>{formatDate(at() * 1000)}</time>}</Show>;
      case 'quota': return <Show when={row.quota} fallback={<span class={css.dim}>—</span>}>{(q) => <span class={css.nowrap}>{q().bytesLimit > 0 ? formatBytes(q().bytesLimit) : t('admin-userlist-quota-unlimited')}</span>}</Show>;
      case 'profile': return (
        <Show when={row.profile} fallback={<span class={css.dim}>{t('admin-userlist-profile-default')}</span>}>
          {(p) => <span class={css.badge} title={t(`admin-userlist-profile-via-${p().via}`)}><Icon name="policy" />{p().name}<Show when={p().via !== 'user'}><span class={srOnly}> ({t(`admin-userlist-profile-via-${p().via}`)})</span></Show></span>}
        </Show>
      );
      case 'linked': return <span class={row.linkedMailboxes === 0 ? css.dim : undefined}>{formatNumber(row.linkedMailboxes)}</span>;
    }
  };
  const RowActions = (p: { row: UserRow }): JSX.Element => (
    <span class={css.actions}>
      <a class={`btn ${ux.iconAction}`} href={configureHref(p.row)} aria-label={t('admin-users-configure-for', { account: p.row.loginAddress })} data-tooltip={t('admin-users-configure')}><Icon name="settings" /></a>
      <button
        type="button" class={ux.iconAction} aria-haspopup="menu" aria-label={t('admin-userlist-more-for', { account: p.row.loginAddress })}
        data-tooltip={t('admin-userlist-more')} onClick={(e) => { setMenuRow(p.row); rowMenu.open(e); }}
      ><Icon name="more" /></button>
    </span>
  );

  // ── Counts ──────────────────────────────────────────────────────────────
  const filtered = (): boolean => activeFilterCount(view().filters) > 0 || view().filters.q.trim() !== '';
  const countText = (): string => {
    const p = page();
    if (!p) return '';
    return filtered() ? t('admin-userlist-count-filtered', { count: p.total, total: p.unfilteredTotal }) : t('admin-userlist-count', { count: p.total });
  };
  const range = (): string => {
    const p = page();
    if (!p || p.total === 0) return '';
    return t('admin-userlist-range', { from: p.offset + 1, to: p.offset + p.rows.length, total: p.total });
  };
  const lastPage = (): number => Math.max(0, Math.ceil((page()?.total ?? 0) / view().pageSize) - 1);

  const clearFilters = (): void => { setText(''); setFilters({ ...view().filters, q: '', domain: [], status: [], twoFactor: [], role: [], profile: [], signin: null, createdFrom: null, createdTo: null }); };
  const Empty = (): JSX.Element => (
    <div class={css.empty} data-testid="users-empty">
      <Show when={filtered()} fallback={<><p class={css.dim}>{t('admin-users-empty')}</p><a class="btn btn--primary" href={hrefFor(sectionPath('users', ['+new']))}>{t('admin-users-provision')}</a></>}>
        <p class={css.dim}>{t('admin-userlist-empty-filtered')}</p>
        <button type="button" class="btn" onClick={clearFilters}>{t('admin-userlist-clear-all')}</button>
      </Show>
    </div>
  );

  const confirmTitle = (change: Pending): string => {
    const n = eligible(change.action, change.rows).length;
    if (change.rows.length === 1 && n === 1) return t(`admin-userlist-confirm-${actionWord(change.action)}-one`, { account: change.rows[0]!.loginAddress });
    return t(`admin-userlist-confirm-${actionWord(change.action)}`, { count: n });
  };
  const confirmBody = (change: Pending): string => {
    const n = eligible(change.action, change.rows).length;
    const skipped = change.rows.length - n;
    const base = t(`admin-userlist-confirm-${actionWord(change.action)}-body`);
    return skipped > 0 ? `${base} ${t('admin-userlist-confirm-skipped', { count: skipped })}` : base;
  };

  return (
    <div class={css.root} data-testid="users-table-root">
      <div class={css.toolbar}>
        <form role="search" class={css.search} onSubmit={(e) => { e.preventDefault(); clearTimeout(timer); setFilters({ ...view().filters, q: text() }); }}>
          <label for="users-search">{t('admin-userlist-search')}</label>
          <input
            ref={searchBox} id="users-search" class={css.searchInput} type="search" value={text()} autocomplete="off" spellcheck={false}
            placeholder={t('admin-userlist-search-placeholder')} aria-describedby="users-count" onInput={(e) => onSearch(e.currentTarget.value)}
          />
        </form>
        <div class={css.toolbarEnd}>
          <p class={css.count} id="users-count" role="status" aria-live="polite" data-testid="users-count">{countText()}</p>
          <button
            type="button" class="btn btn--ghost" aria-pressed={prefs().density === 'compact'}
            onClick={() => savePrefs({ ...prefs(), density: prefs().density === 'compact' ? 'comfortable' : 'compact' })}
          >{t('admin-userlist-compact')}</button>
          <span class={css.anchor}>
            <button
              ref={columnsButton} type="button" class="btn btn--ghost" aria-expanded={columnsOpen()} aria-controls="users-columns"
              onClick={() => setColumnsOpen(!columnsOpen())}
            >{t('admin-userlist-columns')}</button>
            <Show when={columnsOpen()}>
              <div
                ref={columnsPanel} id="users-columns" class={`${css.popover} ${css.popoverEnd}`} role="group" aria-label={t('admin-userlist-columns')}
                onKeyDown={(e) => { if (e.key === 'Escape') { e.preventDefault(); setColumnsOpen(false); columnsButton?.focus(); } }}
              >
                <fieldset class={css.options}>
                  <legend class={css.popoverTitle}>{t('admin-userlist-columns-legend')}</legend>
                  <For each={COLUMNS.filter((c) => c.id !== 'user')}>
                    {(c) => (
                      <label class={css.option}>
                        <input type="checkbox" checked={visible(c.id)} onChange={(e) => setColumn(c.id, e.currentTarget.checked)} />
                        {t(c.label)}
                      </label>
                    )}
                  </For>
                </fieldset>
              </div>
            </Show>
          </span>
        </div>
      </div>

      <UserFilterBar
        filters={view().filters} facets={page()?.facets ?? null} onChange={(f) => { setText(f.q); setFilters(f); }}
        presets={prefs().presets} onApplyPreset={applyPreset} onSavePreset={savePreset} onDeletePreset={deletePreset}
      />

      <Show when={selected().size > 0}>
        <div class={css.bulk} role="region" aria-label={t('admin-userlist-bulk')} data-testid="users-bulk">
          <span class={css.bulkCount}>{t('admin-userlist-selected', { count: selected().size })}</span>
          <Show when={can('users.flags')}>
            <button type="button" class="btn" onClick={() => setPending({ action: 'block', rows: selectedRows() })}><Icon name="lock" />{t('admin-userlist-block')}</button>
            <button type="button" class="btn" onClick={() => setPending({ action: 'unblock', rows: selectedRows() })}><Icon name="unlock" />{t('admin-userlist-unblock')}</button>
          </Show>
          <Show when={can('users.revoke_sessions')}>
            <button type="button" class="btn" onClick={() => setPending({ action: 'signout', rows: selectedRows() })}><Icon name="signOut" />{t('admin-userlist-sign-out')}</button>
          </Show>
          <Show when={can('settings.user_values') && (page()?.facets.profiles.length ?? 0) > 0}>
            <button type="button" class="btn" aria-haspopup="menu" onClick={(e) => profileMenu.open(e)}><Icon name="policy" />{t('admin-userlist-bulk-profile')}</button>
          </Show>
          <button type="button" class="btn btn--ghost" onClick={() => setSelected(new Set<string>())}>{t('admin-userlist-clear-selection')}</button>
        </div>
      </Show>

      <Show when={error()}>
        <div class={css.empty}>
          <p class={css.dim} role="alert">{error()}</p>
          <button type="button" class="btn" onClick={() => void reload()}>{t('admin-userlist-retry')}</button>
        </div>
      </Show>
      <Feedback kind="error" as="toast" message={failure()} />
      <Feedback kind="success" as="toast" message={notice()} />

      <Show when={!error() || page()}>
        <Show when={!phone()} fallback={
          <div aria-busy={loading()}>
            <label class={css.pageSize}>{t('admin-userlist-sort-by')}
              <select value={`${view().sort}:${view().dir}`} onChange={(e) => { const [sort, dir] = e.currentTarget.value.split(':') as [SortKey, 'asc' | 'desc']; setView({ ...view(), sort, dir, page: 0 }); }}>
                <For each={COLUMNS}>{(c) => <>
                  <option value={`${c.sort}:asc`}>{t('admin-userlist-sort-asc', { column: t(c.label) })}</option>
                  <option value={`${c.sort}:desc`}>{t('admin-userlist-sort-desc', { column: t(c.label) })}</option>
                </>}</For>
              </select>
            </label>
            <Show when={page() || !loading()} fallback={<ul class={css.cards} aria-hidden="true"><For each={[0, 1, 2, 3]}>{() => <li class={css.card}><span class={`${css.skeletonCell} ${css.skeletonShort}`} /><span class={`${css.skeletonCell} ${css.skeletonLong}`} /><span /></li>}</For></ul>}>
              <Show when={rows().length > 0} fallback={<Empty />}>
                <ul class={css.cards} aria-label={t('admin-users-title')} data-testid="users-cards">
                  <For each={rows()}>
                    {(row) => (
                      <li class={css.card} data-selected={selected().has(row.accountId)} data-account={row.accountId}>
                        <input type="checkbox" checked={selected().has(row.accountId)} aria-label={t('admin-userlist-select', { account: row.loginAddress })} onChange={(e) => toggle(row.accountId, e.currentTarget.checked)} />
                        <div class={css.cardBody}>
                          <Person row={row} />
                          <div class={css.cardMeta}><Status row={row} /><span>{t('admin-userlist-col-last-seen')}: <LastSeen row={row} /></span><span>{t('admin-userlist-col-last-signin')}: <LastSignin row={row} /></span></div>
                        </div>
                        <RowActions row={row} />
                      </li>
                    )}
                  </For>
                </ul>
              </Show>
            </Show>
          </div>
        }>
          <div class={css.scroller}>
            <table class={`${css.table} ${prefs().density === 'compact' ? css.compact : ''}`} aria-busy={loading()} aria-rowcount={page() ? page()!.total + 1 : -1} data-testid="users-table">
              <caption class={srOnly}>{t('admin-userlist-caption')}</caption>
              <thead>
                <tr>
                  <th scope="col" class={css.selectCell}>
                    <input
                      type="checkbox" checked={allSelected()} ref={(el) => createEffect(() => { el.indeterminate = someSelected(); })}
                      disabled={rows().length === 0} aria-label={t('admin-userlist-select-page')}
                      onChange={(e) => { const on = e.currentTarget.checked; setSelected(on ? new Set(rows().map((r) => r.accountId)) : new Set<string>()); }}
                    />
                  </th>
                  <For each={columns()}>
                    {(c) => (
                      <th scope="col" aria-sort={ariaSort(c.sort)} data-column={c.id}>
                        <button type="button" class={css.sort} onClick={() => sortBy(c.sort)}>
                          {t(c.label)}
                          <span aria-hidden="true"><Icon name={ariaSort(c.sort) === 'ascending' ? 'chevronUp' : ariaSort(c.sort) === 'descending' ? 'chevronDown' : 'sort'} /></span>
                        </button>
                      </th>
                    )}
                  </For>
                  <th scope="col" class={css.actionsCell}><span class={srOnly}>{t('admin-userlist-actions')}</span></th>
                </tr>
              </thead>
              <tbody ref={tbody}>
                <Show when={page() || !loading()} fallback={
                  <For each={[0, 1, 2, 3, 4, 5]}>
                    {(i) => (
                      <tr aria-hidden="true" data-skeleton>
                        <td class={css.selectCell}><span class={`${css.skeletonCell} ${css.skeletonShort}`} /></td>
                        <For each={columns()}>{(_c, j) => <td><span class={`${css.skeletonCell} ${[css.skeletonLong, css.skeletonMid, css.skeletonShort][(i + j()) % 3]}`} /></td>}</For>
                        <td class={css.actionsCell} />
                      </tr>
                    )}
                  </For>
                }>
                  <Show when={rows().length > 0} fallback={<tr><td colSpan={columns().length + 2}><Empty /></td></tr>}>
                    <For each={rows()}>
                      {(row, i) => (
                        <tr
                          class={css.row} data-row data-account={row.accountId} data-selected={selected().has(row.accountId)}
                          aria-rowindex={(page()?.offset ?? 0) + i() + 2}
                          tabindex={i() === Math.min(activeRow(), rows().length - 1) ? 0 : -1}
                          onFocus={(e) => { if (e.target === e.currentTarget) setActiveRow(i()); }}
                          onKeyDown={(e) => onRowKey(e, i(), row)}
                        >
                          <td class={css.selectCell}>
                            <input type="checkbox" checked={selected().has(row.accountId)} aria-label={t('admin-userlist-select', { account: row.loginAddress })} onChange={(e) => toggle(row.accountId, e.currentTarget.checked)} />
                          </td>
                          <For each={columns()}>{(c) => <td data-column={c.id}>{cell(c.id, row)}</td>}</For>
                          <td class={css.actionsCell}><RowActions row={row} /></td>
                        </tr>
                      )}
                    </For>
                  </Show>
                </Show>
              </tbody>
            </table>
          </div>
        </Show>
      </Show>
      <span class={srOnly} role="status">{loading() && !page() ? t('admin-userlist-loading') : ''}</span>

      <Show when={(page()?.total ?? 0) > 0}>
        <nav class={css.pager} aria-label={t('admin-userlist-paging')}>
          <span class={css.count}>{range()}</span>
          <div class={css.pagerButtons}>
            <label class={css.pageSize}>{t('admin-userlist-page-size')}
              <select value={String(view().pageSize)} onChange={(e) => setView({ ...view(), pageSize: Number(e.currentTarget.value), page: 0 })}>
                <For each={PAGE_SIZES}>{(n) => <option value={String(n)}>{n}</option>}</For>
              </select>
            </label>
            <button type="button" class="btn" disabled={view().page === 0} onClick={() => setView({ ...view(), page: view().page - 1 })}>{t('admin-userlist-prev')}</button>
            <button type="button" class="btn" disabled={view().page >= lastPage()} onClick={() => setView({ ...view(), page: view().page + 1 })}>{t('admin-userlist-next')}</button>
          </div>
        </nav>
      </Show>

      {rowMenu.menu()}
      {profileMenu.menu()}
      <Show when={pending()}>
        {(change) => (
          <ConfirmDialog
            open title={confirmTitle(change())} body={confirmBody(change())}
            confirmLabel={t(`admin-userlist-confirm-${actionWord(change().action)}-action`)}
            danger={change().action !== 'unblock' && typeof change().action !== 'object'}
            {...(change().action === 'delete' && change().rows.length === 1 ? { typedConfirmation: change().rows[0]!.directoryAddress } : {})}
            onConfirm={() => run(change())} onCancel={() => setPending(null)}
          />
        )}
      </Show>
    </div>
  );
}
