import {test,expect} from '../csrf.ts';
import {signInManually,messageRow,drawerButton,storageAuthorityHeaders} from '../helpers.ts';
import {sendRawSmtp} from '../crypto-helpers.ts';
test('compact native reader icons keep helpful names and account-saved visible labels @phone',async({page},info)=>{
 const subject=`Reader ribbon ${Date.now()}`;await sendRawSmtp({from:'ribbon@example.org',message:`From: Ribbon sender <ribbon@example.org>\r\nTo: testuser@example.org\r\nSubject: ${subject}\r\nMessage-ID: <ribbon-${Date.now()}@example.org>\r\nContent-Type: text/plain\r\n\r\nMessage action ribbon.\r\n`});
 await signInManually(page,{username:'testuser',password:'testpass'});await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
 const previous=(await(await page.request.get('/api/settings')).json()).settings['reader.actionStyle'];const headers=await storageAuthorityHeaders(page.request);expect(previous).toBeTruthy();
 try{
  expect((await page.request.put('/api/settings/reader.actionStyle',{headers,data:{value:'icons'}})).ok()).toBe(true);await page.reload();await page.getByRole('tab',{name:'Mail',exact:true}).click();await messageRow(page,subject).click();
  const toolbar=page.getByRole('toolbar',{name:'Message actions',exact:true});const reply=toolbar.getByRole('button',{name:'Reply',exact:true});await expect(reply).toBeVisible();await expect(reply.locator('svg')).toHaveCount(1);expect(await reply.evaluate(node=>node.classList.contains('reader-action--compact'))).toBe(true);
  await page.keyboard.press('Tab');await reply.focus();await expect(page.getByRole('tooltip')).toContainText('Reply to the sender in a new compose tab.');await page.keyboard.press('Escape');
  await page.screenshot({path:info.outputPath('reader-ribbon-icons.png'),fullPage:true});
  await reply.click();await expect(page.getByLabel('To',{exact:true})).toHaveValue(/ribbon@example.org/);await page.getByRole('tab',{name:'Mail',exact:true}).click();
  if(info.project.name==='phone')await drawerButton(page).click();await page.locator('.sidebar__settings').click();await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:'Message display',exact:true}).click();await page.getByRole('combobox',{name:'Message action ribbon',exact:true}).selectOption('labels');
  await expect.poll(async()=>(await(await page.request.get('/api/settings')).json()).settings['reader.actionStyle'].value).toBe('labels');await page.reload();await page.getByRole('tab',{name:'Mail',exact:true}).click();await messageRow(page,subject).click();await expect(reply).toBeVisible();expect(await reply.evaluate(node=>node.classList.contains('reader-action--compact'))).toBe(false);expect(await reply.locator('span').evaluate(node=>getComputedStyle(node).position)).not.toBe('absolute');
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1&&document.documentElement.scrollHeight<=innerHeight+1)).toBe(true);await page.screenshot({path:info.outputPath('reader-ribbon-labels.png'),fullPage:true});
 }finally{const restored=previous.userValue===undefined?await page.request.delete('/api/settings/reader.actionStyle',{headers}):await page.request.put('/api/settings/reader.actionStyle',{headers,data:{value:previous.userValue}});expect(restored.ok(),await restored.text()).toBe(true);}
});
