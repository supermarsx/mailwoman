// t101: the console's Interface density set `data-density` on :root, but no
// console rule read it, so the choice changed nothing. Table cells now take
// their block padding from it.
import { expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const css = readFileSync(join(__dirname, 'admin.css.ts'), 'utf8');

it('every console density sets the data-table cell padding', () => {
  expect(css).toContain("globalStyle(':root', { vars: { [cellPadBlock]: vars.space[3] } });");
  expect(css).toContain("globalStyle(':root[data-density=\"compact\"]', { vars: { [cellPadBlock]: vars.space[2] } });");
  expect(css).toContain("globalStyle(':root[data-density=\"relaxed\"]', { vars: { [cellPadBlock]: vars.space[4] } });");
  expect(css).toContain('padding: `${cellPadBlock} ${vars.space[4]}`');
  expect(css).toContain('paddingBottom: `calc(${cellPadBlock} - 1px)`');
});
