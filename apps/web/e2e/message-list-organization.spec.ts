import { test, expect, type Page } from '@playwright/test';
import { login } from './helpers.ts';
test.use({ serviceWorkers: 'block' }); test.setTimeout(75000);
test.afterEach(async ({page}) => { await page.unrouteAll({behavior:'wait'}); });
type Entry = {value:unknown;source:string;locked:boolean};
type JmapBody = {methodCalls:[string,Record<string,unknown>,string][]};
async function fixture(page:Page) {
  await page.setViewportSize({width:1280,height:1000});
  const entries:Record<string,Entry> = {
    'mail.messageListOrder':{value:'newest',source:'user',locked:false}, 'mail.messageListGroup':{value:'none',source:'user',locked:false},
    'mail.messageListStyle':{value:'cards',source:'user',locked:false}, 'mail.list.pageSize':{value:20,source:'global',locked:true},
    'notifications.askPermission':{value:false,source:'global',locked:true},
  };
  const writes:{key:string;value:unknown}[]=[], moves:Record<string,unknown>[]=[], queries:Record<string,unknown>[]=[]; const moved=new Set<string>();
  await page.route(/\/api\/settings(?:\/mail\.messageList(?:Order|Group))?(?:\?.*)?$/,async route=>{
    const request=route.request(),key=new URL(request.url()).pathname.split('/').at(-1)!;
    if(request.method()==='PUT'){const value=(request.postDataJSON()as{value:unknown}).value;entries[key]={value,source:'user',locked:false};writes.push({key,value});await route.fulfill({status:200,json:entries[key]});return;}
    await route.fulfill({status:200,json:{settings:entries}});
  });
  await page.route(/\/jmap(?:\/api)?(?:\?.*)?$/,async route=>{
    const body=route.request().postDataJSON()as JmapBody;
    // An actual move gesture is recorded and acknowledged locally. Never send
    // an Email/set to the standing backend from this regression fixture.
    if(body.methodCalls.some(([name])=>name==='Email/set')){
      const responses=body.methodCalls.map(([name,args,id])=>{
        if(name!=='Email/set')return [name,{},id];const update=(args['update']??{})as Record<string,unknown>;
        for(const [message,patch]of Object.entries(update)){moves.push({message,patch});moved.add(message);}
        return [name,{accountId:args['accountId'],oldState:'s',newState:'s2',updated:Object.fromEntries(Object.keys(update).map(id=>[id,null]))},id];
      });await route.fulfill({status:200,json:{methodResponses:responses,sessionState:'s'}});return;
    }
    // Leave unrelated long-lived JMAP reads outside the fixture handler, so
    // unrouteAll(wait) never waits on a real polling request during teardown.
    if(!body.methodCalls.every(([name])=>['Email/query','Email/get','Mailbox/get'].includes(name))){await route.continue();return;}
    // Query/get responses are wholly synthetic; no real read is needed and
    // cancellation of superseded pages cannot leave a delayed proxy response.
    const result = {methodResponses:body.methodCalls.map(([name,args,id])=>[name,{accountId:args['accountId'],state:'organization',queryState:'organization'},id]),sessionState:'organization'} as {methodResponses:[string,Record<string,unknown>,string][]};
    const query=body.methodCalls.find(([name])=>name==='Email/query');
    const queryArgs=query?.[1],filter=(queryArgs?.['filter']??{})as Record<string,unknown>,box=typeof filter['inMailbox']==='string'?filter['inMailbox']:'inbox';
    const corpus=Array.from({length:120},(_,index)=>({id:`organization-${index}`,mailboxIds:{[box]:true},from:[{name:`Sender ${String(index%12).padStart(2,'0')}`,email:`sender${String(index%12).padStart(2,'0')}@example.test`}],to:[{name:null,email:'testuser@example.org'}],subject:`Organization message ${String(index).padStart(3,'0')}`,receivedAt:new Date(Date.UTC(2026,10,9,12,0,index)).toISOString(),sentAt:new Date(Date.UTC(2026,10,8,12,0,index)).toISOString(),preview:`Organization preview ${index}`,keywords:{$seen:true},size:1000+index*100,mailwomanPriority:index%3===0?'high':index%3===1?'normal':'low',hasAttachment:false,htmlBody:[],textBody:[]})).filter(email=>!moved.has(email.id));
    const sort=(queryArgs?.['sort']??[])as{property:string;isAscending?:boolean;timeZone?:string}[];
    corpus.sort((a,b)=>{for(const rule of sort){
      const value=(email:typeof a):string|number=>{
        if(rule.property==='mailwomanGroupReceived'||rule.property==='mailwomanGroupSent')return new Intl.DateTimeFormat('en-CA',{timeZone:rule.timeZone??'UTC',year:'numeric',month:'2-digit',day:'2-digit'}).format(new Date(rule.property==='mailwomanGroupSent'?email.sentAt:email.receivedAt));
        if(rule.property==='from'||rule.property==='mailwomanGroupFrom')return email.from[0]!.email.toLowerCase();
        if(rule.property==='mailwomanGroupSubject')return email.subject.replace(/^(?:(?:re|fw|fwd)\s*:\s*)+/i,'').trim().toLowerCase();
        if(rule.property==='mailwomanGroupSize')return email.size<25600?0:email.size<102400?1:email.size<1048576?2:email.size<10485760?3:4;
        if(rule.property==='subject')return email.subject;
        if(rule.property==='size')return email.size;
        if(rule.property==='importance')return ({high:2,normal:1,low:0} as Record<string,number>)[email.mailwomanPriority]??1;
        return rule.property==='sentAt'?email.sentAt:email.receivedAt;
      };
      const av=value(a),bv=value(b),comp=typeof av==='number'&&typeof bv==='number'?av-bv:String(av).localeCompare(String(bv));if(comp)return rule.isAscending===false?-comp:comp;
    }return a.id.localeCompare(b.id);});
    const position=Number(queryArgs?.['position']??0),limit=Number(queryArgs?.['limit']??20),slice=corpus.slice(position,position+limit);
    if(queryArgs&&limit>0)queries.push(queryArgs);
    for(const [name,args]of result.methodResponses){
      if(name==='Mailbox/get'){
        args['mailwomanInitialSync']='ready';
        const boxes=[['inbox','Inbox','inbox'],['sent','Sent','sent'],['archive','Archive','archive'],['trash','Trash','trash']].map(([id,name,role],index)=>({id,name,role,parentId:null,sortOrder:index,totalEmails:id==='inbox'?120:0,unreadEmails:0,mailwomanSynced:true}));args['list']=[...boxes,...Array.from({length:40},(_,index)=>({id:`organization-folder-${index}`,name:`Project ${String(index).padStart(2,'0')}`,parentId:null,role:null,sortOrder:100+index,totalEmails:0,unreadEmails:0,mailwomanSynced:true}))];
      }
      if(name==='Email/query'){args['ids']=slice.map(email=>email.id);args['position']=position;args['total']=corpus.length;}
      if(name==='Email/get'){const get=body.methodCalls.find(([method])=>method==='Email/get')?.[1];const ids=get?.['ids'];args['list']=Array.isArray(ids)?corpus.filter(email=>ids.includes(email.id)):slice;args['notFound']=[];}
    }
    if (route.request().failure()) return;
    await route.fulfill({status:200,json:result});
  });
  await login(page);const list=page.locator('section.list:visible');await expect(list.locator('.list__row').first()).toBeVisible();return{list,writes,moves,queries};
}
async function orderMenu(page:Page){await page.getByTestId('list-order-menu').click();const menu=page.getByRole('menu',{name:'Order by',exact:true});await expect(menu).toBeVisible();return menu;}
async function group(page:Page,label:string){const menu=await orderMenu(page);await menu.getByRole('menuitem',{name:'Group by',exact:true}).click();const groups=page.getByRole('menu',{name:'Group by',exact:true});await groups.getByRole('menuitemradio',{name:label,exact:true}).click();}
test('all twelve sort options have icons and the seven grouping options are checked and persisted',async({page})=>{
  const state=await fixture(page),menu=await orderMenu(page);
  const labels=['Newest first','Oldest first','Sender','Sender, Z to A','Subject','Subject, Z to A','Sent, newest first','Sent, oldest first','Importance, high first','Importance, low first','Size, largest first','Size, smallest first'];
  for(const label of labels){const item=menu.getByRole('menuitemradio',{name:label,exact:true});await expect(item).toBeVisible();await expect(item.locator('svg[data-icon]').first()).toBeVisible();}
  await expect(menu.getByRole('menuitemradio')).toHaveCount(12);
  await menu.getByRole('menuitemradio',{name:'Size, largest first',exact:true}).click();await expect.poll(()=>state.writes.at(-1)).toEqual({key:'mail.messageListOrder',value:'size'});
  await group(page,'From');await expect.poll(()=>state.writes.at(-1)).toEqual({key:'mail.messageListGroup',value:'sender'});
  const again=await orderMenu(page);await again.getByRole('menuitem',{name:'Group by',exact:true}).click();const groups=page.getByRole('menu',{name:'Group by',exact:true});await expect(groups.getByRole('menuitemradio')).toHaveCount(7);await expect(groups.getByRole('menuitemradio',{name:'From',exact:true})).toHaveAttribute('aria-checked','true');
  await page.keyboard.press('Escape');await page.keyboard.press('Escape');await page.reload();await expect(page.getByTestId('list-order-menu')).toBeVisible();await group(page,'No grouping');
});
test('sender and importance headers remain separate from rows through tall viewport virtualization',async({page},info)=>{
  const state=await fixture(page);await group(page,'From');
  const headers=state.list.locator('li[role="presentation"]');await expect(headers.first()).toContainText('Sender 00');
  const headBox=await headers.first().boundingBox(),firstRowBox=await state.list.locator('.list__slot').first().boundingBox();expect(headBox).not.toBeNull();expect(firstRowBox).not.toBeNull();expect(firstRowBox!.y).toBeGreaterThanOrEqual(headBox!.y+headBox!.height-1);
  const before=await state.list.locator('.list__row').count();await page.setViewportSize({width:1280,height:2200});
  await expect.poll(()=>state.list.locator('.list__row').count()).toBeGreaterThan(before);
  await expect.poll(()=>state.queries.length).toBeGreaterThan(1);
  await expect(state.list).toHaveAttribute('aria-busy','false');
  const positions=await state.list.locator('.list__slot').evaluateAll(nodes=>nodes.map(node=>{const rect=node.getBoundingClientRect();return{top:rect.top,bottom:rect.bottom};}).sort((a,b)=>a.top-b.top));for(let i=1;i<positions.length;i++)expect(positions[i]!.top).toBeGreaterThanOrEqual(positions[i-1]!.bottom-1);
  await group(page,'Importance');await expect(headers.first()).toContainText('High importance');
  await expect.poll(()=>state.queries.at(-1)?.['sort']).toEqual([{property:'importance',isAscending:false},{property:'receivedAt',isAscending:false}]);
  await expect(state.list.locator('.list__row').first()).toContainText('Organization message 117');
  await state.list.locator('.list__scroll').evaluate(element=>{element.scrollTop=2800;});await expect(headers.filter({hasText:'Normal importance'})).toBeInViewport();
  await page.screenshot({path:info.outputPath('grouped-list-tall.png')});
});
test('Move to filters folders while typing, Escape returns to the parent menu, and a picked folder uses the intended target',async({page})=>{
  const state=await fixture(page);await state.list.locator('.list__row').first().click({button:'right',position:{x:10,y:10}});
  const move=page.getByRole('menuitem',{name:'Move to…',exact:true});await move.click();const folders=page.getByRole('menu',{name:'Move to…',exact:true}),search=folders.getByRole('searchbox',{name:'Search folders'});
  await expect(search).toBeFocused();await search.fill('Project 25');await expect(folders.getByRole('menuitem')).toHaveCount(1);await expect(folders.getByRole('menuitem',{name:'Project 25',exact:true})).toBeVisible();
  await search.press('Escape');await expect(folders).toHaveCount(0);await expect(move).toBeFocused();expect(state.moves).toEqual([]);
  await move.click();const reopened=page.getByRole('menu',{name:'Move to…',exact:true});await reopened.getByRole('searchbox',{name:'Search folders'}).fill('Project 25');await reopened.getByRole('menuitem',{name:'Project 25',exact:true}).click();
  await expect.poll(()=>state.moves.length).toBe(1);expect(JSON.stringify(state.moves[0])).toContain('organization-folder-25');await expect(reopened).toHaveCount(0);
});
