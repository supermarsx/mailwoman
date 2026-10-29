import { test, expect, type Page, type Request } from '@playwright/test';
import {
  engineLogin,
  ENGINE_CREDS,
  gotoKeys,
  generatePgpKey,
  uid,
  waitForInboxMessage,
  openMessage,
} from './crypto-helpers.ts';
import { composeSurface, readerContent } from './helpers.ts';

/**
 * 26.12 compose crypto WIRE-ASSERTION live E2E — the §8 security headline (audit #3,
 * plan §2 Batch-4). "Unit-green ≠ wired": a message that claims to be encrypted but
 * goes out in cleartext is a CRITICAL failure, so this asserts the BYTES on the wire,
 * using the REAL WASM crypto worker + real engine. Every send's outgoing `/jmap/api`
 * request bodies are captured (the Email/set create carries the message body).
 *
 * All four send paths are wire-asserted GREEN (fixed in 26.12; see
 * .orchestration/logs/t12-fix-compose-sign.md):
 *   • encrypt — the body is a real armored `PGP MESSAGE`, the plaintext marker appears
 *     in NO outgoing request (genuinely encrypted, §8), and it DECRYPTS back to the
 *     marker on receipt through the real worker (reversible, not garbage).
 *   • plain — the body is the exact `<p>…</p>` HTML, NO PGP armor (byte-unchanged).
 *   • sign-only — the body is a clear-signed inline `PGP SIGNED MESSAGE`: the cleartext
 *     stays readable AND carries a signature block (BUG B fix: the mw-crypto wasm `sign`
 *     now honors `detached:false`, emitting a real Cleartext Signature Framework message
 *     via rPGP instead of discarding the body behind a bare detached signature).
 *   • encrypt+sign folded — a signed-AND-encrypted `PGP MESSAGE` is drafted, sent, and
 *     decrypts back on receipt with the embedded signature VERIFIED (BUG A fix:
 *     `worker.entry.ts` unwraps `unlockKey`'s `{keyRef}`→string at the worker boundary
 *     so `signWithKeyRef` reaches the wasm `encrypt` as a string, honoring the frozen
 *     `contracts/crypto.ts`).
 *
 * Serial; each test mints its OWN PGP key in its OWN browser context (the private
 * backup is context-local under zero-access), so encrypt (newest-own) and decrypt
 * (first own key with a LOCAL private backup) resolve to the SAME keypair — even
 * across re-runs, since only this context holds that key's private backup.
 */
test.describe.configure({ mode: 'serial' });

const self = ENGINE_CREDS.selfAddress;
const passphrase = `t12-sign-pass-${uid()}`;

/** Collect every outgoing JMAP request body sent while `fn` runs. */
async function captureJmap(page: Page, fn: () => Promise<void>): Promise<string[]> {
  const bodies: string[] = [];
  const onReq = (req: Request): void => {
    if (req.method() === 'POST' && req.url().includes('/jmap/api')) {
      const d = req.postData();
      if (d !== null) bodies.push(d);
    }
  };
  page.on('request', onReq);
  try {
    await fn();
    await page.waitForTimeout(300); // let the in-flight send flush into the capture
  } finally {
    page.off('request', onReq);
  }
  return bodies;
}

/** Mint a PGP key for the self address IN THIS test's context. The private backup is
 *  context-local (zero-access), so each test mints its own; the "first own key with a
 *  local private backup" (decrypt) is then this context's key — the same one encrypt
 *  picks (newest-own) — so the round-trip is deterministic even across re-runs (same
 *  discipline as the V4 crypto-pgp spec). */
async function mintKey(page: Page): Promise<void> {
  await gotoKeys(page);
  await generatePgpKey(page, { email: self, passphrase, name: 'Test User' });
}

/**
 * Send as `self` (the key's User ID). The Greenmail login is the bare `testuser`, so
 * the default identity's From has no domain; the stack offers `self` as a second
 * identity (MW_ALLOWED_FROMS). The reader looks the signer's key up by the From
 * address, and a From without a domain matches no key, so a message that must
 * VERIFY on receipt has to be sent from the address its signing key is bound to.
 * Since e0c99a79 the From picker is shown by the ribbon's From toggle (folded into
 * "More Recipients commands" at narrower widths, t38-e5); same steps as
 * compose-options.spec.ts.
 */
