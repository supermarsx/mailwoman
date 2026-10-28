import {test,expect} from '../csrf.ts';

test('Nextcloud uses dedicated admin pages and persists approved authority without sending credentials @phone',async({page},info)=>{
  await page.waitForTimeout(4000);
  await page.goto('/admin');
  await page.getByLabel('Admin username').fill('root');await page.getByLabel('Password',{exact:true}).fill('hunter2');
  const loggedIn=page.waitForResponse(response=>response.url().endsWith('/admin/login')&&response.request().method()==='POST');
  await page.getByRole('button',{name:'Sign in',exact:true}).click();expect((await loggedIn).status()).toBe(200);
  await page.goto('/admin#/nextcloud');
  await expect(page.getByRole('heading',{name:'Nextcloud instances',exact:true})).toBeVisible();
  await page.getByRole('button',{name:'Add Nextcloud instance',exact:true}).click();
  await expect(page).toHaveURL(/#\/nextcloud\/new$/);
  await expect(page.getByRole('heading',{name:'Nextcloud instances',exact:true})).toHaveCount(0);
  const name=`Browser cloud ${info.project.name} ${Date.now()}`;
  await page.getByLabel(/^Display name/).fill(name);
  await page.getByLabel(/^HTTPS installation URL/).fill('https://cloud.example/nextcloud/');
  await expect(page.getByLabel(/^Additional certificate authorities/)).toBeVisible();
  // Keep this fixture disabled: configuration acceptance must not contact a cloud.
  await expect(page.getByLabel('Allow users to link this instance')).not.toBeChecked();
  await page.getByRole('button',{name:'Save',exact:true}).click();
  const confirmation=page.getByRole('dialog',{name:'Confirm it is you'});
  await confirmation.getByLabel('Password',{exact:true}).fill('hunter2');
  await confirmation.getByRole('button',{name:'Confirm and continue'}).click();
  // The list row opens the editor through an "Open <name>" action (24978c23).
  await expect(page.getByRole('button',{name:`Open ${name}`,exact:true})).toBeVisible();
  await page.getByRole('button',{name:`Open ${name}`,exact:true}).click();
  await page.reload();
  await expect(page.getByLabel(/^Display name/)).toHaveValue(name);
  await expect(page.getByLabel(/^HTTPS installation URL/)).toHaveValue('https://cloud.example/nextcloud/');
  await expect(page.getByLabel('Allow users to link this instance')).not.toBeChecked();
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
  await page.screenshot({path:info.outputPath('nextcloud-instance-editor.png'),fullPage:true});
  await page.getByLabel('Allow users to link this instance').scrollIntoViewIfNeeded();
  const save=await page.getByRole('button',{name:'Save',exact:true}).boundingBox();
  expect(save?.width).toBeLessThan(240);
  await page.screenshot({path:info.outputPath('nextcloud-instance-actions.png'),fullPage:true});
});
