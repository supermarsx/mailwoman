// t59: the reading-pane zoom against a live server built from the tree.
// Three fixtures are delivered over Greenmail SMTP first (plain text, a 900px
// wide HTML newsletter, a message with an inline and a remote image). The body
// is a sandboxed frame: its zoom is read from the frame document itself.
// Screenshots go to MW_T59_SHOTS when set (else the test output dir).
import {test,expect} from '../csrf.ts';
import type {Page,TestInfo} from '@playwright/test';
import net from 'node:net';
import path from 'node:path';

const smtpPort=Number(process.env['MW_E2E_SMTP_PORT']??42225);
const png='iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAIAAAAlC+aJAAAAeUlEQVR4nO3PQQkAMAzAwMqpfxWTNRF7HINABFzm7H7dcEEDWtCAFjSgBQ1oQQNa0IAWNKAFDWhBA1rQgBY0oAUNaEEDWtCAFjSgBQ1oQQNa0IAWNKAFDWhBA1rQgBY0oAUNaEEDWtCAFjSgBQ1oQQNa0IAWNKAFj13vDoEA84tnsQAAAABJRU5ErkJggg==';
const run=Date.now().toString(36);
const subjects={plain:`Zoom plain ${run}`,wide:`Zoom newsletter ${run}`,images:`Zoom images ${run}`};
const wideRows=Array.from({length:6},(_,i)=>`<tr><td width="300" style="padding:12px;background:#eef3ff">Column A row ${i+1}: weekly highlights.</td><td width="300" style="padding:12px">Column B row ${i+1}: newsletter copy in a fixed-width table.</td><td width="300" style="padding:12px;background:#fff6e5">Column C row ${i+1}</td></tr>`).join('');
const fixtures=[
 {subject:subjects.plain,type:'text/plain; charset=utf-8',body:'Hello,\r\n\r\nThis plain-text message checks the reader zoom. The quick brown fox jumps over the lazy dog, and again, so the paragraph wraps across several lines of the reading pane and reflow is visible when the level changes.\r\n\r\nSecond paragraph.'},
 {subject:subjects.wide,type:'text/html; charset=utf-8',body:`<html><body><table width="900" cellpadding="0" cellspacing="0" style="border:1px solid #ccc"><tr><td colspan="3" style="padding:16px;font-size:22px;font-weight:bold;background:#123;color:#fff">The Weekly Newsletter</td></tr>${wideRows}</table><p>Footer below the 900px table.</p></body></html>`},
 {subject:subjects.images,type:'text/html; charset=utf-8',body:`<html><body><p>An inline image, then a remote one.</p><p><img alt="inline square" width="64" height="64" src="data:image/png;base64,${png}"> Inline 64x64 image.</p><p><img alt="remote banner" width="320" height="80" src="http://203.0.113.10/banner.png"></p><p>Text after the images.</p></body></html>`},
];
function deliver(m:{subject:string;type:string;body:string},n:number):Promise<void>{
 return new Promise((resolve,reject)=>{
  const socket=net.connect(smtpPort,'127.0.0.1');
  const data=`From: Zoom Fixture <zoom@example.net>\r\nTo: testuser@example.org\r\nSubject: ${m.subject}\r\nDate: ${new Date(Date.now()+n*1000).toUTCString().replace('GMT','+0000')}\r\nMessage-ID: <t59-${run}-${n}@example.net>\r\nMIME-Version: 1.0\r\nContent-Type: ${m.type}\r\n\r\n${m.body}\r\n.\r\n`;
  const steps=['EHLO t59\r\n','MAIL FROM:<zoom@example.net>\r\n','RCPT TO:<testuser@example.org>\r\n','DATA\r\n',data,'QUIT\r\n'];
  let i=0,buffer='';
  socket.on('data',chunk=>{buffer+=chunk.toString();if(!buffer.endsWith('\r\n'))return;const last=buffer.trim().split('\r\n').at(-1)!;if(/^\d{3}-/.test(last))return;buffer='';if(/^[45]/.test(last)){socket.destroy();reject(new Error(last));return;}if(i<steps.length)socket.write(steps[i++]!);});
  socket.on('close',()=>resolve());socket.on('error',reject);
 });
}
test.describe.configure({timeout:300000});
let seeded=false;
test.beforeEach(async()=>{if(!seeded){for(const [n,m] of fixtures.entries())await deliver(m,n);seeded=true;}await new Promise(resolve=>setTimeout(resolve,3100));});

