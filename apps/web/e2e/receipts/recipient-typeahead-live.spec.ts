import {test,expect} from '../csrf.ts';
import {withUserSetting} from '../helpers.ts';

test('recipient typeahead selects with keyboard and real pointer without submitting',async({page,playwright,baseURL},info)=>{
  const restoreMode=await withUserSetting(playwright.request,baseURL!,'mail.draftSaveMode','manual');
  let restoreWindow=async()=>{};
  try {
  restoreWindow=await withUserSetting(playwright.request,baseURL!,'mail.composeWindow','tab');
  await page.goto('/');
  await page.getByLabel('Email or username',{exact:true}).fill('testuser');
  await page.getByLabel('Password',{exact:true}).fill('testpass');
  const signedIn=page.waitForResponse(response=>new URL(response.url()).pathname==='/api/login'&&response.request().method()==='POST');
  await page.getByRole('button',{name:'Sign in',exact:true}).click();
  const {accountId}=await (await signedIn).json() as {accountId:string};
  await expect(page.locator('.mail-pane')).toBeVisible();
  const suffix=`${info.project.name}-${Date.now()}`;
  const email=`typeahead-${suffix}@example.org`,name=`Recipient ${suffix}`;
  async function jmap(method:string,args:Record<string,unknown>){
    for(let attempt=0;;attempt++){
      const response=await page.request.post('/jmap/api',{data:{using:['urn:ietf:params:jmap:core','urn:mailwoman:contacts'],methodCalls:[[method,{accountId,...args},'contact']]}});
      const body=await response.json() as {limit?:string;methodResponses?:Array<[string,Record<string,unknown>,string]>};
      if(response.status()===429&&body.limit==='maxConcurrentRequests'&&attempt<4){await page.waitForTimeout(500);continue;}
      expect(response.ok(),`fixture JMAP ${response.status()}: ${JSON.stringify(body)}`).toBe(true);
      return body.methodResponses![0]!;
    }
  }
  const created=await jmap('ContactCard/set',{create:{c:{name:{full:name},emails:[{context:'work',value:email,pref:1}]}}});
  expect(created[0]).toBe('ContactCard/set');
  const id=(created[1].created as Record<string,{id:string}>).c!.id;
  try{
    await page.reload();await expect(page.locator('.mail-pane')).toBeVisible();
    if((page.viewportSize()?.width??1280)<600)await page.locator('header.shell__bar').getByRole('button',{name:'Compose',exact:true}).click();
    else await page.locator('.sidebar__compose').click();
    const compose=page.locator('form.compose').last(),to=compose.getByRole('combobox',{name:'To',exact:true});
    await to.fill(`typeahead-${suffix}`);
    const option=page.getByRole('option',{name:new RegExp(email.replaceAll('.','\\.'))});
    await expect(option).toBeVisible();
    await expect(to).toHaveAttribute('aria-expanded','true');
    await to.press('ArrowDown');await expect(to).toHaveAttribute('aria-activedescendant',/.+/);
    await to.press('Enter');await expect(to).toHaveValue(`${name} <${email}>, `);
    await expect(compose).toBeVisible();await expect(to).toHaveAttribute('aria-expanded','false');
    await to.fill(`typeahead-${suffix}`);await expect(option).toBeVisible();
    if(info.project.use.hasTouch)await option.tap();else await option.click();
    await expect(to).toHaveValue(`${name} <${email}>, `);await expect(to).toBeFocused();
    await to.fill(`typeahead-${suffix}`);await expect(option).toBeVisible();
    await to.press('Escape');await expect(option).not.toBeVisible();await expect(compose).toBeVisible();
    await page.screenshot({path:info.outputPath('recipient-typeahead.png'),fullPage:true});
  }finally{
    try{
      const compose=page.locator('form.compose').last();
      if(await compose.isVisible()){
        await compose.getByRole('button',{name:'Discard account draft',exact:true}).click();
        await page.getByTestId('compose-discard-confirm').getByRole('button',{name:'Discard',exact:true}).click();
        await expect(compose).not.toBeVisible();
      }
    }finally{const removed=await jmap('ContactCard/set',{destroy:[id]});expect(removed[0]).toBe('ContactCard/set');}
  }
  }finally{try{await restoreWindow();}finally{await restoreMode();}}
});
