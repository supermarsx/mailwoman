// t68: against a live server built from the tree.
//  (a) In a detached message window, Flag, Read, zoom and Show images act in that
//      window: it stays open, the action takes effect, and with remote content
//      allowed the body still renders through the remote-frame bridge (which now
//      takes a body only from its parent or the parent's opener).
//  (b) At 1440 and 390px the remote-images bar's tooltips open below the bar, so
//      they never cover the reader action row; More takes the first click.
// Screenshots go to MW_T68_SHOTS when set (else the test output dir).
import {test,expect} from '../csrf.ts';
import type {Locator,Page,TestInfo} from '@playwright/test';
import net from 'node:net';
import path from 'node:path';

const smtpPort=Number(process.env['MW_E2E_SMTP_PORT']??42225);
const run=Date.now().toString(36);
const subject=`Detached actions ${run}`;
const html='<html><body><p>A message with a remote image.</p><p><img alt="remote banner" width="320" height="80" src="http://203.0.113.10/t68-banner.png"></p><p>Text after the image.</p></body></html>';
function deliver():Promise<void>{
 return new Promise((resolve,reject)=>{
  const socket=net.connect(smtpPort,'127.0.0.1');
  const data=`From: Detached Fixture <detached@example.net>\r\nTo: testuser@example.org\r\nSubject: ${subject}\r\nDate: ${new Date().toUTCString().replace('GMT','+0000')}\r\nMessage-ID: <t68-${run}@example.net>\r\nMIME-Version: 1.0\r\nContent-Type: text/html; charset=utf-8\r\n\r\n${html}\r\n.\r\n`;
  const steps=['EHLO t68\r\n','MAIL FROM:<detached@example.net>\r\n','RCPT TO:<testuser@example.org>\r\n','DATA\r\n',data,'QUIT\r\n'];
  let i=0,buffer='';
  socket.on('data',chunk=>{buffer+=chunk.toString();if(!buffer.endsWith('\r\n'))return;const last=buffer.trim().split('\r\n').at(-1)!;if(/^\d{3}-/.test(last))return;buffer='';if(/^[45]/.test(last)){socket.destroy();reject(new Error(last));return;}if(i<steps.length)socket.write(steps[i++]!);});
  socket.on('close',()=>resolve());socket.on('error',reject);
 });
}
test.describe.configure({timeout:300000});
let seeded=false;
test.beforeEach(async()=>{if(!seeded){await deliver();seeded=true;await new Promise(resolve=>setTimeout(resolve,3100));}});

const shot=(info:TestInfo,name:string)=>{const dir=process.env['MW_T68_SHOTS'];return dir?path.join(dir,`${info.project.name}-${name}.png`):info.outputPath(`${name}.png`);};
const reader=(page:Page)=>page.locator('section.reader').filter({visible:true}).first();
async function signIn(page:Page):Promise<void>{
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(page.locator('.list__row').first()).toBeVisible({timeout:60000});
}
async function open(page:Page):Promise<Locator>{
 const row=page.locator('.list__row').filter({hasText:subject}).first();
 await expect(row).toBeVisible({timeout:60000});await row.click();
 await expect(page.locator('.reader__subject').filter({visible:true}).first()).toHaveText(subject);
 return row;
}
/** The document the body frame shows: its own srcdoc, or the bridge's nested frame's. */
async function bodySource(scope:Page):Promise<string>{
 const outer=reader(scope).locator('iframe.reader__frame').first();
 const direct=await outer.getAttribute('srcdoc');if(direct)return direct;
 try{const bridge=await (await outer.elementHandle())?.contentFrame();return bridge?await bridge.evaluate(()=>document.querySelector('iframe')?.getAttribute('srcdoc')??''):'';}catch{return '';}
}
/** Whether a real pointer press at the centre of `target` lands on it. */
async function hitsItself(target:Locator):Promise<boolean>{
 return target.evaluate(element=>{const box=element.getBoundingClientRect();const hit=element.ownerDocument.elementFromPoint(box.left+box.width/2,box.top+box.height/2);return hit!==null&&(hit===element||element.contains(hit));});
}

