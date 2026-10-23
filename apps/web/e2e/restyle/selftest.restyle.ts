import { check, type Finding } from './checks.ts';
import { test, expect } from './probes.ts';

/**
 * The checks in checks.ts on a fixed page, without the app or a stack: the good
 * page must give no finding, and each case on the bad page must give exactly the
 * finding it was built for. Runs in the desktop-dark project only.
 */

const BASE_STYLE = `
  body { background: #111111; color: #f2efe6; font: 14px/1.4 sans-serif; margin: 16px; }
  button, input { font: inherit; color: #f2efe6; }
  :focus-visible { outline: 3px solid #f5c400; outline-offset: 2px; }
  .plain { background: none; border: 0; padding: 4px; }
`;

const GOOD = `
  <style>${BASE_STYLE}</style>
  <input aria-label="good field" style="border: 2px solid #bdb8ad; background: #0a0a0a">
  <button style="border: 2px solid #bdb8ad; background: transparent">Bordered</button>
  <button style="border: 2px solid #4d7cff; background: #4d7cff; color: #0b0b0b">Filled</button>
  <ul>
    <li><button class="plain list__row list__row--unread" style="font-weight: 700">Unread row</button></li>
    <li><button class="plain list__row list__row--active" style="box-shadow: inset 4px 0 0 #4d7cff">Selected row</button></li>
  </ul>
  <p>Running text with <a href="#a" style="color: #8fb0ff; text-decoration: underline">a link</a> inside it.</p>
  <div role="tablist"><button role="tab" aria-selected="true" class="plain" style="border-bottom: 3px solid #4d7cff">Tab</button></div>
`;

/** Each case triggers one check; `want` is the finding's check name. */
const BAD_CASES: { want: string; html: string }[] = [
  { want: 'axe color-contrast', html: '<p style="color: #333333">Low contrast text</p>' },
  {
    want: 'non-text boundary < 3:1',
    html: '<input aria-label="weak field" style="border: 1px solid #222222; background: #111111">',
  },
  {
    want: 'focus: no outline or ring',
    html: '<button class="plain" style="outline: none" data-nofocus>No focus ring</button><style>[data-nofocus]:focus-visible { outline: none; }</style>',
  },
  {
    want: 'focus indicator < 3:1',
    html: '<button class="plain" data-lowfocus>Weak focus ring</button><style>[data-lowfocus]:focus-visible { outline: 3px solid #1c1c1c; }</style>',
  },
  { want: 'unread row: colour only', html: '<button class="plain list__row list__row--unread" style="color: #8fb0ff">Unread, colour only</button>' },
  { want: 'selected row: colour only', html: '<button class="plain list__row list__row--active" style="background: #1d3a8f">Selected, colour only</button>' },
  { want: 'in-text link not underlined', html: '<p>Running text with <a href="#b" style="color: #8fb0ff; text-decoration: none">a bare link</a> inside it.</p>' },
  { want: 'active tab: no bar', html: '<div role="tablist"><button role="tab" aria-selected="true" class="plain" style="background: #1d3a8f">Tab, fill only</button></div>' },
];

const names = (f: Finding[]) => f.map((x) => `${x.check}: ${x.element} — ${x.detail}`);

test.describe('restyle checks on a fixed page', () => {
  test.beforeEach(({ viewportName, theme }) => {
    test.skip(viewportName !== 'desktop' || theme !== 'dark', 'runs once, in desktop-dark');
  });

  test('the good page gives no finding', async ({ page }) => {
    await page.setContent(GOOD);
    const r = await check(page, []);
    expect(r.checked.nonText, 'boundary-checked controls').toBe(3);
    expect(r.checked.unread + r.checked.selected + r.checked.links + r.checked.tabs).toBe(4);
    expect(names(r.findings)).toEqual([]);
  });

  for (const c of BAD_CASES) {
    test(`a seeded defect gives "${c.want}"`, async ({ page }) => {
      await page.setContent(`<style>${BASE_STYLE}</style>${c.html}`);
      const r = await check(page, []);
      expect(names(r.findings).length, names(r.findings).join('\n')).toBeGreaterThan(0);
      expect(r.findings.map((f) => f.check)).toEqual(r.findings.map(() => c.want));
    });
  }
});
