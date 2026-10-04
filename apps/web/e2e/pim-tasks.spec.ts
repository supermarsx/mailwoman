import { test, expect, type Page } from '@playwright/test';
import { engineLogin, gotoModule, reloadToShell, uid } from './pim-helpers.ts';

/**
 * V3 Tasks live E2E (plan §3 e12): drive the real Tasks module against the
 * engine's auto-seeded default VTODO list. Create a task → it appears in the
 * list; the My Day view is reachable and correctly filters (a task with no due
 * date is NOT in My Day); completing a task toggles it done; and a created task
 * survives a reload (engine round-trip).
 *
 * Editing (26.20 t28-e5): a task's title, due date and priority are edited
 * inline and a task can be deleted; each is asserted across a full reload, so
 * the proof is the engine's stored row, not the optimistic local state.
 *
 * NOTE (a UI gap, not patched here): the Tasks module exposes no per-task "add
 * to My Day" control, so a task cannot be PINNED into My Day through the UI even
 * though the `Task/*` surface + the tasks slice (`setMyDay`) support it. A task
 * does enter My Day through its due date, which the edit form sets.
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

  test('rename a task, set its due date and priority → all three survive a reload', async ({ page }) => {
    await engineLogin(page);
    await gotoModule(page, 'tasks');

    const tag = uid();
    const before = `Draft the memo ${tag}`;
    const after = `Send the memo ${tag}`;
    await addTask(page, before);
    const list = tasks(page).getByRole('list', { name: 'Tasks' });
    await expect(list.getByText(before)).toBeVisible();
    // Precondition: the new title is not already there, and the task has no due
    // date (it is not in My Day).
    await expect(tasks(page).getByText(after)).toHaveCount(0);

    await tasks(page).getByRole('button', { name: `Edit ${before}` }).click();
    const form = tasks(page).getByRole('form', { name: `Edit ${before}` });
    await form.getByLabel('Title').fill(after);
    const d = new Date();
    const p = (n: number): string => String(n).padStart(2, '0');
    const today = `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
    await form.getByLabel('Due date').fill(today);
    await form.getByLabel('Priority').selectOption({ label: 'High' });
    await form.getByRole('button', { name: 'Save' }).click();
    await expect(list.getByText(after)).toBeVisible();

    await reloadToShell(page);
    await gotoModule(page, 'tasks');
    const row = tasks(page).getByRole('list', { name: 'Tasks' }).locator('li', { hasText: after });
    await expect(row).toBeVisible();
    await expect(tasks(page).getByText(before)).toHaveCount(0);
    await expect(row.getByText(today)).toBeVisible();
    await expect(row.getByLabel('High priority')).toBeVisible();

    // Due today → the engine-stored due date puts it in My Day.
    await tasks(page).getByRole('button', { name: 'My Day' }).click();
    await expect(tasks(page).getByRole('list', { name: 'My Day' }).getByText(after)).toBeVisible();
  });

  test('delete a task → it is gone after a reload; declining the confirm keeps it', async ({ page }) => {
    await engineLogin(page);
    await gotoModule(page, 'tasks');

    const title = `Obsolete chore ${uid()}`;
    await addTask(page, title);
    await expect(tasks(page).getByText(title)).toBeVisible();

    // Declining: the task is still stored.
    await tasks(page).getByRole('button', { name: `Edit ${title}` }).click();
    await tasks(page).getByRole('button', { name: 'Delete', exact: true }).click();
    await tasks(page).getByRole('button', { name: 'Keep task' }).click();
    await tasks(page).getByRole('button', { name: 'Cancel' }).click();
    await reloadToShell(page);
    await gotoModule(page, 'tasks');
    await expect(tasks(page).getByText(title)).toBeVisible();

    // Confirming: it is destroyed in the engine.
    await tasks(page).getByRole('button', { name: `Edit ${title}` }).click();
    await tasks(page).getByRole('button', { name: 'Delete', exact: true }).click();
    await tasks(page).getByRole('button', { name: 'Delete task' }).click();
    await expect(tasks(page).getByText(title)).toHaveCount(0);
    await reloadToShell(page);
    await gotoModule(page, 'tasks');
    // The list has loaded (the add form is there) and the task is not in it.
    await expect(tasks(page).getByLabel('New task title')).toBeVisible();
    await expect(tasks(page).getByText(title)).toHaveCount(0);
  });
});
