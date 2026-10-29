import {test,expect} from '../csrf.ts';
import {execFileSync} from 'node:child_process';
import type {Page} from '@playwright/test';

// t48: a linked mailbox keeps working after its private session ends; when
// upstream refuses its password, choosing it in the picker (a switcher only)
// opens Renew access, and Settings › Linked mailboxes shows the expired state
// and manages the link (rename, primary, reorder, renew, unlink) - no unlink and
// link again.
// Stack: node apps/web/e2e/receipts/linked-renew-fixture.mjs (prints the env).
const DB=process.env['MW_T48_DB']??'';
const GREENMAIL=process.env['MW_T48_GREENMAIL_API']??'http://127.0.0.1:46780';
const SHOTS=process.env['MW_SCREENSHOT_DIR'];

/** What the session clocks see when the linked mailbox's own session lapsed. */
function expireChildSessions():void{
  execFileSync('python',['-c',
    'import sqlite3,sys;c=sqlite3.connect(sys.argv[1]);n=c.execute("UPDATE sessions SET activity_at=\'2000-01-01T00:00:00+00:00\' WHERE linked_internal<>0").rowcount;c.commit();print(n)',DB]);
}
/** "The password was changed upstream": the fixture's Greenmail comes back on
 *  the same ports (same server, same storage authority) with a new password
 *  for the child. (Greenmail 2.1's user API cannot change one in place.) */
const CONTAINER=process.env['MW_T48_GREENMAIL_CONTAINER']??'mw-linked-renew-greenmail-46280';
const IMAP_PORT=process.env['MW_T48_IMAP_PORT']??'46743';
let childPassword='child-pass';
async function setChildPassword(page:Page,password:string):Promise<void>{
  if(password===childPassword)return;
  execFileSync('docker',['rm','-f',CONTAINER]);
  const api=new URL(GREENMAIL).port;
  execFileSync('docker',['run','-d','--name',CONTAINER,'-p',`127.0.0.1:${IMAP_PORT}:3143`,'-p',`127.0.0.1:${api}:8080`,
    '-e',`GREENMAIL_OPTS=-Dgreenmail.setup.test.all -Dgreenmail.hostname=0.0.0.0 -Dgreenmail.users.login=email -Dgreenmail.users=parent:parent-pass@example.org,child:${password}@child.test`,
    'greenmail/standalone:2.1.4']);
  await expect.poll(async()=>(await page.request.get(`${GREENMAIL}/api/user`).catch(()=>null))?.ok()??false,{timeout:60000}).toBe(true);
  childPassword=password;
}
async function drawer(page:Page){
  const open=page.getByRole('button',{name:'Open folders and apps',exact:true});
  if(await open.isVisible()&&await open.getAttribute('aria-expanded')!=='true')await open.click();
}
async function openMenu(page:Page){
  const trigger=page.locator('button[data-account-indicator="picker"]');
  if(!await trigger.isVisible())await drawer(page);
  if(await trigger.getAttribute('aria-expanded')!=='true')await trigger.click();
  const menu=page.getByRole('menu',{name:'Mailbox account',exact:true});
  await expect(menu).toBeVisible();
  return menu;
}
async function closeMenu(page:Page){
  const trigger=page.locator('button[data-account-indicator="picker"]');
  if(await trigger.count()>0&&await trigger.getAttribute('aria-expanded')==='true')await page.keyboard.press('Escape');
}
async function openSettings(page:Page){
  await closeMenu(page);
  const settings=page.locator('.sidebar, #shell-nav').getByRole('button',{name:'Settings',exact:true}).first();
  if(!await settings.isVisible())await drawer(page);
  await settings.click();
  await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:'Linked mailboxes',exact:true}).click();
  await expect(page.getByRole('heading',{name:'Linked mailboxes',exact:true})).toBeVisible();
}
async function shot(page:Page,project:string,name:string){
  if(SHOTS)await page.screenshot({path:`${SHOTS}/${project}-${name}`});
}
const SIZES=[{name:'1440',width:1440,height:900},{name:'390',width:390,height:844}] as const;

async function resize(page:Page,size:{width:number;height:number}){
  await page.setViewportSize(size);
  // The shell picks its phone or desktop layout at load.
  await page.reload();
  await expect(page.locator('[data-account-indicator]').first()).toBeAttached();
}
async function signIn(page:Page){
  await page.goto('/');
  await page.getByLabel('Email or username',{exact:true}).fill('parent@example.org');
  await page.getByLabel('Password',{exact:true}).fill('parent-pass');
  await page.getByRole('button',{name:'Sign in',exact:true}).click();
  await expect(page.getByLabel('Email or username',{exact:true})).toHaveCount(0);
}

