// t95: the one resolver behind every admin breadcrumb level.
//
// The route is `#/<section>/<rest...>`. Each rest segment is one of:
//   • a page word (`new`, `edit`, `connections`, a tab id): its `admin-crumb-*`
//     label from admin-shell.ftl;
//   • a settings registry key (`#/settings/mail.focusedInbox`,
//     `#/privacy/policy/privacy.trackers`): the registry label (`settingLabel`);
//   • a name that is already the display name (a domain, a hosting mailbox): as is;
//   • an entity id (a user, an admin, a role, an SSO backend, an IdP client, a
//     plugin, a feature profile, a Nextcloud instance, a hosting connection, an
//     SSH tunnel, an egress route or destination, a domain backend): its display
//     name, fetched once and cached. The trail renders at once; a level whose
//     name is not known yet is `null` (the trail draws a placeholder), never the id.
// Whatever has no label falls back to words made from the segment (`humanize`),
// or to the entity's kind ("User") for an opaque id, and the miss is logged at
// debug level. The section level uses the section table's label.

import { createSignal } from 'solid-js';
import { t, loadCatalog } from '../../../i18n';
import { humanize, isOpaqueId, logMissingCrumbLabel } from '../../../components/breadcrumbs/humanize.ts';
import { onAdminSessionEnded } from '../../../api/adminSession.ts';

// ── Catalogs ──────────────────────────────────────────────────────────────────
// The labels below come from these catalogs. Until they have loaded, a label
// still reads as its message id, so the level waits (placeholder) instead.
export const CRUMB_CATALOGS = [
  'admin-shell', 'admin', 'admin-settings', 'admin-identity', 'admin-admins', 'privacy', 'plugins',
  'admin-personal', 'admin-features', 'admin-branding', 'admin-analytics',
  // t97: the Security centre's Requirements tab label lives with the centre.
  'admin-security',
  // t105: the Diagnostics sub-tab labels. Until that catalog exists, loadCatalog
  // skips it and those words fall back to humanized text.
  'admin-diagnostics',
] as const;
const [catalogsReady, setCatalogsReady] = createSignal(false);
let catalogsLoad: Promise<void> | undefined;
/** Resolves once the label catalogs have loaded (or failed to). */
export function crumbCatalogsLoaded(): Promise<void> {
  catalogsLoad ??= Promise.allSettled(CRUMB_CATALOGS.map((c) => loadCatalog(c))).then(() => { setCatalogsReady(true); });
  return catalogsLoad;
}
function ensureCatalogs(): boolean {
  void crumbCatalogsLoaded();
  return catalogsReady();
}

const MESSAGE_ID = /^[a-z][a-z0-9]*(?:-[a-z0-9]+)+$/;

/** A translated label, `null` while its catalog loads, else the humanized fallback. */
function translated(id: string, fallback: string, kind: string): string | null {
  const text = t(id);
  if (text !== id) return text;
  if (!ensureCatalogs()) return null;
  logMissingCrumbLabel(kind, fallback);
  return humanize(fallback) || fallback;
}

/** The label of a section from the section table (whose labels are `t()` calls). */
export function sectionCrumbLabel(id: string, label: () => string): string | null {
  const text = label();
  if (!MESSAGE_ID.test(text)) return text;
  if (!ensureCatalogs()) return null;
  logMissingCrumbLabel('section', id);
  return humanize(id) || id;
}

// ── Page words ────────────────────────────────────────────────────────────────
/** Route segments that name a page or a tab rather than an item. Each has an
 *  `admin-crumb-<word>` label in admin-shell.ftl. `+new`/`+register` are the
 *  marker spellings some screens use. */
export const CRUMB_WORDS: readonly string[] = [
  'new', 'edit', 'clone', 'guided', 'overview', 'configure', 'clients', 'register', 'manage',
  'connections', 'diagnostics', 'policy', 'aliases', 'alias-new', 'alias-edit', 'alias-clone', 'backends',
  'bans', 'locks', 'geography', 'geoip', 'protection', 'inbound', 'deployment', 'console', 'sessions',
  'audit', 'statistics', 'telemetry',
  'directory', 'webhooks', 'keys', 'hosting', 'ssh', 'domains', 'mailboxes',
  'application', 'authentication', 'jobs', 'mode', 'destinations', 'proxies', 'totp', 'security',
  'requirements',
  // t105: the Diagnostics sub-tabs (#/diagnostics/<tab>).
  'server', 'connectivity', 'posture', 'tools', 'history', 'report',
];
const WORD_ALIASES: Readonly<Record<string, string>> = { '+new': 'new', '+register': 'register' };

