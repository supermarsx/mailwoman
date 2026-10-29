// Drive the app colour picker (t81) from a spec: open it from its trigger
// (named "<label>: <value>"), type a hex value and press Enter to commit.
import { expect, type Locator, type Page } from '@playwright/test';

const escape = (text: string): string => text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

/** The trigger of the colour field labelled `label` inside `scope`. */
export function colorTrigger(scope: Page | Locator, label: string): Locator {
  return scope.getByRole('button', { name: new RegExp(`^${escape(label)}: `) });
}

/** Open the picker labelled `label`, type `hex`, commit with Enter. */
export async function pickColor(scope: Page | Locator, label: string, hex: string, trigger: Locator = colorTrigger(scope, label)): Promise<void> {
  await trigger.click();
  const dialog = scope.getByRole('dialog', { name: label, exact: true });
  await expect(dialog).toBeVisible();
  const field = dialog.getByRole('textbox', { name: 'Hex', exact: true });
  await field.fill(hex);
  await field.press('Enter');
  await expect(dialog).toBeHidden();
}
