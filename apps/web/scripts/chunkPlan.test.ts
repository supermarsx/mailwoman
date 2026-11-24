// node --experimental-strip-types --test scripts/chunkPlan.test.ts (pnpm run size:test)
import test from 'node:test';
import assert from 'node:assert/strict';
import {
  CHUNK_LAYERS, ROOTS, chunkPlan, moduleKey, packageOf, planChunks, resolveOwners, rootChunk, sectionChunk,
  type ChunkName, type PlanChunk, type PlanGraph,
} from './chunkPlan.ts';

const ROOT = '/repo/apps/web';
const id = (key: string): string => (key.startsWith('\0') || key.startsWith('/') ? key : `${ROOT}/${key}`);

/** A fixture graph: key -> [static imports, dynamic imports]. The first key is the entry. */
function graph(spec: Record<string, [string[], string[]?]>): PlanGraph {
  const keys = Object.keys(spec);
  return {
    getModuleIds: () => keys.map(id),
    getModuleInfo: (full) => {
      const key = keys.find((k) => id(k) === full);
      if (key === undefined) return null;
      const [imports, dynamic = []] = spec[key]!;
      return { importedIds: imports.map(id), dynamicallyImportedIds: dynamic.map(id), isEntry: key === keys[0] };
    },
  };
}
const placed = (g: PlanGraph): Record<string, PlanChunk> => {
  const { chunks } = planChunks(g, ROOT);
  return Object.fromEntries([...chunks].map(([full, chunk]) => [moduleKey(full, ROOT), chunk]));
};

test('module keys are relative to apps/web; packages are found in the pnpm layout', () => {
  assert.equal(moduleKey(`${ROOT}/src/App.tsx`, ROOT), 'src/App.tsx');
  assert.equal(moduleKey('C:\\repo\\apps\\web\\src\\App.tsx', 'C:\\repo\\apps\\web'), 'src/App.tsx');
  assert.equal(moduleKey('\0vite/preload-helper.js', ROOT), '\0vite/preload-helper.js');
  assert.equal(packageOf('node_modules/.pnpm/solid-js@1.9.14/node_modules/solid-js/web/dist/web.js'), 'solid-js');
  assert.equal(packageOf('node_modules/.pnpm/@fluent+bundle@0.18.0/node_modules/@fluent/bundle/esm/index.js'), '@fluent/bundle');
  assert.equal(packageOf('\0/repo/apps/web/node_modules/.pnpm/altcha@3.5.0/node_modules/altcha/dist/external/altcha.js?commonjs-es-import'), 'altcha');
  assert.equal(packageOf('src/App.tsx'), null);
});

test('representative dynamic import targets map to their chunks, each by exactly one rule', () => {
  const expected: Record<string, ChunkName> = {
    'src/screens/Login.tsx': 'login',
    'src/screens/Mailbox.tsx': 'mail',
    'src/screens/Templates.tsx': 'mail-lazy',
    'src/components/Compose.tsx': 'compose',
    'src/components/compose/RichTextEditor.tsx': 'editor',
    'src/screens/Settings.tsx': 'settings',
    'src/screens/Admin/index.tsx': 'admin',
    'src/screens/Admin/Compliance/DpiaTab.tsx': 'admin-section',
    'src/screens/Admin/Dashboard/Dashboard.tsx': 'admin',
    'src/shell/mounts.tsx': 'pim',
    'src/modules/calendar/index.tsx': 'pim',
    'src/modules/keys/index.tsx': 'crypto-ui',
    'src/viewers/PdfViewer.tsx': 'viewers',
    'src/components/properties/MessageProperties.tsx': 'dialogs',
    'src/about/ThirdPartyList.tsx': 'about',
    'src/about/texts/a.json': 'licences',
    'src/brands/paths.finance.generated.ts': 'brand-paths',
    'src/brands/registry.generated.ts': 'brand-index',
    'src/components/StepUpDialog.tsx': 'core-lazy',
    'src/platform/tauri.ts': 'core-lazy',
    'src/modules/passwd/ForcedPasswordChange.tsx': 'rare',
    'node_modules/.pnpm/altcha@3.5.0/node_modules/altcha/dist/external/altcha.js': 'vendor-altcha',
  };
  for (const [key, chunk] of Object.entries(expected)) {
    assert.equal(rootChunk(key), chunk, key);
    const rules = ROOTS.filter(([pattern]) => pattern.test(key));
    assert.ok(rules.length <= 1, `${key} matches ${rules.length} rules`);
  }
  assert.equal(rootChunk('src/components/SomethingNew.tsx'), null);
  // Only dynamic import targets are roots; a rule never claims a helper module.
  assert.equal(rootChunk('src/viewers/linkTrust.ts'), null);
});

