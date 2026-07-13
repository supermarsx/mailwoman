import { test, expect, type Page } from '@playwright/test';
import { engineLogin, gotoModule, reloadToShell, uid } from './pim-helpers.ts';

/**
 * V3 Tasks live E2E (plan §3 e12): drive the real Tasks module against the
 * engine's auto-seeded default VTODO list. Create a task → it appears in the
 * list; the My Day view is reachable and correctly filters (a task with no due
 * date is NOT in My Day); completing a task toggles it done; and a created task
 * survives a reload (engine round-trip).
 *
 * NOTE (reported as a UI gap, not patched here — e12 owns e2e only): the Tasks
 * module exposes no per-task "add to My Day" control, and the add-task form has
 * no due-date field, so a task cannot be pinned into My Day through the current
 * UI even though the `Task/*` surface + the tasks slice (`setMyDay`) support it.
 * This spec therefore verifies My Day reachability + its filter, not UI pinning.
 */

const tasks = (page: Page) => page.locator('[data-module="tasks"]');

async function addTask(page: Page, title: string): Promise<void> {
  await tasks(page).getByLabel('New task title').fill(title);
  await tasks(page).getByRole('button', { name: 'Add' }).click();
}

test.describe('Tasks module through the real UI (engine mode)', () => {
  test('create a task → it appears in the list', async ({ page }) => {
    await engineLogin(page);
    await gotoModule(page, 'tasks');

    const title = `Write the report ${uid()}`;
    await addTask(page, title);

    await expect(tasks(page).getByRole('list', { name: 'Tasks' }).getByText(title)).toBeVisible();
  });

  test('My Day view is reachable and excludes an unscheduled task', async ({ page }) => {
    await engineLogin(page);
    await gotoModule(page, 'tasks');

    const title = `Someday task ${uid()}`;
    await addTask(page, title);
    await expect(tasks(page).getByText(title)).toBeVisible();

    await tasks(page).getByRole('button', { name: 'My Day' }).click();
    const myDay = tasks(page).getByRole('list', { name: 'My Day' });
    await expect(myDay).toBeVisible();
    // A task with no due date is not part of My Day (the engine-side filter).
    await expect(myDay.getByText(title)).toHaveCount(0);
  });

  test('completing a task flips it done (Complete → Reopen)', async ({ page }) => {
    await engineLogin(page);
    await gotoModule(page, 'tasks');

    const title = `Close the ticket ${uid()}`;
    await addTask(page, title);

    const complete = tasks(page).getByRole('checkbox', { name: `Complete ${title}` });
    await expect(complete).toBeVisible();
    // A single click (not `.check()`): completing optimistically re-renders the row
    // and swaps the checkbox node, so a state-verifying `.check()` would race the
    // detach. The flipped "Reopen …" label below is the assertion of "done".
    await complete.click();

    // The toggle's accessible name flips to "Reopen …" once the task is done.
    await expect(tasks(page).getByRole('checkbox', { name: `Reopen ${title}` })).toBeVisible();
  });

  test('a created task persists across a full reload (engine round-trip)', async ({ page }) => {
    await engineLogin(page);
    await gotoModule(page, 'tasks');

    const title = `Persisted task ${uid()}`;
    await addTask(page, title);
    await expect(tasks(page).getByText(title)).toBeVisible();

    await reloadToShell(page);
    await gotoModule(page, 'tasks');
    await expect(tasks(page).getByText(title)).toBeVisible();
  });
});
