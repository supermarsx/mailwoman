import {test,expect} from '../csrf.ts';
import type {Locator,Page} from '@playwright/test';

// t86: mailbox avatars and Settings section icons, live against a server built
// from the tree. Stack: node apps/web/e2e/receipts/linked-live-fixture.mjs (see
// that file), then
//   MW_RECEIPTS_BASE_URL=... npx playwright test -c apps/web/e2e/receipts/playwright.config.ts mailbox-avatar-live
// Projects: chromium and firefox at 1440, phone (Chromium) at 390; each runs in
// light and dark. MW_SCREENSHOT_DIR keeps a copy of every screenshot.

const SECTION_ICONS:Record<string,string>={
  'Linked mailboxes':'mailboxes',Identities:'identityProvider',Signatures:'signature','Security centre':'shield',Privacy:'eyeOff',
  'Message display':'mailRead','Composing and replies':'compose','Folders and sidebar':'folder',Labels:'tag','Mail rules':'filter',
  'Saved searches':'search','Read receipts':'mailCheck','Import and export mail':'transfer',
  Appearance:'palette','Keyboard & display':'keyboard',Notifications:'bell','Time zones':'clock',
  'Nextcloud accounts':'nextcloud','Package tracking':'package','API access':'key','Server information':'serverMetadata',
};

async function drawer(page:Page){
  const open=page.getByRole('button',{name:'Open folders and apps',exact:true});
  if(await open.isVisible()&&await open.getAttribute('aria-expanded')!=='true')await open.click();
}
async function picker(page:Page){
  const trigger=page.locator('button[data-account-indicator="picker"]');
  if(!await trigger.isVisible())await drawer(page);
  await expect(trigger).toBeVisible();
  return trigger;
}
async function openMenu(page:Page){
  const trigger=await picker(page);
  if(await trigger.getAttribute('aria-expanded')!=='true')await trigger.click();
  const menu=page.getByRole('menu',{name:'Mailbox account',exact:true});
  await expect(menu).toBeVisible();
  return menu;
}
async function closeMenu(page:Page){
  const trigger=page.locator('button[data-account-indicator="picker"]');
  if(await trigger.count()>0&&await trigger.getAttribute('aria-expanded')==='true')await page.keyboard.press('Escape');
}
async function openSettings(page:Page,section:string){
  await closeMenu(page);
  const settings=page.locator('.sidebar, #shell-nav').getByRole('button',{name:'Settings',exact:true}).first();
  if(!await settings.isVisible())await drawer(page);
  await settings.click();
  await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:section,exact:true}).click();
}
async function shot(page:Page,info:{outputPath(name:string):string;project:{name:string}},name:string){
  await page.screenshot({path:info.outputPath(name),fullPage:false});
  const dir=process.env['MW_SCREENSHOT_DIR'];
  if(dir)await page.screenshot({path:`${dir}/${info.project.name}-${name}`,fullPage:false});
}
const rgb=(hex:string)=>{const n=parseInt(hex.slice(1),16);return `rgb(${n>>16}, ${(n>>8)&255}, ${n&255})`;};
/** The core light or dark pack on :root (as state/slices/theme.ts sets it; see undo-redo-live). */
async function setTheme(page:Page,theme:'light'|'dark'){
  await page.emulateMedia({colorScheme:theme});
  await page.evaluate(value=>{document.documentElement.setAttribute('data-theme',value);document.documentElement.setAttribute('data-appearance',value);},theme);
}
async function colours(avatar:Locator){return avatar.evaluate(el=>{const s=getComputedStyle(el);return {bg:s.backgroundColor,fg:s.color};});}

/** Type a hex value into a ColorField and commit it. */
async function setColour(field:Locator,hex:string){
  await field.locator('button').first().click();
  const pop=field.locator('[data-color-popover]');
  await expect(pop).toBeVisible();
  const input=pop.locator('[data-color-hex]');
  await input.fill(hex);
  await input.press('Enter');
  await expect(pop).toBeHidden();
}

