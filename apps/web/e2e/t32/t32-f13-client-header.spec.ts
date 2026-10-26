import { readFileSync } from 'node:fs';
import { test, expect, type Request } from '@playwright/test';

/**
 * The web client names itself on sign-in (t32-cfg §4.18, t32-f13): the built bundle
 * sends `X-Mailwoman-Client: browser/<apps/web package version>` with `POST
 * /api/login`, and no such header on the requests it makes before that.
 *
 * Only the request is checked, so the spec runs against any stack: whether the
 * credentials are accepted does not matter here.
 */

const VERSION = (JSON.parse(readFileSync(new URL('../../package.json', import.meta.url), 'utf8')) as { version: string })
  .version;

test('the sign-in request carries X-Mailwoman-Client: browser/<version>', async ({ page }) => {
  const before: Request[] = [];
  page.on('request', (r) => {
    if (new URL(r.url()).pathname.includes('/api/') && !new URL(r.url()).pathname.endsWith('/api/login')) before.push(r);
  });

  await page.goto('/');
  const form = page.getByRole('form', { name: 'Sign in' });
  await expect(form).toBeVisible();
  await form.getByLabel('Email or username').fill('ada@example.org');
  await form.getByLabel('Password', { exact: true }).fill('not-the-password');

  const signIn = page.waitForRequest((r) => r.method() === 'POST' && new URL(r.url()).pathname.endsWith('/api/login'));
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  const req = await signIn;

  expect(await req.headerValue('x-mailwoman-client')).toBe(`browser/${VERSION}`);
  expect(before.length, 'the boot requests were seen').toBeGreaterThan(0);
  for (const r of before) {
    expect(await r.headerValue('x-mailwoman-client'), `${r.method()} ${r.url()}`).toBeNull();
  }
});
