import {test,expect} from '../csrf.ts';
import {sendRawSmtp} from '../crypto-helpers.ts';

test('related searches and selected-message moves use real account-owned messages',async({page},info)=>{
 const suffix=`${Date.now()}-${info.project.name}`,sender=`interactions-${suffix}@example.org`,subjects=[`Interaction A ${suffix}`,`Interaction B ${suffix}`];
 for(const subject of subjects)await sendRawSmtp({from:sender,to:'testuser@example.org',message:`From: ${sender}\r\nTo: testuser@example.org\r\nSubject: ${subject}\r\nMessage-ID: <${subject.replaceAll(' ','-')}@example.org>\r\n\r\nDisposable interaction fixture.\r\n`});
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
 const session=await(await page.request.get('/jmap/session')).json(),accountId=session.primaryAccounts['urn:ietf:params:jmap:mail'];
 const call=async(name:string,args:Record<string,unknown>)=>{const response=await page.request.post('/jmap/api',{data:{using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[[name,{accountId,...args},'c']]}});expect(response.ok()).toBe(true);const body=await response.json();expect(body.methodResponses[0][0],JSON.stringify(body)).toBe(name);return body.methodResponses[0][1];};
 const boxes=await call('Mailbox/get',{}),inbox=boxes.list.find((box:{role:string})=>box.role==='inbox');
 const created=await call('Mailbox/set',{create:{target:{name:`Interaction ${suffix}`}}});const folderId=created.created?.target?.id;expect(folderId,JSON.stringify(created)).toBeTruthy();
 const ids:string[]=[];
 try {
  await expect.poll(async()=>{const query=await call('Email/query',{filter:{from:sender}});return query.ids.length;}).toBe(2);
  ids.push(...(await call('Email/query',{filter:{from:sender}})).ids);
  await page.reload();await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
  const row=(subject:string)=>page.locator('.list__row').filter({hasText:subject});
  await expect(row(subjects[0]!)).toBeVisible();
  const openMenu=async()=>{if(info.project.name==='phone')await row(subjects[0]!).locator('..').getByTestId('msg-more-toggle').click();else await row(subjects[0]!).click({button:'right'});};
  await openMenu();await page.getByRole('menuitem',{name:'Search this sender'}).click();await expect(row(subjects[0]!)).toBeVisible();await expect(row(subjects[1]!)).toBeVisible();
  await openMenu();await page.getByRole('menuitem',{name:'Search the same subject'}).click();await expect(row(subjects[0]!)).toBeVisible();await expect(row(subjects[1]!)).toHaveCount(0);
  if(info.project.name==='phone')await page.getByRole('button',{name:'Open folders and apps'}).click();await page.locator('.sidebar__box').filter({hasText:/^Inbox/}).first().click();
  if(info.project.name==='phone'){const close=page.getByRole('button',{name:'Close folders and apps'});if(await close.isVisible())await close.click();await page.getByRole('button',{name:'Select messages',exact:true}).click();await row(subjects[0]!).click();await row(subjects[1]!).click();}
  else {await row(subjects[0]!).click({modifiers:['Control']});await row(subjects[1]!).click({modifiers:['Control']});}
  await expect(page.getByTestId('bulk-count')).toContainText('2');
  if(info.project.name==='phone')await page.getByTestId('bulk-move').selectOption(folderId);
  else await row(subjects[0]!).dragTo(page.locator('.sidebar__box').filter({hasText:`Interaction ${suffix}`}));
  await expect.poll(async()=>{const result=await call('Email/get',{ids,properties:['id','mailboxIds']});return result.list.filter((message:{mailboxIds:Record<string,boolean>})=>message.mailboxIds[folderId]).length;}).toBe(2);
  await expect(row(subjects[0]!)).toHaveCount(0);await expect(row(subjects[1]!)).toHaveCount(0);
  await page.screenshot({path:info.outputPath('selected-messages-moved.png'),fullPage:true});
 } finally {
  if(ids.length)await call('Email/set',{update:Object.fromEntries(ids.map(id=>[id,{mailboxIds:{[inbox.id]:true}}]))});
  const remaining=await call('Email/query',{filter:{inMailbox:folderId}});if(remaining.ids.length===0)await call('Mailbox/set',{destroy:[folderId]});
 }
});
