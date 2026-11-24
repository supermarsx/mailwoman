// t52w: the load-time reads in one request (`GET /api/bootstrap`, server
// `crates/mw-server/src/bootstrap.rs`, docs/api/bootstrap.md).
//
// The server answers `{"v":1,"sections":{"<name>":{status,etag,body}}}`, each
// section being what its own GET route answers to this same request. This
// module holds that answer as a seed: the shared GET helper (`apiFetch`) asks
// `takeSeed` once before it goes to the network, and a seeded path is answered
// from the section instead. Callers, their caches and their error handling are
// unchanged; nothing here is a second API.
//
// Rules, each enforced in `takeSeed`:
//   * single use: a path is served from the seed once per page load, the next
//     read goes to the network;
//   * only a section the route answered 200 is a seed. Any other status (401,
//     403, 409, 428, 503, a section too large) is not used, so the caller sends
//     its own request and handles the refusal as it always did;
//   * one account: the seed belongs to the session `me` names (account id and
//     storage authority). It is dropped as soon as another account becomes
//     active (a linked mailbox, a new sign-in) or the active one ends (sign-out),
//     and it is never consulted while a linked mailbox is selected or when the
//     request names another account in its fence headers. With no `me` (signed
//     out) only the public sections are kept;
//   * short life: a seed not used within SEED_LIFETIME_MS of its arrival is
//     dropped, so a read made later (after a change pushed over the socket, for
//     example) always reaches the server;
//   * the outer request failing in any way (network error, any status other
//     than 200, HTML from an older server, a shape or version this client does
//     not know) means "no seed": every read goes to its own route.
//
// Warm loads: the response is `Cache-Control: private, no-cache` with a strong
// ETag and `Vary: Cookie`. The request uses the default cache mode, so the
// browser keeps the last answer in its own HTTP cache, revalidates it with
// `If-None-Match` and hands this module the stored body when the server answers
// 304. No copy is written to localStorage or sessionStorage, and a different
// cookie (another sign-in) never matches the stored answer. A 304 that does
// reach this module (a caller that set the header itself) is treated as absent.
//
// Imports: no `http.ts` (which imports this module); the request itself is sent
// by `startBootstrap` in `http.ts`.

import { observePreferenceAccount, preferenceAccount } from './accountPreferenceLifecycle.ts';
import { activeLinkedMailbox } from './linkedMailboxSession.ts';

/** The bootstrap format this client understands. */
export const BOOTSTRAP_VERSION = 1;
/** How long an unused seed stays usable after the answer arrived. */
export const SEED_LIFETIME_MS = 15_000;
/** A bootstrap answer slower than this is given up; the reads go out on their own. */
export const BOOTSTRAP_WAIT_MS = 10_000;

/** Section name → the GET route it stands for (without the deploy prefix). */
export const SECTION_PATHS: Readonly<Record<string, string>> = Object.freeze({
  me: '/api/me',
  jmapSession: '/jmap/session',
  publicConfig: '/api/public-config',
  cookiesConfig: '/api/cookies/config',
  accounts: '/api/accounts',
  settings: '/api/settings',
  appearance: '/api/account/appearance',
  preferences: '/api/account/preferences',
  features: '/api/features',
  notifications: '/api/account/notifications',
  policy: '/api/policy',
  restriction: '/api/privacy/restriction',
  uiPlugins: '/api/ui-plugins',
  assistConfig: '/api/assist/config',
  version: '/api/version',
  focusedSenders: '/api/account/focused-senders',
  drafts: '/api/account/drafts',
  notificationCenter: '/api/account/notification-center',
});
/** Its path carries the primary account id (`?accountId=`), taken from `me`. */
const CUSTOMIZATION_SECTION = 'appearanceCustomization';
const CUSTOMIZATION_PATH = '/api/account/appearance/customization';
/** Sections the server answers the same way to every visitor. */
const PUBLIC_SECTIONS = new Set(['publicConfig', 'cookiesConfig']);
/** Sections whose `etag` is the route's own ETag (a validator its If-None-Match accepts). */
const ROUTE_ETAGS = new Set(['me', 'jmapSession', 'settings', 'appearance', 'notifications', 'assistConfig', 'uiPlugins']);
/** Request headers a seeded answer may ignore or check; any other header means "ask the route". */
const SEEDABLE_HEADERS = new Set(['accept', 'accept-language', 'content-type', 'if-none-match', 'x-mailwoman-account', 'x-mailwoman-storage-authority', 'x-mailwoman-client']);

