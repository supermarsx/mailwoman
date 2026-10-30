// Analysis behind check-loaded.mjs (t112): message ids that would render as raw
// ids because they are missing from `en`, or because the catalog that defines
// them is not loaded where they are shown.
//
// check-catalog.mjs proves that every literal `t('id')` exists in SOME en
// catalog. That is not enough: catalogs are lazy (`loadCatalog('<module>')`), so
// an id that lives in `admin-features.ftl` but is shown by the admin nav renders
// as `admin-nav-features` until something else happens to load that catalog.
// t() also takes ids from tables (`labelId: 'x'`, `label: () => t('x')`,
// `cond ? 'a' : 'b'`), which the literal regex there does not see.
//
// Two checks, both from the TypeScript AST of every non-test file under src/:
//
//   missing   an id-shaped literal in a message-id position (a t() argument,
//             either branch of a conditional or `??` inside one, an
//             `...Id`/`...Key`/`label`/`title`/… property value, or an entry of
//             a table whose other entries are en ids) that is not an en id.
//   unloaded  an en id used in file F, where no catalog defining it is
//             guaranteed to be loaded whenever F is rendered.
//
// "Guaranteed loaded" for a file F is a dataflow over the static import graph
// (static `import`, `export … from`, and `import('…')`; `import type` is not an
// edge):
//
//   G(root) = ALWAYS ∪ own(root)
//   G(F)    = ALWAYS ∪ own(F) ∪ ⋂ { G(I) : I imports F }
//
// own(F) is the set of catalogs F loads itself (`loadCatalog('m')`,
// `useCatalog('m')`, `preloadCatalogs([...])`, or `X.map((c) => loadCatalog(c))`
// over a string array `X` declared in F). A catalog loaded by a CHILD does not
// count for its parent: the admin nav lists every section, but a section's
// catalog loads only when that section is opened, so the nav would show the id.
// The intersection over importers means a shared module only counts a catalog
// that every screen using it loads.

import { readFile } from 'node:fs/promises';
import { dirname, join, relative, resolve, sep } from 'node:path';
import { parseFtlIds, walk } from './catalog.mjs';

/** Catalogs registered at startup (catalog.ts bundles en/common eagerly). */
export const ALWAYS = new Set(['common']);

/** Files whose catalog loads are not guaranteed (idle prefetch may never run). */
export const UNGUARANTEED_LOADERS = new Set(['boot/idlePrefetch.ts']);

/** A Fluent message id with at least two hyphens (`admin-nav-features`). */
export const ID_SHAPE = /^[a-z][a-z0-9]*(?:-[a-z0-9]+){2,}$/;

/** Property names whose string value is a message id passed to t() later. */
const ID_PROPERTY = /^(?:[a-zA-Z]*(?:Id|Key)|label|title|description|help|hint|placeholder|tooltip|message|text|caption|summary)$/;

const LOAD_FNS = new Set(['loadCatalog', 'useCatalog']);

/** Map `module -> Set<id>` from the en catalog directory. */
export async function enModules(enDir) {
  const files = await walk(enDir, (n) => n.endsWith('.ftl'));
  const byModule = new Map();
  for (const f of files) {
    const mod = f.slice(f.lastIndexOf(sep) + 1).replace(/\.ftl$/, '');
    byModule.set(mod, parseFtlIds(await readFile(f, 'utf8')));
  }
  return byModule;
}

/** `id -> Set<module>` from `module -> Set<id>`. */
export function idIndex(byModule) {
  const index = new Map();
  for (const [mod, ids] of byModule) {
    for (const id of ids) {
      if (!index.has(id)) index.set(id, new Set());
      index.get(id).add(mod);
    }
  }
  return index;
}

const calleeName = (ts, expr) =>
  ts.isIdentifier(expr) ? expr.text : ts.isPropertyAccessExpression(expr) ? expr.name.text : undefined;

const stringOf = (ts, n) =>
  n && (ts.isStringLiteral(n) || ts.isNoSubstitutionTemplateLiteral(n)) ? n.text : undefined;

/**
 * Parse one file. Returns its import specifiers, the catalogs it loads, the
 * id-shaped literals in message-id positions (`candidates`), and every string
 * literal (`literals`, used for the loaded check against en ids).
 */
