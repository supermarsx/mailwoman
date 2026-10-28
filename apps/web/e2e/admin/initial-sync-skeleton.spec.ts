import {test,expect} from '../csrf.ts';
import {signInManually} from '../helpers.ts';

// First sync after sign-in: no text banner, no premature "No messages". The
// list shows skeleton rows (empty folder) or keeps its rows under a thin
// decorative progress line, and settles once the engine reports it indexed.
test('first-sync progress shows skeletons or a progress line, never a banner @phone',async({page},info)=>{
 let status='syncing',synced=false;
 await page.route('**/jmap/api',async route=>{
  const request=route.request().postDataJSON();
  const call=request?.methodCalls?.find((entry:unknown[])=>entry[0]==='Mailbox/get'&&entry[2]==='sync');
  if(!call){await route.continue();return;}
  await route.fulfill({json:{methodResponses:[['Mailbox/get',{accountId:call[1].accountId,mailwomanInitialSync:status,list:(call[1].ids??[]).map((id:string)=>({id,mailwomanSynced:synced}))},'sync']]}});
 });
 await signInManually(page,{username:'testuser',password:'testpass'});
 const progress=page.locator('.list [data-skeleton="messages"], .list [data-testid="sync-progress"]').first();
 await expect(progress).toBeVisible();
 await expect(page.getByText(/Synchronizing your messages/)).toHaveCount(0);
 await expect(page.locator('.list__empty')).toHaveCount(0);
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
 await page.screenshot({path:info.outputPath('first-sync.png'),fullPage:true});
 status='ready';synced=true;
 await expect(page.locator('.list [data-skeleton="messages"], .list [data-testid="sync-progress"]')).toHaveCount(0);
});