export function wordOf(segment: string): string | undefined {
  const word = WORD_ALIASES[segment] ?? segment;
  return CRUMB_WORDS.includes(word) ? word : undefined;
}

export function wordLabel(word: string): string | null {
  return translated(`admin-crumb-${word}`, word, 'word');
}

// ── Entities ──────────────────────────────────────────────────────────────────
export type EntityKind =
  | 'user' | 'admin' | 'role' | 'sso' | 'idp-client' | 'plugin' | 'ui-plugin' | 'profile'
  | 'nextcloud' | 'hosting' | 'ssh' | 'egress-proxy' | 'egress-destination' | 'backend';

export const ENTITY_KINDS: readonly EntityKind[] = [
  'user', 'admin', 'role', 'sso', 'idp-client', 'plugin', 'ui-plugin', 'profile',
  'nextcloud', 'hosting', 'ssh', 'egress-proxy', 'egress-destination', 'backend',
];

/** Fetch the display name of one item; `undefined` when it has none or is gone.
 *  `parent` is the containing item (the domain of a backend). */
export type EntityFetcher = (id: string, parent?: string) => Promise<string | undefined>;

const named = <T>(rows: readonly T[], id: string, key: (r: T) => string, name: (r: T) => string | undefined): string | undefined => {
  const row = rows.find((r) => key(r) === id);
  const value = row ? name(row)?.trim() : undefined;
  return value ? value : undefined;
};

// The production fetchers. Each module is imported on demand, so the clients of
// a section the admin never opens stay out of the console's first chunk.
export const DEFAULT_FETCHERS: Readonly<Record<EntityKind, EntityFetcher>> = {
  async user(id) {
    const { adminRequest } = await import('./adminApi.ts');
    const r = await adminRequest<{ label?: string; loginAddress?: string; address?: string; legacyBinding?: boolean }>(`/users/${encodeURIComponent(id)}/identity`);
    const d = r.data;
    return (d.legacyBinding ? d.loginAddress ?? d.label : d.label) || d.loginAddress || d.address;
  },
  async admin(id) {
    const { adminsApi } = await import('../../../api/admin/admins.ts');
    return named(await adminsApi.accounts(), id, (a) => a.id, (a) => a.displayName || a.username);
  },
  async role(id) {
    const { adminsApi } = await import('../../../api/admin/admins.ts');
    return named(await adminsApi.roles(), id, (r) => r.id, (r) => r.name);
  },
  async sso(id) {
    const { createHttpSsoAdminApi } = await import('../../../modules/sso');
    const api = createHttpSsoAdminApi();
    const own = named(await api.list('deployment'), id, (r) => r.id, (r) => r.displayName);
    if (own) return own;
    const { domainsApi } = await import('../../../api/admin/domains.ts');
    const domains = await domainsApi.list();
    for (const d of domains) {
      const found = named(await api.list(`domain:${d.name}`), id, (r) => r.id, (r) => r.displayName);
      if (found) return found;
    }
    return undefined;
  },
  async 'idp-client'(id) {
    const { identityProviderApi } = await import('../../../api/admin/identityProvider.ts');
    return (await identityProviderApi.client(id)).name || undefined;
  },
  async plugin(id) {
    const { createHttpPluginsApi } = await import('../../../state/slices/plugins.ts');
    return named(await createHttpPluginsApi().list(), id, (p) => p.id, (p) => p.name);
  },
  async 'ui-plugin'(id) {
    const { createHttpUiPluginsApi } = await import('../UiPlugins/service.ts');
    return named(await createHttpUiPluginsApi().list(), id, (p) => p.id, (p) => p.name);
  },
  async profile(id) {
    const { featuresApi } = await import('../Features/api.ts');
    return named((await featuresApi.load()).data.profiles, id, (p) => p.id, (p) => p.name);
  },
  async nextcloud(id) {
    const { nextcloudAccountsApi } = await import('../../../api/nextcloudAccounts.ts');
    return named((await nextcloudAccountsApi.instances()).instances, id, (i) => i.id, (i) => i.name);
  },
  async hosting(id) {
    const { hostingApi } = await import('../../../api/admin/hosting.ts');
    return named(await hostingApi.list(), id, (c) => c.id, (c) => c.name);
  },
  async ssh(id) {
    const { sshTunnelApi } = await import('../../../api/admin/ssh-tunnels.ts');
    return named(await sshTunnelApi.list(), id, (c) => c.id, (c) => c.name);
  },
  async 'egress-proxy'(id) {
    const { createHttpEgressAdminApi } = await import('../Egress.tsx');
    return named(await createHttpEgressAdminApi().list(), id, (r) => r.id, (r) => `${r.scheme}://${r.host}:${r.port}`);
  },
  async 'egress-destination'(id) {
    const { egressDestinationsApi } = await import('../../../api/admin/egressDestinations.ts');
    return named((await egressDestinationsApi.list()).destinations, id, (d) => d.id, (d) => d.destination);
  },
  async backend(id, parent) {
    if (!parent) return undefined;
    const { domainsApi, domainBackends } = await import('../../../api/admin/domains.ts');
    const detail = await domainsApi.detail(parent);
    const row = domainBackends(detail.upstream).find((b) => b.id === id);
    return row ? `${row.protocol.toUpperCase()} · ${t(`admin-crumb-backend-${row.role}`)}` : undefined;
  },
};