async function sendAsSelf(dialog: ReturnType<Page['locator']>): Promise<void> {
  const fromToggle = dialog.getByRole('button', { name: 'From', exact: true });
  const moreRecipients = dialog.getByRole('button', { name: 'More Recipients commands', exact: true });
  const from = dialog.getByRole('combobox', { name: 'From', exact: true });
  await expect(async () => {
    if (await from.isVisible()) return;
    if (!(await fromToggle.isVisible()) && (await moreRecipients.isVisible())
      && (await moreRecipients.getAttribute('aria-expanded')) !== 'true') await moreRecipients.click();
    if ((await fromToggle.getAttribute('aria-pressed', { timeout: 1_000 })) !== 'true') await fromToggle.click({ timeout: 1_000 });
    await expect(from).toBeVisible({ timeout: 1_000 });
  }).toPass({ timeout: 15_000 });
  let value: string | undefined;
  await expect(async () => {
    const options = await from
      .locator('option')
      .evaluateAll((els) => els.map((o) => ({ value: (o as HTMLOptionElement).value, text: o.textContent ?? '' })));
    value = options.find((o) => o.text.includes(`<${self}>`))?.value;
    expect(value, `the stack offers ${self} as a sending identity (MW_ALLOWED_FROMS): ${JSON.stringify(options)}`).toBeTruthy();
  }).toPass({ timeout: 10_000 });
  await from.selectOption(value!);
}

async function openCompose(
  page: Page,
  subject: string,
  body: string,
  opts: { fromSelf?: boolean } = {},
): Promise<ReturnType<Page['locator']>> {
  await page.getByRole('button', { name: 'Compose', exact: true }).click();
  // Compose is a workspace tab region since 24fb6149 (helpers.composeSurface).
  const dialog = composeSurface(page);
  await expect(dialog).toBeVisible();
  if (opts.fromSelf === true) await sendAsSelf(dialog);
  await dialog.getByLabel('To', { exact: true }).fill(self);
  await dialog.getByLabel('Subject', { exact: true }).fill(subject);
  await dialog.getByLabel('Body', { exact: true }).fill(body);
  // The crypto banner and the encrypt/sign toggles are on the ribbon's Security tab (e0c99a79).
  await dialog.getByRole('tab', { name: 'Security', exact: true }).click();
  return dialog;
}

/** Unlock the signing key in the composer via the passphrase panel (Enter to unlock —
 *  the submit button sits below the fold in the tall composer). */
async function unlockSigning(dialog: ReturnType<Page['locator']>): Promise<void> {
  // The sign toggle lives on the compose ribbon's Security tab.
  await dialog.getByRole('tab', { name: 'Security', exact: true }).click();
  await dialog.locator('[data-testid="sign-toggle"]').check();
  const panel = dialog.locator('[data-testid="compose-sign-unlock"]');
  await expect(panel).toBeVisible();
  const pass = panel.locator('[data-testid="sign-passphrase"]');
  await pass.fill(passphrase);
  await pass.press('Enter');
  await expect(panel).toBeHidden({ timeout: 20_000 });
}

/** Submit the composer's form directly (the footer Send sits below the fold). */
async function submitCompose(dialog: ReturnType<Page['locator']>): Promise<void> {
  await dialog.locator('form.compose').evaluate((f) => (f as HTMLFormElement).requestSubmit());
  await expect(dialog).toBeHidden();
}

