import {test,expect} from '../csrf.ts';
import {ADMIN} from '../admin/admin-path.ts';
import {pinComposerRibbonOnPhone,revealRibbonControl,useTabbedWorkspace,withUserSetting} from '../helpers.ts';
import type {AdminSetting} from '../../src/api/admin/settings.ts';

test('personal and shared response templates insert literal text and replace only after consent',async({page,playwright,baseURL},info)=>{
 const url=baseURL!;const restoreEnabled=await withUserSetting(playwright.request,url,'mail.templates.enabled',true);
 const restorePersonal=await withUserSetting(playwright.request,url,'mail.templates.personal',[]);
 const restoreDraftMode=await withUserSetting(playwright.request,url,'mail.draftSaveMode','manual');
 const unpin=await pinComposerRibbonOnPhone(playwright.request,url,info.project.name);
 const admin=await playwright.request.newContext({baseURL:url});let restoreLayout=async()=>{};
 const key='mail.templates.shared',endpoint=`${ADMIN}/settings/${key}`;let original:AdminSetting|undefined;
 try{
  expect((await admin.post(`${ADMIN}/login`,{data:{username:process.env['MW_ADMIN_USER']??'root',password:process.env['MW_ADMIN_PASSWORD']??'hunter2'}})).status()).toBe(200);
  const before=await admin.get(endpoint);original=await before.json() as AdminSetting;
  expect((await admin.put(endpoint,{headers:{'if-match':before.headers()['etag']!},data:{globalValue:[{id:'browser-shared',title:'Shared greeting',body:'Shared <b>literal</b> text'}],allowUserOverride:false,locked:false}})).status()).toBe(200);
  await page.setViewportSize({width:info.project.name==='phone'?390:1440,height:900});await page.goto('/');
  await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();
  await expect(page.getByRole('region',{name:'Messages',exact:true})).toBeVisible();restoreLayout=await useTabbedWorkspace(page);
  const settings=page.locator('.sidebar, #shell-nav').getByRole('button',{name:'Settings',exact:true}).first();
  if(!await settings.isVisible())await page.locator('header.shell__bar button[aria-controls="shell-nav"]').first().click();await settings.click();
  await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:'Composing and replies',exact:true}).click();
  const library=page.getByRole('region',{name:'Response templates',exact:true});await library.getByRole('button',{name:'Add template'}).click();
  await library.getByLabel('Template name').fill('Personal greeting');await library.getByLabel('Message text').fill('Personal <img src=x>\nSecond line');
  await library.getByRole('button',{name:'Save templates'}).click();await expect(library.getByRole('button',{name:'Save templates'})).toBeDisabled();
  await expect.poll(async()=>((await(await page.request.get('/api/settings')).json()).settings['mail.templates.personal'].value)).toMatchObject([{title:'Personal greeting',body:'Personal <img src=x>\nSecond line'}]);
  await page.getByRole('tab',{name:'Mail',exact:true}).click();
  if(info.project.name==='phone')await page.locator('header.shell__bar').getByRole('button',{name:'Compose',exact:true}).click();else await page.locator('.sidebar__compose').click();
  const compose=page.locator('form.compose').last(),editor=compose.locator('.compose-rt__surface .ProseMirror');await expect(editor).toBeVisible();
  await compose.getByLabel('To',{exact:true}).fill('recipient@example.org');await compose.getByLabel('Subject',{exact:true}).fill('Template fixture');await editor.fill('Original body');
  const ribbon=page.locator('[data-bar="composerRibbon"]').last();
  async function picker(){await ribbon.getByRole('tab',{name:'Insert',exact:true}).click();const button=page.getByRole('button',{name:'Response templates',exact:true});if(info.project.name==='phone'){const group=ribbon.getByTestId('compose-response-templates');await expect(group).toHaveAttribute('data-folded','true');const more=group.getByRole('button',{name:'More Response templates commands',exact:true});if(await more.getAttribute('aria-expanded')!=='true')await more.click();await expect(button).toBeInViewport();}else await revealRibbonControl(page,ribbon,button);await button.click();return page.getByRole('dialog',{name:'Response templates',exact:true});}
  let panel=await picker();await panel.getByLabel('Find a template').fill('Personal');await expect(panel.getByRole('button',{name:'Shared greeting · Shared'})).toHaveCount(0);await panel.getByRole('button',{name:'Personal greeting · Personal'}).click();await panel.getByRole('button',{name:'Insert text',exact:true}).click();
  await expect(editor).toContainText('Personal <img src=x>');await expect(editor.locator('img')).toHaveCount(0);await expect(editor).toContainText('Original body');
  panel=await picker();await panel.getByLabel('Find a template').fill('Shared');await panel.getByRole('button',{name:'Shared greeting · Shared'}).click();await panel.getByRole('button',{name:'Replace message text'}).click();
  await page.getByRole('alertdialog',{name:'Replace your message text?'}).getByRole('button',{name:'Cancel',exact:true}).click();await expect(editor).toContainText('Original body');
  if(!await panel.isVisible())panel=await picker();
  await panel.getByRole('button',{name:'Replace message text'}).click();await page.getByRole('alertdialog',{name:'Replace your message text?'}).getByRole('button',{name:'Replace message text'}).click();await expect(editor).toHaveText('Shared <b>literal</b> text');await expect(editor.locator('b')).toHaveCount(0);
  await editor.click();await page.keyboard.press('Control+z');await expect(editor).toContainText('Original body');await expect(compose.getByLabel('To',{exact:true})).toHaveValue('recipient@example.org');await expect(compose.getByLabel('Subject',{exact:true})).toHaveValue('Template fixture');
  await page.screenshot({path:info.outputPath('response-templates.png'),fullPage:true});
  await compose.getByRole('button',{name:'Discard account draft',exact:true}).click();await page.getByTestId('compose-discard-confirm').getByRole('button',{name:'Discard',exact:true}).click();await expect(compose).toHaveCount(0);
 }finally{
  try{if(original){const current=await admin.get(endpoint);const headers={'if-match':current.headers()['etag']!};const restored=original.updatedAt===null?await admin.delete(endpoint,{headers}):await admin.put(endpoint,{headers,data:{globalValue:original.globalValue,allowUserOverride:original.allowUserOverride,locked:original.globalLocked}});expect([200,204]).toContain(restored.status());}}finally{await admin.dispose();await restoreLayout();await restorePersonal();await restoreEnabled();await restoreDraftMode();await unpin();}
 }
});
