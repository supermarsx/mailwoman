// node --test scripts/check-load-budget.test.mjs (pnpm run size:test)
// A valid fixture graph passes; each rule has a negative control.
import test from 'node:test';
import assert from 'node:assert/strict';
import { checkLoadBudget, invalidated, staticCycles } from './check-load-budget.mjs';

const policy = {
  layers: { 'vendor-core': 0, 'brand-paths': 0, core: 1, shared: 3, login: 7, mail: 7, settings: 7, rare: 9, entry: 10 },
  leaf: ['rare'], data: ['brand-paths'], vendor: ['vendor-core'], entryImportExceptions: ['mail'],
};
const chunk = (plan, modules, imports = [], dynamicImports = [], extra = {}) => ({ plan, isEntry: false, isDynamicEntry: false, imports, dynamicImports, css: [], modules, ...extra });

function fixture() {
  const chunks = {
    'assets/index-a.js': chunk('entry', ['index.html', 'src/main.tsx'], ['assets/core-a.js', 'assets/vendor-core-a.js'], ['assets/login-a.js', 'assets/mail-a.js', 'assets/rare-a.js'], { isEntry: true }),
    'assets/vendor-core-a.js': chunk('vendor-core', ['node_modules/solid-js/dist/solid.js']),
    'assets/core-a.js': chunk('core', ['src/api/basePath.ts'], ['assets/vendor-core-a.js'], ['assets/brand-paths-a.js'], { css: ['assets/core-a.css'] }),
    'assets/brand-paths-a.js': chunk('brand-paths', ['src/brands/paths.finance.generated.ts'], [], [], { isDynamicEntry: true }),
    'assets/shared-a.js': chunk('shared', ['src/components/Button.tsx'], ['assets/core-a.js', 'assets/vendor-core-a.js']),
    'assets/login-a.js': chunk('login', ['src/screens/Login.tsx'], ['assets/core-a.js', 'assets/vendor-core-a.js'], [], { isDynamicEntry: true, css: ['assets/login-a.css'] }),
    'assets/mail-a.js': chunk('mail', ['src/screens/Mailbox.tsx', 'src/components/MessageList.tsx'], ['assets/core-a.js', 'assets/shared-a.js', 'assets/vendor-core-a.js', 'assets/index-a.js'], ['assets/settings-a.js'], { isDynamicEntry: true, css: ['assets/mail-a.css'] }),
    'assets/settings-a.js': chunk('settings', ['src/screens/Settings.tsx'], ['assets/core-a.js', 'assets/shared-a.js'], [], { isDynamicEntry: true }),
    'assets/rare-a.js': chunk('rare', ['src/modules/passwd/ForcedPasswordChange.tsx'], ['assets/settings-a.js', 'assets/core-a.js'], [], { isDynamicEntry: true }),
  };
  const html = '<script type="module" crossorigin src="./assets/index-a.js"></script><link rel="modulepreload" href="./assets/core-a.js"><link rel="stylesheet" href="./assets/core-a.css">';
  const sw = 'const ASSET_DIGESTS = {};\nconst OFFLINE_ASSETS = ["/assets/index-a.js","/assets/core-a.js"];\n';
  const distFiles = [...Object.keys(chunks), 'assets/core-a.css', 'assets/login-a.css', 'assets/mail-a.css', 'assets/boot-a.js', 'index.html', 'sw.js'];
  const budget = {
    critical: { maxJs: 3, maxBrBytes: 1000 },
    routes: {
      login: { root: 'src/screens/Login.tsx', maxJs: 4, maxCss: 2 },
      mailbox: { root: 'src/screens/Mailbox.tsx', maxJs: 5, maxCss: 2, maxBrBytes: 1000 },
      settings: { root: 'src/screens/Settings.tsx', after: 'mailbox', maxJs: 1, maxCss: 0 },
    },
    totals: { jsChunks: 9, nonAdminChunks: 9, cssFiles: 3, precacheEntries: 2 },
    otherJs: ['^assets/boot-'],
    probes: [
      { module: 'src/components/MessageList.tsx', maxRenamed: 2, forbid: ['vendor-core', 'brand-paths'] },
      { module: 'src/api/basePath.ts', maxRenamed: 7, forbid: ['vendor-core', 'brand-paths'] },
    ],
  };
  return { report: { schema: 1, policy, chunks }, html, sw, distFiles, budget, sizes: () => ({ raw: 100, br: 40, gz: 50 }) };
}
const run = (f) => checkLoadBudget(f);
const failsWith = (f, pattern) => {
  const { failures } = run(f);
  assert.ok(failures.some((m) => pattern.test(m)), `expected a failure matching ${pattern}, got:\n${failures.join('\n')}`);
};

