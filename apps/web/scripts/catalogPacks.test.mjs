// node --experimental-strip-types --test scripts/catalogPacks.test.mjs
import test from 'node:test';
import assert from 'node:assert/strict';
import {readdirSync, readFileSync} from 'node:fs';
import {join, relative} from 'node:path';
import {fileURLToPath} from 'node:url';
import {FluentBundle, FluentResource} from '@fluent/bundle';
import {
  ADMIN_NAMESPACES, ADMIN_PACK_NAMESPACES_LOADED_OUTSIDE_ADMIN, APP_RESIDENT_ADMIN_NAMESPACES,
  buildPacks, isAppCatalogPack, mergeEnglishFallback, packKind, packNamespaces, readLocaleSources, splitEntries,
} from './catalogPacks.ts';

const root = fileURLToPath(new URL('../', import.meta.url));
const sources = readLocaleSources(join(root, 'locales'));
const allNamespaces = [...new Set([...sources.values()].flatMap((files) => [...files.keys()]))].sort();

test('splitEntries keeps patterns, selectors, attributes and terms whole and drops comments', () => {
  const entries = splitEntries([
    '# comment', '## group', '-brand = Mailwoman', '',
    'one = One', 'two =', '    Line one', '', '    Line three', '    .title = Attr',
    'count = { $n ->', '    [one] a one', '   *[other] { $n } others', '  }', '', 'last = L',
  ].join('\r\n'));
  assert.deepEqual([...entries.keys()], ['-brand', 'one', 'two', 'count', 'last']);
  assert.equal(entries.get('two'), 'two =\n    Line one\n\n    Line three\n    .title = Attr');
  assert.equal(entries.get('count'), 'count = { $n ->\n    [one] a one\n   *[other] { $n } others\n  }');
  assert.equal(entries.get('one'), 'one = One');
});

test('merge appends English text only for ids the locale lacks in every namespace', () => {
  const fixture = new Map([
    ['en', new Map([['a', 'x = EN x\ny = EN y\n'], ['b', 'z = EN z\nonly = EN only\n'], ['c', 'c1 = EN c1\n']])],
    ['de', new Map([['a', 'x = DE x\n'], ['b', 'y = DE y\nz = DE z']])],
  ]);
  const merged = mergeEnglishFallback(fixture, 'de');
  assert.deepEqual([...merged.keys()], ['a', 'b', 'c']);
  assert.equal(merged.get('a'), 'x = DE x\n', 'y is defined in de/b, so it is not copied');
  assert.equal(merged.get('b'), 'y = DE y\nz = DE z\n\nonly = EN only\n');
  assert.equal(merged.get('c'), '\nc1 = EN c1\n', 'an English-only namespace is present in the locale pack');
  assert.equal(mergeEnglishFallback(fixture, 'en').get('a'), 'x = EN x\ny = EN y\n');
});

test('every namespace is in exactly one pack of every locale, and the lists have no stale or unclassified names', () => {
  assert.ok(allNamespaces.length >= 120);
  for (const name of ADMIN_NAMESPACES) assert.ok(allNamespaces.includes(name), `ADMIN_NAMESPACES lists ${name}, which has no .ftl file`);
  assert.equal(new Set(ADMIN_NAMESPACES).size, ADMIN_NAMESPACES.length);
  for (const name of APP_RESIDENT_ADMIN_NAMESPACES) { assert.ok(allNamespaces.includes(name)); assert.equal(packKind(name), 'app'); }
  const unclassified = allNamespaces.filter((name) => name.startsWith('admin') && packKind(name) === 'app' && !APP_RESIDENT_ADMIN_NAMESPACES.includes(name));
  assert.deepEqual(unclassified, [], 'admin* namespaces must be in ADMIN_NAMESPACES or APP_RESIDENT_ADMIN_NAMESPACES');
  assert.deepEqual(ADMIN_NAMESPACES.filter((name) => !name.startsWith('admin')), []);
  const packs = buildPacks(sources);
  assert.equal(packs.length, sources.size * 2);
  for (const locale of sources.keys()) {
    const mine = packs.filter((pack) => pack.locale === locale);
    const app = Object.keys(mine.find((pack) => pack.kind === 'app').files);
    const admin = Object.keys(mine.find((pack) => pack.kind === 'admin').files);
    assert.deepEqual(app.filter((name) => admin.includes(name)), [], `${locale}: a namespace is in both packs`);
    assert.deepEqual([...app, ...admin].sort(), packNamespaces(sources, locale), `${locale}: packs do not cover the namespaces`);
    assert.deepEqual(packNamespaces(sources, locale), allNamespaces, `${locale}: every locale carries every namespace`);
    assert.ok(admin.every((name) => name.startsWith('admin')) && app.every((name) => packKind(name) === 'app'));
  }
});

