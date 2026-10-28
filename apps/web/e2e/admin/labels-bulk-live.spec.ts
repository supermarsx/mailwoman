import {test,expect} from '../csrf.ts';
import {signInManually,messageRow,drawerButton,storageAuthorityHeaders} from '../helpers.ts';
import {sendRawSmtp} from '../crypto-helpers.ts';
import {readFile} from 'node:fs/promises';

test('account labels, touch selection, attached forward and one mbox archive work together @phone',async({page},info)=>{
  const stamp=Date.now();const subjects=[`Bulk Alpha ${stamp}`,`Bulk Beta ${stamp}`];
  for(const subject of subjects)await sendRawSmtp({from:'sender@example.test',message:`From: sender@example.test\r\nTo: testuser@example.org\r\nDate: ${new Date().toUTCString()}\r\nMessage-ID: <${subject.replaceAll(' ','-')}@example.test>\r\nSubject: ${subject}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nOriginal message for ${subject}.\r\n`});
  await signInManually(page,{username:'testuser',password:'testpass'});
  await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
  const original=await page.request.get('/api/account/preferences');expect(original.ok(),await original.text()).toBe(true);const previous=await original.json();
  try{
    if(info.project.name==='phone')await drawerButton(page).click();await page.locator('.sidebar__settings').click();await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:'Labels',exact:true}).click();
    await page.getByRole('button',{name:'New label'}).click();await expect(page.getByRole('table')).toHaveCount(0);
    const name=`Travel ${stamp}`;const keyword=`travel-${stamp}`;await page.getByLabel('Label name',{exact:true}).fill(name);await page.getByLabel('Label colour',{exact:true}).fill('#ffff00');await page.getByLabel('Label icon',{exact:true}).selectOption('star');await page.getByRole('button',{name:'Save label',exact:true}).click();
    await expect(page.getByRole('table')).toContainText(name);await expect.poll(async()=>(await(await page.request.get('/api/account/preferences')).json()).preferences.tags?.[keyword]?.name).toBe(name);
    await page.reload();await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();await page.getByRole('tab',{name:'Mail',exact:true}).click();
    await page.getByRole('button',{name:'Select messages',exact:true}).click();for(const subject of subjects)await messageRow(page,subject).click();
    const bar=page.getByRole('toolbar',{name:'Selected messages'});await expect(bar).toContainText('2 selected');
    const updated=page.waitForRequest(request=>request.url().endsWith('/jmap/api')&&request.method()==='POST'&&request.postDataJSON()?.methodCalls?.some((call:unknown[])=>call[0]==='Email/set'));
    await bar.getByRole('combobox',{name:'Add label',exact:true}).selectOption(keyword);
    const mutation=(await updated).postDataJSON().methodCalls.find((call:unknown[])=>call[0]==='Email/set')[1];const ids=Object.keys(mutation.update);expect(ids).toHaveLength(2);
    await expect.poll(async()=>{
      const response=await page.request.post('/jmap/api',{data:{using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[['Email/get',{accountId:mutation.accountId,ids,properties:['keywords']},'g']]}});
      const result=await response.json();return result.methodResponses.find((call:unknown[])=>call[2]==='g')?.[1]?.list?.every((email:{keywords:Record<string,boolean>})=>email.keywords[keyword]===true);
    }).toBe(true);
    let submissions=0;page.on('request',request=>{if(request.url().endsWith('/jmap/api')&&request.method()==='POST'&&request.postDataJSON()?.methodCalls?.some((call:unknown[])=>call[0]==='EmailSubmission/set'))submissions++;});
    await bar.getByRole('button',{name:'Forward selected messages as attachments'}).click();await expect(page.getByTestId('compose-attachments').locator('li')).toHaveCount(2);await expect(page.getByLabel('To',{exact:true})).toHaveValue('');expect(submissions).toBe(0);
    await page.screenshot({path:info.outputPath('bulk-forward.png'),fullPage:true});await page.getByRole('tab',{name:'Mail',exact:true}).click();
    const download=page.waitForEvent('download');await bar.getByRole('button',{name:'Download selected messages as mbox'}).click();
    const proof=page.getByRole('dialog',{name:'Confirm it is you'});if(await proof.waitFor({state:'visible',timeout:3000}).then(()=>true,()=>false)){await proof.getByLabel('Password',{exact:true}).fill('testpass');await proof.getByRole('button',{name:'Confirm and continue',exact:true}).click();}
    const archive=await download;expect(archive.suggestedFilename()).toBe('messages.mbox');const path=await archive.path();expect(path).toBeTruthy();const text=await readFile(path!,'utf8');for(const subject of subjects)expect(text).toContain(subject);expect(text.match(/^From /gm)).toHaveLength(2);
    expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
  }finally{
    const current=await page.request.get('/api/account/preferences');const restored=await page.request.put('/api/account/preferences',{headers:{...(await storageAuthorityHeaders(page.request)),'if-match':current.headers()['etag']!},data:previous});expect(restored.ok(),await restored.text()).toBe(true);
  }
});