test('a valid layered graph passes, with the expected route and probe numbers', () => {
  const { failures, stats } = run(fixture());
  assert.deepEqual(failures, []);
  assert.deepEqual([stats.routes.login.js, stats.routes.login.css], [4, 2]);
  assert.deepEqual([stats.routes.mailbox.js, stats.routes.mailbox.css], [5, 2]);
  assert.deepEqual([stats.routes.settings.addsJs, stats.routes.settings.addsCss], [1, 0]);
  assert.deepEqual(stats.probes['src/components/MessageList.tsx'].chunks, ['entry', 'mail']);
  assert.equal(stats.probes['src/api/basePath.ts'].renamed, 7);
  assert.equal(stats.totals.precacheEntries, 2);
});

test('invalidation follows static and dynamic importers; cycles are found', () => {
  const chunks = { a: { imports: ['b'], dynamicImports: [] }, b: { imports: [], dynamicImports: ['c'] }, c: { imports: [], dynamicImports: [] }, d: { imports: [], dynamicImports: [] } };
  assert.deepEqual([...invalidated(chunks, 'c')].sort(), ['a', 'b', 'c']);
  assert.deepEqual(staticCycles(chunks), []);
  chunks.b.imports.push('a');
  assert.equal(staticCycles(chunks).length, 1);
});

test('negative: a screen importing another screen', () => {
  const f = fixture();
  f.report.chunks['assets/settings-a.js'].imports.push('assets/login-a.js');
  failsWith(f, /layering: settings \(layer 7\) imports login/);
});

test('negative: importing the entry without an exception', () => {
  const f = fixture();
  f.report.chunks['assets/settings-a.js'].imports.push('assets/index-a.js');
  failsWith(f, /layering: settings imports the entry chunk/);
});

test('negative: a vendor chunk importing app code, a data chunk importing anything', () => {
  let f = fixture();
  f.report.chunks['assets/vendor-core-a.js'].imports.push('assets/core-a.js');
  failsWith(f, /vendor chunk vendor-core imports app chunk core/);
  f = fixture();
  f.report.chunks['assets/brand-paths-a.js'].imports.push('assets/vendor-core-a.js');
  failsWith(f, /data chunk brand-paths imports/);
});

test('negative: a lower layer importing a higher one, and a static cycle', () => {
  const f = fixture();
  f.report.chunks['assets/core-a.js'].imports.push('assets/shared-a.js');
  failsWith(f, /layering: core \(layer 1\) imports shared \(layer 3\)/);
  failsWith(f, /circular chunk imports: /);
});

test('negative: a chunk the plan does not name', () => {
  const f = fixture();
  f.report.chunks['assets/mixed-a.js'] = chunk(null, ['src/x.ts', 'src/y.ts']);
  f.distFiles.push('assets/mixed-a.js');
  failsWith(f, /mixed-a\.js is not a chunk the plan names/);
});