const shot=(info:TestInfo,name:string)=>{const dir=process.env['MW_T59_SHOTS'];return dir?path.join(dir,`${info.project.name}-${name}.png`):info.outputPath(`${name}.png`);};
async function signIn(page:Page):Promise<void>{
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(page.locator('.list__row').first()).toBeVisible();
}
async function open(page:Page,subject:string):Promise<void>{
 const row=page.locator('.list__row').filter({hasText:subject}).first();
 await expect(page.locator('.list__row').first()).toBeAttached({timeout:60000});
 // At phone width an open message covers the list: use its tab, or go back to Mail.
 const tab=page.getByRole('tab',{name:subject,exact:true});
 const mailTab=page.getByRole('tab',{name:'Mail',exact:true});
 await expect.poll(async()=>await row.isVisible()||await tab.isVisible()||await mailTab.isVisible(),{timeout:60000}).toBe(true);
 if(!(await row.isVisible())&&await tab.isVisible())await tab.click();
 else{if(!(await row.isVisible())){const mail=page.getByRole('tab',{name:'Mail',exact:true});if(await mail.isVisible())await mail.click();}
  await expect(row).toBeVisible({timeout:60000});await row.click();}
 await expect(page.locator('.reader__subject').filter({visible:true}).first()).toHaveText(subject);
}
/** The message document's source. The body frame runs no script (sandbox=""), so
 *  nothing can be evaluated inside it: read the srcdoc the app gave it, either on
 *  the reader's own frame or, in the remote-content bridge, on the nested frame. */
async function source(page:Page):Promise<string>{
 const outer=reader(page).locator('iframe.reader__frame').first();
 await expect(outer).toBeAttached();
 const direct=await outer.getAttribute('srcdoc');
 if(direct!==null&&direct!=='')return direct;
 let html='';
 await expect.poll(async()=>{
  try{const bridge=await (await outer.elementHandle())?.contentFrame();html=bridge?await bridge.evaluate(()=>document.querySelector('iframe')?.getAttribute('srcdoc')??''):'';}catch{html='';}
  return html.includes('Content-Security-Policy');
 }).toBe(true);
 return html;
}
const frameZoom=async(page:Page)=>{const m=/html\{zoom:([0-9.]+)\}/.exec(await source(page));return m?Number(m[1]):1;};
/** Pixel box of an element inside the body frame, when the engine can measure it. */
async function imgBox(page:Page,selector:string){try{return await reader(page).frameLocator('iframe.reader__frame').first().locator(selector).first().boundingBox({timeout:3000})??await reader(page).frameLocator('iframe.reader__frame').first().frameLocator('iframe').first().locator(selector).first().boundingBox({timeout:3000});}catch{try{return await reader(page).frameLocator('iframe.reader__frame').first().frameLocator('iframe').first().locator(selector).first().boundingBox({timeout:3000});}catch{return null;}}}
const reader=(page:Page)=>page.locator('section.reader').filter({visible:true}).first();
const level=(page:Page)=>reader(page).getByTestId('reader-zoom-level');
async function zoomControl(page:Page,which:'in'|'out'|'reset'):Promise<void>{
 const inline=reader(page).getByTestId(which==='reset'?'reader-zoom-level':`reader-zoom-${which}`);
 if(await inline.isVisible())return inline.click();
 // Leave any tooltip the last click opened, so it does not cover More.
 await page.mouse.move(1,1);await page.keyboard.press('Escape');
 await reader(page).getByTestId('reader-more').click();
 const name=which==='in'?'Zoom in':which==='out'?'Zoom out':/^Reset zoom/;
 await page.getByRole('menuitem',{name}).click();
}
async function expectLevel(page:Page,percent:number):Promise<void>{
 await expect.poll(()=>frameZoom(page)).toBeCloseTo(percent/100,2);
 const label=level(page);if(await label.isVisible())await expect(label).toHaveText(`${percent}%`);
}
/** Leave the stored level at 100% for the next test, through the control itself. */
async function resetToDefault(page:Page):Promise<void>{if(await frameZoom(page)!==1)await zoomControl(page,'reset');await expectLevel(page,100);}

