import {test,expect} from '../csrf.ts';
import {signInManually,messageRow,drawerButton,storageAuthorityHeaders} from '../helpers.ts';
import {sendRawSmtp} from '../crypto-helpers.ts';

test('contact pictures stay local, respect account choice and fall back to generated initials @phone',async({page},info)=>{
 const stamp=Date.now(),address=`photo-${stamp}@example.org`,subject=`Contact photo ${stamp}`;
 await sendRawSmtp({from:address,message:`From: Ana Pereira <${address}>\r\nTo: testuser@example.org\r\nDate: ${new Date().toUTCString()}\r\nMessage-ID: <photo-${stamp}@example.org>\r\nSubject: ${subject}\r\nContent-Type: text/plain\r\n\r\nLocal picture fixture.\r\n`});
 await signInManually(page,{username:'testuser',password:'testpass'});await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
 const me=await(await page.request.get('/api/me')).json();const accountId=me.accountId;
 const existing=await(await page.request.get('/api/settings')).json();const previous=existing.settings['contacts.avatarMode'];const headers=await storageAuthorityHeaders(page.request);let contacts:string[]=[];
 const call=async(name:string,args:object)=>{const response=await page.request.post('/jmap/api',{data:{using:['urn:ietf:params:jmap:core','urn:mailwoman:contacts'],methodCalls:[[name,{accountId,...args},'c']]}});expect(response.ok()).toBe(true);const body=await response.json();expect(body.methodResponses[0][0]).toBe(name);return body.methodResponses[0][1];};
 try{
  const image='iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGNQKHjwHwAElAJwfN7sMwAAAABJRU5ErkJggg==';
  const result=await call('ContactCard/import',{blob:`BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Ana Pereira\r\nEMAIL:${address}\r\nPHOTO;ENCODING=b;TYPE=PNG:${image}\r\nEND:VCARD\r\n`});expect(result.count).toBe(1);contacts=result.imported;
  if(info.project.name==='phone')await drawerButton(page).click();await page.locator('.sidebar__settings').click();await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:'Message display',exact:true}).click();
  const choices=page.getByRole('combobox',{name:'Sender and contact pictures',exact:true});await expect(choices).toBeEnabled();await choices.selectOption('contacts');await expect.poll(async()=>(await(await page.request.get('/api/settings')).json()).settings['contacts.avatarMode'].value).toBe('contacts');
  const remote:string[]=[];page.on('request',request=>{const url=new URL(request.url());if(['http:','https:'].includes(url.protocol)&&url.origin!==new URL(page.url()).origin)remote.push(url.origin);});
  await page.reload();await page.getByRole('tab',{name:'Mail',exact:true}).click();const row=messageRow(page,subject);await expect(row.locator('.person-avatar img')).toBeVisible({timeout:30000});await expect.poll(()=>row.locator('.person-avatar img').evaluate(node=>(node as HTMLImageElement).naturalWidth)).toBe(1);expect(remote).toEqual([]);
  await page.screenshot({path:info.outputPath('contact-picture.png'),fullPage:true});
  expect((await page.request.put('/api/settings/contacts.avatarMode',{headers,data:{value:'initials'}})).ok()).toBe(true);await page.reload();await page.getByRole('tab',{name:'Mail',exact:true}).click();await expect(row.locator('.person-avatar')).toHaveText('AP');await expect(row.locator('img')).toHaveCount(0);
  expect((await page.request.put('/api/settings/contacts.avatarMode',{headers,data:{value:'off'}})).ok()).toBe(true);await page.reload();await page.getByRole('tab',{name:'Mail',exact:true}).click();await expect(row).toBeVisible();await expect(row.locator('.person-avatar')).toHaveCount(0);expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
 }finally{
  if(contacts.length)await call('ContactCard/set',{destroy:contacts});
  const restored=previous?.userValue===undefined?await page.request.delete('/api/settings/contacts.avatarMode',{headers}):await page.request.put('/api/settings/contacts.avatarMode',{headers,data:{value:previous.userValue}});expect(restored.ok(),await restored.text()).toBe(true);
 }
});