export interface BootstrapSection {
  readonly status: number;
  readonly etag?: string;
  readonly body: unknown;
  readonly error?: string;
}
interface Identity { readonly accountId: string; readonly storageAuthority?: string }
interface Seed {
  readonly identity: Identity | null;
  /** Path (with query) → section; an entry is removed when it is served. */
  readonly sections: Map<string, { readonly name: string; readonly section: BootstrapSection }>;
  readonly expires: number;
}

let pending: Promise<Seed | null> | null = null;
let seed: Seed | null = null;
/** Bumped whenever the seed is dropped; an answer adopted under an older one is discarded. */
let generation = 0;
/** An account was active since the seed was adopted (a later `null` is a sign-out). */
let activated = false;
let now: () => number = () => Date.now();

function drop(): void {
  generation += 1;
  pending = null;
  seed = null;
}

observePreferenceAccount((account) => {
  if (pending === null && seed === null) return;
  if (account === null) {
    // `init` and `login` suspend before their first read; only a suspension
    // after an account was active is a sign-out or a switch.
    if (activated) drop();
    return;
  }
  const identity = seed?.identity;
  if (seed !== null && (identity === null || identity === undefined || !sameAccount(identity, account.id, account.storageAuthority))) {
    drop();
    return;
  }
  activated = true;
});