export function analyzeFile(ts, fileName, src) {
  const sf = ts.createSourceFile(fileName, src, ts.ScriptTarget.Latest, true,
    fileName.endsWith('.tsx') ? ts.ScriptKind.TSX : ts.ScriptKind.TS);
  const imports = []; // { spec, names: string[] | null }  (null: the whole module)
  const reexports = []; // { spec, names: Map<exported, local> | null }  (null: export *)
  const loads = new Set();
  const candidates = []; // { id, line, why }
  const literals = []; // { id, line }
  const arrays = new Map(); // const NAME = ['a', 'b'] -> strings
  const loopedArrays = []; // NAME in NAME.map((c) => loadCatalog(c))
  const lineOf = (n) => sf.getLineAndCharacterOfPosition(n.getStart(sf)).line + 1;
  const ignored = (line) => {
    const lines = src.split(/\r?\n/);
    return /i18n-ignore-id:\s*\S/.test(lines[line - 1] ?? '') || /i18n-ignore-id:\s*\S/.test(lines[line - 2] ?? '');
  };

  // Collect the id-shaped literals of an expression in a message-id position:
  // the literal itself, both branches of `?:`, both sides of `??`/`||`, parens,
  // `as` casts. A call (`t(kind())`) or identifier is not followed.
  const idLiterals = (e, out = []) => {
    if (!e) return out;
    const s = stringOf(ts, e);
    if (s !== undefined) { out.push(e); return out; }
    if (ts.isParenthesizedExpression(e) || ts.isAsExpression(e) || ts.isNonNullExpression(e) || ts.isSatisfiesExpression?.(e)) return idLiterals(e.expression, out);
    if (ts.isConditionalExpression(e)) { idLiterals(e.whenTrue, out); idLiterals(e.whenFalse, out); return out; }
    if (ts.isBinaryExpression(e) && (e.operatorToken.kind === ts.SyntaxKind.QuestionQuestionToken || e.operatorToken.kind === ts.SyntaxKind.BarBarToken)) {
      idLiterals(e.right, out); return out;
    }
    if (ts.isArrowFunction(e) && !ts.isBlock(e.body)) return idLiterals(e.body, out);
    return out;
  };
  const tArgs = new Set(); // literal nodes passed straight to t()
  const addCandidates = (e, why) => {
    for (const lit of idLiterals(e)) {
      if (why === 't()') tArgs.add(lit);
      const id = lit.text;
      if (ID_SHAPE.test(id)) candidates.push({ id, line: lineOf(lit), why });
    }
  };

  const visit = (n) => {
    if ((ts.isImportDeclaration(n) || ts.isExportDeclaration(n)) && n.moduleSpecifier && ts.isStringLiteral(n.moduleSpecifier)) {
      const spec = n.moduleSpecifier.text;
      if (ts.isImportDeclaration(n)) {
        const c = n.importClause;
        if (c?.isTypeOnly !== true) {
          let names = null;
          if (c && !c.name && c.namedBindings && ts.isNamedImports(c.namedBindings)) {
            names = c.namedBindings.elements.filter((e) => !e.isTypeOnly).map((e) => (e.propertyName ?? e.name).text);
          }
          if (names === null || names.length > 0) imports.push({ spec, names });
        }
      } else if (!n.isTypeOnly) {
        if (!n.exportClause) reexports.push({ spec, names: null });
        else if (ts.isNamedExports(n.exportClause)) {
          const m = new Map();
          for (const e of n.exportClause.elements) if (!e.isTypeOnly) m.set(e.name.text, (e.propertyName ?? e.name).text);
          if (m.size > 0) reexports.push({ spec, names: m });
        } else reexports.push({ spec, names: null }); // export * as ns
      }
    }
    if (ts.isCallExpression(n)) {
      if (n.expression.kind === ts.SyntaxKind.ImportKeyword) {
        const s = stringOf(ts, n.arguments[0]);
        if (s !== undefined) imports.push({ spec: s, names: null });
      }
      const name = calleeName(ts, n.expression);
      if (name === 't' && n.arguments.length > 0) addCandidates(n.arguments[0], 't()');
      if (name && LOAD_FNS.has(name)) {
        const s = stringOf(ts, n.arguments[0]);
        if (s !== undefined) loads.add(s);
      }
      if (name === 'preloadCatalogs' && n.arguments[0]) {
        const a = n.arguments[0];
        if (ts.isArrayLiteralExpression(a)) for (const el of a.elements) { const s = stringOf(ts, el); if (s) loads.add(s); }
        else if (ts.isIdentifier(a)) loopedArrays.push(a.text);
      }
      // NAME.map((c) => loadCatalog(c)) / NAME.forEach(...)
      if (ts.isPropertyAccessExpression(n.expression) && ['map', 'forEach'].includes(n.expression.name.text)
        && ts.isIdentifier(n.expression.expression)) {
        const fn = n.arguments[0];
        const body = fn && (ts.isArrowFunction(fn) || ts.isFunctionExpression(fn)) ? fn.body : undefined;
        let callsLoad = false;
        const scan = (x) => { if (ts.isCallExpression(x) && LOAD_FNS.has(calleeName(ts, x.expression) ?? '')) callsLoad = true; ts.forEachChild(x, scan); };
        if (body) scan(body);
        if (callsLoad) loopedArrays.push(n.expression.expression.text);
      }
    }
    if (ts.isVariableDeclaration(n) && ts.isIdentifier(n.name) && n.initializer) {
      let init = n.initializer;
      while (ts.isAsExpression(init) || ts.isSatisfiesExpression?.(init)) init = init.expression;
      if (ts.isArrayLiteralExpression(init)) {
        arrays.set(n.name.text, init.elements.map((el) => stringOf(ts, el)).filter((s) => s !== undefined));
      }
    }
    if (ts.isPropertyAssignment(n)) {
      const key = ts.isIdentifier(n.name) || ts.isStringLiteral(n.name) ? n.name.text : undefined;
      if (key && ID_PROPERTY.test(key) && !/testid$/i.test(key)) addCandidates(n.initializer, `property ${key}`);
    }
    if (ts.isStringLiteral(n) || ts.isNoSubstitutionTemplateLiteral(n)) {
      literals.push({ id: n.text, line: lineOf(n), node: n });
    }
    ts.forEachChild(n, visit);
  };
  visit(sf);
  for (const name of loopedArrays) for (const s of arrays.get(name) ?? []) loads.add(s);

  return {
    imports,
    reexports,
    loads,
    candidates: candidates.filter((c) => !ignored(c.line)),
    literals: literals.filter((l) => !ignored(l.line)).map(({ id, line, node }) => ({ id, line, node, direct: tArgs.has(node) })),
    sf,
  };
}

