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

/**
 * 26.12 compose crypto WIRE-ASSERTION live E2E — the §8 security headline (audit #3,
 * plan §2 Batch-4). "Unit-green ≠ wired": a message that claims to be encrypted but
 * goes out in cleartext is a CRITICAL failure, so this asserts the BYTES on the wire,
 * using the REAL WASM crypto worker + real engine. Every send's outgoing `/jmap/api`
 * request bodies are captured (the Email/set create carries the message body).
 *
 * GREEN (proven working):
 *   • encrypt — the body is a real armored `PGP MESSAGE`, the plaintext marker appears
 *     in NO outgoing request (genuinely encrypted, §8), and it DECRYPTS back to the
 *     marker on receipt through the real worker (reversible, not garbage).
 *   • plain — the body is the exact `<p>…</p>` HTML, NO PGP armor (byte-unchanged).
 *
 * ESCALATED — sign-on-send (audit #3's residual) is BROKEN end-to-end in a real
 * browser (two independent wire holes; both `test.fail`-tracked, both flip to
 * UNEXPECTED PASS once fixed → the owner removes the annotation). See
 * .orchestration/logs/t12-e-e2e-web.md + state.md Escalations for repro/expected/actual.
 *
 *   BUG A — encrypt+sign folded via `signWithKeyRef`: the worker's `unlockKey` returns
 *   `{keyRef}` (mw-crypto wasm `UnlockOut{keyRef}`) but the FROZEN `contracts/crypto.ts`
 *   declares `unlockKey(): Promise<KeyRef=string>`; `worker.entry.ts` normalizes
 *   unlockKey's INPUT but never unwraps its OUTPUT, so `Compose.tsx` stores `{keyRef}`
 *   and passes it as `signWithKeyRef`, and the wasm `encrypt` (sign_with_key_ref:
 *   Option<String>) panics `expected a string` → no signed ciphertext is drafted.
 *   Minimal fix: unwrap `{keyRef}`→string in worker.entry.ts (frozen contract/wasm
 *   untouched).
 *
 *   BUG B — sign-only clear-sign: `clearSignBody` calls the worker `sign` with
 *   `detached:false` intending an inline `PGP SIGNED MESSAGE`, but the wasm `sign`
 *   IGNORES `detached` (`#[allow(dead_code)]`) and always returns a DETACHED signature
 *   (`pgp::sign_detached`). `clearSignBody` then sends ONLY that `-----BEGIN PGP
 *   SIGNATURE-----` armor as the body, so the actual message text is DISCARDED and the
 *   result is not a clear-signed message. Minimal fix: honor `detached:false` in the
 *   wasm `sign` (emit a real cleartext-signed message) OR have `clearSignBody` assemble
 *   the cleartext-signature framework around the detached signature.
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

async function openCompose(page: Page, subject: string, body: string): Promise<ReturnType<Page['locator']>> {
  await page.getByRole('button', { name: 'Compose', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: 'Compose message' });
  await expect(dialog).toBeVisible();
  await dialog.getByLabel('To', { exact: true }).fill(self);
  await dialog.getByLabel('Subject', { exact: true }).fill(subject);
  await dialog.getByLabel('Body', { exact: true }).fill(body);
  return dialog;
}

/** Unlock the signing key in the composer via the passphrase panel (Enter to unlock —
 *  the submit button sits below the fold in the tall composer). */
async function unlockSigning(dialog: ReturnType<Page['locator']>): Promise<void> {
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
  await expect(dialog.locator('[data-testid="encrypted-draft-indicator"]')).toBeVisible({
    timeout: 20_000,
  });

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
  await expect(page.frameLocator('iframe.reader__frame').getByText(marker)).toBeVisible({
    timeout: 20_000,
  });
});

// KNOWN-BROKEN + ESCALATED (BUG B, see file header + state.md). Asserts the INTENDED
// clear-signed behavior (an inline PGP SIGNED MESSAGE whose cleartext stays readable);
// today the wire body is a bare detached PGP SIGNATURE with the message text discarded,
// so the first assertion fails. Marked test.fail to keep the suite green while tracking
// the bug; flips to UNEXPECTED PASS once the clear-sign is emitted correctly.
test('sign-only — the wire body is a clear-signed PGP SIGNED MESSAGE with a real signature block', async ({
  page,
}) => {
  test.fail(true, 'compose-crypto wire hole: worker sign ignores detached:false; clearSignBody loses the body — escalated');
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

// KNOWN-BROKEN + ESCALATED (see file header + state.md Escalations). Drives the
// encrypt+sign fold; currently the worker keyRef mismatch prevents a signed ciphertext
// from ever being drafted, so the encrypted-draft indicator never appears. Marked
// test.fail so the suite stays green while tracking the bug; flips to UNEXPECTED PASS
// once worker.entry.ts unwraps unlockKey's `{keyRef}`.
test('encrypt+sign (folded via signWithKeyRef) produces a signed-and-encrypted draft', async ({
  page,
}) => {
  test.fail(true, 'compose-crypto wire hole: unlockKey `{keyRef}` vs frozen string contract — escalated');
  test.setTimeout(90_000);
  const token = uid();
  const subject = `enc-sign ${token}`;
  const marker = `SIGNED_ENCRYPTED_${token}`;

  await engineLogin(page);
  await mintKey(page);

  const dialog = await openCompose(page, subject, marker);
  await expect(dialog.locator('[data-testid="compose-crypto-banner"]')).toHaveAttribute(
    'data-capability',
    'e2ee',
    { timeout: 20_000 },
  );
  // Unlock the signing key FIRST, then encrypt so the first encrypt folds the signature.
  await unlockSigning(dialog);
  await dialog.locator('[data-testid="encrypt-toggle"]').check();
  // BUG: the fold throws in the worker (`expected a string`), so no encrypted draft is
  // produced — this assertion times out (the tracked failure).
  await expect(dialog.locator('[data-testid="encrypted-draft-indicator"]')).toBeVisible({
    timeout: 8_000,
  });
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
