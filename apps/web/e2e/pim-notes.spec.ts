import { test, expect, type Page } from '@playwright/test';
import { engineLogin, gotoModule, reloadToShell, uid } from './pim-helpers.ts';

/**
 * V3 Notes live E2E (plan §3 e12): drive the real Notes module against the
 * engine's `Note/*` surface. Create a note, give it a title, tag + color it, and
 * find it by title search; pinning reorders the list (pinned first); and a
 * created note survives a reload (engine round-trip). Note bodies are sealed at
 * rest server-side (asserted at the Rust/engine level by e8); here we exercise
 * the plaintext title/tag/color/pin surface the client drives.
 */

const notes = (page: Page) => page.locator('[data-module="notes"]');
const options = (page: Page) => notes(page).getByRole('option');

/**
 * Report which of two notes appears first in the (search-scoped) list, or `null`
 * while either is momentarily absent (mid-render). Robust to any other notes in
 * the account — it compares the two titles' positions. Poll it so the read never
 * races a re-render.
 */
async function orderOf(
  page: Page,
  alpha: string,
  beta: string,
): Promise<'alpha-first' | 'beta-first' | null> {
  const texts = await options(page).allInnerTexts();
  const ia = texts.findIndex((t) => t.includes(alpha));
  const ib = texts.findIndex((t) => t.includes(beta));
  if (ia < 0 || ib < 0) return null;
  return ia < ib ? 'alpha-first' : 'beta-first';
}

/** Create a note and rename it via the detail editor; returns nothing. */
async function newNote(page: Page, title: string): Promise<void> {
  await notes(page).getByRole('button', { name: '+ New note' }).click();
  const editor = notes(page).getByLabel('Note editor');
  await expect(editor).toBeVisible();
  // Wait until the freshly-created note ("Untitled note") is the SELECTED note
  // before renaming — otherwise the fill would race the selection switch and
  // rename the previously-open note instead of the new one.
  const titleInput = editor.getByLabel('Note title');
  await expect(titleInput).toHaveValue('Untitled note');
  await titleInput.fill(title);
  // The renamed note shows up in the list (pinned-first, newest-first) — wait for
  // the list row so downstream selection/ordering assertions are stable.
  await expect(options(page).filter({ hasText: title })).toBeVisible();
}

test.describe('Notes module through the real UI (engine mode)', () => {
  test('create → title → tag → color, and search finds it by title', async ({ page }) => {
    await engineLogin(page);
    await gotoModule(page, 'notes');

    const title = `Meeting notes ${uid()}`;
    await newNote(page, title);

    const editor = notes(page).getByLabel('Note editor');
    // Tag it.
    await editor.getByLabel('Add tag').fill('project');
    await editor.getByLabel('Add tag').press('Enter');
    await expect(editor.getByText('#project')).toBeVisible();
    // Color it.
    const color = editor.getByLabel('Color #facc15');
    await color.click();
    await expect(color).toHaveAttribute('aria-pressed', 'true');

    // Title search finds exactly this (uniquely-titled) note. Scope the count to
    // the unique title so the assertion is immune to other notes in the account.
    await notes(page).getByLabel('Search notes').fill(title);
    await expect(options(page).filter({ hasText: title })).toHaveCount(1);
    await expect(options(page).filter({ hasText: title })).toBeVisible();
  });

  test('pinning a note moves it ahead of a newer note (pinned-first ordering)', async ({ page }) => {
    await engineLogin(page);
    await gotoModule(page, 'notes');

    const tag = uid();
    const alpha = `Alpha ${tag}`;
    const beta = `Beta ${tag}`;
    await newNote(page, alpha);
    await newNote(page, beta); // newer → sorts above Alpha while both unpinned

    // Scope the visible list to just this run's two notes via the search box.
    await notes(page).getByLabel('Search notes').fill(tag);
    await expect(options(page).filter({ hasText: alpha })).toBeVisible();
    await expect(options(page).filter({ hasText: beta })).toBeVisible();

    // Pin Alpha; it should sort ahead of Beta (pinned-first ordering).
    await options(page).filter({ hasText: alpha }).click();
    await notes(page).getByLabel('Note editor').getByRole('button', { name: 'Pin note' }).click();
    await expect(
      notes(page).getByLabel('Note editor').getByRole('button', { name: 'Unpin note' }),
    ).toBeVisible();

    await expect.poll(() => orderOf(page, alpha, beta)).toEqual('alpha-first');
  });

  test('a created note persists across a full reload (engine round-trip)', async ({ page }) => {
    await engineLogin(page);
    await gotoModule(page, 'notes');

    const title = `Persisted note ${uid()}`;
    await newNote(page, title);

    await reloadToShell(page);
    await gotoModule(page, 'notes');
    await notes(page).getByLabel('Search notes').fill(title);
    await expect(options(page).filter({ hasText: title })).toBeVisible();
  });
});
