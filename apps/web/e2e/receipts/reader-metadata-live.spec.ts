import {test,expect} from '../csrf.ts';
import {sendRawSmtp} from '../crypto-helpers.ts';

test('reader metadata stays inline and opens accessible details @phone',async({page},info)=>{
 await page.route('**/*',async route=>{
  const url=new URL(route.request().url());
  // Deterministic header presentation only; mailbox/authentication remain real.
  if(route.request().headers()['range']?.startsWith('bytes=0-')&&url.pathname.includes('download'))return route.fulfill({status:200,contentType:'message/rfc822',body:'X-Spam-Status: Yes, score=6.2 required=5.0 tests=TEST_RULE\r\n\r\n'});
  return route.continue();
 });
 // View conversation is offered only for a thread of two or more messages
 // (Conversation.tsx), so the spec opens a reply in a thread of its own.
 const token=`Reader metadata ${info.project.name} ${Date.now()}`,root=`<${token.replaceAll(' ','-')}-root@example.org>`;
 await sendRawSmtp({from:'metadata@example.org',to:'testuser@example.org',message:`From: metadata@example.org\r\nTo: testuser@example.org\r\nSubject: ${token}\r\nMessage-ID: ${root}\r\n\r\nThread start.\r\n`});
 await sendRawSmtp({from:'metadata@example.org',to:'testuser@example.org',message:`From: metadata@example.org\r\nTo: testuser@example.org\r\nSubject: Re: ${token}\r\nMessage-ID: <${token.replaceAll(' ','-')}-reply@example.org>\r\nIn-Reply-To: ${root}\r\nReferences: ${root}\r\n\r\nThread reply.\r\n`});
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 const opened=page.locator('.list__row').filter({hasText:`Re: ${token}`}).first();await expect(opened).toBeVisible({timeout:30000});await opened.click();
 const row=page.locator('.reader__metadata-line').filter({visible:true}).first();await expect(row).toBeVisible();await expect(row.locator('time')).toBeVisible();
 const auth=row.getByRole('button').filter({hasText:'DMARC'});await expect(auth).toContainText('DMARC');await expect(row.locator('.spam-score')).toBeVisible();await expect(row.locator('.spam-score__chip strong')).toHaveText('6.2 / 5');
 const geometry=await row.evaluate(node=>{const parts=[node.querySelector('time')!,node.querySelector('button')!,node.querySelector('.spam-score')!].map(item=>item.getBoundingClientRect());return {tops:parts.map(p=>p.top),overflow:document.documentElement.scrollWidth-innerWidth,rowOverflow:node.scrollWidth-node.clientWidth};});
 expect(Math.max(...geometry.tops)-Math.min(...geometry.tops)).toBeLessThan(24);expect(geometry.overflow).toBeLessThanOrEqual(1);expect(geometry.rowOverflow).toBeLessThanOrEqual(1);
 await auth.click();await expect(page.getByRole('dialog',{name:'Message security details'})).toBeVisible();await page.keyboard.press('Escape');await expect(page.getByRole('dialog',{name:'Message security details'})).toHaveCount(0);
 const conversation=page.getByRole('button',{name:'View conversation',exact:true});await expect(conversation).toBeVisible();// ReaderAction (23d6db6d/ca8cb1cf): icon only, its label screen-reader only, its help a title that TooltipLayer moves to data-tooltip-title.
 await expect(conversation.locator('svg')).toHaveCount(1);expect(await conversation.evaluate(node=>[...node.querySelectorAll('span')].every(span=>{const r=span.getBoundingClientRect();return r.width<=1&&r.height<=1;}))).toBe(true);await expect(conversation).toHaveAttribute('data-tooltip-title','View conversation');
 await row.locator('.time-helper summary').click();await expect(row.locator('.time-helper')).toHaveAttribute('open','');await row.locator('.time-helper summary').click();
 await page.evaluate(()=>document.documentElement.setAttribute('data-theme','light'));await page.screenshot({path:info.outputPath('reader-metadata-light.png'),fullPage:true});
 await page.evaluate(()=>document.documentElement.setAttribute('data-theme','dark'));await page.screenshot({path:info.outputPath('reader-metadata-dark.png'),fullPage:true});
});
