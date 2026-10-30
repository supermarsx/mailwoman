import {test,expect} from '../csrf.ts';

test('bulk mode keeps its actions visible before selection and after Select none',async({page},info)=>{
 await page.goto('/');
 await page.getByLabel('Email or username',{exact:true}).fill('testuser');
 await page.getByLabel('Password',{exact:true}).fill('testpass');
 await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(page.locator('.list__row').first()).toBeVisible();
 const toggle=page.getByRole('button',{name:'Select messages',exact:true});
 const bar=page.getByRole('toolbar',{name:'Selection actions',exact:true});
 await expect(toggle).toHaveAttribute('aria-pressed','false');
 await expect(bar).toHaveCount(0);
 await toggle.click();
 await expect(toggle).toHaveAttribute('aria-pressed','true');
 await expect(bar).toBeVisible();
 await expect(bar.getByTestId('bulk-count')).toHaveText('0 selected');
 await expect(bar.getByTestId('bulk-archive')).toBeDisabled();
 await page.screenshot({path:info.outputPath('bulk-mode-empty.png')});
 await bar.getByRole('button',{name:'Select all',exact:true}).click();
 await expect(bar.getByTestId('bulk-count')).not.toHaveText('0 selected');
 await expect(bar.getByTestId('bulk-archive')).toBeEnabled();
 await bar.getByRole('button',{name:'Select none',exact:true}).click();
 await expect(bar).toBeVisible();
 await expect(bar.getByTestId('bulk-count')).toHaveText('0 selected');
 await expect(toggle).toHaveAttribute('aria-pressed','true');
 await toggle.click();await expect(bar).toHaveCount(0);
 await toggle.click();await expect(bar).toBeVisible();
 await bar.getByRole('button',{name:'Clear selection',exact:true}).click();
 await expect(bar).toHaveCount(0);await expect(toggle).toHaveAttribute('aria-pressed','false');
 // The reader's list-layout toggle belongs to an individual message pane.
 const layoutToggle=page.getByRole('button',{name:'Show or hide message list',exact:true});
 await expect(layoutToggle).toHaveCount(0);
 const folderTab=page.getByRole('tab',{selected:true});const folderName=await folderTab.getAttribute('aria-label')??await folderTab.innerText();
 await page.locator('.list__row').first().dblclick();await expect(layoutToggle).toBeVisible();
 await page.getByRole('tab',{name:folderName,exact:true}).click();await expect(layoutToggle).toHaveCount(0);
 const compose=info.project.name==='phone'?page.locator('header.shell__bar').getByRole('button',{name:'Compose',exact:true}):page.getByRole('button',{name:'Compose',exact:true});
 await compose.click();await expect(page.getByLabel('Subject',{exact:true})).toBeVisible();await expect(layoutToggle).toHaveCount(0);
 await page.getByRole('tab',{name:folderName,exact:true}).click();await expect(toggle).toBeVisible();
});