for(const width of [1440,390]){
 test(`zooms only the message body with buttons, shortcuts and Ctrl+wheel at ${width}px`,async({page},info)=>{
  await page.setViewportSize({width,height:900});
  await signIn(page);
  await open(page,subjects.plain);
  // Start from 100% whatever an earlier run stored (that it is still there is persistence).
  await resetToDefault(page);
  const chrome=await page.evaluate(()=>getComputedStyle(document.querySelector('.reader__subject')!).fontSize);
  await page.screenshot({path:shot(info,`${width}-plain-100`)});
  // The action row stays one line; at phone width the zoom group folds into More.
  const toolbar=reader(page).locator('.reader__toolbar').first();
  const rowBoxes=await toolbar.locator('[data-reader-item]:not([hidden]), [data-testid="reader-more"]').evaluateAll(nodes=>nodes.filter(n=>(n as HTMLElement).offsetParent!==null).map(n=>Math.round(n.getBoundingClientRect().top)));
  expect(Math.max(...rowBoxes)-Math.min(...rowBoxes)).toBeLessThan(8);
  if(width<=390){
   await expect(reader(page).getByTestId('reader-zoom')).toBeHidden();
   await reader(page).getByTestId('reader-more').click();
   await page.screenshot({path:shot(info,`${width}-more-menu`)});
   await expect(page.getByRole('menuitem',{name:'Zoom in'})).toBeVisible();
   await page.keyboard.press('Escape');
  }

  await zoomControl(page,'in');await expectLevel(page,110);
  await zoomControl(page,'in');await expectLevel(page,125);
  await zoomControl(page,'out');await expectLevel(page,110);
  await zoomControl(page,'reset');await expectLevel(page,100);

  // Keyboard, with focus in the reader.
  await page.locator('.reader__subject').filter({visible:true}).first().focus();
  await page.keyboard.press('Control+Equal');await expectLevel(page,110);
  await page.keyboard.press('Control+Equal');await expectLevel(page,125);
  await page.keyboard.press('Control+Minus');await expectLevel(page,110);
  await page.keyboard.press('Control+Digit0');await expectLevel(page,100);

  // Ctrl + wheel over the body.
  const box=(await reader(page).locator('iframe.reader__frame').first().boundingBox())!;
  await page.mouse.move(box.x+box.width/2,box.y+Math.min(box.height/2,120));
  await page.keyboard.down('Control');await page.mouse.wheel(0,-120);await page.keyboard.up('Control');
  await expectLevel(page,110);
  await page.keyboard.down('Control');await page.mouse.wheel(0,120);await page.mouse.wheel(0,120);await page.keyboard.up('Control');
  await expectLevel(page,90);

  // The app UI did not scale.
  expect(await page.evaluate(()=>getComputedStyle(document.querySelector('.reader__subject')!).fontSize)).toBe(chrome);
  expect(await page.evaluate(()=>getComputedStyle(document.documentElement).zoom||'1')).toBe('1');

  // With focus in the list the app leaves Ctrl+= / Ctrl+- / Ctrl+0 to the browser.
  await page.locator('.list__row').filter({visible:true}).first().focus({timeout:5000}).catch(()=>undefined);
  const listFocused=await page.evaluate(()=>document.activeElement?.closest('.list')!==null&&document.activeElement?.closest('section.reader')===null);
  if(listFocused){
   await page.evaluate(()=>{(window as unknown as {t59:boolean[]}).t59=[];window.addEventListener('keydown',e=>{if(e.ctrlKey&&e.key!=='Control')(window as unknown as {t59:boolean[]}).t59.push(e.defaultPrevented);});});
   await page.keyboard.press('Control+Equal');await page.keyboard.press('Control+Minus');await page.keyboard.press('Control+Digit0');
   expect(await page.evaluate(()=>(window as unknown as {t59:boolean[]}).t59)).toEqual([false,false,false]);
  }
  await expectLevel(page,90);
  info.annotations.push({type:'list-focus',description:String(listFocused)});

  // Wide newsletter at 200%: horizontal scroll inside the frame only.
  await open(page,subjects.wide);
  for(let i=0;i<6;i++)await zoomControl(page,'in');
  await expectLevel(page,200);
  await expect.poll(()=>source(page)).toContain('The Weekly Newsletter');
  expect(await page.evaluate(()=>document.documentElement.scrollWidth-document.documentElement.clientWidth)).toBeLessThanOrEqual(1);
  await page.screenshot({path:shot(info,`${width}-newsletter-200`)});

  // Images scale with the text.
  await open(page,subjects.images);
  await expectLevel(page,200);
  const big=await imgBox(page,'img[alt="inline square"]');
  await page.screenshot({path:shot(info,`${width}-images-200`)});
  await zoomControl(page,'reset');await expectLevel(page,100);
  const small=await imgBox(page,'img[alt="inline square"]');
  info.annotations.push({type:'image-scale',description:big&&small?String(big.width/small.width):'not measurable in this engine'});
  if(big&&small)expect(big.width/small.width).toBeCloseTo(2,1);
  await page.screenshot({path:shot(info,`${width}-images-100`)});
  await zoomControl(page,'in');await zoomControl(page,'in');await zoomControl(page,'in');await expectLevel(page,150);
  await page.screenshot({path:shot(info,`${width}-images-150`)});

  // Persists across a reload, for every message.
  await page.reload();
  await open(page,subjects.plain);
  await expectLevel(page,150);
  await page.screenshot({path:shot(info,`${width}-plain-150-after-reload`)});

  // Print uses 100%.
  expect(await source(page)).toContain('@media print{html{zoom:1!important}}');
  await resetToDefault(page);
 });
}

test('the stored level applies in a detached reader window',async({page},info)=>{
 await page.setViewportSize({width:1440,height:900});
 await signIn(page);
 await open(page,subjects.plain);
 await resetToDefault(page);
 await zoomControl(page,'in');await zoomControl(page,'in');await expectLevel(page,125);
 const row=page.locator('.list__row').filter({hasText:subjects.plain}).first();
 await row.dblclick();
 const tab=page.getByRole('tab',{name:subjects.plain,exact:true});
 await expect(tab).toBeVisible();
 const pending=page.waitForEvent('popup');
 await tab.press('Shift+F10');await page.getByRole('menuitem',{name:`Open ${subjects.plain} in a new window`,exact:true}).click();
 const popup=await pending;
 await expect(popup.locator('.reader__subject').filter({visible:true}).first()).toHaveText(subjects.plain,{timeout:60000});
 await expectLevel(popup,125);
 await popup.screenshot({path:shot(info,'detached-125')});
 // The shared level is reset from the main window (actions inside the popup
 // are covered by reader-detached-actions-live.spec.ts, t68).
 await popup.close();
 await open(page,subjects.wide);
 await expectLevel(page,125);
 await resetToDefault(page);
});
