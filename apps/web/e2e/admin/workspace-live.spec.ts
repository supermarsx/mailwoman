import { test, expect } from '../csrf.ts';
import { signInManually, composeSurface } from '../helpers.ts';
import { messageRow } from '../helpers.ts';
import { sendRawSmtp } from '../crypto-helpers.ts';

test('independent message bodies survive tab switching and tiling @phone', async ({page}) => {
  const stamp=Date.now();
  for(const name of ['Alpha','Beta']) await sendRawSmtp({from:'sender@example.test',message:`From: sender@example.test\r\nTo: testuser@example.org\r\nDate: ${new Date().toUTCString()}\r\nMessage-ID: <workspace-${stamp}-${name}@example.test>\r\nSubject: ${name} ${stamp}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nIndependent ${name} body.\r\n`});
  await page.goto('/');await signInManually(page,{username:'testuser',password:'testpass'});
  await messageRow(page,`Alpha ${stamp}`).click();
  await expect(page.locator('.workspace-pane:visible .reader__subject')).toHaveText(`Alpha ${stamp}`);
  await page.getByRole('tab',{name:'Mail',exact:true}).click();
  await messageRow(page,`Beta ${stamp}`).click();
  await expect(page.locator('.workspace-pane:visible .reader__subject')).toHaveText(`Beta ${stamp}`);
  await page.getByRole('tab',{name:`Alpha ${stamp}`,exact:true}).click();
  await expect(page.locator('.workspace-pane:visible .reader__subject')).toHaveText(`Alpha ${stamp}`);
  if(test.info().project.name!=='phone') {
    await page.getByRole('button',{name:'Side by side',exact:true}).click();
    await expect(page.locator('.workspace-pane:visible .reader__subject')).toHaveCount(2);
    await page.getByRole('separator',{name:'Resize workspace panes'}).focus();
    await page.getByRole('separator',{name:'Resize workspace panes'}).press('ArrowRight');
    await page.getByRole('button',{name:'Tabs',exact:true}).click();
  }
  await page.getByRole('button',{name:'Show or hide reading pane'}).click();
  await expect(page.locator('.mail-pane')).toBeVisible();
  await page.getByRole('button',{name:'Show or hide reading pane'}).click();
  await expect(page.locator('.workspace-pane:visible .reader__subject')).toHaveText(`Alpha ${stamp}`);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
});

test('retained drafts, sectioned settings and account-saved pane layout @phone', async ({page}) => {
  await page.goto('/');
  await signInManually(page,{username:'testuser',password:'testpass'});
  await expect(page.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible();
  const prefs=()=>page.request.get('/api/account/preferences');
  await expect.poll(async()=> (await prefs()).status()).toBe(200);
  const compact=page.getByRole('button',{name:'Compact folder icons'});
  if(await compact.getAttribute('aria-pressed')==='true') await compact.click();
  await compact.click();
  await expect.poll(async()=> (await (await prefs()).json()).preferences.folderSidebarMode).toBe('compact');
  await page.reload();
  await expect(compact).toHaveAttribute('aria-pressed','true');
  await compact.click();
  if(test.info().project.name!=='phone') {
    const separator=page.getByRole('separator',{name:'Resize folders sidebar'});
    await separator.focus();await separator.press('Home');await separator.press('ArrowRight');
    await expect.poll(async()=> (await (await prefs()).json()).preferences.folderSidebarWidth).toBe(170);
    await page.getByRole('button',{name:'Hide folders sidebar'}).click();
    await expect(page.locator('.sidebar')).toHaveCSS('display','none');
    await page.getByRole('button',{name:'Show folders sidebar'}).click();
  }
  await page.getByRole('button',{name:'Compose',exact:true}).click();
  await composeSurface(page).getByLabel('Subject',{exact:true}).fill('Retained alpha');
  // f1a656bb names a composer tab after its subject.
  const first=page.getByRole('tab',{name:'Retained alpha',exact:true});
  await expect(first).toBeVisible();
  await page.getByRole('button',{name:'Compose',exact:true}).click();
  await composeSurface(page).getByLabel('Subject',{exact:true}).fill('Retained beta');
  await first.click();
  await expect(composeSurface(page).getByLabel('Subject',{exact:true})).toHaveValue('Retained alpha');
  if(test.info().project.name==='phone') await page.getByRole('button',{name:'Open folders and apps'}).click();
  await page.getByRole('button',{name:'Settings',exact:true}).click();
  await expect(page.getByRole('region',{name:'Settings',exact:true})).toBeVisible();
  await expect(page.getByRole('navigation',{name:'Settings sections'})).toBeVisible();
  await first.click();
  await expect(composeSurface(page).getByLabel('Subject',{exact:true})).toHaveValue('Retained alpha');
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  await page.screenshot({path:test.info().outputPath('workspace.png'),fullPage:true});
});