/**
 * Table entries: an id-shaped literal that sits in an object or array literal
 * where at least two sibling values are en ids is a message id too
 * (`{ ok: 'x-y-ok', err: 'x-y-err' }`, `['a-b-c', 'a-b-d']`).
 */
export function tableCandidates(ts, literals, enIds) {
  const out = [];
  const byParent = new Map();
  for (const l of literals) {
    let p = l.node.parent;
    let holder = l.node;
    if (p && ts.isPropertyAssignment(p) && p.initializer === l.node) { holder = p; p = p.parent; }
    if (!p || !(ts.isObjectLiteralExpression(p) || ts.isArrayLiteralExpression(p))) continue;
    if (holder !== l.node && ts.isPropertyAssignment(holder) && holder.name === l.node) continue;
    if (!byParent.has(p)) byParent.set(p, []);
    byParent.get(p).push(l);
  }
  for (const group of byParent.values()) {
    const known = group.filter((l) => enIds.has(l.id)).length;
    if (known < 2) continue;
    for (const l of group) if (!enIds.has(l.id) && ID_SHAPE.test(l.id)) out.push({ id: l.id, line: l.line, why: 'table entry' });
  }
  return out;
}

/** Resolve a relative import specifier to a source file in `files`, or undefined. */
function resolveImport(fromFile, spec, fileSet) {
  if (!spec.startsWith('.')) return undefined;
  const base = resolve(dirname(fromFile), spec.split('?')[0]);
  const tries = [base, `${base}.ts`, `${base}.tsx`, join(base, 'index.ts'), join(base, 'index.tsx')];
  if (base.endsWith('.js')) tries.push(base.replace(/\.js$/, '.ts'), base.replace(/\.js$/, '.tsx'));
  return tries.find((p) => fileSet.has(p));
}

/** Greatest-fixpoint of G over the import graph. Returns `file -> Set<module>`. */
export function guaranteedLoads(allFiles, info, allModulesIter, entries) {
  const allModules = [...allModulesIter];
  const fileSet = new Set(allFiles);
  const importers = new Map(allFiles.map((f) => [f, new Set()]));
  // An import through a barrel (`export { X } from './x'`) is an edge to the
  // file that defines the imported name, not to every file the barrel lists.
  const link = (from, to, names, seen) => {
    if (!to || to === from || seen.has(to)) return;
    seen.add(to);
    importers.get(to).add(from);
    for (const r of info.get(to).reexports) {
      const next = resolveImport(to, r.spec, fileSet);
      if (names === null) { link(from, next, null, seen); continue; }
      if (r.names === null) { link(from, next, names, seen); continue; }
      const picked = names.filter((nm) => r.names.has(nm)).map((nm) => r.names.get(nm));
      if (picked.length > 0) link(from, next, picked, seen);
    }
  };
  for (const f of allFiles) {
    for (const { spec, names } of info.get(f).imports) link(f, resolveImport(f, spec, fileSet), names, new Set());
  }
  // Only what the entry reaches is ever rendered; a dead importer must not
  // narrow what its (live) children can rely on.
  const live = reachable(allFiles, importers, entries);
  const files = allFiles.filter((f) => live.has(f));
  for (const f of files) importers.set(f, new Set([...importers.get(f)].filter((i) => live.has(i))));
  const own = (f) => new Set([...ALWAYS, ...info.get(f).loads]);
  const G = new Map(files.map((f) => [f, importers.get(f).size === 0 ? own(f) : new Set(allModules)]));
  let changed = true;
  while (changed) {
    changed = false;
    for (const f of files) {
      const imps = [...importers.get(f)];
      if (imps.length === 0) continue;
      let inter = new Set(G.get(imps[0]));
      for (const i of imps.slice(1)) { const gi = G.get(i); inter = new Set([...inter].filter((m) => gi.has(m))); }
      const next = new Set([...own(f), ...inter]);
      if (next.size !== G.get(f).size) { G.set(f, next); changed = true; }
    }
  }
  // U(F): catalogs loaded by F or by ANY file that imports it, transitively.
  const U = new Map(files.map((f) => [f, own(f)]));
  changed = true;
  while (changed) {
    changed = false;
    for (const f of files) {
      const u = U.get(f);
      const before = u.size;
      for (const i of importers.get(f)) for (const m of U.get(i)) u.add(m);
      if (u.size !== before) changed = true;
    }
  }
  return { G, U, importers, live };
}