test('reader actions act inside a detached message window and keep it open',async({page},info)=>{
 test.skip(info.project.name==='phone','a detached window is a desktop feature');
 await page.setViewportSize({width:1440,height:900});
 await signIn(page);
 const row=await open(page);
 await row.dblclick();
 const tab=page.getByRole('tab',{name:subject,exact:true});await expect(tab).toBeVisible();
 const pending=page.waitForEvent('popup');
 await tab.press('Shift+F10');await page.getByRole('menuitem',{name:`Open ${subject} in a new window`,exact:true}).click();
 const popup=await pending;
 const view=reader(popup);
 await expect(popup.locator('.reader__subject').filter({visible:true}).first()).toHaveText(subject,{timeout:60000});
 let closed=false;popup.on('close',()=>{closed=true;});

 const flag=view.getByTestId('reader-flag');
 const before=await flag.getAttribute('aria-pressed');
 await flag.click();
 await expect(flag).toHaveAttribute('aria-pressed',before==='true'?'false':'true');
 const read=view.getByTestId('reader-toggle-read');
 const readLabel=await read.getAttribute('aria-label');
 await read.click();
 await expect(read).not.toHaveAttribute('aria-label',readLabel!);
 await read.click();
 await expect(read).toHaveAttribute('aria-label',readLabel!);
 const level=view.getByTestId('reader-zoom-level');
 const start=(await level.textContent())?.trim();
 await view.getByTestId('reader-zoom-in').click();
 await expect(level).not.toHaveText(start!);
 await level.click();
 await expect(level).toHaveText('100%');
 expect(closed,'the detached window stays open').toBe(false);
 await expect(page.getByText('Returned to the main window to complete this action.')).toHaveCount(0);

 // Show images: the body is re-sent through the bridge in the popup.
 const bar=view.getByTestId('remote-content-bar');
 await bar.getByRole('button',{name:'Show images'}).click();
 await expect(bar).toHaveAttribute('data-state','shown');
 await expect.poll(()=>bodySource(popup),{timeout:30000}).toContain('Text after the image.');
 expect(closed).toBe(false);
 await popup.screenshot({path:shot(info,'detached-after-actions')});
 // Restore flag and image state so a re-run starts from the same place.
 await bar.getByRole('button',{name:/^Undo/}).click();
 await expect(bar).toHaveAttribute('data-state','blocked');
 await flag.click();await expect(flag).toHaveAttribute('aria-pressed',before==='true'?'true':'false');
 expect(closed).toBe(false);
 await popup.close();
});

for(const width of [1440,390]){
 test(`the remote-images tooltip leaves the reader actions clickable at ${width}px`,async({page},info)=>{
  test.skip(info.project.name==='phone'&&width===1440,'phone runs the 390px leg');
  await page.setViewportSize({width,height:info.project.name==='phone'?844:900});
  await signIn(page);
  await open(page);
  const view=reader(page);
  const bar=view.getByTestId('remote-content-bar');await expect(bar).toBeVisible();
  const undo=bar.getByRole('button',{name:/^Undo/});if(await undo.isVisible())await undo.click();
  const summary=bar.locator('[role="status"]');
  // The pointer rests on the summary (what t59 saw after a More menu closed over it).
  await summary.hover();
  const tooltip=page.getByRole('tooltip');
  await expect(tooltip).toBeVisible();
  const [tip,anchor,toolbar]=await Promise.all([tooltip.boundingBox(),summary.boundingBox(),view.locator('.reader__toolbar').boundingBox()]);
  expect(tip!.y,'tooltip opens below its anchor').toBeGreaterThanOrEqual(Math.floor(anchor!.y+anchor!.height));
  expect(tip!.y,'tooltip does not reach the action row').toBeGreaterThan(toolbar!.y+toolbar!.height);
  await page.screenshot({path:shot(info,`banner-tooltip-${width}`)});
  const target=view.getByTestId(width<=390?'reader-more':'reader-flag');
  expect(await hitsItself(target),'the action row is not covered').toBe(true);
  if(width<=390){
   // No pointer move away and no Escape first: the first click opens More.
   await target.click({timeout:5000});
   await expect(page.getByRole('menu',{name:'More actions'})).toBeVisible();
   await page.screenshot({path:shot(info,`more-open-${width}`)});
   await page.keyboard.press('Escape');
  }
 });
}
