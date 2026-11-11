import {test,expect} from '../csrf.ts';
import {login} from '../helpers.ts';

test('unified mail scopes every query and opens the original mailbox',async({page})=>{
 const linked={id:'linked-fixture',accountId:'fixture-account',storageAuthority:'msa1_fixture_linked_mailbox',address:'second@example.org',displayName:'Second mailbox',status:'ready',revision:'v1'};
 const accounts=new Set<string>();
 await page.route('**/api/settings',async route=>{
  if(route.request().method()!=='GET')return route.continue();
  const response=await route.fetch();
  const body=await response.json();
  body.settings['mail.unifiedInbox']={value:true,source:'user',locked:false};
  await route.fulfill({response,json:body});
 });
 await page.route('**/api/accounts',async route=>{
  const response=await route.fetch();
  const body=await response.json();
  body.accounts=[linked];
  body.primaryMailbox=null;
  body.policy={...body.policy,enabled:true,maxAccounts:4};
  await route.fulfill({response,json:body});
 });
 await page.route('**/jmap/api',async route=>{
  const request=route.request(), headers=request.headers();
  const account=headers['x-mailwoman-account'], authority=headers['x-mailwoman-storage-authority'];
  if(request.method()!=='POST'||!account||!authority)return route.continue();
  const child=headers['x-mailwoman-link']===linked.id;
  if(child){
   expect(account).toBe(linked.accountId);
   expect(authority).toBe(linked.storageAuthority);
  }else expect(account).not.toBe(linked.accountId);
  const calls=request.postDataJSON().methodCalls as [string,Record<string,unknown>,string][];
  const folder=child?'secondary-inbox':'primary-inbox';
  const subject=child?'From the linked account':'From the primary account';
  const id='same-message-id';
  const list=calls.map(([name,args,tag])=>{
   expect(args['accountId']).toBe(account);
   if(name==='Mailbox/get')return [name,{accountId:account,list:[{id:folder,role:'inbox',name:'Inbox',parentId:null}]},tag];
   if(name==='Email/query'){
    expect(args['filter']).toMatchObject({inMailbox:folder});
    accounts.add(account);
    return [name,{accountId:account,ids:[id],position:0},tag];
   }
   return [name,{accountId:account,list:[{id,subject,receivedAt:child?'2026-11-10T22:00:00Z':'2026-11-10T20:00:00Z',mailboxIds:{[folder]:true},from:[{email:'sender@example.org'}]}]},tag];
  });
  await route.fulfill({json:{methodResponses:list,sessionState:'fixture'}});
 });
 await login(page);
 await page.locator('#shell-nav').getByRole('button',{name:'Unified mail',exact:true}).click();
 const unified=page.getByRole('region',{name:'Unified mail',exact:true});
 const first=unified.getByRole('link',{name:'From the linked account'});
 await expect(first).toHaveAttribute('href',/mailbox=linked-fixture.*folder=secondary-inbox/);
 await expect(unified.getByRole('link',{name:'From the primary account'})).toHaveAttribute('href',/message=same-message-id&folder=primary-inbox/);
 await expect(unified.getByRole('row').nth(1)).toContainText('From the linked account');
 expect(accounts.size).toBe(2);
 await unified.getByRole('combobox',{name:'Account',exact:true}).selectOption(linked.id);
 await expect(first).toBeVisible();
 await expect(unified.getByRole('link',{name:'From the primary account'})).toHaveCount(0);
});
