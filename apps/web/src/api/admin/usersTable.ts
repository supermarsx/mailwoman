// `GET /admin/users/table` client (t107): the admin Users table. The server
// searches, filters, sorts and pages (crates/mw-server/src/admin/users_table.rs);
// this module carries the view (filters + sort + page) between the URL fragment,
// the request and the screen, so a view is shareable and survives a reload.

import { adminRequest } from '../../screens/Admin/shell/adminApi.ts';
import type { UserFeatureFlags } from '../../state/slices/admin.ts';

export type UserState = 'active' | 'blocked' | 'disabled' | 'pending' | 'deleted';
export type TwoFactor = 'passkey' | 'totp' | 'none';

export interface UserRow {
  accountId: string;
  username: string;
  domain: string;
  loginAddress: string;
  directoryAddress: string;
  legacyBinding: boolean;
  displayName: string | null;
  /** The lifecycle status as stored. */
  status: 'active' | 'disabled' | 'deleting' | 'deleted';
  /** What the table shows. */
  state: UserState;
  deletedAt: number | null;
  lastSigninAt: number | null;
  /** Last authenticated activity, independent of the most recent sign-in. */
  lastSeenAt: number | null;
  createdAt: number | null;
  twoFactor: TwoFactor;
  twoFactorMethods: TwoFactor[];
  adminRole: string | null;
  profile: { id: string; name: string; via: 'user' | 'group' | 'domain' } | null;
  linkedMailboxes: number;
  quota: { bytesLimit: number; msgLimit: number } | null;
  flags: UserFeatureFlags;
}

export interface UserTablePage {
  rows: UserRow[];
  total: number;
  unfilteredTotal: number;
  offset: number;
  limit: number;
  facets: { domains: string[]; roles: string[]; profiles: { id: string; name: string }[] };
}

export const SORT_KEYS = ['name', 'domain', 'status', 'twoFactor', 'role', 'lastSeen', 'lastSignin', 'created', 'quota', 'profile', 'linked'] as const;
export type SortKey = (typeof SORT_KEYS)[number];

export const STATES: readonly UserState[] = ['active', 'pending', 'blocked', 'disabled', 'deleted'];
export const FACTORS: readonly TwoFactor[] = ['passkey', 'totp', 'none'];

/** `never`, `older:<days>` or `within:<days>`. */
export type SigninFilter = string;

export interface UserFilters {
  q: string;
  domain: string[];
  status: UserState[];
  twoFactor: TwoFactor[];
  /** `admin`, `none` or a role name. */
  role: string[];
  /** A profile id or `none`. */
  profile: string[];
  signin: SigninFilter | null;
  /** `YYYY-MM-DD`, inclusive, in the admin's time zone. */
  createdFrom: string | null;
  createdTo: string | null;
}

export interface UserView {
  filters: UserFilters;
  sort: SortKey;
  dir: 'asc' | 'desc';
  /** Zero-based. */
  page: number;
  pageSize: number;
}

export const PAGE_SIZES = [25, 50, 100] as const;

export function emptyFilters(): UserFilters {
  return { q: '', domain: [], status: [], twoFactor: [], role: [], profile: [], signin: null, createdFrom: null, createdTo: null };
}

export function defaultView(): UserView {
  return { filters: emptyFilters(), sort: 'lastSeen', dir: 'desc', page: 0, pageSize: 50 };
}

/** How many filters are set (the search text not counted). */
export function activeFilterCount(f: UserFilters): number {
  return [f.domain, f.status, f.twoFactor, f.role, f.profile].filter((v) => v.length > 0).length
    + (f.signin ? 1 : 0) + (f.createdFrom || f.createdTo ? 1 : 0);
}

const DATE = /^\d{4}-\d{2}-\d{2}$/u;
const SIGNIN = /^(never|older:\d{1,5}|within:\d{1,5})$/u;

function split(value: string | undefined): string[] {
  return (value ?? '').split(',').map((v) => v.trim()).filter((v) => v !== '');
}

/** The view a fragment query names. Unknown values are dropped, not guessed at. */
export function viewFromQuery(query: Readonly<Record<string, string>> | undefined): UserView {
  const q = query ?? {};
  const view = defaultView();
  view.filters.q = (q['q'] ?? '').slice(0, 200);
  view.filters.domain = split(q['domain']).map((d) => d.toLowerCase());
  view.filters.status = split(q['status']).filter((s): s is UserState => (STATES as readonly string[]).includes(s));
  view.filters.twoFactor = split(q['tfa']).filter((s): s is TwoFactor => (FACTORS as readonly string[]).includes(s));
  view.filters.role = split(q['role']);
  view.filters.profile = split(q['profile']);
  const signin = q['signin'] ?? '';
  view.filters.signin = SIGNIN.test(signin) ? signin : null;
  const [from = '', to = ''] = (q['created'] ?? '').split('..');
  view.filters.createdFrom = DATE.test(from) ? from : null;
  view.filters.createdTo = DATE.test(to) ? to : null;
  const sort = q['sort'] ?? '';
  if ((SORT_KEYS as readonly string[]).includes(sort)) view.sort = sort as SortKey;
  view.dir = q['dir'] === 'asc' || q['dir'] === 'desc' ? q['dir'] : view.sort === 'lastSeen' ? 'desc' : 'asc';
  const page = Number(q['page'] ?? '1');
  view.page = Number.isInteger(page) && page > 1 ? page - 1 : 0;
  const size = Number(q['size'] ?? '');
  if ((PAGE_SIZES as readonly number[]).includes(size)) view.pageSize = size;
  return view;
}

