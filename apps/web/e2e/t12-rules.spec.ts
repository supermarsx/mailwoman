import { test, expect, type Page } from '@playwright/test';
import { engineLogin } from './helpers.ts';
import { uid } from './crypto-helpers.ts';

/**
 * 26.12 Sieve rules live E2E (audit #1, SPEC §6.1/§10.5, plan §2 Batch-4).
 *
 * The web rule builder is new in 26.12 (there was no rules module before). This
 * drives it through the REAL UI against the engine's `MailRule` JMAP surface (which
 * persists `mw_sieve::Rule` and, where ManageSieve is advertised, uploads the
 * generated Sieve):
 *
 *   • create a rule in the condition/action builder → it saves and lists;
 *   • the "where it runs" indicator renders (server-Sieve vs engine);
 *   • the raw-Sieve editor shows the SOURCE generated from that rule (the codegen
 *     output — the round-trip inverse the Rust parser reads back, unit-proven by
 *     `mw-sieve` with 45 round-trip tests) and lints clean;
 *   • the dry-run preview fires the rule against a matching sample and not a
 *     non-matching one;
 *   • the rule survives a full page reload — proving it round-tripped through the
 *     ENGINE (MailRule/set → MailRule/get), not client state.
 */

/** Open the Settings dialog and wait for the Rules section (authenticated only). */
async function openRules(page: Page): Promise<void> {
  await page.getByRole('button', { name: 'Settings' }).first().click();
  await expect(page.getByRole('dialog', { name: 'Settings' })).toBeVisible();
  // The Rules section mounts under the authenticated block; its heading is stable.
  await expect(page.getByRole('heading', { name: 'Rules & filters' })).toBeVisible();
}

test('rule builder → save → raw-Sieve source + where-it-runs + dry-run, persisted through the engine', async ({
  page,
}) => {
  test.setTimeout(60_000);

  const token = uid();
  const ruleName = `Route ${token}`;
  const sender = `boss-${token}@example.com`;
  const mailbox = `Work-${token}`;

  await engineLogin(page);
  await openRules(page);

  const settings = page.getByRole('dialog', { name: 'Settings' });

  // 1) New rule → the condition/action builder opens.
  await settings.getByRole('button', { name: 'New rule' }).click();
  const builder = settings.getByRole('form', { name: 'Rule builder' });
  await expect(builder).toBeVisible();

  // Name + a From-contains condition + a Move-to-mailbox action (a Sieve-expressible
  // rule → runs as server Sieve).
  await builder.getByLabel('Rule name').fill(ruleName);
  await builder.getByLabel('Match value').first().fill(sender);
  // The default first action is "Move to mailbox"; fill its target.
  await builder.getByLabel('Action value').first().fill(mailbox);

  // 2) The where-it-runs indicator shows this rule runs as server Sieve.
  await expect(builder.getByText('Server (Sieve)').first()).toBeVisible();

  // 3) Save → the rule lands in the list (MailRule/set create → reload).
  await builder.getByRole('button', { name: 'Save rule' }).click();
  await expect(settings.getByText(ruleName)).toBeVisible();

  // 4) Raw-Sieve editor: the SOURCE generated from the saved rule set.
  await settings.getByRole('tab', { name: 'Raw Sieve' }).click();
  const raw = settings.getByLabel('Raw Sieve source');
  await expect(raw).toBeVisible();
  const source = await raw.inputValue();
  expect(source, 'rule name comment').toContain(`# rule: ${ruleName}`);
  expect(source, 'From condition codegen').toContain('address :contains "from"');
  expect(source, 'sender value').toContain(sender);
  expect(source, 'Move action codegen').toContain(`fileinto "${mailbox}"`);
  // The generated source lints clean (require covers fileinto).
  await expect(settings.getByText('No problems found.')).toBeVisible();

  // 5) Dry-run preview: a matching From fires the rule; a non-matching one does not.
  await settings.getByRole('tab', { name: 'Dry run' }).click();
  const results = settings.getByRole('list', { name: 'Dry-run results' });
  await settings.getByLabel('From').fill(sender);
  await expect(results.getByText(ruleName)).toBeVisible();
  await expect(results.getByText(`move to ${mailbox}`)).toBeVisible();
  // A non-matching sender: the rule row shows "No match".
  await settings.getByLabel('From').fill(`stranger-${token}@nowhere.example`);
  await expect(results.getByText('No match').first()).toBeVisible();

  // 6) Engine round-trip: reload the page, reopen Rules — the rule persisted server-
  // side (it came back from MailRule/get, not in-memory state).
  await page.reload();
  await expect(page.getByRole('button', { name: 'Compose' })).toBeVisible();
  await openRules(page);
  await expect(page.getByRole('dialog', { name: 'Settings' }).getByText(ruleName)).toBeVisible();
});
