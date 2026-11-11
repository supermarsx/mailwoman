import {test,expect} from '../csrf.ts';
import {login,messageRow} from '../helpers.ts';
import {deflateSync} from 'node:zlib';
const crc=(b:Buffer)=>{let c=0xffffffff;for(const v of b){c^=v;for(let i=0;i<8;i++)c=c&1?(c>>>1)^0xedb88320:c>>>1;}return(c^0xffffffff)>>>0;};
const chunk=(name:string,data:Buffer)=>{const bytes=Buffer.concat([Buffer.from(name),data]),len=Buffer.alloc(4),sum=Buffer.alloc(4);len.writeUInt32BE(data.length);sum.writeUInt32BE(crc(bytes));return Buffer.concat([len,bytes,sum]);};
const head=Buffer.alloc(13);head.writeUInt32BE(40);head.writeUInt32BE(30,4);head[8]=8;head[9]=6;
const pixels=Buffer.alloc(30*(1+40*4),100);for(let row=0;row<30;row++)pixels[row*(1+40*4)]=0;
const png=Buffer.concat([Buffer.from([137,80,78,71,13,10,26,10]),chunk('IHDR',head),chunk('IDAT',deflateSync(pixels)),chunk('IEND',Buffer.alloc(0))]);
for(const missingUuid of [false,true])test(`copies 23 visible message images into a reply draft (randomUUID unavailable: ${missingUuid})`,async({page})=>{
 if(missingUuid)await page.addInitScript(()=>Object.defineProperty(crypto,'randomUUID',{value:undefined,configurable:true}));
 const ids=new Set<string>();let uploads=0;let signed=0;
 const html='<p>Newsletter content</p>'+Array.from({length:23},(_,i)=>`<img alt="image-${i}" src="https://fixture.example/image-${i}.png" width="40" height="30">`).join('');
 await page.route('**/api/settings',async route=>{
  const response=await route.fetch();const body=await response.json();
  if(body.settings)body.settings['mail.replyImages']={value:'allow',source:'user',locked:false};
  await route.fulfill({response,json:body});
 });
 await page.route('**/api/rc/policy',async route=>{
  const response=await route.fetch();const body=await response.json();
  await route.fulfill({response,json:{...body,mode:'proxy',inlineParts:true}});
 });
 await page.route(/\/api\/rc\/sign$/,async route=>{
  const body=route.request().postDataJSON();signed+=body.urls?.length??0;
  await route.fulfill({json:{images:(body.urls??[]).map((url:string,i:number)=>({url,src:`/api/rc/i/fixture-${i}.token`})),parts:[],files:[],refused:[]}});
 });
 await page.route(/\/api\/rc\/i\//,route=>route.fulfill({contentType:'image/png',body:png}));
 await page.route(/\/jmap\/download\//,route=>route.fulfill({contentType:'image/png',body:png}));
 await page.route(/\/jmap\/upload\//,async route=>{
  uploads++;const accountId=new URL(route.request().url()).pathname.split('/').at(-1);
  await route.fulfill({status:201,json:{accountId,blobId:`fixture-upload-${uploads}`,type:'image/png',size:png.length}});
 });
 await page.route(/\/jmap(?:\/api)?$/,async route=>{
  const response=await route.fetch();const body=await response.json();
  for(const [name,args] of body.methodResponses??[])if(name==='Email/get')for(const row of args.list??[]){
   if(row.subject==='Lunch tomorrow?')ids.add(row.id);
   if(!ids.has(row.id))continue;
   row.htmlBody=[{partId:'1',type:'text/html'}];row.bodyValues={'1':{value:html,isTruncated:false}};row.textBody=[];
  }
  await route.fulfill({response,json:body});
 });
 await login(page);await messageRow(page,'Lunch tomorrow?').click();
 await expect(page.getByText('23/23 images shown',{exact:true})).toBeVisible();
 await page.getByRole('button',{name:'Reply',exact:true}).first().click();
 const composer=page.locator('form.compose').last();await expect(composer).toBeVisible();
 await expect.poll(()=>uploads).toBe(23);
 expect(signed).toBeGreaterThanOrEqual(23);
 const editor=composer.getByRole('tabpanel',{name:'Visual editor'}).frameLocator('iframe');
 await expect(editor.locator('img')).toHaveCount(23);
 await expect.poll(()=>editor.locator('img').evaluateAll(images=>images.filter(image=>(image as HTMLImageElement).naturalWidth>0).length)).toBe(23);
});