/** Files reachable from `entries` along import edges (`importers` reversed). */
export function reachable(files, importers, entries) {
  const children = new Map(files.map((f) => [f, []]));
  for (const [to, froms] of importers) for (const from of froms) children.get(from).push(to);
  const seen = new Set();
  const stack = entries.filter((e) => children.has(e));
  while (stack.length > 0) {
    const f = stack.pop();
    if (seen.has(f)) continue;
    seen.add(f);
    stack.push(...children.get(f));
  }
  return seen;
}

/** The app's entry module (index.html loads /src/main.tsx). */
export const ENTRIES = ['main.tsx'];

/** Run both checks over `srcDir`. */
export async function checkLoaded(ts, { srcDir, enDir, repoRoot, allowlist = [] }) {
  const files = (await walk(srcDir, (n) => /\.tsx?$/.test(n) && !/\.(test|spec)\.tsx?$/.test(n) && !n.endsWith('.d.ts')))
    .filter((f) => {
      const rel = relative(srcDir, f).split(sep).join('/');
      return !rel.startsWith('i18n/') && !rel.startsWith('test/');
    });
  const byModule = await enModules(enDir);
  const index = idIndex(byModule);
  const enIds = new Set(index.keys());
  const info = new Map();
  for (const f of files) {
    const a = analyzeFile(ts, f, await readFile(f, 'utf8'));
    if (UNGUARANTEED_LOADERS.has(relative(srcDir, f).split(sep).join('/'))) a.loads = new Set();
    info.set(f, a);
  }
  // A file nothing reaches from the entry is never shown; it is checked once
  // something mounts it.
  const stale = [];
  for (const entry of allowlist) {
    const f = join(repoRoot, entry.file);
    const a = info.get(f);
    const used = a && a.literals.some((l) => entry.catalogs.some((c) => byModule.get(c)?.has(l.id)));
    if (!a || !used || typeof entry.reason !== 'string' || entry.reason.trim() === '') { stale.push(entry); continue; }
    for (const c of entry.catalogs) a.loads.add(c);
  }
  const { G, U, live } = guaranteedLoads(files, info, byModule.keys(), ENTRIES.map((e) => join(srcDir, e)));

  const missing = [];
  const unloaded = [];
  for (const f of files) {
    if (!live.has(f)) continue;
    const a = info.get(f);
    const rel = relative(repoRoot, f).split(sep).join('/');
    const seen = new Set();
    for (const c of [...a.candidates, ...tableCandidates(ts, a.literals, enIds)]) {
      const k = `${c.id}:${c.line}`;
      if (seen.has(k) || enIds.has(c.id)) continue;
      seen.add(k);
      missing.push({ id: c.id, file: rel, line: c.line, why: c.why });
    }
    const g = G.get(f);
    const u = U.get(f);
    const reported = new Set();
    for (const l of a.literals) {
      const mods = index.get(l.id);
      if (!mods || !l.id.includes('-') || reported.has(l.id)) continue;
      // Shown here (`t('id')`): the catalog must be loaded on every way in.
      // Only named here (a table value rendered by an importer): some file on
      // the way in must load it.
      const have = l.direct ? g : u;
      if ([...mods].some((m) => have.has(m))) continue;
      reported.add(l.id);
      unloaded.push({ id: l.id, file: rel, line: l.line, modules: [...mods] });
    }
  }
  return { missing, unloaded, stale, fileCount: live.size };
}