/** The fragment query of a view: only what differs from the default. */
export function viewToQuery(view: UserView): Record<string, string> {
  const out: Record<string, string> = {};
  const f = view.filters;
  if (f.q.trim() !== '') out['q'] = f.q.trim();
  if (f.domain.length) out['domain'] = f.domain.join(',');
  if (f.status.length) out['status'] = f.status.join(',');
  if (f.twoFactor.length) out['tfa'] = f.twoFactor.join(',');
  if (f.role.length) out['role'] = f.role.join(',');
  if (f.profile.length) out['profile'] = f.profile.join(',');
  if (f.signin) out['signin'] = f.signin;
  if (f.createdFrom || f.createdTo) out['created'] = `${f.createdFrom ?? ''}..${f.createdTo ?? ''}`;
  if (view.sort !== 'lastSeen') out['sort'] = view.sort;
  if (view.dir !== (view.sort === 'lastSeen' ? 'desc' : 'asc')) out['dir'] = view.dir;
  if (view.page > 0) out['page'] = String(view.page + 1);
  if (view.pageSize !== 50) out['size'] = String(view.pageSize);
  return out;
}

/** Local midnight of a `YYYY-MM-DD` date, in epoch seconds (`end` = its last second). */
export function dayEdge(date: string, end: boolean): number {
  const [y, m, d] = date.split('-').map(Number) as [number, number, number];
  const at = new Date(y, m - 1, d + (end ? 1 : 0), 0, 0, 0, 0).getTime() / 1000;
  return Math.floor(end ? at - 1 : at);
}

/** The API query string of a view. */
export function requestQuery(view: UserView): string {
  const f = view.filters;
  const p = new URLSearchParams();
  if (f.q.trim() !== '') p.set('q', f.q.trim());
  if (f.domain.length) p.set('domain', f.domain.join(','));
  if (f.status.length) p.set('status', f.status.join(','));
  if (f.twoFactor.length) p.set('twoFactor', f.twoFactor.join(','));
  if (f.role.length) p.set('role', f.role.join(','));
  if (f.profile.length) p.set('profile', f.profile.join(','));
  if (f.signin) p.set('signin', f.signin);
  if (f.createdFrom) p.set('createdFrom', String(dayEdge(f.createdFrom, false)));
  if (f.createdTo) p.set('createdTo', String(dayEdge(f.createdTo, true)));
  p.set('sort', view.sort);
  p.set('dir', view.dir);
  p.set('offset', String(view.page * view.pageSize));
  p.set('limit', String(view.pageSize));
  return p.toString();
}

export async function loadUserTable(view: UserView, signal?: AbortSignal): Promise<UserTablePage> {
  return (await adminRequest<UserTablePage>(`/users/table?${requestQuery(view)}`, signal ? { signal } : {})).data;
}

// ── Row actions (one request per user, each audited server-side) ──────────────

interface Lifecycle { status: string }

/** Block (`true`) or unblock one user's Mailwoman login: read the revision, then write. */
export async function setLoginBlocked(accountId: string, blocked: boolean): Promise<void> {
  const path = `/users/${encodeURIComponent(accountId)}/lifecycle`;
  const current = await adminRequest<Lifecycle>(path);
  if (!current.etag) throw new Error('Missing revision');
  await adminRequest<Lifecycle>(path, { method: 'PUT', ifMatch: current.etag, body: { disabled: blocked } });
}

/** End every Mailwoman session of one user. */
export async function signOutUser(accountId: string): Promise<void> {
  await adminRequest(`/users/${encodeURIComponent(accountId)}/revoke-sessions`, { method: 'POST' });
}

/** Give one user a feature profile (`null` removes the user's own assignment). */
export async function assignProfile(accountId: string, profileId: string | null, current: string | null): Promise<void> {
  const revision = async (): Promise<string> => {
    const read = await adminRequest('/features');
    if (!read.etag) throw new Error('Missing revision');
    return read.etag;
  };
  if (current !== null && current !== profileId) {
    await adminRequest('/features/assignments', { method: 'DELETE', ifMatch: await revision(), body: { scopeKind: 'user', scopeId: accountId, profileId: current, priority: 100 } });
  }
  if (profileId !== null && profileId !== current) {
    await adminRequest('/features/assignments', { method: 'PUT', ifMatch: await revision(), body: { scopeKind: 'user', scopeId: accountId, profileId, priority: 100 } });
  }
}

/** Delete one user's local Mailwoman access (the upstream mailbox is kept). */
export async function deleteAccess(accountId: string): Promise<void> {
  const path = `/users/${encodeURIComponent(accountId)}/lifecycle`;
  const current = await adminRequest<Lifecycle>(path);
  if (!current.etag) throw new Error('Missing revision');
  await adminRequest<Lifecycle>(path, { method: 'DELETE', ifMatch: current.etag });
}
