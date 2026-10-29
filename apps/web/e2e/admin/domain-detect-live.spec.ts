import {test,expect} from '../csrf.ts';
import { ADMIN } from './admin-path.ts';
import {adminLogin} from './admin-login.ts';

/**
 * The guided domain wizard's "Discover configuration" runs server-side discovery
 * (POST /admin/domains/{name}/detect) with no route mock. The admin-detect egress
 * class refuses private and loopback targets and no DNS override exists, so this
 * spec discovers a real public domain: gmail.com publishes RFC 6186 SRV records
 * (_imaps._tcp -> imap.gmail.com:993, _submission._tcp -> smtp.gmail.com:587) and
 * no /.well-known/jmap session, so the SRV rung answers. Needs outbound DNS and
 * HTTPS from the server; run it on the engine-mode admin stack.
 *
 * MW_E2E_DETECT_SHOT  optional extra path for the result screenshot
 */
test('guided domain discovery shows the server-detected endpoints @phone',async({page},info)=>{
 await adminLogin(page);
 const status=await (await page.request.get(`${ADMIN}/domains/status`)).json();
 expect(status.mode,'run this on the mail-engine admin stack').toBe('engine');
 await page.goto(`${ADMIN}#/domains/new/guided`);
 await page.getByLabel('Domain name',{exact:true}).fill('gmail.com');
 await page.getByRole('button',{name:'Next',exact:true}).click();
 await page.getByLabel('Connection',{exact:true}).selectOption('advanced');
 const detected=page.waitForResponse(r=>r.url().endsWith(`${ADMIN}/domains/gmail.com/detect`)&&r.request().method()==='POST',{timeout:45_000});
 await page.getByRole('button',{name:'Discover configuration',exact:true}).click();
 const response=await detected;
 expect(response.status(),await response.text()).toBe(200);
 const body=await response.json();
 expect(body).toMatchObject({name:'gmail.com',source:'srv',saved:false,upstream:{
  imap:{host:'imap.gmail.com',port:993,tls:'implicit'},smtp:{host:'smtp.gmail.com',port:587,tls:'starttls'}}});
 expect(body.upstream.pop3).toBeUndefined();
 await expect(page.getByText(/^Discovered source: \u2068?srv\u2069?\./u)).toBeVisible();
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
 await page.screenshot({path:info.outputPath('domain-detect-result.png'),fullPage:true});
 const extra=process.env['MW_E2E_DETECT_SHOT'];
 if(extra)await page.screenshot({path:extra.replace('{project}',info.project.name),fullPage:true});
 await page.getByRole('button',{name:'Review discovered endpoints',exact:true}).click();
 const drafts=page.getByRole('table',{name:'Backend setup drafts'});
 await expect(drafts).toContainText('IMAP');
 await expect(drafts).toContainText('SMTP');
 await expect(drafts).not.toContainText('POP3');
 // The review step lists each adopted backend's host and port.
 await page.getByRole('button',{name:'Next',exact:true}).click();
 await expect(page.getByText(/imap\.gmail\.com:993/u)).toBeVisible();
 await expect(page.getByText(/smtp\.gmail\.com:587/u)).toBeVisible();
 await page.screenshot({path:info.outputPath('domain-detect-review.png'),fullPage:true});
 // Discovery proposes; it never saves.
 expect((await page.request.get(`${ADMIN}/domains/gmail.com`)).status()).toBe(404);
});
