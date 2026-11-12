import {test,expect} from '../csrf.ts';
import {ADMIN} from './admin-path.ts';

test('guided contracts and notification resolution fit the compliance view @phone',async({page})=>{
 const data={contract:[] as Record<string,unknown>[],transfer:[],control:[],processor:[{id:'provider',name:'Example hosting',builtin:false,enabled:true}],catalog:[]};
 let delivery='uncertain';
 await page.route(`**${ADMIN}/compliance/governance`,route=>route.fulfill({json:data}));
 await page.route(`**${ADMIN}/compliance/records`,async route=>{const body=route.request().postDataJSON();const row={id:'contract-1',builtin:false,enabled:true,name:body.name,...body.fields};data.contract.push(row);await route.fulfill({status:201,json:row});});
 await page.route(`**${ADMIN}/compliance/mail`,route=>route.fulfill({json:{passwordConfigured:false,items:[{id:'mail-1',event:'dsr-admin:request-1',status:delivery,at:1800000000,attempts:1,error:'Acceptance unknown'}]}}));
 await page.route(`**${ADMIN}/compliance/mail/mail-1/resolve`,async route=>{expect(route.request().postDataJSON()).toEqual({action:'retry',acceptDuplicateRisk:true});delivery='pending';await route.fulfill({json:{status:delivery}});});
 await page.goto(ADMIN);await page.getByLabel('Admin username').fill('root');await page.getByLabel('Password',{exact:true}).fill('hunter2');const signedIn=page.waitForResponse(response=>response.url().endsWith(`${ADMIN}/login`)&&response.request().method()==='POST');await page.getByRole('button',{name:'Sign in',exact:true}).click();expect((await signedIn).status()).toBe(200);
 await page.goto(`${ADMIN}#/compliance/contracts`);const view=page.getByTestId('governance-contract');await view.getByRole('button',{name:'Add record',exact:true}).click();
 const dialog=page.getByTestId('governance-editor');await dialog.getByLabel('Name',{exact:true}).fill('Hosting DPA');await dialog.getByLabel('Owner',{exact:true}).fill('Privacy team');await dialog.getByLabel('Counterparty',{exact:true}).fill('Example hosting');await dialog.getByRole('button',{name:'Next',exact:true}).click();
 await dialog.getByRole('combobox',{name:'Sub-processor',exact:true}).selectOption('provider');await dialog.getByLabel('Contract document URL',{exact:true}).fill('https://example.org/contracts/v1');await dialog.getByRole('button',{name:'Next',exact:true}).click();await expect(dialog.getByText('https://example.org/contracts/v1',{exact:true})).toBeVisible();await dialog.getByRole('button',{name:'Save',exact:true}).click();
 await expect(view.getByRole('rowheader',{name:'Hosting DPA',exact:true})).toBeVisible();
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
 await page.goto(`${ADMIN}#/compliance/notifications`);const notices=page.getByTestId('compliance-notifications');await notices.getByRole('button',{name:'Retry',exact:true}).click();const confirmation=page.getByRole('alertdialog');await expect(confirmation).toContainText('may deliver a duplicate');await confirmation.getByRole('button',{name:'Retry',exact:true}).click();await expect(notices.getByText('Queued',{exact:true})).toBeVisible();
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
});
