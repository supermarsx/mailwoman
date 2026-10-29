import { test, expect, type Page } from '@playwright/test';
import { composeSurface } from './helpers.ts';
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
  // .first(): the merge test creates two cards with one name on purpose.
  await expect(contactList(page).getByText(fullName).first()).toBeVisible();
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

    // t39 R6: the import is a page of its own (source list, review detail).
    await contacts(page).getByRole('button', { name: 'Import…' }).click();
    const importPage = contacts(page).getByTestId('contacts-import-page');
    await expect(importPage.getByRole('heading', { level: 1, name: 'Import contacts' })).toBeFocused();
    await importPage.getByLabel('Paste vCard or CSV').fill(vcard);
    await importPage.getByRole('button', { name: 'Preview' }).click();
    await importPage.getByRole('button', { name: /Import 1 contact/ }).click();
    await expect(importPage.getByText(/Imported 1 contact/)).toBeVisible();
    await importPage.getByRole('button', { name: 'Done' }).click();

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
    // On the desktop workspace it opens as a tab region, not a dialog (24fb6149).
    await page.getByRole('button', { name: 'Compose' }).click();
    const dialog = composeSurface(page);
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

  // t39 R6: membership and merge pages, persisted through the engine.
  test('group membership set on its page survives a reload', async ({ page }) => {
    await engineLogin(page);
    await gotoModule(page, 'contacts');
    const tag = uid();
    const name = `Dorothy Vaughan ${tag}`;
    await createContact(page, name, `dorothy${tag}@example.org`);
    const group = `Team ${tag}`;
    await contacts(page).getByRole('button', { name: 'New group' }).click();
    await contacts(page).getByLabel('New group name').fill(group);
    await contacts(page).getByRole('button', { name: 'Create' }).click();
    await contacts(page).getByRole('button', { name: 'Group membership', exact: true }).click();
    const membership = contacts(page).getByTestId('contacts-membership-page');
    await membership.getByRole('button', { name: new RegExp(group) }).click();
    const picker = membership.getByRole('combobox', { name: 'Members' });
    await picker.fill(`dorothy${tag}`);
    await expect(membership.getByRole('option', { name: new RegExp(name) })).toBeVisible();
    await picker.press('Enter');
    await expect(membership.getByRole('button', { name: `Remove ${name}` })).toBeVisible();
    await expect(picker).toBeEnabled();

    await reloadToShell(page);
    await gotoModule(page, 'contacts');
    await contacts(page).getByRole('button', { name: 'Group membership', exact: true }).click();
    const after = contacts(page).getByTestId('contacts-membership-page');
    await after.getByRole('button', { name: new RegExp(group) }).click();
    await expect(after.getByRole('button', { name: `Remove ${name}` })).toBeVisible();
    await expect(after.getByText('1 member')).toBeVisible();
  });

  test('a merge on the merge page keeps one card after a reload', async ({ page }) => {
    await engineLogin(page);
    await gotoModule(page, 'contacts');
    const tag = uid();
    const name = `Mary Jackson ${tag}`;
    await createContact(page, name, `mary${tag}@work.example.org`);
    await createContact(page, name, `mary${tag}@home.example.org`);
    await contacts(page).getByRole('button', { name: 'Find duplicates' }).click();
    const merge = contacts(page).getByTestId('contacts-merge-page');
    await merge.getByRole('navigation', { name: 'Possible duplicates' }).getByRole('button', { name: new RegExp(name) }).click();
    const preview = merge.getByRole('region', { name: 'Merged preview' });
    await expect(preview.getByText(`mary${tag}@work.example.org`)).toBeVisible();
    await expect(preview.getByText(`mary${tag}@home.example.org`)).toBeVisible();
    await merge.getByRole('button', { name: 'Merge contacts' }).click();
    await expect(merge.getByRole('status')).toHaveText(`Merged into ${name}.`);
    await merge.getByRole('button', { name: 'Back to contacts' }).click();

    await reloadToShell(page);
    await gotoModule(page, 'contacts');
    await contacts(page).getByLabel('Search contacts').fill(name);
    await expect(contactList(page).getByText(name)).toHaveCount(1);
    await contactList(page).getByText(name).click();
    const card = contacts(page).getByRole('article', { name: `Contact ${name}` });
    await expect(card.getByText(`mary${tag}@work.example.org`)).toBeVisible();
    await expect(card.getByText(`mary${tag}@home.example.org`)).toBeVisible();
  });
});