for(const scheme of ['light','dark'] as const){
  test(`mailbox avatars: icon and letter with colours in table, picker and switcher; persists; section icons (${scheme})`,async({page},info)=>{
    page.setDefaultTimeout(15000);
    await page.emulateMedia({colorScheme:scheme});
    const errors:string[]=[];page.on('pageerror',error=>errors.push(error.message));
    const csp:string[]=[];page.on('console',m=>{if(/Content Security Policy|Refused to apply/i.test(m.text()))csp.push(m.text());});
    await page.goto('/');
    await page.getByLabel('Email or username',{exact:true}).fill('parent@example.org');
    await page.getByLabel('Password',{exact:true}).fill('parent-pass');
    await page.getByRole('button',{name:'Sign in',exact:true}).click();
    await expect(page.getByLabel('Email or username',{exact:true})).toHaveCount(0);
    await expect(page.locator('[data-account-indicator]').first()).toBeAttached();
    const primary=await (await page.request.get('/api/me')).json() as {storageAuthority:string};
    const owner={'x-mailwoman-storage-authority':primary.storageAuthority};
    // Start from one linked child and the generated avatars.
    let listing=await (await page.request.get('/api/accounts',{headers:owner})).json() as {accounts:{id:string;address?:string;revision:string}[];layout:{revision:string;order:string[];nicknames:Record<string,string>}};
    if(!listing.accounts.some(a=>a.address==='child@child.test')){
      expect((await page.request.post('/api/security/step-up',{data:{password:'parent-pass'}})).ok()).toBe(true);
      const linked=await page.request.post('/api/accounts/link',{headers:owner,data:{username:'child@child.test',password:'child-pass',displayName:'Child'}});
      expect(linked.ok(),await linked.text()).toBe(true);
      listing=await (await page.request.get('/api/accounts',{headers:owner})).json();
    }
    const link=listing.accounts.find(a=>a.address==='child@child.test')!.id;
    await expect(page.locator('html')).toHaveAttribute('data-theme',/.+/);
    const reset=await page.request.put('/api/accounts/layout',{headers:owner,data:{revision:listing.layout.revision,order:listing.layout.order,nicknames:{},avatars:{}}});
    expect(reset.ok(),await reset.text()).toBe(true);
    await page.reload();
    await expect(page.locator('[data-account-indicator]').first()).toBeAttached();
    await setTheme(page,scheme);

    // Section icons: every nav entry draws its own glyph.
    await openSettings(page,'Linked mailboxes');
    const nav=page.getByRole('navigation',{name:'Settings sections'});
    let seen=0;
    for(const [section,icon] of Object.entries(SECTION_ICONS)){
      const button=nav.getByRole('button',{name:section,exact:true});
      if(await button.count()===0)continue;
      seen++;
      // Lazy section catalogs can replace the navigation between reading its
      // label and scrolling it. Re-resolve that same section, not a stale node.
      await expect(async()=>{
        await expect(button.locator('svg[data-icon]'),section).toHaveAttribute('data-icon',icon);
        await button.scrollIntoViewIfNeeded();
        await expect(button.locator('svg[data-icon]')).toBeVisible();
        const box=await button.locator('svg[data-icon]').boundingBox();
        expect(box?.width,section).toBeGreaterThan(10);
      }).toPass({timeout:10000});
    }
    // Sections the server does not offer are hidden; most are always there.
    expect(seen).toBeGreaterThanOrEqual(15);
    await shot(page,info,`${scheme}-settings-section-icons.png`);

    // The child: an office icon, white on blue.
    const row=page.locator(`[data-linked-row][data-key="${link}"]`);
    const rowAvatar=row.locator('[data-linked-avatar] .person-avatar');
    const before=await rowAvatar.boundingBox();
    await row.locator('[data-linked-avatar]').click();
    const editor=page.locator(`[data-avatar-editor="${link}"]`);
    await expect(editor).toBeVisible();
    await editor.locator('[data-avatar-mode="icon"]').click();
    const office=editor.locator('[data-avatar-choice="office"]');
    await office.hover();
    await expect(office).toHaveAttribute('data-tooltip','Office building');
    await expect(office).toHaveAccessibleName('Office building');
    await office.click();
    await expect(editor.locator('[data-avatar-preview] .person-avatar')).toHaveAttribute('data-avatar-icon','office');
    await setColour(editor.locator('[data-avatar-bg] [data-color-field]'),'#1d4ed8');
    await setColour(editor.locator('[data-avatar-fg] [data-color-field]'),'#ffffff');
    await expect(editor.locator('[data-avatar-contrast]')).toHaveAttribute('data-avatar-contrast','ok');
    await shot(page,info,`${scheme}-avatar-editor-icon.png`);
    await editor.locator('[data-avatar-save]').click();
    await expect(editor).toBeHidden();
    await expect(rowAvatar).toHaveAttribute('data-avatar-icon','office');
    expect(await colours(rowAvatar)).toEqual({bg:rgb('#1d4ed8'),fg:rgb('#ffffff')});
    const after=await rowAvatar.boundingBox();
    expect(Math.abs(after!.width-before!.width)).toBeLessThan(0.5);
    expect(Math.abs(after!.height-before!.height)).toBeLessThan(0.5);

    // The sign-in mailbox: a two-letter avatar, dark text on yellow; then a
    // manual colour that fails AA is called out and Auto fixes it.
    const main=page.locator('[data-linked-row][data-key="main"]');
    await main.locator('[data-linked-avatar]').click();
    const mainEditor=page.locator('[data-avatar-editor="main"]');
    await expect(mainEditor).toBeVisible();
    await mainEditor.locator('[data-avatar-letter]').fill('Me');
    await setColour(mainEditor.locator('[data-avatar-bg] [data-color-field]'),'#fde68a');
    await setColour(mainEditor.locator('[data-avatar-fg] [data-color-field]'),'#f5f5f5');
    await expect(mainEditor.locator('[data-avatar-contrast="low"]')).toBeVisible();
    await shot(page,info,`${scheme}-avatar-editor-low-contrast.png`);
    await mainEditor.locator('[data-avatar-use-auto]').click();
    await expect(mainEditor.locator('[data-avatar-contrast]')).toHaveAttribute('data-avatar-contrast','ok');
    await setColour(mainEditor.locator('[data-avatar-fg] [data-color-field]'),'#7c2d12');
    await expect(mainEditor.locator('[data-avatar-contrast]')).toHaveAttribute('data-avatar-contrast','ok');
    await mainEditor.locator('[data-avatar-save]').click();
    await expect(mainEditor).toBeHidden();
    const mainAvatar=main.locator('[data-linked-avatar] .person-avatar');
    await expect(mainAvatar).toHaveText('Me');
    expect(await colours(mainAvatar)).toEqual({bg:rgb('#fde68a'),fg:rgb('#7c2d12')});
    await shot(page,info,`${scheme}-linked-table-avatars.png`);

    // The sidebar switcher (trigger) and the picker entries.
    const check=async()=>{
      const trigger=await picker(page);
      const head=trigger.locator('.person-avatar');
      await expect(head).toHaveText('Me');
      expect(await colours(head)).toEqual({bg:rgb('#fde68a'),fg:rgb('#7c2d12')});
      const menu=await openMenu(page);
      const child=menu.locator(`[data-mailbox-item][data-link="${link}"] .person-avatar`);
      await expect(child).toHaveAttribute('data-avatar-icon','office');
      expect(await colours(child)).toEqual({bg:rgb('#1d4ed8'),fg:rgb('#ffffff')});
      await expect(menu.locator('[data-mailbox-item][data-link=""] .person-avatar')).toHaveText('Me');
    };
    await check();
    await shot(page,info,`${scheme}-picker-avatars.png`);
    await closeMenu(page);

    // It is kept on the server: the same after a reload.
    await page.reload();
    await expect(page.locator('[data-account-indicator]').first()).toBeAttached();
    await setTheme(page,scheme);
    await check();
    await closeMenu(page);

    // Switching to the child: the switcher shows its icon, in a box the same
    // size as before (no shift in the sidebar head).
    const head=(await picker(page)).locator('.person-avatar');
    const headBefore=await head.boundingBox();
    const menu=await openMenu(page);
    await menu.locator(`[data-mailbox-item][data-link="${link}"]`).click();
    await expect(page.locator('button[data-account-indicator="picker"]')).toHaveAttribute('data-linked','true');
    const childHead=(await picker(page)).locator('.person-avatar');
    await expect(childHead).toHaveAttribute('data-avatar-icon','office');
    const headAfter=await childHead.boundingBox();
    expect(Math.abs(headAfter!.width-headBefore!.width)).toBeLessThan(0.5);
    expect(Math.abs(headAfter!.height-headBefore!.height)).toBeLessThan(0.5);
    await shot(page,info,`${scheme}-switcher-child.png`);
    // Back to the sign-in mailbox for the next run.
    const back=await openMenu(page);
    await back.locator('[data-mailbox-item][data-link=""]').click();
    await expect(page.locator('button[data-account-indicator="picker"]')).toHaveAttribute('data-linked','false');

    expect(errors).toEqual([]);
    expect(csp).toEqual([]);
  });
}
