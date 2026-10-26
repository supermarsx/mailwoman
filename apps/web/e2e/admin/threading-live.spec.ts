import {test,expect} from '../csrf.ts';
import {signInManually} from '../helpers.ts';
import {sendRawSmtp} from '../crypto-helpers.ts';

test('references create a real expandable conversation that survives reload @phone',async({page},info)=>{
 const stamp=Date.now();const subject=`Thread acceptance ${stamp}`;const first=`thread-root-${stamp}@example.org`;
 for(const [index,title] of [subject,`Re: ${subject}`].entries())await sendRawSmtp({from:`sender${index}@example.org`,message:`From: sender${index}@example.org\r\nTo: testuser@example.org\r\nDate: ${new Date(stamp+index*1000).toUTCString()}\r\nSubject: ${title}\r\nMessage-ID: <${index===0?first:`thread-reply-${stamp}@example.org`}>\r\n${index?`In-Reply-To: <${first}>\r\nReferences: <${first}>\r\n`:''}Content-Type: text/plain\r\n\r\nConversation member ${index}.\r\n`});
 await signInManually(page,{username:'testuser',password:'testpass'});
 const head=page.getByTestId('thread-head').filter({hasText:subject});await expect(head).toHaveCount(1);await expect(head.getByLabel('2 messages',{exact:true})).toBeVisible();
 const toggle=head.locator('..').getByTestId('thread-toggle');await expect(toggle).toHaveAttribute('aria-expanded','false');await toggle.click();await expect(toggle).toHaveAttribute('aria-expanded','true');await expect(page.locator('.list__row').filter({hasText:subject})).toHaveCount(3);
 await page.screenshot({path:info.outputPath('thread-expanded.png'),fullPage:true});await toggle.click();await expect(page.locator('.list__row').filter({hasText:subject})).toHaveCount(1);
 await page.reload();await expect(head).toHaveCount(1);await expect(head.getByLabel('2 messages',{exact:true})).toBeVisible();expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1&&document.documentElement.scrollHeight<=innerHeight+1)).toBe(true);
});
