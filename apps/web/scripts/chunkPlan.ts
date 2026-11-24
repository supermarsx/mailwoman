/**
 * t52w part 4: the explicit chunk layout of the web client.
 *
 * Without this, Rollup's default splitting gave 318 JS and 101 CSS files, and
 * one changed line renamed 222 hashed files: lazy chunks imported shared code
 * from the entry and from each other by hashed name, so one new hash cascaded
 * through most of the graph (t39x audit, one-line-change.txt).
 *
 * Rollup's name for a chunk covers the names of every chunk it imports,
 * statically or dynamically. So a changed module renames its chunk and every
 * chunk that imports that one, transitively. The layout keeps those paths short:
 *
 *   vendor-core   third-party code the entry needs (solid-js, @fluent, idb, ...)
 *   core          app modules statically reachable from index.html, except the
 *                 entry's own modules
 *   core-lazy     what core loads lazily (step-up dialog, offline sync, Tauri):
 *                 imports nothing above core
 *   shared        lazy modules the mailbox and another screen both reach
 *   dialogs       what shared loads lazily: imports nothing above shared
 *   shared-lazy   lazy modules two screens other than the mailbox reach
 *   editor        ProseMirror and the editor UI
 *   <screen>      login, mail, compose, settings, admin, pim, viewers,
 *                 crypto-ui, plugins, about: what one screen alone reaches
 *   admin-lazy    what the admin shell shares with screens other than the
 *                 mailbox
 *   mail-lazy     panes of the mailbox loaded on demand (templates, unified
 *                 inbox); may import mail
 *   admin, admin-common, admin-section.<path>
 *                 the admin console: the shell with its first view
 *                 (dashboard), code several sections share, and one chunk per
 *                 section; no other screen loads them
 *   rare          screens most sessions never load; may import any screen
 *   unplanned     dynamic import targets no ROOTS rule names yet: the build
 *                 still works, and check-load-budget.mjs fails until a rule
 *                 places them
 *   entry         main.tsx, App.tsx and the route table
 *
 * A chunk imports only from layers below it (rare and the entry excepted, and
 * `mail` imports the entry for one module: ENTRY_IMPORT_EXCEPTIONS). Data
 * chunks (brand-paths, licences) import nothing, so a change in
 * app code never renames them.
 *
 * Every module outside the entry's static closure belongs to the screens whose
 * ROOTS (dynamic import targets) reach it through static imports; resolveOwners
 * turns that set into one chunk. Route boundaries stay dynamic imports: a root is
 * never placed in a chunk that its importer loads statically
 * (check-load-budget.mjs asserts this on the built graph).
 *
 * vanilla-extract `*.css.ts` modules and their virtual `.vanilla.css` modules are
 * ordinary members of the graph, so they land in their owner's chunk and their
 * CSS in that chunk's one CSS file (cssCodeSplit stays on).
 */

import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import type { Rollup } from 'vite';
type Plugin = Rollup.Plugin;

/**
 * Chunks and their layers. `entry` holds index.html's module: a manual chunk
 * that contains the entry module becomes the entry chunk (Vite builds apps
 * without preserved entry signatures, so Rollup needs no facade). It is named
 * explicitly because Rollup moves an unassigned module into the manual chunk of
 * any module that imports it.
 */
export const CHUNK_LAYERS = {
  'vendor-core': 0,
  'brand-paths': 0,
  licences: 0,
  'vendor-altcha': 0,
  core: 1,
  'core-lazy': 2,
  shared: 3,
  dialogs: 4,
  'admin-lazy': 4.5,
  'shared-lazy': 5,
  editor: 6,
  login: 7,
  mail: 7,
  compose: 7,
  settings: 7,
  admin: 7,
  pim: 7,
  viewers: 7,
  'crypto-ui': 7,
  plugins: 7,
  about: 7,
  'mail-lazy': 8,
  'admin-common': 8,
  'admin-section': 9,
  rare: 10,
  unplanned: 10,
  entry: 11,
} as const;
export type ChunkName = keyof typeof CHUNK_LAYERS;
/**
 * A chunk of the plan: a ChunkName, or one admin console section,
 * `admin-section.<path>`, which has the layer of `admin-section`.
 */