test('negative: a route root loaded statically by its parent, or not a dynamic entry', () => {
  let f = fixture();
  f.report.chunks['assets/core-a.js'].imports.push('assets/login-a.js');
  failsWith(f, /route login: src\/screens\/Login\.tsx is in login, which the entry loads statically/);
  f = fixture();
  f.report.chunks['assets/mail-a.js'].isDynamicEntry = false;
  failsWith(f, /route mailbox: chunk mail of src\/screens\/Mailbox\.tsx is not a dynamic entry/);
  f = fixture();
  f.budget.routes.mailbox.root = 'src/screens/Gone.tsx';
  failsWith(f, /route mailbox: root module src\/screens\/Gone\.tsx is in no chunk/);
});

test('negative: route, critical path and totals over budget', () => {
  let f = fixture();
  f.budget.routes.mailbox.maxJs = 4;
  failsWith(f, /route mailbox: 5 JS files > 4/);
  f = fixture();
  f.budget.routes.settings.maxJs = 0;
  failsWith(f, /route settings: 1 JS files > 0/);
  f = fixture();
  f.budget.routes.mailbox.maxBrBytes = 200;
  failsWith(f, /route mailbox: 280 brotli bytes > 200/);
  f = fixture();
  f.budget.critical.maxJs = 2;
  failsWith(f, /critical path: 3 JS files > 2/);
  f = fixture();
  f.budget.totals.cssFiles = 2;
  failsWith(f, /totals: cssFiles 3 > 2/);
});

test('negative: a probe renaming too many files or a forbidden chunk', () => {
  let f = fixture();
  f.budget.probes[0].maxRenamed = 1;
  failsWith(f, /probe src\/components\/MessageList\.tsx: renames 2 hashed files > 1/);
  f = fixture();
  f.report.chunks['assets/brand-paths-a.js'].imports.push('assets/core-a.js');
  failsWith(f, /probe src\/api\/basePath\.ts: renames brand-paths/);
});

test('negative: a stale report (missing file, unknown JS in dist)', () => {
  let f = fixture();
  f.distFiles = f.distFiles.filter((file) => file !== 'assets/settings-a.js');
  failsWith(f, /names assets\/settings-a\.js, which dist does not have/);
  f = fixture();
  f.distFiles.push('assets/Login-old.js');
  failsWith(f, /neither in the chunk report nor in otherJs.*Login-old/);
});

test('negative: a non-console route loading an admin section chunk', () => {
  const f = fixture();
  f.report.policy.layers['admin-section'] = 9;
  f.report.policy.admin = ['admin', 'admin-section', 'admin-common'];
  f.report.chunks['assets/admin-section.Users-a.js'] = chunk('admin-section.Users', ['src/screens/Admin/Users.tsx'], ['assets/core-a.js'], [], { isDynamicEntry: true });
  f.distFiles.push('assets/admin-section.Users-a.js');
  f.budget.totals.jsChunks = 10;
  f.budget.probes[1].maxRenamed = 8;
  assert.deepEqual(run(f).failures, [], 'an unreferenced section chunk is fine, and its layer comes from admin-section');
  f.report.chunks['assets/rare-a.js'].imports.push('assets/admin-section.Users-a.js');
  f.budget.routes.rare = { root: 'src/modules/passwd/ForcedPasswordChange.tsx' };
  failsWith(f, /route rare: its static closure contains admin console chunks admin-section\.Users/);
  f.budget.routes.rare.console = true;
  assert.ok(!run(f).failures.some((m) => /admin console chunks/.test(m)), 'a console route may');
});

test('negative: a dynamic import target no chunk rule names', () => {
  const f = fixture();
  f.report.unrootedDynamicTargets = ['src/components/NewLazyThing.tsx'];
  failsWith(f, /unplanned: src\/components\/NewLazyThing\.tsx is a dynamic import target no ROOTS rule/);
});

test('negative: index.html loading something other than the entry chunk', () => {
  const f = fixture();
  f.html = f.html.replace('index-a.js', 'index-b.js');
  failsWith(f, /index\.html loads assets\/index-b\.js, but the entry chunk is assets\/index-a\.js/);
});
