// Tests for the AST analysis behind no-hardcoded-strings.mjs.
// Run: node --test scripts/i18n/*.test.mjs   (needs `typescript` in apps/web/node_modules)

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { analyzeSource } from './lib/hardcoded.mjs';

const repoRoot = fileURLToPath(new URL('../../', import.meta.url));
const ts = createRequire(join(repoRoot, 'apps/web/package.json'))('typescript');

const kinds = (fileName, src) => analyzeSource(ts, fileName, src).map((f) => [f.kind, f.text]);

test('multi-line JSX text is found, with its first line', () => {
  const src = [
    'export const A = () => (',
    '  <p>',
    '    These plugins were admitted',
    '    without a verified signature.',
    '  </p>',
    ');',
  ].join('\n');
  const [f] = analyzeSource(ts, 'a.tsx', src);
  assert.equal(f.kind, 'jsx-text');
  assert.equal(f.text, 'These plugins were admitted without a verified signature.');
  assert.equal(f.line, 3);
  assert.equal(f.multiline, true);
});

test('JSX text next to an expression and string children are found', () => {
  assert.deepEqual(kinds('a.tsx', 'const A = (n) => <p>Hello {n} there{"Bye now"}</p>;'), [
    ['jsx-text', 'Hello'],
    ['jsx-text', 'there'],
    ['jsx-text', 'Bye now'],
  ]);
});

test('t() calls, glyphs, entities and message ids are not findings', () => {
  const src = `const A = () => (
    <div aria-label={t('mail-x')} title="mail-outbox-sent">
      {t('common-ok')} · &lt; → 3
    </div>
  );`;
  assert.deepEqual(kinds('a.tsx', src), []);
});

test('user-visible attributes, including ternary and ?? branches', () => {
  const src = `const A = (x) => <>
    <img alt="Company logo" />
    <input placeholder={'Search mail'} />
    <button title={x ? 'Open' : 'Close'} aria-label={x ?? 'Menu'} class="btn-primary" />
  </>;`;
  assert.deepEqual(kinds('a.tsx', src), [
    ['attr:alt', 'Company logo'],
    ['attr:placeholder', 'Search mail'],
    ['attr:title', 'Open'],
    ['attr:title', 'Close'],
    ['attr:aria-label', 'Menu'],
  ]);
});

test('label/description/title properties in .ts tables', () => {
  const src = `export const T = [
    { id: 'a', label: 'Fix grammar', description: \`Rewrite \${x} text\` },
    { id: 'b', label: 'tasks-priority-high', tracking: { label: '0.06em' } },
    { id: 'c', name: 'Not a UI property' },
  ];`;
  assert.deepEqual(kinds('t.ts', src), [
    ['prop:label', 'Fix grammar'],
    ['prop:description', 'Rewrite text'],
  ]);
});

test('alert/confirm/prompt and Notification text', () => {
  const src = `confirm('Delete this?'); window.alert("Saved"); new Notification('New message', {}); console.log('Debug text');`;
  assert.deepEqual(kinds('t.ts', src), [
    ['call:confirm', 'Delete this?'],
    ['call:alert', 'Saved'],
    ['call:Notification', 'New message'],
  ]);
});

test('browser-locale formatting is found; explicit locales and time-zone lookups are not', () => {
  const src = `
    d.toLocaleString();
    d.toLocaleDateString(undefined, { month: 'short' });
    new Intl.DateTimeFormat(undefined, { hour: 'numeric' });
    new Intl.NumberFormat();
    Intl.DateTimeFormat([]);
    new Intl.NumberFormat(locale, {});
    Intl.DateTimeFormat().resolvedOptions().timeZone;
  `;
  assert.deepEqual(kinds('t.ts', src), [
    ['locale-format', 'toLocaleString()'],
    ['locale-format', 'toLocaleDateString()'],
    ['locale-format', 'Intl.DateTimeFormat(undefined)'],
    ['locale-format', 'Intl.NumberFormat()'],
    ['locale-format', 'Intl.DateTimeFormat([])'],
  ]);
});

test('i18n-ignore with a reason accepts a site; without a reason it does not', () => {
  const src = [
    'const a = <p>Mailwoman</p>; // i18n-ignore: product name',
    '// i18n-ignore: API key prefix shown as is',
    'const b = <code>mwk_ prefix</code>;',
    'const c = <p>Kept</p>; // i18n-ignore:',
  ].join('\n');
  assert.deepEqual(kinds('a.tsx', src), [['jsx-text', 'Kept']]);
});