let fetchers: Readonly<Record<EntityKind, EntityFetcher>> = DEFAULT_FETCHERS;
/** Tests replace the fetchers; `undefined` restores the production ones. Clears the cache. */
export function setEntityFetchers(next?: Partial<Record<EntityKind, EntityFetcher>>): void {
  fetchers = { ...DEFAULT_FETCHERS, ...next };
  clearEntityNames();
}

// name, or `false` once a fetch found no name (the fallback is shown then).
const [names, setNames] = createSignal<ReadonlyMap<string, string | false>>(new Map());
const fetchedAt = new Map<string, number>();
const inFlight = new Set<string>();
/** A name shown again is refreshed in the background at most this often, so a
 *  rename shows up on the next visit without the trail flashing back to loading. */
const REFRESH_MS = 5000;

export function clearEntityNames(): void {
  fetchedAt.clear();
  inFlight.clear();
  setNames(new Map());
}
onAdminSessionEnded(clearEntityNames);

/** Seed or update a name the caller already knows (a screen that just saved it). */
export function rememberEntityName(kind: EntityKind, id: string, name: string, parent?: string): void {
  const key = cacheKey(kind, id, parent);
  fetchedAt.set(key, Date.now());
  setNames((m) => new Map(m).set(key, name));
}

const cacheKey = (kind: EntityKind, id: string, parent?: string): string => `${kind}\u0000${parent ?? ''}\u0000${id}`;

function request(kind: EntityKind, id: string, parent?: string): void {
  const key = cacheKey(kind, id, parent);
  if (inFlight.has(key)) return;
  const at = fetchedAt.get(key);
  if (at !== undefined && Date.now() - at < REFRESH_MS) return;
  inFlight.add(key);
  fetchedAt.set(key, Date.now());
  void (async () => {
    let value: string | false = false;
    try {
      value = (await fetchers[kind](id, parent))?.trim() || false;
    } catch {
      value = false;
    }
    if (!inFlight.delete(key)) return; // cleared meanwhile (sign-out)
    // A refresh that fails keeps the name already shown.
    if (value === false && typeof names().get(key) === 'string') return;
    // An unchanged name changes nothing on screen.
    if (names().get(key) === value) return;
    setNames((m) => new Map(m).set(key, value));
  })();
}

/** The entity's kind, as a noun ("User"), for an item whose name is unknown. */
export function entityNoun(kind: EntityKind): string | null {
  return translated(`admin-crumb-entity-${kind}`, kind, 'entity');
}

/** The display name of an item, `null` while it is fetched, the fallback when none. */
export function entityLabel(kind: EntityKind, id: string, parent?: string): string | null {
  const key = cacheKey(kind, id, parent);
  request(kind, id, parent);
  const value = names().get(key);
  if (typeof value === 'string') return value;
  if (value === undefined) return null;
  logMissingCrumbLabel(kind, id);
  if (isOpaqueId(id)) return entityNoun(kind);
  return humanize(id) || entityNoun(kind);
}

