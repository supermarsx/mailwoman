import {test,expect} from '@playwright/test';
import {login} from './helpers.ts';

test.afterEach(async({page})=>page.unrouteAll({behavior:'wait'}));

test('measures a tall list and fills its resized viewport with bounded page requests',async({page},info)=>{
  await page.setViewportSize({width:1500,height:1600});
  await page.route(/\/api\/settings(?:\?.*)?$/,async route=>{
    const response=await route.fetch(),body=await response.json();
    for(const [key,value] of Object.entries({'notifications.askPermission':false,'mail.list.pageSize':20,'mail.list.conversations':false}))body.settings[key]={value,source:'user',locked:false};
    await route.fulfill({response,json:body});
  });
  await page.route(/\/api\/account\/preferences(?:\?.*)?$/,async route=>{
    expect(route.request().method()).toBe('GET');const response=await route.fetch(),body=await response.json();
    body.preferences={...body.preferences,readingPane:'right',readerCollapsed:false,messageListCollapsed:false,workspaceLayout:'tabs',markRead:'never'};
    await route.fulfill({response,json:body});
  });
  const calls:{position:number;limit:number}[]=[];
  let inbox='';
  const email=(id:string)=>({id,threadId:id,mailboxIds:{[inbox]:true},from:[{name:'Viewport sender',email:'viewport@example.org'}],to:[{email:'testuser@example.org'}],subject:`Viewport message ${Number(id.slice(9))+1}`,receivedAt:'2026-11-09T09:00:00Z',preview:'Synthetic local browser fixture',keywords:{$seen:true},hasAttachment:false});
  await page.route(/\/jmap(?:\/api)?$/,async route=>{
    if(route.request().method()!=='POST')return route.continue();
    const request=route.request().postDataJSON() as {methodCalls:[string,Record<string,unknown>,string][]};
    const response=await route.fetch(),body=await response.json() as {methodResponses:[string,Record<string,unknown>,string][]};
    for(const [method,args] of body.methodResponses)if(method==='Mailbox/get')for(const box of args['list'] as Record<string,unknown>[]??[])if(box['role']==='inbox'){inbox=String(box['id']);box['totalEmails']=1000;box['unreadEmails']=0;}
    // Unread/count probes use limit:0; they are not list-page requests.
    for(const [method,args,tag] of body.methodResponses)if(method==='Email/query'&&request.methodCalls.some(call=>call[2]===tag&&call[1]['limit']===0))Object.assign(args,{ids:[],total:0});
    const query=request.methodCalls.find(call=>call[0]==='Email/query'&&Number(call[1]['limit']??20)>0);
    let ids:string[]|undefined;
    if(query){
      const position=Number(query[1]['position']??0),limit=Number(query[1]['limit']??20);
      calls.push({position,limit});ids=Array.from({length:Math.min(limit,1000-position)},(_,index)=>`viewport-${position+index}`);
      for(const [method,args,tag] of body.methodResponses)if(method==='Email/query'&&tag===query[2])Object.assign(args,{ids,position,total:1000,queryState:'viewport-fixture'});
    }
    for(const [method,args,tag] of body.methodResponses)if(method==='Email/get'){
      const explicit=request.methodCalls.find(call=>call[2]===tag)?.[1]['ids'];
      const requested=Array.isArray(explicit)?explicit:ids;
      if(requested?.every(id=>typeof id==='string'&&id.startsWith('viewport-')))Object.assign(args,{list:requested.map(id=>email(String(id))),notFound:[]});
    }
    await route.fulfill({response,json:body});
  });
  await login(page);
  const scroller=page.locator('.list__scroll:visible');await expect(scroller).toBeVisible();
  const geometry=()=>scroller.evaluate(node=>{
    const view=node.getBoundingClientRect(),rows=[...node.querySelectorAll('.list__row')].map(row=>row.getBoundingClientRect());
    return{height:node.clientHeight,scrollHeight:node.scrollHeight,top:node.scrollTop,rendered:rows.length,lastBottom:Math.max(...rows.map(row=>row.bottom)),bottom:view.bottom,rowHeight:rows[0]?.height??0};
  });
  await expect.poll(async()=>{const g=await geometry();return g.lastBottom>=g.bottom-g.rowHeight;}).toBe(true);
  const before=await geometry();expect(before.rendered).toBeGreaterThan(20);
  const initialCalls=calls.length;expect(initialCalls).toBeGreaterThanOrEqual(2);
  await page.setViewportSize({width:1500,height:2600});
  await expect.poll(async()=>{const g=await geometry();return g.height>before.height&&g.lastBottom>=g.bottom-g.rowHeight&&g.rendered>before.rendered;}).toBe(true);
  await expect.poll(()=>calls.length).toBeGreaterThan(initialCalls);
  const grown=await geometry();expect(grown.top).toBe(0);expect(grown.scrollHeight).toBeGreaterThan(grown.height);
  const settled=calls.length;await page.waitForTimeout(1200);expect(calls).toHaveLength(settled);
  expect(calls.every(call=>call.limit===20)).toBe(true);expect(calls.length).toBeLessThanOrEqual(8);
  expect(calls.map(call=>call.position)).toEqual(calls.map((_,index)=>index*20));
  console.log(info.project.name,JSON.stringify({before,grown,calls}));await page.screenshot({path:info.outputPath('tall-list-filled.png')});
});