test('encrypt — the wire body is genuinely encrypted (no cleartext leak) and decrypts back on receipt', async ({
  page,
}) => {
  test.setTimeout(150_000);
  const token = uid();
  const subject = `enc ${token}`;
  const marker = `PLAINTEXT_SECRET_${token}`;

  await engineLogin(page);
  await mintKey(page);

  const dialog = await openCompose(page, subject, marker);
  await expect(dialog.locator('[data-testid="compose-crypto-banner"]')).toHaveAttribute(
    'data-capability',
    'e2ee',
    { timeout: 20_000 },
  );

  await dialog.locator('[data-testid="encrypt-toggle"]').check();
  // The ribbon shows a check beside the toggle; the text is the toggle's description (t38-e5).
  await expect(dialog.locator('[data-testid="encrypted-draft-indicator"]')).toHaveText('Draft encrypted on this device.', {
    timeout: 20_000,
  });
  await expect(dialog.locator('[data-testid="encrypt-toggle"]')).toHaveAccessibleDescription('Draft encrypted on this device.');
  await expect(dialog.locator('.cmpcrypto-toggle__drafted')).toBeVisible();

  const bodies = await captureJmap(page, () => submitCompose(dialog));

  // WIRE ASSERTION (§8): the outgoing body is a real armored PGP MESSAGE...
  const encryptedReq = bodies.find((b) => b.includes('-----BEGIN PGP MESSAGE-----'));
  expect(encryptedReq, 'an outgoing request carrying an armored PGP MESSAGE').toBeTruthy();
  expect(encryptedReq!).toContain('-----END PGP MESSAGE-----');
  // ...and the plaintext marker went out in NO request (genuinely encrypted, not clear).
  for (const b of bodies) {
    expect(b, 'plaintext marker must never appear on the wire').not.toContain(marker);
  }

  // Decrypt-on-receipt through the real worker: the plaintext renders → it was truly
  // encrypted-then-decryptable (not garbage / not a plaintext passthrough).
  await waitForInboxMessage(page, subject, 120_000);
  await openMessage(page, subject);
  await expect(page.locator('[data-testid="reader-decrypt"]')).toBeVisible();
  await page.locator('[data-testid="decrypt-passphrase"]').fill(passphrase);
  await page.locator('[data-testid="decrypt-submit"]').click();
  await expect((await readerContent(page)).contentFrame().getByText(marker)).toBeVisible({ // nested content frame since 99f51b81
    timeout: 20_000,
  });
});

// BUG B fix (26.12): the mw-crypto wasm `sign` now honors `detached:false` and emits a
// real inline PGP SIGNED MESSAGE (Cleartext Signature Framework), so the cleartext body
// stays readable AND carries a verifiable signature block — the body is no longer lost.
test('sign-only — the wire body is a clear-signed PGP SIGNED MESSAGE with a real signature block', async ({
  page,
}) => {
  test.setTimeout(90_000);
  const token = uid();
  const subject = `sign-only ${token}`;
  const marker = `CLEARSIGNED_BODY_${token}`;

  await engineLogin(page);
  await mintKey(page);

  const dialog = await openCompose(page, subject, marker);
  await unlockSigning(dialog); // sign, encrypt OFF
  const bodies = await captureJmap(page, () => submitCompose(dialog));

  const signedReq = bodies.find((b) => b.includes('-----BEGIN PGP SIGNED MESSAGE-----'));
  expect(signedReq, 'an outgoing request carrying a clear-signed PGP SIGNED MESSAGE').toBeTruthy();
  // The cleartext stays READABLE (clear-signed, not encrypted)...
  expect(signedReq!, 'clear-signed cleartext is readable').toContain(marker);
  // ...and it carries a real detached signature block.
  expect(signedReq!).toContain('-----BEGIN PGP SIGNATURE-----');
  expect(signedReq!).toContain('-----END PGP SIGNATURE-----');
  // A sign-only send is NOT encrypted — no PGP MESSAGE armor.
  expect(signedReq!).not.toContain('-----BEGIN PGP MESSAGE-----');
});

