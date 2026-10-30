// Tests for the analysis behind check-loaded.mjs (t112).
// Run: node --test scripts/i18n/*.test.mjs   (needs `typescript` in apps/web/node_modules)

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { mkdtemp, mkdir, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { checkLoaded } from './lib/loaded.mjs';

const repoRoot = fileURLToPath(new URL('../../', import.meta.url));
const ts = createRequire(join(repoRoot, 'apps/web/package.json'))('typescript');

/** Write a fake app (`src/…`, `en/…`) and run the check on it. */
async function run(files, allowlist = []) {
  const root = await mkdtemp(join(tmpdir(), 'mw-loaded-'));
  try {
    for (const [path, body] of Object.entries(files)) {
      await mkdir(dirname(join(root, path)), { recursive: true });
      await writeFile(join(root, path), body);
    }
    return await checkLoaded(ts, { srcDir: join(root, 'src'), enDir: join(root, 'en'), repoRoot: root, allowlist });
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}

// The t98 shape: the nav table shows a label from a catalog that only the
// section's own screen loads.
const navApp = (navCatalog) => ({
  'en/common.ftl': 'common-ok-label = OK\n',
  'en/shell.ftl': `shell-nav-home = Home\n${navCatalog === 'shell' ? 'shell-nav-features = Features\n' : ''}`,
  'en/features.ftl': `features-page-title = Feature profiles\n${navCatalog === 'features' ? 'shell-nav-features = Features\n' : ''}`,
  'src/main.tsx': "import { Shell } from './Shell.tsx';\nexport default Shell;\n",
  'src/Shell.tsx': [
    "import { loadCatalog, t } from './i18n';",
    "import { SECTIONS } from './sections.ts';",
    "export function Shell() { loadCatalog('shell'); return SECTIONS.map((s) => s.label()); }",
  ].join('\n'),
  'src/sections.ts': [
    "import { t } from './i18n';",
    'export const SECTIONS = [',
    "  { id: 'home', label: () => t('shell-nav-home') },",
    "  { id: 'features', label: () => t('shell-nav-features'), load: () => import('./Features.tsx') },",
    '];',
  ].join('\n'),
  'src/Features.tsx': [
    "import { loadCatalog, t } from './i18n';",
    "import { SECTIONS } from './sections.ts';",
    "export function Features() { loadCatalog('features'); return [t('features-page-title'), SECTIONS.length]; }",
  ].join('\n'),
});

test('a nav label from a catalog only its child screen loads is reported', async () => {
  const r = await run(navApp('features'));
  assert.deepEqual(r.unloaded.map((u) => [u.id, u.file, u.modules]), [['shell-nav-features', 'src/sections.ts', ['features']]]);
  assert.deepEqual(r.missing, []);
});

test('the same label in the catalog the shell loads passes', async () => {
  const r = await run(navApp('shell'));
  assert.deepEqual(r.unloaded, []);
  assert.deepEqual(r.missing, []);
});

test('ids missing from en are reported from t(), conditionals, id properties and tables', async () => {
  const r = await run({
    'en/common.ftl': 'common-a-one = A\ncommon-a-two = B\ncommon-a-three = C\n',
    'src/main.tsx': [
      "import { t } from './i18n';",
      "const TABLE = { one: 'common-a-one', two: 'common-a-two', four: 'common-a-four' };",
      "const item = { labelId: 'common-a-five', testId: 'common-a-six' };",
      "export const x = [t('common-a-seven'), t(Math.random() > 0.5 ? 'common-a-one' : 'common-a-eight'), TABLE, item];",
    ].join('\n'),
  });
  assert.deepEqual(r.missing.map((m) => m.id).sort(), ['common-a-eight', 'common-a-five', 'common-a-four', 'common-a-seven']);
});

test('an import through a barrel links only to the file that defines the name', async () => {
  const r = await run({
    'en/common.ftl': 'common-x-y = x\n',
    'en/rules.ftl': 'rules-builder-title = Rules\n',
    'src/main.tsx': "import { Settings } from './rules/index.ts';\nexport default Settings;\n",
    'src/rules/index.ts': "export { Settings } from './Settings.tsx';\nexport { Builder } from './Builder.tsx';\n",
    'src/rules/Settings.tsx': "import { loadCatalog } from '../i18n';\nimport { Builder } from './Builder.tsx';\nexport function Settings() { loadCatalog('rules'); return Builder(); }\n",
    'src/rules/Builder.tsx': "import { t } from '../i18n';\nexport function Builder() { return t('rules-builder-title'); }\n",
  });
  assert.deepEqual(r.unloaded, []);
});

test('a shared part used by a screen that does not load its catalog is reported; the allowlist accepts it with a reason', async () => {
  const app = {
    'en/common.ftl': 'common-x-y = x\n',
    'en/dialog.ftl': 'dialog-cancel-label = Cancel\n',
    'src/main.tsx': "import { A } from './A.tsx';\nimport { B } from './B.tsx';\nexport default [A, B];\n",
    'src/A.tsx': "import { loadCatalog } from './i18n';\nimport { Dialog } from './Dialog.tsx';\nexport function A() { loadCatalog('dialog'); return Dialog(); }\n",
    'src/B.tsx': "import { Dialog } from './Dialog.tsx';\nexport function B() { return Dialog(); }\n",
    'src/Dialog.tsx': "import { t } from './i18n';\nexport function Dialog() { return t('dialog-cancel-label'); }\n",
  };
  const r = await run(app);
  assert.deepEqual(r.unloaded.map((u) => u.id), ['dialog-cancel-label']);
  const ok = await run(app, [{ file: 'src/Dialog.tsx', catalogs: ['dialog'], reason: 'test' }]);
  assert.deepEqual(ok.unloaded, []);
  assert.deepEqual(ok.stale, []);
  const stale = await run(app, [{ file: 'src/B.tsx', catalogs: ['dialog'], reason: 'test' }]);
  assert.equal(stale.stale.length, 1);
});

test('a file nothing reaches from main.tsx is not checked', async () => {
  const r = await run({
    'en/common.ftl': 'common-x-y = x\n',
    'en/about.ftl': 'about-tp-title = Third parties\n',
    'src/main.tsx': 'export default 1;\n',
    'src/about/List.tsx': "import { t } from '../i18n';\nexport const L = () => t('about-tp-title');\n",
  });
  assert.deepEqual(r.unloaded, []);
});
