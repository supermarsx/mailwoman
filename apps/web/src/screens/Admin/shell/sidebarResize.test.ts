import { expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

// 1d4df784 sized a vertical ResizeHandle by stretching it (height:auto, align-self:stretch).
// That only works when its parent is a flex or grid container: the admin sidebar's sticky
// 100vh wrapper was a block box, so the separator collapsed to 0 px and could not be dragged
// (e2e/admin/sidebar-layout.spec.ts). vitest renders no vanilla-extract CSS, so read the rule.
it('the admin sidebar resize wrapper is a flex box so its separator spans the full height', () => {
  const source = readFileSync(resolve(process.cwd(), 'src/screens/Admin/shell/shell.css.ts'), 'utf8');
  const rule = /export const sidebarResize = style\(\{([\s\S]*?)\n\}\);/.exec(source)?.[1];
  expect(rule).toBeDefined();
  const base = rule!.split(/\n\s*(?:selectors|'@media'):/)[0]!.replace(/\/\/.*$/gm, '');
  expect(base).toMatch(/display:\s*'(?:flex|grid)'/);
  expect(base).toMatch(/height:\s*'100vh'/);
  const handle = readFileSync(resolve(process.cwd(), 'src/components/ResizeHandle.css.ts'), 'utf8');
  expect(handle).toMatch(/alignSelf:\s*'stretch'/);
});
