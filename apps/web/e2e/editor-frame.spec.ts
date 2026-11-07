import {createServer,type ViteDevServer} from 'vite';
import {expect,test} from '@playwright/test';

/**
 * Regression: the native HTML editor lives in an iframe whose document is replaced
 * (reload, re-attach). Its view-colour constructable stylesheet was kept across
 * documents and re-adopted into the new one, which Firefox ("Adopted style sheet's
 * constructor document must match ...") and Chromium ("Sharing constructed
 * stylesheets in multiple documents") both reject. Serves the editor from Vite on
 * an ephemeral port; needs no mw-server.
 */
let server: ViteDevServer; let origin = '';
test.beforeAll(async () => {
  server = await createServer({configFile: new URL('../vite.config.ts', import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, '$1'), server: {port: 0, host: '127.0.0.1'}, logLevel: 'error'});
  await server.listen();
  const address = server.httpServer!.address();
  origin = `http://127.0.0.1:${typeof address === 'object' && address ? address.port : 0}`;
});
test.afterAll(async () => { await server?.close(); });

test('toggling views, themes and replacing the editor frame raises no adopted-stylesheet error', async ({page}) => {
  const errors: string[] = [];
  page.on('pageerror', error => errors.push(error.message));
  page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
  await page.goto(`${origin}/e2e/fixtures/html-editor-harness.html`);
  await page.waitForSelector('iframe', {state: 'attached'});
  await page.waitForTimeout(1000);
  for (let i = 0; i < 2; i++) {
    await page.getByRole('tab', {name: /source/i}).click();
    await page.getByRole('tab', {name: /visual/i}).click();
    await page.evaluate(() => { const root = document.documentElement; root.setAttribute('data-theme', root.getAttribute('data-theme') === 'dark' ? 'light' : 'dark'); });
    await page.waitForTimeout(400);
  }
  await page.evaluate(() => (document.querySelector('iframe')!.contentWindow as Window).location.reload());
  await page.waitForTimeout(1200);
  await page.evaluate(() => { const frame = document.querySelector('iframe')!; const parent = frame.parentElement!; parent.removeChild(frame); parent.appendChild(frame); });
  await page.waitForTimeout(1200);
  expect(errors.filter(text => /adopted style ?sheet|constructed stylesheets/i.test(text))).toEqual([]);
});

test('local-scheme references in a draft or quoted original never reach the editor frame', async ({page}) => {
  const errors: string[] = [];
  page.on('pageerror', error => errors.push(error.message));
  page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
  const html = '<p>x</p><img src="file:///C:/Users/a/image001.png"><img src="file:///"><a href="file:///C:/x">l</a>'
    + '<table background="file:///C:/b.png"><tr><td style="background:url(file:///C:/c.png)">t</td></tr></table>'
    + '<style>p{background:url(file:///C:/d.png)}@font-face{font-family:f;src:url(file:///C:/f.woff)}</style>'
    + '<svg><image xlink:href="file:///C:/e.svg"/></svg><video poster="file:///C:/p.png"></video>';
  await page.goto(`${origin}/e2e/fixtures/html-editor-harness.html?html=${encodeURIComponent(html)}`);
  await page.waitForSelector('iframe', {state: 'attached'});
  await page.waitForTimeout(2500);
  await page.getByRole('tab', {name: /source/i}).click();
  await page.getByRole('tab', {name: /visual/i}).click();
  await page.waitForTimeout(1500);
  expect(errors.filter(text => /file:\/|may not load or link/i.test(text))).toEqual([]);
});