test('owner sets resolve by the documented rules', () => {
  const r = (owners: PlanChunk[], pkg: string | null = null): PlanChunk => resolveOwners(new Set(owners), pkg);
  assert.equal(r(['settings']), 'settings');
  assert.equal(r(['mail', 'settings']), 'shared');
  assert.equal(r(['login', 'mail']), 'core');
  assert.equal(r(['settings', 'pim']), 'shared-lazy');
  assert.equal(r(['compose', 'editor']), 'editor');
  assert.equal(r(['compose', 'editor', 'settings']), 'shared-lazy');
  assert.equal(r(['mail', 'mail-lazy']), 'mail', 'a pane and its parent only: the parent');
  assert.equal(r(['mail-lazy', 'settings']), 'shared-lazy', 'a pane shares with another screen: not the parent');
  assert.equal(r(['rare', 'settings']), 'settings', 'a leaf owner yields to a screen');
  assert.equal(r(['rare']), 'rare');
  assert.equal(r(['core-lazy', 'mail']), 'core', 'core-lazy never imports above core');
  assert.equal(r(['dialogs', 'settings']), 'shared', 'dialogs never import above shared');
  assert.equal(r(['dialogs', 'login', 'mail']), 'core', 'the lowest layer wins');
  assert.equal(r(['mail'], 'prosemirror-view'), 'editor', 'ProseMirror always goes to the editor');
  // The admin console.
  assert.equal(r(['admin', 'admin-section.Users']), 'admin', 'shell and a section: the shell');
  assert.equal(r(['admin-section.Users']), 'admin-section.Users');
  assert.equal(r(['admin-section.Users', 'admin-section.Roles']), 'admin-common', 'sections only, not the shell');
  assert.equal(r(['admin', 'settings']), 'admin-lazy');
  assert.equal(r(['admin', 'mail']), 'core', 'the shell and the mailbox: core, never shared');
  assert.equal(r(['admin-section.Users', 'mail']), 'shared', 'a section and the mailbox: shared, off the shell');
  assert.equal(r(['admin-section.Users', 'settings']), 'shared-lazy');
  assert.equal(r(['admin-section.Users', 'login']), 'core');
});

test('one chunk per admin section, named by its path', () => {
  assert.equal(sectionChunk('src/screens/Admin/Users.tsx'), 'admin-section.Users');
  assert.equal(sectionChunk('src/screens/Admin/Compliance/index.tsx'), 'admin-section.Compliance');
  assert.equal(sectionChunk('src/screens/Admin/Privacy/OverviewTab.tsx'), 'admin-section.Privacy-OverviewTab');
});

test('layers: every chunk has one, the entry is the top', () => {
  const top = Math.max(...Object.values(CHUNK_LAYERS));
  assert.equal(CHUNK_LAYERS.entry, top);
  assert.ok(CHUNK_LAYERS['vendor-core'] < CHUNK_LAYERS.core);
  assert.ok(CHUNK_LAYERS.core < CHUNK_LAYERS.shared && CHUNK_LAYERS.shared < CHUNK_LAYERS.mail);
});

