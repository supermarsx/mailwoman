import {test,expect} from '../csrf.ts';
import {sendRawSmtp} from '../crypto-helpers.ts';
import {postJmap,withUserSetting} from '../helpers.ts';

test('inline forwarding preserves safe newsletter structure through the production editor @phone',async({page,playwright,baseURL},info)=>{
 test.setTimeout(120_000);
 const restores:Array<()=>Promise<void>>=[];
 const subject=`Forward layout ${info.project.name} ${Date.now()}`;
 let emailId='';
 const html='<div style="font-family:Arial;font-size:14px;color:#123456;line-height:1.4"><p>Newsletter opening</p><table style="width:280px;border-collapse:separate"><tr><td style="padding:12px;background-color:#eeeeee;vertical-align:middle"><h2>Forward heading</h2><ul><li>First item</li><li>Second item</li></ul><table style="width:180px"><tr><td style="padding:8px;border:2px solid #112233">Nested content</td></tr></table></td></tr></table><div style="text-align:right">Closing line</div></div>';
 let accountId='';
 const call=async(name:string,args:Record<string,unknown>)=>{const response=await postJmap(page.request,{data:{using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[[name,{accountId,...args},'fixture']]}});expect(response.ok()).toBe(true);const body=await response.json();expect(body.methodResponses[0][0]).toBe(name);return body.methodResponses[0][1];};
 try{
  for(const [key,value]of [['mail.forwardMode','inline'],['mail.composeWindow','tab'],['mail.draftSaveMode','manual']] as const)restores.push(await withUserSetting(playwright.request,baseURL!,key,value));
  await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect(page.getByRole('searchbox',{name:'Search mail'})).toBeVisible();
  accountId=(await(await page.request.get('/api/me')).json()).accountId;
  await sendRawSmtp({from:'newsletter@example.org',to:'testuser@example.org',message:`From: newsletter@example.org\r\nTo: testuser@example.org\r\nSubject: ${subject}\r\nDate: ${new Date().toUTCString()}\r\nMessage-ID: <${Date.now()}-layout@example.org>\r\nMIME-Version: 1.0\r\nContent-Type: text/html; charset=utf-8\r\nContent-Transfer-Encoding: base64\r\n\r\n${Buffer.from(html).toString('base64')}\r\n`});
  await expect.poll(async()=>(await call('Email/query',{filter:{subject}})).ids.length,{timeout:30000}).toBe(1);emailId=(await call('Email/query',{filter:{subject}})).ids[0];
  await page.reload();const row=page.locator('.list__row').filter({hasText:subject});await expect(row).toBeVisible({timeout:30000});await row.click();
  await expect(page.frameLocator('iframe.reader__frame').getByText('Nested content',{exact:true})).toBeVisible();
  await page.getByTestId('reader-forward').click();
  const editor=page.locator('form.compose .ProseMirror').last();await expect(editor).toBeVisible();await expect(editor.locator('table')).toHaveCount(2);await expect(editor.locator('td h2')).toHaveText('Forward heading');await expect(editor.locator('td li')).toHaveCount(2);
  expect(await editor.locator('table').first().evaluate(node=>(node as HTMLElement).style.width)).toBe('280px');expect(await editor.locator('table table').evaluate(node=>(node as HTMLElement).style.width)).toBe('180px');
  expect(await editor.locator('table table td').evaluate(node=>getComputedStyle(node).borderTopWidth)).toBe('2px');
  await editor.click();await page.keyboard.press('Control+End');await page.keyboard.type('Forwarding note');
  await expect(editor).toContainText('Forwarding note');await expect(editor.locator('table')).toHaveCount(2);
  await expect.poll(()=>page.evaluate(expected=>{const drafts=JSON.parse(localStorage.getItem('mw.compose.drafts.v2')??'[]') as {subject:string;bodyHtml:string}[];return drafts.find(draft=>draft.subject.includes(expected))?.bodyHtml??'';},subject)).toContain('Forwarding note');
  await page.screenshot({path:info.outputPath('forward-newsletter-layout.png'),fullPage:true});
  const compose=page.locator('form.compose').last();await compose.getByRole('button',{name:'Discard account draft',exact:true}).click();await page.getByTestId('compose-discard-confirm').getByRole('button',{name:'Discard',exact:true}).click();
 }finally{if(emailId)await call('Email/set',{destroy:[emailId]});for(const restore of restores.reverse())await restore();}
});
