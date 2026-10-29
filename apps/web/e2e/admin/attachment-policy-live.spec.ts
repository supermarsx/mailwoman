import {test,expect} from '../csrf.ts';
import { ADMIN } from './admin-path.ts';
import {signInManually,messageRow,sidebarInbox,drawerButton} from '../helpers.ts';
import {sendRawSmtp} from '../crypto-helpers.ts';

// The spec owns its fixture: it stores a block rule for .exe and a warning rule for
// .txt, delivers a message carrying one of each, and restores both settings after.
const rules={'attachments.blockedExtensions':['exe'],'attachments.warnExtensions':['txt']} as const;

test('stored attachment policy blocks renamed downloads and requires warning acknowledgement @phone',async({page,playwright,baseURL},info)=>{
  const admin=await playwright.request.newContext({...baseURL?{baseURL}:{}});
  const saved:Array<{key:string;body:{globalValue:unknown;globalLocked:boolean}}>=[];
  try{
    await page.waitForTimeout(3100);await admin.get(ADMIN);
    expect((await admin.post(`${ADMIN}/login`,{data:{username:'root',password:'hunter2'}})).status()).toBe(200);
    for(const [key,value] of Object.entries(rules)){
      const current=await admin.get(`${ADMIN}/settings/${key}`);expect(current.status()).toBe(200);
      saved.push({key,body:await current.json()});
      const put=await admin.put(`${ADMIN}/settings/${key}`,{data:{globalValue:value},headers:{'if-match':current.headers()['etag']!}});
      expect(put.ok(),await put.text()).toBe(true);
    }
    const subject=`Attachment policy ${info.project.name} ${Date.now()}`;const boundary=`policy-${Date.now()}`;
    const part=(name:string,type:string,body:string)=>`--${boundary}\r\nContent-Type: ${type}; name="${name}"\r\nContent-Disposition: attachment; filename="${name}"\r\nContent-Transfer-Encoding: base64\r\n\r\n${Buffer.from(body).toString('base64')}\r\n`;
    await sendRawSmtp({from:'sender@example.test',message:`From: sender@example.test\r\nTo: testuser@example.org\r\nDate: ${new Date().toUTCString()}\r\nMessage-ID: <${boundary}@example.test>\r\nSubject: ${subject}\r\nMIME-Version: 1.0\r\nContent-Type: multipart/mixed; boundary="${boundary}"\r\n\r\n--${boundary}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nAttachment policy fixture.\r\n`+
      part('policy-danger.exe','application/octet-stream','MZ not really a program')+part('policy-safe.txt','text/plain','plain text attachment')+`--${boundary}--\r\n`});
    await page.goto('/');await signInManually(page,{username:'testuser',password:'testpass'});
    await expect(page.getByRole('toolbar',{name:'Workspace layout'})).toBeVisible({timeout:60_000});
    await expect(async()=>{if(page.viewportSize()!.width<761)await drawerButton(page).click();await sidebarInbox(page).click();await expect(messageRow(page,subject)).toBeVisible({timeout:3000});}).toPass({timeout:45_000});
    await messageRow(page,subject).click();
    await page.getByRole('option',{name:/policy-danger\.exe/}).click();
    await expect(page.getByRole('alert').filter({hasText:'administrator has blocked'})).toBeVisible();
    await page.screenshot({path:info.outputPath('blocked-attachment.png'),fullPage:true});
    // Return to the original reader tab, keeping the blocked attachment tab open.
    await page.getByRole('tab',{name:subject}).click();
    await page.getByRole('option',{name:/policy-safe\.txt/}).click();
    await expect(page.getByRole('button',{name:'I understand — continue'})).toBeVisible();
    await page.getByRole('button',{name:'I understand — continue'}).click();
    await expect(page.getByRole('link',{name:'Download file'})).toBeVisible();
    await page.screenshot({path:info.outputPath('acknowledged-attachment.png'),fullPage:true});
    expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
    await page.goto(ADMIN);await page.getByLabel('Admin username').fill('root');await page.getByLabel('Password',{exact:true}).fill('hunter2');
    await page.waitForTimeout(3100);
    const login=page.waitForResponse(response=>new URL(response.url()).pathname===`${ADMIN}/login`&&response.request().method()==='POST');
    await page.getByRole('button',{name:'Sign in',exact:true}).click();expect((await login).status()).toBe(200);
    await page.goto(`${ADMIN}#/attachments`);
    await expect(page.getByRole('heading',{name:'Delivery and preview rules'})).toBeVisible();
    await expect(page.getByRole('heading',{name:'Blocked filename extensions'})).toBeVisible();
    await page.screenshot({path:info.outputPath('admin-attachment-policy.png'),fullPage:true});
    expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  }finally{
    for(const {key,body} of saved){
      const current=await admin.get(`${ADMIN}/settings/${key}`);
      const put=await admin.put(`${ADMIN}/settings/${key}`,{data:{globalValue:body.globalValue??null,locked:body.globalLocked},headers:{'if-match':current.headers()['etag']!}});
      expect(put.ok(),await put.text()).toBe(true);
    }
    await admin.dispose();
  }
});