test('a linked mailbox survives an ended session; an expired one renews from the picker; Settings manages links',async({page},info)=>{
  test.skip(info.project.name==='phone','the spec sets 1440 and 390 itself');
  test.skip(!DB,'needs MW_T48_DB from linked-renew-fixture.mjs');
  test.setTimeout(300000);
  page.setDefaultTimeout(15000);
  const project=info.project.name;
  const errors:string[]=[];page.on('pageerror',error=>errors.push(error.message));
  childPassword='';
  await setChildPassword(page,'child-pass');
  await page.setViewportSize({width:1440,height:900});
  await signIn(page);
  const primary=await (await page.request.get('/api/me')).json() as {storageAuthority:string};
  const owner={'x-mailwoman-storage-authority':primary.storageAuthority};
  expect((await page.request.post('/api/security/step-up',{data:{password:'parent-pass'}})).ok()).toBe(true);
  for(const stale of (await (await page.request.get('/api/accounts',{headers:owner})).json()).accounts as {id:string;revision:string}[])
    expect((await page.request.delete(`/api/accounts/${stale.id}`,{headers:{...owner,'if-match':JSON.stringify(stale.revision)}})).ok()).toBe(true);
  await page.request.put('/api/accounts/primary',{headers:owner,data:{linkId:null}});
  const linked=await page.request.post('/api/accounts/link',{headers:owner,data:{username:'child@child.test',password:'child-pass',displayName:'Child'}});
  expect(linked.status(),await linked.text()).toBe(201);
  const link=(await linked.json()).account.id as string;

  // 1. The child's own session ends: it still opens, with no message.
  expireChildSessions();
  await page.reload();
  let menu=await openMenu(page);
  await menu.locator(`[data-mailbox-item][data-link="${link}"]`).click();
  await expect(page.locator('button[data-account-indicator="picker"]')).toHaveAttribute('data-linked','true');
  await expect(page.getByRole('dialog',{name:/Renew access/})).toHaveCount(0);
  await expect(page.getByText(/Access (expired|to this mailbox expired)|[Uu]nlink this mailbox/)).toHaveCount(0);
  menu=await openMenu(page);
  await menu.locator('[data-mailbox-item][data-link=""]').click();
  await expect(page.locator('button[data-account-indicator="picker"]')).toHaveAttribute('data-linked','false');

  // 2. The password is changed upstream. The picker shows the entry with no
  //    status and no management (switcher only) at both widths.
  await setChildPassword(page,'child-pass-2');
  expireChildSessions();
  for(const size of SIZES){
    await resize(page,size);
    menu=await openMenu(page);
    const entry=menu.locator(`[data-mailbox-item][data-link="${link}"]`);
    await expect(entry).toHaveAttribute('data-expired','true');
    await expect(menu.getByRole('menuitem')).toHaveCount(0);
    await expect(menu.locator('[data-mailbox-grip],[data-mailbox-first],[data-mailbox-rename],[data-mailbox-renew]')).toHaveCount(0);
    await expect(menu.getByText(/^(Primary|Primary mailbox|Sign-in account|Renew access)$/)).toHaveCount(0);
    // "Access expired" is screen-reader text only: nothing is drawn.
    const hidden=(await menu.getByText('Access expired',{exact:true}).boundingBox())!;
    expect(hidden.width*hidden.height).toBeLessThanOrEqual(1);
    await shot(page,project,`t48-picker-switcher-expired-${size.name}.png`);
    await closeMenu(page);
  }
  await resize(page,{width:1440,height:900});

  // 3. Choosing the expired mailbox opens Renew access (never a dead end);
  //    after renewing it opens, live, with no reload.
  let loads=0;page.on('load',()=>loads++);
  menu=await openMenu(page);
  await menu.locator(`[data-mailbox-item][data-link="${link}"]`).click();
  const dialog=page.getByRole('dialog',{name:/Renew access to/});
  await expect(dialog).toBeVisible();
  await expect(dialog).not.toContainText(/[Uu]nlink/);
  await dialog.getByLabel('Password',{exact:true}).fill('wrong-password');
  await dialog.getByRole('button',{name:'Renew access',exact:true}).click();
  await expect(dialog.getByRole('alert')).toContainText('The password was not accepted');
  await shot(page,project,'t48-renew-dialog-1440.png');
  await dialog.getByLabel('Password',{exact:true}).fill('child-pass-2');
  await dialog.getByRole('button',{name:'Renew access',exact:true}).click();
  await expect(dialog).toHaveCount(0);
  await expect(page.locator('button[data-account-indicator="picker"]')).toHaveAttribute('data-linked','true');
  menu=await openMenu(page);
  await expect(menu.locator(`[data-mailbox-item][data-link="${link}"]`)).not.toHaveAttribute('data-expired','true');
  await expect(menu.locator(`[data-mailbox-item][data-link="${link}"]`)).toHaveAttribute('aria-checked','true');
  expect(loads,'renewed and opened without a reload').toBe(0);
  await menu.locator('[data-mailbox-item][data-link=""]').click();
  const after=(await (await page.request.get('/api/accounts',{headers:owner})).json()).accounts as {id:string;status:string;renewal:string}[];
  expect(after).toEqual([expect.objectContaining({id:link,status:'ready',renewal:'automatic'})]);

  // 4. Expired again (sign-out of the child would also do): Settings shows the
  //    status icon and Renew access in its row.
  await setChildPassword(page,'child-pass-3');
  expireChildSessions();
  await openSettings(page);
  for(const size of SIZES){
    await resize(page,size);
    await openSettings(page);
    const row=page.locator(`[data-linked-row][data-key="${link}"]`);
    await expect(row.locator('[data-linked-status]')).toHaveAttribute('data-tooltip','Access expired');
    await expect(row.getByRole('button',{name:/Access to .*child@child\.test.* expired\. Renew access/})).toBeVisible();
    await row.locator('[data-linked-renew]').hover();
    await shot(page,project,`t48-settings-expired-${size.name}.png`);
  }
  await resize(page,{width:1440,height:900});
  await openSettings(page);
  await page.locator(`[data-linked-row][data-key="${link}"] [data-linked-renew]`).click();
  const settingsDialog=page.getByRole('dialog',{name:/Renew access to/});
  await settingsDialog.getByLabel('Password',{exact:true}).fill('child-pass-3');
  await settingsDialog.getByRole('button',{name:'Renew access',exact:true}).click();
  await expect(settingsDialog).toHaveCount(0);
  await expect(page.locator(`[data-linked-row][data-key="${link}"] [data-linked-status]`)).toHaveCount(0);

  // 5. Settings manages the links: rename, primary star, keyboard reorder.
  const row=page.locator(`[data-linked-row][data-key="${link}"]`);
  await row.locator('[data-linked-rename]').click();
  const nickname=row.locator('[data-linked-nickname]');
  await nickname.fill(`Work ${project}`);
  await nickname.press('Enter');
  await expect(row.locator('[data-linked-name]')).toContainText(`Work ${project}`);
  await row.locator('[data-linked-set-first]').click();
  await expect(row.locator('[data-linked-set-first]')).toHaveAttribute('aria-pressed','true');
  await row.locator('[data-linked-grip]').focus();
  await page.keyboard.press('Alt+ArrowUp');
  await expect(page.locator('[data-linked-row]').first()).toHaveAttribute('data-key',link);
  const saved=await (await page.request.get('/api/accounts',{headers:owner})).json() as {primaryMailbox:string|null;layout:{order:string[];nicknames:Record<string,string>}};
  expect(saved.primaryMailbox).toBe(link);
  expect(saved.layout.order[0]).toBe(link);
  expect(saved.layout.nicknames[link]).toBe(`Work ${project}`);
  for(const size of SIZES){
    await resize(page,size);
    await openSettings(page);
    const current=page.locator(`[data-linked-row][data-key="${link}"]`);
    const buttons=current.locator('[data-linked-actions] button');
    await expect(buttons).toHaveCount(3);
    for(const button of await buttons.all()){
      await expect(button).toHaveAttribute('data-tooltip',/.+/);
      expect((await button.innerText()).trim()).toBe('');
      if(size.width===390){const box=(await button.boundingBox())!;expect(box.height).toBeGreaterThanOrEqual(43.5);expect(box.width).toBeGreaterThanOrEqual(43.5);}
    }
    const grip=(await current.locator('[data-linked-grip]').boundingBox())!;
    expect(grip.width,'slim grip').toBeLessThanOrEqual(20);
    await expect(page.getByText('Sign-in account',{exact:true})).toHaveCount(0);
    expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
    await current.locator('[data-linked-unlink]').hover();
    await shot(page,project,`t48-settings-after-${size.name}.png`);
    // The picker keeps the saved order, and only switches.
    menu=await openMenu(page);
    await expect(menu.locator('[data-mailbox-item]').first()).toHaveAttribute('data-link',link);
    await shot(page,project,`t48-picker-after-${size.name}.png`);
    await closeMenu(page);
  }
  await resize(page,{width:1440,height:900});
  await openSettings(page);

  // 6. Unlink is destructive and confirms in the shared dialog.
  await page.locator(`[data-linked-row][data-key="${link}"] [data-linked-unlink]`).click();
  const confirm=page.getByRole('alertdialog',{name:/^Unlink .*\?$/});
  await expect(confirm).toBeVisible();
  await confirm.getByRole('button',{name:'Confirm unlink',exact:true}).click();
  const proof=page.getByRole('dialog',{name:'Confirm it is you'});
  if(await proof.isVisible().catch(()=>false)){
    await proof.getByLabel('Password',{exact:true}).fill('parent-pass');
    await proof.getByRole('button',{name:'Confirm and continue',exact:true}).click();
  }
  await expect.poll(async()=>((await (await page.request.get('/api/accounts',{headers:owner})).json()).accounts as unknown[]).length).toBe(0);
  await page.reload();
  await openSettings(page);
  await expect(page.locator('[data-linked-empty]')).toBeVisible();
  await shot(page,project,'t48-settings-empty-1440.png');
  await setChildPassword(page,'child-pass');
  expect(errors).toEqual([]);
});