function sourceFiles(dir) {
  return readdirSync(dir, {withFileTypes: true}).flatMap((entry) => {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) return sourceFiles(full);
    return /\.tsx?$/.test(entry.name) && !/\.test\.tsx?$/.test(entry.name) ? [full] : [];
  });
}

test('admin-pack namespaces are loaded by name only inside the admin console, except the reviewed list', () => {
  const outside = new Set();
  const adminDir = join(root, 'src/screens/Admin');
  for (const file of sourceFiles(join(root, 'src'))) {
    if (!relative(adminDir, file).startsWith('..')) continue;
    for (const match of readFileSync(file, 'utf8').matchAll(/(?:loadCatalog|useCatalog)\(\s*['"]([a-z0-9-]+)['"]/g)) outside.add(match[1]);
  }
  const offenders = [...outside].filter((name) => packKind(name) === 'admin' && !ADMIN_PACK_NAMESPACES_LOADED_OUTSIDE_ADMIN.includes(name));
  assert.deepEqual(offenders, [], 'a screen outside the admin console loads an admin-pack namespace: classify it as app or list it');
  for (const name of ADMIN_PACK_NAMESPACES_LOADED_OUTSIDE_ADMIN) assert.ok(outside.has(name), `${name} is no longer loaded outside the admin console; remove it from the list`);
  for (const name of outside) assert.ok(allNamespaces.includes(name), `loaded namespace ${name} has no .ftl file`);
});

test('app pack file names are recognised for precache and admin packs are not', () => {
  assert.ok(isAppCatalogPack('/assets/catalog.en.app-AbC_123.json'));
  assert.ok(isAppCatalogPack('/assets/catalog.pt-BR.app-AbC_123.json'));
  assert.ok(!isAppCatalogPack('/assets/catalog.en.admin-AbC_123.json'));
  assert.ok(!isAppCatalogPack('/assets/other.en.app-AbC_123.json'));
});

// Reference = the registry's lookup today: the id comes from the locale's bundle
// when any loaded namespace of the locale defines it, otherwise from English.
const bundleOf = (locale, texts) => {
  const bundle = new FluentBundle(locale, {useIsolating: false});
  for (const text of texts) bundle.addResource(new FluentResource(text), {allowOverrides: false});
  return bundle;
};
const render = (bundle, id) => {
  const message = bundle.getMessage(id);
  const out = {};
  for (const [key, pattern] of [['value', message.value], ...Object.entries(message.attributes)]) {
    if (pattern == null) continue;
    const errors = [];
    out[key] = [bundle.formatPattern(pattern, undefined, errors), errors.length];
  }
  return out;
};
const ids = (bundle) => [...bundle._messages.keys()];

test('every id that resolves today resolves to the same strings from the packs', () => {
  const packs = buildPacks(sources);
  const english = bundleOf('en', [...sources.get('en').values()]);
  let compared = 0, errorCountDifferences = 0;
  const improved = [];
  for (const locale of sources.keys()) {
    const reference = bundleOf(locale, [...sources.get(locale).values()]);
    const fromPacks = bundleOf(locale, packs.filter((pack) => pack.locale === locale).flatMap((pack) => Object.values(pack.files)));
    const universe = new Set([...ids(reference), ...ids(english)]);
    for (const id of universe) {
      const today = render(reference.hasMessage(id) ? reference : english, id);
      assert.ok(fromPacks.hasMessage(id), `${locale}: ${id} resolves today but not from the packs`);
      const packed = render(fromPacks, id);
      for (const key of Object.keys(today)) {
        if (packed[key]?.[0] === today[key][0]) { if (packed[key][1] !== today[key][1]) errorCountDifferences++; continue; }
        // The one allowed difference: today a locale message that references a message or term the locale
        // lacks renders the unresolved reference (`{id}`, resolver error); the pack has the English text there.
        assert.ok(today[key][1] > packed[key]?.[1], `${locale}: ${id} ${key}: ${JSON.stringify(today[key])} vs ${JSON.stringify(packed[key])}`);
        improved.push(`${locale}/${id}`);
      }
      assert.deepEqual(Object.keys(packed), Object.keys(today), `${locale}: ${id} attribute set`);
      compared++;
    }
    assert.equal(ids(fromPacks).length, universe.size, `${locale}: the packs define ids that resolve nowhere today`);
  }
  assert.ok(compared > 20000, `compared ${compared} id/locale pairs`);
  console.log(`catalogPacks: ${compared} id/locale pairs compared; ${improved.length} differ only where today's render has an unresolved reference (${improved.slice(0, 5).join(', ')}); ${errorCountDifferences} equal strings with different error counts`);
});
