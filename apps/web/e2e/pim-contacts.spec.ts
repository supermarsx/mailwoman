import { test, expect, type Page } from '@playwright/test';
import { engineLogin, gotoModule, reloadToShell, uid } from './pim-helpers.ts';

/**
 * V3 Contacts live E2E (plan §3 e12): drive the real Contacts module against the
 * engine's `AddressBook/*` / `ContactCard/*` surface. Create a contact → it
 * appears in the list; favoriting it surfaces it under the Favorites filter;
 * importing a vCard lands the contact; and — the cross-module wiring — a created
 * contact autocompletes as a recipient in Compose (`[data-testid=
 * "contact-suggestion"]`). A created contact also survives a reload (engine
 * round-trip).
 */

const contacts = (page: Page) => page.locator('[data-module="contacts"]');
const contactList = (page: Page) => contacts(page).getByRole('list', { name: 'Contact list' });

/** Create a contact through the real editor (full name + one email). */
async function createContact(page: Page, fullName: string, email: string): Promise<void> {
  await contacts(page).getByRole('button', { name: 'New contact' }).click();
  const form = contacts(page).getByRole('form', { name: 'New contact' });
  await expect(form).toBeVisible();
  await form.getByLabel('Full name').fill(fullName);
  await form.getByLabel('Email 1', { exact: true }).fill(email);
  await form.getByRole('button', { name: 'Save' }).click();
  await expect(contactList(page).getByText(fullName)).toBeVisible();
}

test.describe('Contacts module through the real UI (engine mode)', () => {
  test('create a contact → it appears in the list → favorite surfaces it', async ({ page }) => {
    await engineLogin(page);
    await gotoModule(page, 'contacts');

    const name = `Grace Hopper ${uid()}`;
    await createContact(page, name, `grace${uid()}@example.org`);

    // Favorite it from the list row. `exact: true` so the star <button> (whose
    // accessible name is exactly "Favorite <name>") is picked, not the enclosing
    // contact row (role="button") whose composed name merely starts with it.
    await contactList(page).getByRole('button', { name: `Favorite ${name}`, exact: true }).click();

    // The Favorites filter now includes the contact (engine-persisted flag).
    await contacts(page).getByRole('button', { name: /Favorites/ }).click();
    await expect(contactList(page).getByText(name)).toBeVisible();
  });

  test('import a vCard → the imported contact lands in the list', async ({ page }) => {
    await engineLogin(page);
    await gotoModule(page, 'contacts');

    const tag = uid();
    const name = `Katherine Johnson ${tag}`;
    const vcard = [
      'BEGIN:VCARD',
      'VERSION:3.0',
      `FN:${name}`,
      `EMAIL:katherine${tag}@example.org`,
      'END:VCARD',
      '',
    ].join('\n');

    await contacts(page).getByRole('button', { name: 'Import…' }).click();
    const dialog = page.getByRole('dialog', { name: 'Import contacts' });
    await dialog.getByLabel('Paste vCard or CSV').fill(vcard);
    await dialog.getByRole('button', { name: 'Preview' }).click();
    await dialog.getByRole('button', { name: /Import 1 contact/ }).click();
    await expect(dialog.getByText(/Imported 1 contact/)).toBeVisible();
    await dialog.getByRole('button', { name: 'Done' }).click();

    await contacts(page).getByLabel('Search contacts').fill(name);
    await expect(contactList(page).getByText(name)).toBeVisible();
  });

  test('a contact autocompletes as a recipient in Compose', async ({ page }) => {
    await engineLogin(page);
    await gotoModule(page, 'contacts');

    const tag = uid();
    const name = `Ada Lovelace ${tag}`;
    const email = `ada${tag}@example.org`;
    await createContact(page, name, email);

    // Compose loads contacts on open, then ranks the in-progress recipient token.
    await page.getByRole('button', { name: 'Compose' }).click();
    const dialog = page.getByRole('dialog', { name: 'Compose message' });
    await expect(dialog).toBeVisible();
    await dialog.getByLabel('To', { exact: true }).fill(`ada${tag}`);

    const suggestion = dialog.locator('[data-testid="contact-suggestion"]');
    await expect(suggestion.first()).toBeVisible();
    await suggestion.first().click();

    // Picking the suggestion inserts `Name <email>` into the To field.
    await expect(dialog.getByLabel('To', { exact: true })).toHaveValue(new RegExp(email));
  });

  test('a created contact persists across a full reload (engine round-trip)', async ({ page }) => {
    await engineLogin(page);
    await gotoModule(page, 'contacts');

    const name = `Persisted Contact ${uid()}`;
    await createContact(page, name, `persist${uid()}@example.org`);

    await reloadToShell(page);
    await gotoModule(page, 'contacts');
    await contacts(page).getByLabel('Search contacts').fill(name);
    await expect(contactList(page).getByText(name)).toBeVisible();
  });
});
