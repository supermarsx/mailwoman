import {test,expect,type Page} from '../csrf.ts';

test('account Drafts resumes on a fresh device and explicit discard removes the owned copy',async({page,browser},info)=>{
 const login=async(p:Page)=>{await p.goto('/');await p.getByLabel('Email or username',{exact:true}).fill('testuser');await p.getByLabel('Password',{exact:true}).fill('testpass');await p.getByRole('button',{name:'Sign in',exact:true}).click();await expect(p.getByRole('button',{name:'Compose',exact:true})).toBeVisible();};
 const compose=async(p:Page)=>{if(info.project.name==='phone')await p.getByRole('button',{name:'Open folders and apps'}).click();await p.locator('#shell-nav').getByRole('button',{name:'Compose',exact:true}).click();};
 await login(page);await compose(page);
 const subject=`Account Draft ${info.project.name} ${Date.now()}`;
 await page.getByLabel('To',{exact:true}).fill('unfinished@');
 await page.getByLabel('Subject',{exact:true}).fill(subject);
 await expect(page.getByLabel('Attach a file',{exact:true})).toBeEnabled();
 await page.getByLabel('Attach a file',{exact:true}).setInputFiles({name:'draft-proof.txt',mimeType:'text/plain',buffer:Buffer.from('durable draft attachment')});
 await expect(page.getByText('draft-proof.txt',{exact:true})).toBeVisible();
 await expect(page.getByTestId('account-draft-status')).toHaveText('Saved to account Drafts',{timeout:45000});
 const saved=await page.evaluate(async(subject)=>{const list=await(await fetch('/api/account/drafts')).json();return list.items.find((row:{summary:{subject:string}})=>row.summary.subject===subject);},subject);
 expect(saved.emailId).toBeTruthy();expect(saved.revision).toBeTruthy();
 await page.getByRole('button',{name:'Close',exact:true}).click();
 const fresh=await browser.newContext({baseURL:process.env['MW_RECEIPTS_BASE_URL']??'http://localhost:42221',viewport:info.project.name==='phone'?{width:390,height:844}:{width:1280,height:900},serviceWorkers:'block'});
 try{
  const other=await fresh.newPage();await login(other);await compose(other);
  expect(await other.evaluate(()=>localStorage.getItem('mw.compose.drafts.v2'))).toBeNull();
  await other.getByTestId('open-drafts').click();
  const list=other.getByRole('region',{name:'Account drafts'});
  await list.getByRole('button',{name:subject,exact:true}).click();
  await expect(other.getByLabel('Subject',{exact:true})).toHaveValue(subject);
  await expect(other.getByLabel('To',{exact:true})).toHaveValue('unfinished@');
  await expect(other.getByText('draft-proof.txt',{exact:true}).first()).toBeVisible();
  await other.screenshot({path:info.outputPath('account-draft-cross-device.png'),fullPage:true});
  expect(await other.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  await other.getByRole('button',{name:'Close',exact:true}).click();
  if(info.project.name==='phone')await other.getByRole('button',{name:'Open folders and apps'}).click();
  await other.locator('#shell-nav').getByRole('button',{name:'Drafts',exact:true}).click();
  await other.locator('.list__row').filter({hasText:subject}).click();
  await expect(other.getByLabel('Subject',{exact:true})).toHaveValue(subject);
  await other.getByTestId('open-drafts').click();
  const row=list.locator('div').filter({has:other.getByRole('button',{name:subject,exact:true})});
  await row.getByRole('button',{name:'Discard account draft',exact:true}).click();
  await other.getByRole('alertdialog').getByRole('button',{name:'Confirm draft removal',exact:true}).click();
  await expect.poll(async()=> (await other.request.get(`/api/account/drafts/${saved.id}`)).status()).toBe(404);
 }finally{await fresh.close();}
});
