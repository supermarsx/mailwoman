import {test,expect} from '../csrf.ts';
import {signInManually} from '../helpers.ts';

test('indexing progress remains bounded and clears on completion @phone',async({page},info)=>{
 let status='syncing';
 await page.route('**/jmap/api',async route=>{
  const request=route.request().postDataJSON();
  const call=request?.methodCalls?.find((entry:unknown[])=>entry[0]==='Mailbox/get'&&entry[2]==='sync');
  if(!call){await route.continue();return;}
  await route.fulfill({json:{methodResponses:[['Mailbox/get',{accountId:call[1].accountId,mailwomanInitialSync:status},'sync']]}});
 });
 await signInManually(page,{username:'testuser',password:'testpass'});
 const notice=page.locator('.mail-sync-notice');await expect(notice).toBeVisible();await expect(notice).toBeInViewport();
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1&&document.documentElement.scrollHeight<=innerHeight+1)).toBe(true);
 await page.screenshot({path:info.outputPath('indexing-notice.png'),fullPage:true});
 status='retrying';await expect(notice).toContainText(/Reconnecting/i);status='ready';await expect(notice).toHaveCount(0);
});
