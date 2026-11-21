import {test} from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp, mkdir, writeFile, rm, realpath} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join, dirname, basename} from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

const checker = fileURLToPath(new URL('./check-catalog-weight.mjs', import.meta.url));
const common = '# Unique English common catalog\ncommon-button = Continue safely\n';
const mail = '# Unique Portuguese mail catalog\nmail-button = Abrir mensagem agora\n';
async function check(options = {}) {
  const root = await mkdtemp(join(tmpdir(), 'mw-catalog-guard-'));
  try {
    for (const path of ['locales/en', 'locales/pt', 'dist/assets']) {
      await mkdir(join(root, path), {recursive:true});
    }
    await writeFile(join(root, 'locales/en/common.ftl'), common);
    await writeFile(join(root, 'locales/pt/mail.ftl'), mail);
    await writeFile(join(root, 'dist/index.html'), '<script type="module" src="./assets/entry.js"></script>');
    const entry = (options.missingCommon ? '' : `const common=${JSON.stringify(common)};`) +
      (options.leak ? `const mail=${JSON.stringify(mail)};` : '') + 'import("./lazy.js");';
    await writeFile(join(root, 'dist/assets/entry.js'), entry);
    await writeFile(join(root, 'dist/assets/lazy.js'), options.legacy ?
      `export default ${JSON.stringify(mail)};` :
      `export default ${JSON.stringify(options.assetLiteral ?? (options.unreferenced ? 'missing.ftl' : './mail-hash.ftl'))};` +
      (options.commentReference ? '/* ./mail-hash.ftl */' : ''));
    if (!options.legacy) {
      await writeFile(join(root, 'dist/assets/mail-hash.ftl'), options.tampered ? mail + 'changed = Stale artifact\n' : mail);
    }
    if (options.missingCommon) await writeFile(join(root, 'dist/assets/common-hash.ftl'), common);
    if (options.missingCommon) await writeFile(join(root, 'dist/assets/common-loader.js'), 'export default "./common-hash.ftl";');
    const result = spawnSync(process.execPath, [checker, '--web-root', root], {encoding:'utf8', timeout:15000});
    assert.equal(result.error, undefined);
    return {status:result.status, text:result.stdout + result.stderr};
  } finally {
    // Only remove the exact owned temporary fixture, never a relocated/reparse
    // target or a parent computed from untrusted checker input.
    const target = await realpath(root);
    assert.equal(dirname(target).toLowerCase(), (await realpath(tmpdir())).toLowerCase());
    assert.ok(basename(target).startsWith('mw-catalog-guard-'));
    await rm(root, {recursive:true, force:true});
  }
}
test('production exact referenced FTL assets preserve the raw eager positive control', async () => {
  const result = await check();
  assert.equal(result.status, 0, result.text);
});
test('legacy lazy raw catalogs remain guarded', async () => {
  const result = await check({legacy:true});
  assert.equal(result.status, 0, result.text);
});
test('stale emitted text fails exact source-byte coverage', async () => {
  const result = await check({tampered:true});
  assert.equal(result.status, 1);
  assert.match(result.text, /NOT FOUND/);
});
test('unreferenced exact assets cannot hide missing bundle coverage', async () => {
  const result = await check({unreferenced:true});
  assert.equal(result.status, 1);
  assert.match(result.text, /NOT FOUND/);
});
test('text assets do not excuse eager raw catalog leaks', async () => {
  const result = await check({leak:true});
  assert.equal(result.status, 1);
  assert.match(result.text, /LEAK/);
});
test('even an exact referenced common asset cannot replace the eager positive control', async () => {
  const result = await check({missingCommon:true});
  assert.equal(result.status, 1);
  assert.match(result.text, /positive control/);
});
for (const [label, options] of [
  ['a comment', {unreferenced:true, commentReference:true}],
  ['a filename suffix', {assetLiteral:'./mail-hash.ftl.old'}],
  ['a foreign origin', {assetLiteral:'https://invalid.example/assets/mail-hash.ftl'}],
  ['a different path', {assetLiteral:'./unrelated/mail-hash.ftl'}],
]) {
  test(`an exact text asset mentioned only in ${label} does not prove a URL reference`, async () => {
    const result = await check(options);
    assert.equal(result.status, 1);
    assert.match(result.text, /NOT FOUND/);
  });
}