export type PlanChunk = ChunkName | `admin-section.${string}`;
/** The ChunkName a plan chunk takes its layer and rules from. */
export function family(chunk: PlanChunk): ChunkName {
  return chunk.startsWith('admin-section.') ? 'admin-section' : (chunk as ChunkName);
}
export const layerOf = (chunk: PlanChunk): number => CHUNK_LAYERS[family(chunk)];
/** Chunks of the admin console's shell (with its first view) and sections. */
export const ADMIN_CHUNKS: ReadonlySet<ChunkName> = new Set(['admin', 'admin-section', 'admin-lazy']);
/** Chunks no screen but the admin console may load. */
export const CONSOLE_CHUNKS: ReadonlySet<ChunkName> = new Set(['admin', 'admin-section', 'admin-common']);
const isAdmin = (chunk: PlanChunk): boolean => ADMIN_CHUNKS.has(family(chunk));

/** Chunks that may import a screen chunk (they are loaded after one, or alone). */
export const LEAF_CHUNKS: ReadonlySet<ChunkName> = new Set(['rare', 'unplanned']);
/** Chunks whose modules import nothing (pure data). */
export const DATA_CHUNKS: ReadonlySet<ChunkName> = new Set(['brand-paths', 'licences']);
/** Chunks made of third-party code only. */
export const VENDOR_CHUNKS: ReadonlySet<ChunkName> = new Set(['vendor-core', 'vendor-altcha']);
/**
 * Chunks a lower chunk loads lazily, with that lower chunk: a module they share
 * with any other screen goes down into it, so they never import above it.
 */
export const BASE_OF: ReadonlyMap<ChunkName, ChunkName> = new Map([['core-lazy', 'core'], ['dialogs', 'shared']]);
/** Panes only their parent screen loads: a module only the two reach goes to the parent. */
export const PARENT_OF: ReadonlyMap<ChunkName, ChunkName> = new Map([['mail-lazy', 'mail']]);

/** Modules of the entry chunk: the bootstrap and the route table. */
export const ENTRY_MODULES: readonly RegExp[] = [
  /^index\.html$/,
  /^\0vite\/modulepreload-polyfill/,
  /^src\/main\.tsx$/,
  /^src\/App\.tsx$/,
  // These two hold the dynamic imports of Mailbox panes (Compose, Settings) and
  // of the idle prefetch targets. In core they would make every screen's name
  // part of core's; in the entry their only cost is that `mail` imports
  // mailboxPaneLoaders from the entry (ENTRY_IMPORT_EXCEPTIONS).
  /^src\/boot\/idlePrefetch\.ts$/,
  /^src\/screens\/mailboxPaneLoaders\.ts$/,
];

/**
 * The only chunks allowed to import from the entry chunk, and why. `mail`
 * imports `mailboxPaneLoaders` (LazyMailboxPanes.tsx shares it with the idle
 * prefetch in App.tsx). A component edit can remove this later.
 */
export const ENTRY_IMPORT_EXCEPTIONS: ReadonlySet<ChunkName> = new Set(['mail']);

/** Third-party packages that form the editor chunk with the editor UI. */
const EDITOR_PACKAGES = /^(?:prosemirror-[a-z-]+|orderedmap|rope-sequence|w3c-keyname)$/;

/**
 * Chunk of each dynamic import target (a root). A dynamic import target that
 * no rule names fails the build. A module matches at most one rule (tested).
 */