// BUG A fix (26.12): `worker.entry.ts` unwraps `unlockKey`'s `{keyRef}`→string at the
// worker boundary, so `signWithKeyRef` reaches the wasm `encrypt` as a string (frozen
// `contracts/crypto.ts` honored). The fold now drafts a signed-AND-encrypted PGP MESSAGE
// that goes out on the wire, carries no cleartext, decrypts back on receipt, and whose
// embedded signature VERIFIES.
test('encrypt+sign (folded via signWithKeyRef) produces a signed-and-encrypted draft that verifies', async ({
  page,
}) => {
  test.setTimeout(150_000);
  const token = uid();
  const subject = `enc-sign ${token}`;
  const marker = `SIGNED_ENCRYPTED_${token}`;

  await engineLogin(page);
  await mintKey(page);

  const dialog = await openCompose(page, subject, marker, { fromSelf: true });
  await expect(dialog.locator('[data-testid="compose-crypto-banner"]')).toHaveAttribute(
    'data-capability',
    'e2ee',
    { timeout: 20_000 },
  );
  // Unlock the signing key FIRST, then encrypt so the first encrypt folds the signature.
  await unlockSigning(dialog);
  await dialog.locator('[data-testid="encrypt-toggle"]').check();
  // The fold succeeds now: a signed-and-encrypted draft is produced.
  // The ribbon shows a check beside the toggle; the text is the toggle's description (t38-e5).
  await expect(dialog.locator('[data-testid="encrypted-draft-indicator"]')).toHaveText('Draft encrypted on this device.', {
    timeout: 20_000,
  });
  await expect(dialog.locator('[data-testid="encrypt-toggle"]')).toHaveAccessibleDescription('Draft encrypted on this device.');
  await expect(dialog.locator('.cmpcrypto-toggle__drafted')).toBeVisible();

  const bodies = await captureJmap(page, () => submitCompose(dialog));

  // WIRE ASSERTION (§8): the outgoing body is a real armored PGP MESSAGE (encrypted)...
  const encryptedReq = bodies.find((b) => b.includes('-----BEGIN PGP MESSAGE-----'));
  expect(encryptedReq, 'an outgoing request carrying an armored PGP MESSAGE').toBeTruthy();
  expect(encryptedReq!).toContain('-----END PGP MESSAGE-----');
  // ...the plaintext marker went out in NO request (genuinely encrypted, not clear)...
  for (const b of bodies) {
    expect(b, 'plaintext marker must never appear on the wire').not.toContain(marker);
  }

  // ...and on receipt it decrypts back to the marker AND the embedded signature verifies
  // (signed-AND-encrypted) — the property BUG A blocked.
  await waitForInboxMessage(page, subject, 120_000);
  await openMessage(page, subject);
  await expect(page.locator('[data-testid="reader-decrypt"]')).toBeVisible();
  await page.locator('[data-testid="decrypt-passphrase"]').fill(passphrase);
  await page.locator('[data-testid="decrypt-submit"]').click();
  await expect((await readerContent(page)).contentFrame().getByText(marker)).toBeVisible({ // nested content frame since 99f51b81
    timeout: 20_000,
  });
  // Expand the security chip and assert the client-side decrypt+verify reported the
  // embedded signature as VERIFIED. The chip renders once the server verdict has
  // loaded, and the header has other aria-expanded controls (View conversation), so
  // match the chip by the dialog it opens (SecurityPanel.tsx).
  const chip = page.locator('.reader__header button[aria-haspopup="dialog"][aria-controls]');
  await expect(chip).toBeVisible({ timeout: 20_000 });
  await chip.click();
  await expect(
    page.getByRole('region', { name: 'Message security details' }).getByText('Signature verified'),
  ).toBeVisible({ timeout: 10_000 });
});

test('plain — the wire body is exact <p>…</p> HTML with no PGP armor (byte-unchanged)', async ({
  page,
}) => {
  test.setTimeout(60_000);
  const token = uid();
  const subject = `plain ${token}`;
  const marker = `PLAIN_BODY_${token}`;

  await engineLogin(page);

  const dialog = await openCompose(page, subject, marker);
  const bodies = await captureJmap(page, () => submitCompose(dialog));

  // Target the Email/set SEND body specifically (`<p>…</p>`), not an incidental
  // Dlp/scan request that may carry the raw bodyText during the capture window.
  const sendReq = bodies.find((b) => b.includes(`<p>${marker}</p>`));
  expect(sendReq, 'an outgoing request carrying the plain body').toBeTruthy();
  expect(sendReq!).toContain(`<p>${marker}</p>`);
  expect(sendReq!).not.toContain('BEGIN PGP MESSAGE');
  expect(sendReq!).not.toContain('BEGIN PGP SIGNED MESSAGE');
  expect(sendReq!).not.toContain('BEGIN PGP SIGNATURE');
});
