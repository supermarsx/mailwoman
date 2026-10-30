import {test,expect} from '../csrf.ts';
import {injectViaSmtp,messageRow,postJmap} from '../helpers.ts';

test('guided search opens before a query and keeps the Trash choice explicit',async({page},info)=>{
  const token=`guided-${info.project.name}-${Date.now()}`;
  const sender=`${token}@example.org`;
  const inboxSubject=`Inbox ${token}`,trashSubject=`Trash ${token}`;
  await injectViaSmtp({from:sender,subject:inboxSubject,text:'Guided search fixture.'});
  await injectViaSmtp({from:sender,subject:trashSubject,text:'Guided search fixture in Trash.'});
  await page.setViewportSize({width:info.project.name==='phone'?390:1440,height:900});
  await page.goto('/');
  await page.getByLabel('Email or username',{exact:true}).fill('testuser');
  await page.getByLabel('Password',{exact:true}).fill('testpass');
  await page.getByRole('button',{name:'Sign in',exact:true}).click();
  const input=page.getByRole('searchbox',{name:'Search mail'});
  await expect(input).toBeVisible();
  const session=await (await page.request.get('/jmap/session')).json();
  const accountId=session.primaryAccounts['urn:ietf:params:jmap:mail'];
  let boxes=await (await postJmap(page.request,{data:{using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[['Mailbox/get',{accountId},'m']]}})).json();
  if(!boxes.methodResponses[0][1].list.some((box:{role:string})=>box.role==='trash')){
    const created=await postJmap(page.request,{data:{using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[['Mailbox/set',{accountId,create:{trash:{name:'Trash'}}},'create']]}});expect(created.ok()).toBe(true);
    boxes=await (await postJmap(page.request,{data:{using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[['Mailbox/get',{accountId},'m']]}})).json();
    await page.reload();await expect(input).toBeVisible();
  }
  const trash=boxes.methodResponses[0][1].list.filter((box:{role:string})=>box.role==='trash').map((box:{id:string})=>box.id) as string[];
  expect(trash.length).toBeGreaterThan(0);
  let trashMessage='';let state='';
  await expect(async()=>{
    const response=await postJmap(page.request,{data:{using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[
      ['Email/query',{accountId,filter:{text:`from:${sender}`}},'q'],
      ['Email/get',{accountId,'#ids':{resultOf:'q',name:'Email/query',path:'/ids'},properties:['subject']},'g'],
    ]}});
    const body=await response.json();const result=body.methodResponses[1][1];
    expect(result.list).toHaveLength(2);state=result.state;
    trashMessage=result.list.find((email:{subject:string})=>email.subject===trashSubject).id;
  }).toPass({timeout:60000});
  const moved=await (await postJmap(page.request,{data:{using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[['Email/set',{accountId,ifInState:state,update:{[trashMessage]:{mailboxIds:{[trash[0]!]:true}}}},'m']]}})).json();
  expect(moved.methodResponses[0][1].updated).toHaveProperty(trashMessage);
  const filters:Record<string,unknown>[]=[];
  page.on('request',request=>{
    if(request.method()!=='POST'||!request.url().endsWith('/jmap/api'))return;
    const body=request.postDataJSON() as {methodCalls:[string,{filter?:Record<string,unknown>},string][]};
    for(const [method,args] of body.methodCalls??[])if(method==='Email/query'&&String(args.filter?.['text']).includes(sender))filters.push(args.filter!);
  });
  const toggle=page.getByRole('button',{name:'Filters',exact:true});
  await toggle.click();
  const panel=page.getByRole('region',{name:'Guided search'});
  await expect(panel).toBeVisible();
  await panel.getByRole('button',{name:'From',exact:true}).click();
  const editor=page.getByRole('dialog',{name:'Edit filter: From'});
  await editor.getByRole('textbox').fill(sender);
  await editor.getByRole('button',{name:'Apply',exact:true}).click();
  await expect(input).toHaveValue(`from:${sender}`);
  await expect.poll(()=>filters.at(-1)?.['inMailboxOtherThan']).toEqual(trash);
  await expect(messageRow(page,inboxSubject)).toBeVisible();
  await expect(messageRow(page,trashSubject)).toHaveCount(0);
  await panel.getByRole('checkbox',{name:'Include Trash'}).check();
  await expect(input).toHaveValue(`from:${sender} include:trash`);
  await expect.poll(()=>filters.at(-1)).toMatchObject({text:`from:${sender}`});
  await expect.poll(()=>filters.at(-1)?.['inMailboxOtherThan']).toBeUndefined();
  expect(filters.at(-1)?.['text']).not.toContain('include:');
  await expect(messageRow(page,inboxSubject)).toBeVisible();
  await expect(messageRow(page,trashSubject)).toBeVisible();
  await page.screenshot({path:info.outputPath('guided-search.png'),fullPage:true});
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
  await panel.getByRole('checkbox',{name:'Include Trash'}).press('Escape');
  await expect(panel).toHaveCount(0);
  await expect(toggle).toBeFocused();
  await expect(input).toHaveValue(`from:${sender} include:trash`);
  await toggle.click();
  await panel.getByRole('checkbox',{name:'Include Trash'}).uncheck();
  await expect(messageRow(page,trashSubject)).toHaveCount(0);
  await panel.getByRole('button',{name:'Folder',exact:true}).click();
  const folderEditor=page.getByRole('dialog',{name:'Edit filter: Folder'});
  await folderEditor.getByRole('combobox').selectOption(trash[0]!);
  await folderEditor.getByRole('button',{name:'Apply',exact:true}).click();
  await expect(messageRow(page,trashSubject)).toBeVisible();
  await expect(messageRow(page,inboxSubject)).toHaveCount(0);
  await expect(panel.getByRole('checkbox',{name:'Include Trash'})).not.toBeChecked();
  await expect(panel.getByRole('checkbox',{name:'Include Trash'})).toBeDisabled();
});
