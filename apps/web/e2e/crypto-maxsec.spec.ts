// `test` from csrf.ts: the floor case writes the admin policy through a request
// context, which has to repeat the CSRF cookie on the stack's strict guard.
import { test, expect, type APIRequestContext } from './csrf.ts';
import { engineLogin, injectHtmlViaSmtp, uid, waitForInboxMessage, openMessage } from './crypto-helpers.ts';

/**
 * V4 max-security opening live E2E (plan §3 e10 item 6 / SPEC §7.2): the reader
 * view-mode dropdown (Full / No media / Plain text; t57, formerly a three-position switch) changes the ACTUAL
 * body render. Flipping to "No media" pins the sandboxed body frame to a media-free
 * CSP (images/media dropped); flipping to "Plain text" renders the message as escaped
 * plain text with no HTML at all. Pure client-side (no WASM worker) — the switch
 * drives the `bodyFrameDoc` CSP/sanitize mode in `viewers/sandbox.ts`.
 */
test('Max-security: the switch strips media (no-media) and renders plain text', async ({ page }) => {
  const token = uid();
  const subject = `MaxSec ${token}`;
  const htmlMark = `HTMLMARK_${token}`;
  const plainMark = `PLAINMARK_${token}`;

  // A message with both a text/plain part (used by plain-text mode) and a text/html
  // part carrying an <img> (used to prove media stripping).
  await injectHtmlViaSmtp({
    from: 'Newsletter <news@remote-example.net>',
    subject,
    text: `${plainMark} plain body`,
    html: `<p>${htmlMark}</p><img src="http://remote-example.net/tracker.png" alt="tracker">`,
  });

  await engineLogin(page);
  await waitForInboxMessage(page, subject);
  await openMessage(page, subject);

  const frame = page.locator('iframe.reader__frame');
  await expect(frame).toBeVisible();
  const switchEl = page.locator('[data-testid="max-security-switch"]');
  // Default posture is full-sanitized (the pre-V4 body render).
  await expect(switchEl).toHaveAttribute('data-mode', 'full-sanitized');

  // Flip to No media → the body frame is pinned to a media-free CSP (no img-src).
  await switchEl.click();
  await page.getByRole('menuitemcheckbox', { name: 'No media', exact: true }).click();
  await expect(switchEl).toHaveAttribute('data-mode', 'sanitized-no-media');
  await expect(async () => {
    const srcdoc = (await frame.getAttribute('srcdoc')) ?? '';
    expect(srcdoc).toContain("default-src 'none'");
    // The media-free CSP has NO img-src/media-src source at all.
    expect(srcdoc).not.toContain('img-src');
    expect(srcdoc).not.toContain('media-src');
  }).toPass({ timeout: 10_000 });

  // Flip to Plain text → the body renders as escaped plain text (no live HTML).
  await switchEl.click();
  await page.getByRole('menuitemcheckbox', { name: 'Plain text', exact: true }).click();
  await expect(switchEl).toHaveAttribute('data-mode', 'plain-text');
  await expect(async () => {
    const srcdoc = (await frame.getAttribute('srcdoc')) ?? '';
    expect(srcdoc).toContain('<pre>');
    expect(srcdoc).toContain(plainMark);
    // No live <img> element — media cannot render in plain-text mode.
    expect(srcdoc).not.toContain('<img');
  }).toPass({ timeout: 10_000 });
});

/** Admin credentials of the engine stack (`MW_ADMIN_USER`/`MW_ADMIN_PASSWORD`). */
const ADMIN = {
  username: process.env['MW_ADMIN_USER'] ?? 'root',
  password: process.env['MW_ADMIN_PASSWORD'] ?? 'hunter2',
};

/** Set the security policy's `maxSecurityFloor` through the admin API, keeping the
 *  other fields as they are (a PUT takes the whole shape). */
async function setFloor(admin: APIRequestContext, floor: string | null): Promise<void> {
  const now = await admin.get('/admin/security-policy');
  expect(now.status(), 'GET /admin/security-policy').toBe(200);
  const p = (await now.json()) as Record<string, unknown>;
  const put = await admin.put('/admin/security-policy', {
    data: {
      minTls: p['minTls'],
      require2fa: p['require2fa'],
      argon2MCost: p['argon2MCost'],
      argon2TCost: p['argon2TCost'],
      argon2PCost: p['argon2PCost'],
      capturePolicy: p['capturePolicy'],
      maxSecurityFloor: floor,
      dlpRulesJson: p['dlpRulesJson'],
    },
  });
  expect(put.status(), `PUT maxSecurityFloor=${String(floor)}: ${await put.text()}`).toBe(200);
}

/**
 * t29-e6: the floor an administrator sets (`PUT /admin/security-policy`
 * `maxSecurityFloor`) reaches the reader through `GET /api/policy`: the switch
 * opens at the floor, the positions below it are disabled, and choosing one does
 * not lower the mode. The floor is put back to none afterwards, because every
 * crypto spec shares this server.
 */
test('Max-security: the admin floor keeps the switch at or above it', async ({ page, playwright, baseURL }) => {
  const admin = await playwright.request.newContext(baseURL === undefined ? {} : { baseURL });
  const login = await admin.post('/admin/login', { data: ADMIN });
  expect(login.status(), 'admin login').toBe(200);

  const token = uid();
  const subject = `MaxSecFloor ${token}`;
  const plainMark = `FLOORPLAIN_${token}`;
  await injectHtmlViaSmtp({
    from: 'Newsletter <news@remote-example.net>',
    subject,
    text: `${plainMark} plain body`,
    html: `<p>FLOORHTML_${token}</p><img src="http://remote-example.net/tracker.png" alt="tracker">`,
  });

  await setFloor(admin, 'plain-text');
  try {
    await engineLogin(page);
    await waitForInboxMessage(page, subject);
    await openMessage(page, subject);

    const switchEl = page.locator('[data-testid="max-security-switch"]');
    await expect(switchEl).toHaveAttribute('data-mode', 'plain-text', { timeout: 15_000 });
    // The view-mode dropdown lists every mode, but those below the floor are disabled.
    await switchEl.click();
    const full = page.getByRole('menuitemcheckbox', { name: 'Full', exact: true });
    await expect(full).toHaveAttribute('aria-disabled', 'true');
    await expect(page.getByRole('menuitemcheckbox', { name: 'No media', exact: true })).toHaveAttribute('aria-disabled', 'true');
    await full.click({ force: true });
    await expect(switchEl).toHaveAttribute('data-mode', 'plain-text');
    const frame = page.locator('iframe.reader__frame');
    await expect(async () => {
      const srcdoc = (await frame.getAttribute('srcdoc')) ?? '';
      expect(srcdoc).toContain('<pre>');
      expect(srcdoc).toContain(plainMark);
      expect(srcdoc).not.toContain('<img');
    }).toPass({ timeout: 10_000 });
  } finally {
    await setFloor(admin, null);
    await admin.dispose();
  }
});