export const ROOTS: ReadonlyArray<readonly [RegExp, ChunkName]> = [
  [/^src\/screens\/Login\.tsx$/, 'login'],
  [/^src\/screens\/Settings\/webauthn\.ts$/, 'login'],
  [/^src\/screens\/Mailbox\.tsx$/, 'mail'],
  [/^src\/screens\/(?:Templates|UnifiedMail)\.tsx$/, 'mail-lazy'],
  [/^src\/components\/properties\/(?:Bulk|Folder|Message)Properties\.tsx$/, 'dialogs'],
  [/^src\/components\/exportDialog\/ExportDialog\.tsx$/, 'dialogs'],
  [/^src\/modules\/contacts\/SaveContactsDialog\.tsx$/, 'dialogs'],
  [/^src\/components\/Compose\.tsx$/, 'compose'],
  [/^src\/components\/compose\/(?:quote|replyImages|htmlSource|richtext|inlineImages)\.ts$/, 'compose'],
  [/^src\/wasm\/mw-sanitize\/mw_sanitize\.js$/, 'editor'],
  [/^src\/components\/compose\/(?:RichTextEditor|HtmlSourceEditor)\.tsx$/, 'editor'],
  [/^src\/screens\/Settings\.tsx$/, 'settings'],
  [/^src\/screens\/Settings\/Signatures\.tsx$/, 'settings'],
  [/^src\/screens\/Admin\/(?:index\.tsx$|shell\/|Dashboard\/)/, 'admin'],
  [/^src\/screens\/Admin\/(?!index\.tsx$|shell\/|Dashboard\/)/, 'admin-section'],
  [/^src\/api\/admin\//, 'admin'],
  [/^src\/api\/nextcloudAccounts\.ts$/, 'admin'],
  [/^src\/state\/slices\/plugins\.ts$/, 'admin'],
  [/^src\/modules\/sso\/index\.ts$/, 'admin'],
  [/^src\/shell\/mounts\.tsx$/, 'pim'],
  [/^src\/modules\/(?:contacts|notes|tasks|notifications|calendar)\/index\.tsx$/, 'pim'],
  [/^src\/modules\/keys\/index\.tsx$/, 'crypto-ui'],
  [/^src\/viewers\/[A-Za-z]+Viewer\.tsx$/, 'viewers'],
  [/^src\/viewers\/originalHtml\.ts$/, 'viewers'],
  [/^src\/about\/texts\/[0-9a-f]\.json$/, 'licences'],
  [/^src\/about\/(?!texts\/)/, 'about'],
  [/^src\/plugins-ui\/Tier\.tsx$/, 'plugins'],
  [/^src\/offline\/(?:nativeBackground|nativeReplaySchedule|synchronize|calendarExpansion)\.ts$/, 'core-lazy'],
  [/^src\/platform\/tauri\.ts$/, 'core-lazy'],
  [/^src\/components\/StepUpDialog\.tsx$/, 'core-lazy'],
  [/^src\/worker\/networkCore\.ts$/, 'core-lazy'],
  [/^src\/observability\/e2eHook\.tsx$/, 'core-lazy'],
  [/^src\/brands\/paths\.[a-z]+\.generated\.ts$/, 'brand-paths'],
  [/^src\/modules\/passwd\/ForcedPasswordChange\.tsx$/, 'rare'],
  [/^src\/screens\/(?:Consent|IdentityConsent)\/index\.tsx$/, 'rare'],
  [/^src\/screens\/Settings\/ProviderConnections\.tsx$/, 'rare'],
];

/** Lazily imported third-party packages that form their own chunk. */
export const PACKAGE_ROOTS: ReadonlyMap<string, ChunkName> = new Map([['altcha', 'vendor-altcha']]);

/** Module id relative to apps/web, forward slashes, query kept. */
export function moduleKey(id: string, root: string): string {
  const slash = id.split('\\').join('/');
  const base = root.split('\\').join('/').replace(/\/?$/, '/');
  if (slash.startsWith('\0')) return slash;
  if (slash.toLowerCase().startsWith(base.toLowerCase())) return slash.slice(base.length);
  const nm = slash.indexOf('/node_modules/');
  return nm >= 0 ? slash.slice(nm + 1) : slash;
}

/** The npm package of a node_modules key (pnpm layout aware), or null. */
export function packageOf(key: string): string | null {
  const match = /(?:^|\/)node_modules\/(?:\.pnpm\/[^/]+\/node_modules\/)?((?:@[^/]+\/)?[^/?]+)/.exec(key.split('?')[0] ?? key);
  return match?.[1] ?? null;
}

export function rootChunk(key: string): ChunkName | null {
  const pkg = packageOf(key);
  if (pkg !== null) return PACKAGE_ROOTS.get(pkg) ?? null;
  for (const [pattern, chunk] of ROOTS) if (pattern.test(key)) return chunk;
  return null;
}

export function isEntryModule(key: string): boolean {
  return ENTRY_MODULES.some((pattern) => pattern.test(key));
}

/** Screen chunks whose modules belong to the editor when compose shares them. */
const EDITOR_FAMILY: ReadonlySet<PlanChunk> = new Set(['editor', 'compose']);

/**
 * The chunk of a module outside the entry closure, from the chunks whose roots
 * reach it (`set`).
 *  1. ProseMirror packages: `editor`.
 *  2. A module a lazily loaded chunk (BASE_OF) shares with anything: its base.
 *  3. Leaf owners count only when no other screen reaches the module.
 *  4. One owner, or a pane and its parent only: that owner, or the parent.
 *  5. Reached from the sign-in screen and another: `core`. Sign-in is the cold
 *     path of a first visit, and core is loaded there anyway.
 *  6. Reached from the mailbox and another screen: `shared`. The mailbox loads
 *     it in any case, so the mailbox path carries no other screen's code.
 *     If the admin shell reaches it too: `core` (see below).
 *  7. Reached only from the admin shell and sections: `admin`, the shell.
 *  8. Reached from the admin shell and another screen (not the mailbox):
 *     `admin-lazy`, never on the mailbox's path.
 *  Code that admin sections reach but the shell does not is placed first (see
 *  the start of the function): one section keeps it, several share
 *  `admin-common`, and with other screens it follows these rules with
 *  `admin-common` standing in for the sections.
 *  9. Reached only from compose and the editor: `editor` (compose loads it).
 * 10. Anything else two screens reach: `shared-lazy`, never on the mailbox's
 *     cold path.
 */
export function resolveOwners(set: ReadonlySet<PlanChunk>, pkg: string | null): PlanChunk {
  if (pkg !== null && EDITOR_PACKAGES.test(pkg)) return 'editor';
  // Code only admin sections reach (not the shell or its first view): one
  // section keeps it; else it is placed as if `admin-common` were one screen,
  // so it never rides along with the shell or into shared-admin.
  const sections = [...set].filter((c) => family(c) === 'admin-section');
  if (sections.length > 0 && !set.has('admin')) {
    if (sections.length === set.size && sections.length === 1) return sections[0]!;
    const rest = new Set<PlanChunk>([...set].filter((c) => family(c) !== 'admin-section'));
    rest.add('admin-common');
    return resolveOwners(rest, pkg);
  }
  const strong = [...set].filter((c) => !LEAF_CHUNKS.has(family(c)));
  const pool = strong.length > 0 ? strong : [...set];
  if (pool.length === 1 && set.size === 1) return pool[0]!;
  // Rules 2, 5 and 6 can all apply; the lowest layer wins, so that no chunk
  // ever imports above its own layer.
  const candidates: ChunkName[] = [];
  for (const chunk of set) { const base = BASE_OF.get(family(chunk)); if (base !== undefined) candidates.push(base); }
  if (candidates.length === 0 && pool.length === 1) return pool[0]!;
  const parents = new Set(pool.map((c) => PARENT_OF.get(family(c)) ?? c));
  const parent = [...parents][0]!;
  if (candidates.length === 0 && parents.size === 1 && pool.includes(parent)) return parent;
  if (pool.includes('login')) candidates.push('core');
  if (pool.includes('mail')) candidates.push('shared');
  if (candidates.length === 0) {
    if (pool.every(isAdmin)) return 'admin';
    if (pool.some(isAdmin)) return 'admin-lazy';
    return pool.every((c) => EDITOR_FAMILY.has(c)) ? 'editor' : 'shared-lazy';
  }
  const low = candidates.reduce((min, c) => (CHUNK_LAYERS[c] < CHUNK_LAYERS[min] ? c : min));
  // The admin console loads shared-admin, never the mailbox's shared code.
  // Code the admin shell (with its first view) and the mailbox both reach: core.
  // In `shared` it would put the mailbox's shared code on the console's first
  // view; in a chunk of its own it cost the mailbox path one more file and
  // 1.7 KB br (measured 2026-11-24, 804.1 against 802.4 KB br).
  return low === 'shared' && pool.some(isAdmin) ? 'core' : low;
}

/** `admin-section.<path below src/screens/Admin, dashes for slashes>`. */
export function sectionChunk(key: string): PlanChunk {
  const path = key.replace(/^src\/screens\/Admin\//, '').replace(/\.[a-z]+$/, '').replace(/\/index$/, '');
  return `admin-section.${path.replace(/[^A-Za-z0-9]+/g, '-')}`;
}

/** The subset of Rollup's ModuleInfo the plan reads. */
export interface PlanModule {
  readonly importedIds: readonly string[];
  readonly dynamicallyImportedIds: readonly string[];
  readonly isEntry: boolean;
}
export interface PlanGraph {
  getModuleIds(): IterableIterator<string> | readonly string[];
  getModuleInfo(id: string): PlanModule | null;
}

export interface Placement {
  /** module id -> chunk. */
  readonly chunks: ReadonlyMap<string, PlanChunk>;
  /** module id -> the screen chunks whose roots reach it (diagnostics). */
  readonly owners: ReadonlyMap<string, ReadonlySet<PlanChunk>>;
  /** Dynamic import targets outside the entry closure that match no ROOTS rule. */
  readonly unrootedDynamicTargets: readonly string[];
}

/**
 * Assign every module of the graph. Pure: the result depends only on the graph
 * and the rules above.
 */
export function planChunks(graph: PlanGraph, root: string): Placement {
  const ids = [...graph.getModuleIds()];
  const info = new Map<string, PlanModule>();
  for (const id of ids) { const m = graph.getModuleInfo(id); if (m) info.set(id, m); }
  const key = new Map(ids.map((id) => [id, moduleKey(id, root)] as const));
  const k = (id: string): string => key.get(id) ?? moduleKey(id, root);

  // 1. The entry's static closure: vendor-core, core or the entry itself.
  const entryClosure = new Set<string>();
  const queue = ids.filter((id) => info.get(id)?.isEntry === true);
  while (queue.length > 0) {
    const id = queue.pop()!;
    if (entryClosure.has(id)) continue;
    entryClosure.add(id);
    queue.push(...(info.get(id)?.importedIds ?? []));
  }
  const chunks = new Map<string, PlanChunk>();
  for (const id of entryClosure) {
    const name = k(id);
    const pkg = packageOf(name);
    chunks.set(id, isEntryModule(name) ? 'entry' : pkg !== null ? (EDITOR_PACKAGES.test(pkg) ? 'editor' : 'vendor-core') : 'core');
  }

  // 2. Roots: the dynamic import targets outside the entry closure, each named
  //    by a ROOTS rule; one no rule names goes to `unplanned` and is reported.
  //    A rule never makes a statically imported module a root: ownership comes
  //    from the graph.
  const roots = new Map<string, PlanChunk>();
  const unrooted = new Set<string>();
  for (const id of ids) for (const target of info.get(id)?.dynamicallyImportedIds ?? []) {
    if (entryClosure.has(target) || roots.has(target) || !info.has(target)) continue;
    const chunk = rootChunk(k(target));
    if (chunk === null) unrooted.add(target);
    // One chunk per admin console section: a section loads only its own code.
    roots.set(target, chunk === 'admin-section' ? sectionChunk(k(target)) : chunk ?? 'unplanned');
  }

  // 3. Owners: the chunks whose roots reach a module through static imports
  //    (never through the entry closure, which is already placed).
  const owners = new Map<string, Set<PlanChunk>>();
  for (const [rootId, chunk] of roots) {
    const seen = new Set<string>();
    const stack = [rootId];
    while (stack.length > 0) {
      const id = stack.pop()!;
      if (seen.has(id) || entryClosure.has(id)) continue;
      seen.add(id);
      let set = owners.get(id);
      if (!set) { set = new Set(); owners.set(id, set); }
      set.add(chunk);
      for (const dep of info.get(id)?.importedIds ?? []) stack.push(dep);
    }
  }
  for (const [id, set] of owners) chunks.set(id, resolveOwners(set, packageOf(k(id))));
  return { chunks, owners, unrootedDynamicTargets: [...unrooted].map(k).sort() };
}

/** Where the build writes the chunk report check-load-budget.mjs reads. */
export const CHUNK_REPORT = 'node_modules/.cache/mailwoman-build/chunk-plan.json';

/**
 * The plan for one build: `manualChunks` for Rollup's output options, and a
 * Rollup plugin that writes the emitted chunk graph, with the layer policy,
 * to CHUNK_REPORT for check-load-budget.mjs. A dynamic import target that no
 * rule names is built into the `unplanned` chunk and listed in the report, and
 * check-load-budget.mjs fails on it: a new lazy boundary is a reviewed
 * decision, but it does not stop a local build.
 */
export function chunkPlan(root: string): { manualChunks: (id: string, meta: PlanGraph) => string | null; plugin: Plugin } {
  let placement: Placement | null = null;
  const manualChunks = (id: string, meta: PlanGraph): string | null => {
    placement ??= planChunks(meta, root);
    return placement.chunks.get(id) ?? null;
  };
  const plugin: Plugin = {
    name: 'mw-chunk-plan',
    buildStart() { placement = null; },
    generateBundle(_options, bundle) {
      const chunks: Record<string, unknown> = {};
      for (const item of Object.values(bundle)) {
        if (item.type !== 'chunk') continue;
        const chunk: Rollup.OutputChunk = item;
        const planned = placement?.chunks;
        const names = new Set(chunk.moduleIds.map((id) => planned?.get(id) ?? '(unplaced)'));
        chunks[chunk.fileName] = {
          plan: names.size === 1 ? [...names][0] : null,
          isEntry: chunk.isEntry,
          isDynamicEntry: chunk.isDynamicEntry,
          imports: chunk.imports,
          dynamicImports: chunk.dynamicImports,
          css: [...(chunk.viteMetadata?.importedCss ?? [])],
          modules: chunk.moduleIds.map((id) => moduleKey(id, root)),
        };
      }
      const report = {
        schema: 1,
        policy: {
          layers: CHUNK_LAYERS,
          leaf: [...LEAF_CHUNKS],
          data: [...DATA_CHUNKS],
          vendor: [...VENDOR_CHUNKS],
          entryImportExceptions: [...ENTRY_IMPORT_EXCEPTIONS],
          admin: [...CONSOLE_CHUNKS],
        },
        unrootedDynamicTargets: placement?.unrootedDynamicTargets ?? [],
        chunks,
      };
      const file = resolve(root, CHUNK_REPORT);
      mkdirSync(dirname(file), { recursive: true });
      writeFileSync(file, `${JSON.stringify(report, null, 1)}\n`);
    },
  };
  return { manualChunks, plugin };
}

/**
 * `pnpm run size:touch` only: with MW_LOAD_TOUCH=<module path relative to
 * apps/web>, append one statement to that module, as the t39x audit did, so a
 * second build shows which hashed files one changed line renames.
 */
export function touchPlugin(root: string, target: string | undefined): Plugin {
  return {
    name: 'mw-load-touch',
    transform(code, id) {
      if (!target || moduleKey(id, root) !== target) return null;
      return { code: `${code}\nconsole.debug("mw-load-touch");\n`, map: null };
    },
  };
}
