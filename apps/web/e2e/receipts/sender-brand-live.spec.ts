import {test,expect} from '../csrf.ts';
import net from 'node:net';

const fixtures=[
 {domain:'olx.pt',icon:'olx'},
 {domain:'trading212.com',icon:'trading212'},
 {domain:'interactivebrokers.com',icon:'interactivebrokers'},
 {domain:'openrouter.ai',icon:'openrouter'},
 {domain:'santander.pt',icon:'santanderpt'},
 {domain:'publico.pt',icon:'publico'},
 {domain:'portaldasfinancas.gov.pt',icon:'government'},
 {domain:'unrecognised.example',icon:null},
 {domain:'olx.pt.attacker.example',icon:null},
] as const;
function deliver(domain:string,subject:string,index:number):Promise<void>{
 return new Promise((resolve,reject)=>{
  const socket=net.connect(Number(process.env['MW_E2E_SMTP_PORT']??42225),process.env['MW_E2E_SMTP_HOST']??'127.0.0.1');
  const data=`From: Sender icon fixture <notice@${domain}>\r\nTo: testuser@example.org\r\nSubject: ${subject}\r\nDate: ${new Date(Date.now()+index*1000).toUTCString()}\r\nMessage-ID: <${subject.replace(/ /gu,'-')}@fixture.example>\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nLocal sender-icon fixture.\r\n.\r\n`;
  const steps=['EHLO sender-icon-fixture\r\n',`MAIL FROM:<notice@${domain}>\r\n`,'RCPT TO:<testuser@example.org>\r\n','DATA\r\n',data,'QUIT\r\n'];let indexStep=0,buffer='';
  socket.setTimeout(15000,()=>{socket.destroy();reject(new Error('SMTP fixture timed out'));});
  socket.on('data',chunk=>{buffer+=chunk.toString();if(!buffer.endsWith('\r\n'))return;const last=buffer.trim().split('\r\n').at(-1)!;if(/^\d{3}-/u.test(last))return;buffer='';if(/^[45]/u.test(last)){socket.destroy();reject(new Error(last));return;}if(indexStep<steps.length)socket.write(steps[indexStep++]!);});
  socket.on('close',()=>{if(indexStep===steps.length)resolve();else reject(new Error('SMTP fixture closed early'));});socket.on('error',reject);
 });
}
test('bundled sender marks render in real message rows under CSP without external image requests',async({page},info)=>{
 const run=`Sender icons ${info.project.name} ${Date.now().toString(36)}`;
 for(const [i,fixture] of fixtures.entries())await deliver(fixture.domain,`${run} ${i}`,i);
 const outside:string[]=[];const csp:string[]=[];
 const origin=new URL(String(info.project.use.baseURL)).origin;
 page.on('request',request=>{if(request.resourceType()==='image'&&/^https?:/u.test(request.url())&&new URL(request.url()).origin!==origin)outside.push(request.url());});
 page.on('console',message=>{if(/content security policy|refused to load the image/iu.test(message.text()))csp.push(message.text());});
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 for(const [i,fixture] of fixtures.entries()){
  const row=page.locator('.list__row').filter({hasText:`${run} ${i}`});await expect(row).toHaveCount(1);await row.scrollIntoViewIfNeeded();
  const avatar=row.locator('.person-avatar');await expect(avatar).toBeVisible();
  if(fixture.icon){await expect(avatar).toHaveAttribute('data-sender-icon',fixture.icon);const img=avatar.locator('img');if(await img.count())await expect.poll(()=>img.evaluate((image:HTMLImageElement)=>image.complete&&image.naturalWidth>0)).toBe(true);else await expect(avatar.locator('svg path').first()).toBeAttached();}
  else await expect(avatar).not.toHaveAttribute('data-sender-icon');
 }
 await page.locator('.list__row').filter({hasText:`${run} 0`}).scrollIntoViewIfNeeded();
 await page.screenshot({path:info.outputPath('sender-icons-real-message-list.png'),fullPage:true});
 expect(outside).toEqual([]);expect(csp).toEqual([]);
});