test('a fixture app is placed by ownership', () => {
  const g = graph({
    'index.html': [['src/main.tsx']],
    'src/main.tsx': [['src/App.tsx', 'node_modules/.pnpm/solid-js@1/node_modules/solid-js/dist/solid.js']],
    'src/App.tsx': [['src/api/http.ts', 'src/boot/idlePrefetch.ts'], ['src/screens/Login.tsx', 'src/screens/Mailbox.tsx', 'src/modules/passwd/ForcedPasswordChange.tsx']],
    'src/boot/idlePrefetch.ts': [['src/screens/mailboxPaneLoaders.ts']],
    'src/screens/mailboxPaneLoaders.ts': [[], ['src/screens/Settings.tsx', 'src/components/Compose.tsx']],
    'src/api/http.ts': [[], ['src/components/StepUpDialog.tsx']],
    'node_modules/.pnpm/solid-js@1/node_modules/solid-js/dist/solid.js': [[]],
    'src/components/StepUpDialog.tsx': [['src/components/Button.tsx']],
    'src/screens/Login.tsx': [['src/brands/registry.ts', 'src/screens/login.css.ts']],
    'src/screens/login.css.ts': [['src/screens/login.css.ts.vanilla.css']],
    'src/screens/login.css.ts.vanilla.css': [[]],
    'src/screens/Mailbox.tsx': [['src/brands/registry.ts', 'src/components/MessageList.tsx', 'src/components/Button.tsx', 'src/screens/mailboxPaneLoaders.ts'], ['src/screens/Templates.tsx']],
    'src/screens/Templates.tsx': [['src/components/MessageList.tsx']],
    'src/components/MessageList.tsx': [[]],
    'src/components/Button.tsx': [[]],
    'src/brands/registry.ts': [[]],
    'src/screens/Settings.tsx': [['src/components/Button.tsx', 'src/components/Table.tsx']],
    'src/components/Compose.tsx': [['src/components/Table.tsx', 'node_modules/.pnpm/prosemirror-view@1/node_modules/prosemirror-view/dist/index.js']],
    'node_modules/.pnpm/prosemirror-view@1/node_modules/prosemirror-view/dist/index.js': [[]],
    'src/components/Table.tsx': [[]],
    'src/modules/passwd/ForcedPasswordChange.tsx': [['src/screens/Settings.tsx', 'src/components/PasswordRules.tsx']],
    'src/components/PasswordRules.tsx': [[]],
  });
  assert.deepEqual(placed(g), {
    'index.html': 'entry',
    'src/main.tsx': 'entry',
    'src/App.tsx': 'entry',
    'src/boot/idlePrefetch.ts': 'entry',
    'src/screens/mailboxPaneLoaders.ts': 'entry',
    'src/api/http.ts': 'core',
    'node_modules/.pnpm/solid-js@1/node_modules/solid-js/dist/solid.js': 'vendor-core',
    'src/components/StepUpDialog.tsx': 'core-lazy',
    'src/components/Button.tsx': 'core', // core-lazy shares it with mail and settings
    'src/screens/Login.tsx': 'login',
    'src/screens/login.css.ts': 'login', // a style module stays with its only owner
    'src/screens/login.css.ts.vanilla.css': 'login',
    'src/brands/registry.ts': 'core', // sign-in shares it
    'src/screens/Mailbox.tsx': 'mail',
    'src/screens/Templates.tsx': 'mail-lazy',
    'src/components/MessageList.tsx': 'mail', // a pane and its parent
    'src/screens/Settings.tsx': 'settings', // the leaf `rare` also imports it
    'src/components/Table.tsx': 'shared-lazy',
    'src/components/Compose.tsx': 'compose',
    'node_modules/.pnpm/prosemirror-view@1/node_modules/prosemirror-view/dist/index.js': 'editor',
    'src/modules/passwd/ForcedPasswordChange.tsx': 'rare',
    'src/components/PasswordRules.tsx': 'rare',
  });
});

test('a dynamic import target no rule names is reported and built into `unplanned`', () => {
  const g = graph({
    'index.html': [['src/main.tsx']],
    'src/main.tsx': [[], ['src/components/NewLazyThing.tsx']],
    'src/components/NewLazyThing.tsx': [['src/components/NewHelper.ts']],
    'src/components/NewHelper.ts': [[]],
  });
  assert.deepEqual(planChunks(g, ROOT).unrootedDynamicTargets, ['src/components/NewLazyThing.tsx']);
  const { manualChunks } = chunkPlan(ROOT);
  assert.equal(manualChunks(id('src/components/NewLazyThing.tsx'), g), 'unplanned');
  assert.equal(manualChunks(id('src/components/NewHelper.ts'), g), 'unplanned');
  assert.equal(manualChunks(id('src/main.tsx'), g), 'entry');
});

test('a dynamic import of a module the entry already loads is not a root', () => {
  const g = graph({
    'index.html': [['src/main.tsx']],
    'src/main.tsx': [['src/api/jmap.ts'], ['src/api/jmap.ts']],
    'src/api/jmap.ts': [[]],
  });
  const plan = planChunks(g, ROOT);
  assert.deepEqual(plan.unrootedDynamicTargets, []);
  assert.equal(plan.chunks.get(id('src/api/jmap.ts')), 'core');
});