// ── Settings keys ─────────────────────────────────────────────────────────────
/** The registry label of a settings key (`admin-settings-key-*`), `null` while
 *  the catalog loads, the humanized key when the registry has no label. */
export function settingCrumbLabel(key: string, label: (key: string) => string): string | null {
  const text = label(key);
  if (text !== key) return text;
  if (!ensureCatalogs()) return null;
  logMissingCrumbLabel('setting', key);
  return humanize(key) || key;
}

// ── Route grammar ─────────────────────────────────────────────────────────────
export type CrumbSpec =
  | { kind: 'word'; word: string }
  | { kind: 'setting'; key: string }
  | { kind: 'name'; text: string }
  | { kind: 'entity'; entity: EntityKind; id: string; parent?: string }
  | { kind: 'unknown'; raw: string };

const entity = (e: EntityKind, id: string, parent?: string): CrumbSpec => (parent === undefined ? { kind: 'entity', entity: e, id } : { kind: 'entity', entity: e, id, parent });

/** Sections whose first segment is an item of one kind (`new`/`+new` excepted). */
const FIRST_ENTITY: Readonly<Record<string, EntityKind>> = {
  users: 'user', admins: 'admin', roles: 'role', sso: 'sso', plugins: 'plugin', 'ui-plugins': 'ui-plugin',
  features: 'profile', nextcloud: 'nextcloud',
};

/** What the segment `rest[i]` of `#/<section>/<rest>` names. */
export function specFor(section: string, rest: readonly string[], i: number): CrumbSpec {
  const seg = rest[i] ?? '';
  const word = wordOf(seg);
  switch (section) {
    case 'settings':
      if (i === 0) return { kind: 'setting', key: seg };
      break;
    case 'privacy':
      if (i === 1 && rest[0] === 'policy') return { kind: 'setting', key: seg };
      break;
    case 'domains':
      if (i === 0) return word === 'new' ? { kind: 'word', word } : { kind: 'name', text: seg };
      if (i === 2 && rest[1] === 'backends' && word !== 'new') return entity('backend', seg, rest[0]);
      if (i === 2 && rest[1]?.startsWith('alias-')) return { kind: 'name', text: seg };
      break;
    case 'identity-provider':
      if (i === 1 && rest[0] === 'clients' && word !== 'new') return entity('idp-client', seg);
      break;
    case 'integrations':
      if (i === 1 && word === undefined && rest[0] === 'hosting') return entity('hosting', seg);
      if (i === 1 && word === undefined && rest[0] === 'ssh') return entity('ssh', seg);
      // A hosting connection's own domains and mailboxes are names already.
      if (i >= 3 && rest[0] === 'hosting' && word === undefined) return { kind: 'name', text: seg };
      break;
    case 'egress':
      if (i === 0 && word === undefined) return entity('egress-proxy', seg);
      if (i === 1 && rest[0] === 'destinations' && word === undefined) return entity('egress-destination', seg);
      break;
    default: {
      const kind = FIRST_ENTITY[section];
      if (kind && i === 0 && word === undefined) return entity(kind, seg);
    }
  }
  return word !== undefined ? { kind: 'word', word } : { kind: 'unknown', raw: seg };
}

/** The label of one spec; `null` while it loads. */
export function specLabel(spec: CrumbSpec, label: (key: string) => string): string | null {
  switch (spec.kind) {
    case 'word': return wordLabel(spec.word);
    case 'setting': return settingCrumbLabel(spec.key, label);
    case 'name': return spec.text;
    case 'entity': return entityLabel(spec.entity, spec.id, spec.parent);
    case 'unknown':
      logMissingCrumbLabel('segment', spec.raw);
      return isOpaqueId(spec.raw) ? translated('admin-crumb-item', 'item', 'word') : humanize(spec.raw) || spec.raw;
  }
}

/** Labels for every rest segment of a route. */
export function restCrumbLabels(section: string, rest: readonly string[], label: (key: string) => string): (string | null)[] {
  return rest.map((_, i) => specLabel(specFor(section, rest, i), label));
}
