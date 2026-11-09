import {test,expect} from '@playwright/test';
import {CREDS,signInManually,messageRow} from './helpers.ts';

test('compact reply banner groups its icons and supports confirmed discard and opening a local draft',async({page},info)=>{
  await page.route(/\/api\/settings(?:\?.*)?$/,async route=>{
    const response=await route.fetch(),body=await response.json();
    body.settings['notifications.askPermission']={value:false,source:'user',locked:false};
    await route.fulfill({response,json:body});
  });
  await page.route(/\/api\/account\/preferences(?:\?.*)?$/,async route=>{
    expect(route.request().method()).toBe('GET');
    const response=await route.fetch(),body=await response.json();
    body.preferences={...body.preferences,readingPane:'right',readerCollapsed:false,messageListCollapsed:false,workspaceLayout:'tabs',markRead:'never'};
    await route.fulfill({response,json:body});
  });
  // The editor may autosave after opening. Keep every draft API request in the
  // browser fixture: this check never creates/changes a shared server draft.
  await page.route(/\/api\/account\/drafts(?:\/[^?]*)?(?:\?.*)?$/,async route=>{
    const headers=route.request().headers();
    if(route.request().method()==='GET'&&new URL(route.request().url()).pathname.endsWith('/drafts'))await route.fulfill({json:{accountId:headers['x-mailwoman-account'],storageAuthority:headers['x-mailwoman-storage-authority'],supported:false,items:[]}});
    else await route.fulfill({status:403,json:{code:'draftsUnsupported'}});
  });
  await signInManually(page,CREDS);await expect(page.getByRole('button',{name:'Compose',exact:true})).toBeVisible();
  const row=messageRow(page,'Lunch tomorrow?');await expect(row).toBeVisible();
  const emailId=await row.getAttribute('data-email-id');expect(emailId).toBeTruthy();
  const me=await (await page.request.get('/api/me')).json() as {accountId:string;storageAuthority:string};
  expect(me.storageAuthority).toBeTruthy();
  const seed=async()=>page.evaluate(({me,emailId})=>{
    const draft={id:'compact-reply-browser-only',accountId:me.accountId,storageAuthority:me.storageAuthority,to:'reply@example.org',subject:'Browser-only compact reply',bodyHtml:'<p>Local browser fixture.</p>',bodyText:'Local browser fixture.',savedAt:Date.now(),source:{emailId,keyword:'$answered'}};
    localStorage.setItem('mw.compose.drafts.v2',JSON.stringify([draft]));
    dispatchEvent(new CustomEvent('mw:drafts-changed',{detail:{kind:'save',id:draft.id,accountId:me.accountId,storageAuthority:me.storageAuthority}}));
  },{me,emailId:emailId!});
  await seed();await row.click();
  const banner=page.getByTestId('reply-in-progress-notice').filter({visible:true});
  await expect(banner).toBeVisible();await expect(banner).toContainText('Reply in progress');
  const open=banner.getByRole('button',{name:'Open draft',exact:true}),discard=banner.getByTestId('reader-discard-reply');
  await expect(open).toBeVisible();await expect(discard).toBeVisible();
  const geometry=await banner.evaluate(node=>{
    const bounds=node.getBoundingClientRect(),buttons=[...node.querySelectorAll('button')].map(button=>{const b=button.getBoundingClientRect();return{x:b.x,y:b.y,width:b.width,height:b.height,text:button.textContent?.trim()};});
    return{height:bounds.height,right:bounds.right,buttons};
  });
  console.log(info.project.name,JSON.stringify(geometry));
  expect(geometry.buttons).toHaveLength(2);expect(geometry.buttons.every(button=>button.text==='')).toBe(true);
  expect(Math.abs(geometry.buttons[0]!.y-geometry.buttons[1]!.y)).toBeLessThanOrEqual(1);
  expect(geometry.buttons[1]!.x-geometry.buttons[0]!.x-geometry.buttons[0]!.width).toBeLessThanOrEqual(6);
  expect(geometry.right-geometry.buttons[1]!.x-geometry.buttons[1]!.width).toBeLessThanOrEqual(12);
  if(info.project.name==='desktop')expect(geometry.height).toBeLessThanOrEqual(36);
  expect(geometry.height).toBeLessThanOrEqual(56);
  await page.screenshot({path:info.outputPath('compact-reply-banner.png')});
  await discard.click();const dialog=page.getByTestId('reply-discard-confirm');await expect(dialog).toBeVisible();
  await dialog.getByRole('button',{name:'Cancel',exact:true}).click();await expect(banner).toBeVisible();
  await discard.click();await dialog.getByRole('button',{name:'Discard',exact:true}).click();await expect(banner).toHaveCount(0);
  expect(await page.evaluate(()=>JSON.parse(localStorage.getItem('mw.compose.drafts.v2')??'[]').length)).toBe(0);
  await seed();await expect(banner).toBeVisible();await open.click();
  await expect(page.getByLabel('Subject',{exact:true})).toHaveValue('Browser-only compact reply');
  await page.screenshot({path:info.outputPath('compact-reply-open.png')});
});
