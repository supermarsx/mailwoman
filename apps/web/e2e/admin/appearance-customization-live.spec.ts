import {test, expect, type Page} from '../csrf.ts';
import { ADMIN } from './admin-path.ts';
import {signInManually, drawerButton} from '../helpers.ts';
import {colorTrigger,pickColor} from '../colorPicker.ts';
async function request(page: Page, path: string, data?: unknown, etag?: string) {
  return page.evaluate(async ({path,data,etag}) => {
    const token = document.cookie.split('; ').find(item => item.startsWith('mw_csrf='))?.slice(8);
    const authority = data !== undefined && path.startsWith('/api/account/') ? (await (await fetch('/api/me')).json()).storageAuthority as string : undefined;
    const response = await fetch(path, {method: data === undefined ? 'GET' : 'PUT', headers: {...(authority ? {'x-mailwoman-storage-authority':authority} : {}), ...(data === undefined ? {} : {'content-type':'application/json'}), ...(token ? {'x-csrf-token':decodeURIComponent(token)} : {}), ...(etag ? {'if-match':etag} : {})}, body:data === undefined ? null : JSON.stringify(data)});
    const text = await response.text();
    return {status:response.status,etag:response.headers.get('etag'),body:text ? JSON.parse(text) : null};
  }, {path,data,etag});
}
test('personal CSS and isolated script persist with separate administrator scope @phone', async ({page,browser,baseURL}) => {
  const adminContext = await browser.newContext({baseURL: baseURL!});
  const admin = await adminContext.newPage();
  await admin.waitForTimeout(4000); await admin.goto(ADMIN);
  await admin.getByLabel('Admin username').fill('root'); await admin.getByLabel('Password',{exact:true}).fill('hunter2');
  await admin.getByRole('button',{name:'Sign in',exact:true}).click();
  await expect(admin.getByRole('button',{name:'Sign in',exact:true})).toHaveCount(0);
  const keys = ['appearance.customCssEnabled','appearance.customJsEnabled'];
  const before = await Promise.all(keys.map(key => request(admin, `${ADMIN}/settings/${key}`)));
  await page.waitForTimeout(4000); await signInManually(page,{username:'testuser',password:'testpass'});
  await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
  const me = await request(page,'/api/me');
  const endpoint = `/api/account/appearance/customization?accountId=${encodeURIComponent(me.body.accountId)}`;
  const adminEndpoint = `${ADMIN}/me/appearance-customization?username=root`;
  const original = await request(page,endpoint), originalAdmin = await request(admin,adminEndpoint);
  const originalAppearance = await request(page,'/api/account/appearance');
  const originalAdminPrefs = await request(admin,`${ADMIN}/me/preferences`);
  expect(original.status).toBe(200); expect(originalAdmin.status).toBe(200);
  try {
    for (const key of keys) {const current=await request(admin,`${ADMIN}/settings/${key}`); expect((await request(admin,`${ADMIN}/settings/${key}`,{globalValue:true},current.etag!)).status).toBe(200);}
    await page.reload(); await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
    if (test.info().project.name === 'phone') await drawerButton(page).click();
    await page.locator('.sidebar__settings').click();
    await page.getByRole('button',{name:'Appearance',exact:true}).click();
    await pickColor(page,'Custom accent colour','#c23f9e');
    await expect.poll(async() => (await request(page,'/api/account/appearance')).body.appearance?.customAccent).toBe('#c23f9e');
    const editor = page.getByRole('region',{name:'Personal appearance customization'});
    await expect(editor.getByText(/Call appearance.set with an object/)).toBeVisible();
    await editor.getByLabel('Restricted CSS').fill('body{border-radius:19px}');
    await editor.getByLabel('Isolated appearance script').fill("appearance.set({density:'compact',font:'mono'});");
    await editor.getByRole('button',{name:'Save customization'}).click();
    await expect(editor.getByRole('status').filter({hasText:'Customization saved.'})).toBeVisible();
    await expect.poll(() => page.locator('.mw-custom-mail').evaluate(element=>getComputedStyle(element).borderRadius)).toBe('19px');
    await expect.poll(() => page.locator('.mw-custom-mail').evaluate(element=>getComputedStyle(element).fontSize)).toBe('13px');
    await page.reload();
    await expect.poll(() => page.locator('.mw-custom-mail').evaluate(element=>getComputedStyle(element).borderRadius)).toBe('19px');
    await admin.goto(`${ADMIN}#/preferences`);
    await pickColor(admin,'Custom accent colour','#126789');
    await expect.poll(async() => (await request(admin,`${ADMIN}/me/preferences`)).body.appearance?.customAccent).toBe('#126789');
    const adminEditor=admin.getByRole('region',{name:'Personal appearance customization'});
    await adminEditor.getByLabel('Restricted CSS').fill('body{border-radius:7px}');
    await adminEditor.getByLabel('Isolated appearance script').fill('');
    await adminEditor.getByRole('button',{name:'Save customization'}).click();
    await expect.poll(() => admin.locator('.mw-custom-admin').evaluate(element=>getComputedStyle(element).borderRadius)).toBe('7px');
    expect(await page.locator('.mw-custom-mail').evaluate(element=>getComputedStyle(element).borderRadius)).toBe('19px');
    expect(await page.evaluate(()=>document.documentElement.scrollWidth <= innerWidth+1)).toBe(true);
    if (test.info().project.name === 'phone') await drawerButton(page).click();
    await page.locator('.sidebar__settings').click();
    await page.getByRole('button',{name:'Appearance',exact:true}).click();
    await expect(colorTrigger(page,'Custom accent colour')).toHaveAttribute('data-color-value','#c23f9e');
    await editor.scrollIntoViewIfNeeded();
    await page.screenshot({path:test.info().outputPath('personal-appearance.png'),fullPage:true});
    await adminEditor.scrollIntoViewIfNeeded();
    await admin.screenshot({path:test.info().outputPath('admin-personal-appearance.png'),fullPage:true});
    await page.goto('/?appearanceSafe=1');
    await expect(page.locator('.mw-custom-mail')).toBeVisible();
    expect(await page.locator('.mw-custom-mail').evaluate(element=>getComputedStyle(element).borderRadius)).not.toBe('19px');
  } finally {
    for (const [target,path,saved] of [[page,endpoint,original],[admin,adminEndpoint,originalAdmin]] as const) {const current=await request(target,path); expect((await request(target,path,{ownerId:current.body.ownerId,document:saved.body.document},current.etag!)).status).toBe(200);}
    for (let i=0;i<keys.length;i++) {const path=`${ADMIN}/settings/${keys[i]}`, current=await request(admin,path); expect((await request(admin,path,{globalValue:before[i]!.body.globalValue},current.etag!)).status).toBe(200);}
    expect((await request(page,'/api/account/appearance',{appearance:originalAppearance.body.appearance ?? {}})).status).toBe(200);
    const adminPrefsNow=await request(admin,`${ADMIN}/me/preferences`);
    expect((await request(admin,`${ADMIN}/me/preferences`,originalAdminPrefs.body,adminPrefsNow.etag!)).status).toBe(200);
    await adminContext.close();
  }
});