function sameAccount(identity: Identity, id: string, storageAuthority: string | undefined): boolean {
  return identity.accountId === id && (identity.storageAuthority ?? undefined) === (storageAuthority ?? undefined);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function validSection(value: unknown): value is BootstrapSection {
  if (!isRecord(value)) return false;
  const { status, etag, error } = value;
  return Number.isInteger(status) && (status as number) >= 100 && (status as number) <= 599
    && (etag === undefined || typeof etag === 'string')
    && (error === undefined || typeof error === 'string')
    && 'body' in value;
}

/**
 * The seed in a bootstrap answer, or null when it cannot be used (see the
 * module comment). Exported for tests.
 */
export async function readBootstrap(response: Response): Promise<Seed | null> {
  if (response.status !== 200) return null;
  if (!(response.headers.get('content-type') ?? '').toLowerCase().includes('json')) return null;
  let parsed: unknown;
  try { parsed = await response.json(); } catch { return null; }
  if (!isRecord(parsed) || parsed['v'] !== BOOTSTRAP_VERSION || !isRecord(parsed['sections'])) return null;
  const given = parsed['sections'];
  const me = given['me'];
  let identity: Identity | null = null;
  if (validSection(me) && me.status === 200 && me.error === undefined && isRecord(me.body)) {
    const { accountId, storageAuthority } = me.body;
    if (typeof accountId === 'string' && accountId !== '' && (storageAuthority === undefined || typeof storageAuthority === 'string')) {
      identity = Object.freeze({ accountId, ...(typeof storageAuthority === 'string' ? { storageAuthority } : {}) });
    }
  }
  const sections = new Map<string, { name: string; section: BootstrapSection }>();
  const keep = (name: string, path: string): void => {
    const section = given[name];
    if (!validSection(section) || section.status !== 200 || section.error !== undefined) return;
    if (identity === null && !PUBLIC_SECTIONS.has(name)) return;
    sections.set(path, { name, section });
  };
  for (const [name, path] of Object.entries(SECTION_PATHS)) keep(name, path);
  if (identity !== null) keep(CUSTOMIZATION_SECTION, `${CUSTOMIZATION_PATH}?accountId=${encodeURIComponent(identity.accountId)}`);
  return { identity, sections, expires: now() + SEED_LIFETIME_MS };
}

/**
 * Use `request` (the `GET /api/bootstrap` already on the wire) as this page
 * load's seed. A later call replaces an earlier one.
 */
export function adoptBootstrap(request: Promise<Response>): void {
  drop();
  activated = preferenceAccount() !== null;
  const adopted = generation;
  let timer: ReturnType<typeof setTimeout> | undefined;
  const late = new Promise<null>((resolve) => { timer = setTimeout(() => resolve(null), BOOTSTRAP_WAIT_MS); });
  const read = request.then(readBootstrap, () => null);
  const settled = Promise.race([read, late]).then((value) => {
    clearTimeout(timer);
    if (adopted !== generation) return null;
    pending = null;
    seed = value;
    // An account that became active while the answer was on the wire must be
    // the one the answer is for.
    const account = preferenceAccount();
    if (value !== null && account !== null && (value.identity === null || !sameAccount(value.identity, account.id, account.storageAuthority))) drop();
    return seed;
  }, () => {
    clearTimeout(timer);
    if (adopted === generation) pending = null;
    return null;
  });
  pending = settled;
}

/** Forget the seed (tests; also safe to call at any time). */
export function clearBootstrap(): void {
  drop();
  activated = false;
}

/** Replace the clock (tests). */
export function setBootstrapClock(clock: (() => number) | null): void {
  now = clock ?? (() => Date.now());
}

/** Whether `path` may have a seed at all, without waiting for the answer. */
function seedable(path: string): boolean {
  if (Object.values(SECTION_PATHS).includes(path)) return true;
  return path.startsWith(`${CUSTOMIZATION_PATH}?`);
}

function synthesize(name: string, section: BootstrapSection, ifNoneMatch: string | null): Response {
  const headers = new Headers({ 'x-mailwoman-bootstrap': '1' });
  const etag = ROUTE_ETAGS.has(name) && section.etag !== undefined && section.etag !== '' ? section.etag : null;
  if (etag !== null) headers.set('etag', etag);
  if (etag !== null && ifNoneMatch !== null && ifNoneMatch.split(',').map((value) => value.trim()).includes(etag)) {
    return new Response(null, { status: 304, headers });
  }
  if (section.body === null) return new Response(null, { status: 200, headers });
  if (typeof section.body === 'string') {
    headers.set('content-type', 'text/plain; charset=utf-8');
    return new Response(section.body, { status: 200, headers });
  }
  headers.set('content-type', 'application/json');
  return new Response(JSON.stringify(section.body), { status: 200, headers });
}

/**
 * The seeded answer for a primary-context GET of `path` (route path without the
 * deploy prefix, with its query), or null when the caller must send its request.
 * Waits for a bootstrap answer still on the wire. `headers` are the request's.
 */
export async function takeSeed(path: string, headers: Headers): Promise<Response | null> {
  if (pending === null && seed === null) return null;
  if (!seedable(path)) return null;
  for (const name of headers.keys()) if (!SEEDABLE_HEADERS.has(name.toLowerCase())) return null;
  const started = generation;
  if (pending !== null) await pending;
  if (started !== generation || seed === null) return null;
  const current = seed;
  if (now() >= current.expires) { drop(); return null; }
  if (activeLinkedMailbox() !== null) { drop(); return null; }
  const entry = current.sections.get(path);
  if (entry === undefined) return null;
  const account = preferenceAccount();
  if (current.identity === null) {
    if (!PUBLIC_SECTIONS.has(entry.name)) return null;
  } else {
    if (account !== null && !sameAccount(current.identity, account.id, account.storageAuthority)) { drop(); return null; }
    const fenceAccount = headers.get('x-mailwoman-account');
    const fenceAuthority = headers.get('x-mailwoman-storage-authority');
    if (fenceAccount !== null && fenceAccount !== current.identity.accountId) return null;
    if (fenceAuthority !== null && fenceAuthority !== current.identity.storageAuthority) return null;
  }
  current.sections.delete(path);
  return synthesize(entry.name, entry.section, headers.get('if-none-match'));
}
