// t79: zooming the message body must not reload it. User report: "theres some
// flickering on zoom in and zoom out a message view". Before t79 every level
// change rebuilt the body document (the level was a rule in its own stylesheet),
// so the frame went blank for a few frames, scrolled back to the top and fetched
// its images again. The app now scales the frame from outside (ReaderZoomBox).
//
// Against a live server built from the tree: one long HTML message with an inline
// (cid) image and remote images is delivered over Greenmail SMTP. Each leg scrolls
// into the body, then steps the zoom in, out and back with the buttons, sampling:
//   - frame loads: `framenavigated`/`frameattached` for any frame, plus `load`
//     events on the reader's own iframe element (must be 0);
//   - blank frames: screenshots of the body box taken right after each step,
//     decoded in a scratch page; a frame with no ink is blank (must be 0), and a
//     DOM check that the message text is in the nested document;
//   - every animation frame in the app document: the reader chrome (subject,
//     action row) and the body box do not move (<= 1 px), and the iframe element
//     is never replaced;
//   - the scroll position inside the body stays put: the same paragraph at the top (+/- 2) and the fraction within 0.1 (images keep their height while text reflows, so the fraction drifts a little;
//     the paragraph is what the reader sees);
//   - image requests (/api/rc/i|p) after the first load: 0;
//   - the inline image's on-screen width is 2x at 200%.
// A Ctrl+wheel burst (12 notches in a row) lands on 200% with no reload and at
// most 3 preference writes. The detached-window leg repeats the steps in the popup.
// Frames and screenshots go to MW_T79_SHOTS when set (else the test output dir).
import {test,expect} from '../csrf.ts';
import type {Frame,Page,TestInfo} from '@playwright/test';
import net from 'node:net';
import path from 'node:path';
import fs from 'node:fs';

const smtpPort=Number(process.env['MW_E2E_SMTP_PORT']??42225);
const run=Date.now().toString(36);
const subject=`T79 zoom stability ${run}`;
const MARK='T79TOP';
// 64x64 opaque mid-grey PNG (the t59 fixture), sent as a cid part.
const PNG='iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAIAAAAlC+aJAAAAeUlEQVR4nO3PQQkAMAzAwMqpfxWTNRF7HINABFzm7H7dcEEDWtCAFjSgBQ1oQQNa0IAWNKAFDWhBA1rQgBY0oAUNaEEDWtCAFjSgBQ1oQQNa0IAWNKAFDWhBA1rQgBY0oAUNaEEDWtCAFjSgBQ1oQQNa0IAWNKAFj13vDoEA84tnsQAAAABJRU5ErkJggg==';
const paragraphs=Array.from({length:220},(_,i)=>`<p>T79P${i} paragraph of a long message, with enough words to wrap across the reading pane at every width and level.</p>`).join('\r\n');
const remote=Array.from({length:4},(_,i)=>`<img alt="remote ${i}" width="300" height="80" src="http://203.0.113.10/t79/${run}/${i}.png">`).join('<br>');
const html=`<p>${MARK} begins</p><p><img alt="inline square" width="64" height="64" src="cid:sq@t79"> Inline 64x64 image.</p><p>${remote}</p>${paragraphs}<p>T79END</p>`;
function deliver():Promise<void>{
 return new Promise((resolve,reject)=>{
  const socket=net.connect(smtpPort,'127.0.0.1');
  const data=`From: Zoom Stability <zoom@example.net>\r\nTo: testuser@example.org\r\nSubject: ${subject}\r\nDate: ${new Date().toUTCString().replace('GMT','+0000')}\r\nMessage-ID: <t79-${run}@example.net>\r\nMIME-Version: 1.0\r\nContent-Type: multipart/related; boundary="rel"\r\n\r\n`+
   `--rel\r\nContent-Type: text/html; charset=utf-8\r\n\r\n${html}\r\n--rel\r\nContent-Type: image/png\r\nContent-Transfer-Encoding: base64\r\nContent-ID: <sq@t79>\r\nContent-Disposition: inline; filename="sq.png"\r\n\r\n${PNG}\r\n--rel--\r\n.\r\n`;
  const steps=['EHLO t79\r\n','MAIL FROM:<zoom@example.net>\r\n','RCPT TO:<testuser@example.org>\r\n','DATA\r\n',data,'QUIT\r\n'];
  let i=0,buffer='';
  socket.on('data',chunk=>{buffer+=chunk.toString();if(!buffer.endsWith('\r\n'))return;const last=buffer.trim().split('\r\n').at(-1)!;if(/^\d{3}-/.test(last))return;buffer='';if(/^[45]/.test(last)){socket.destroy();reject(new Error(last));return;}if(i<steps.length)socket.write(steps[i++]!);});
  socket.on('close',()=>resolve());socket.on('error',reject);
 });
}
test.describe.configure({timeout:300000});
let seeded=false;
// eslint-disable-next-line no-empty-pattern -- Playwright passes fixtures first; only the test info is needed.
test.beforeEach(async({},info)=>{test.skip(info.project.name==='phone','widths are set per leg; the phone project repeats chromium');if(!seeded){await deliver();seeded=true;await new Promise(r=>setTimeout(r,3100));}});

const shotDir=(info:TestInfo):string=>{const dir=process.env['MW_T79_SHOTS'];const d=dir?path.join(dir,process.env['MW_T79_LABEL']??'run'):info.outputPath('t79');fs.mkdirSync(d,{recursive:true});return d;};
async function signIn(page:Page):Promise<void>{
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(page.locator('.list__row').first()).toBeVisible({timeout:60000});
}
const reader=(page:Page)=>page.locator('section.reader').filter({visible:true}).first();
async function open(page:Page):Promise<void>{
 const shown=page.locator('.reader__subject').filter({visible:true}).first();
 if(await shown.count()&&await shown.textContent()===subject)return;
 const row=page.locator('.list__row').filter({hasText:subject}).first();
 await expect(row).toBeVisible({timeout:60000});await row.click();
 await expect(page.locator('.reader__subject').filter({visible:true}).first()).toHaveText(subject);
}
/** The message document: the frame whose parent is the remote-content bridge (or the reader frame itself). */
function body(page:Page):Frame|undefined{
 const frames=page.frames();
 return frames.find(f=>f.parentFrame()?.url().includes('/api/rc/frame'))??frames.find(f=>f!==page.mainFrame()&&f.url()==='about:srcdoc');
}
async function bodyReady(page:Page):Promise<Frame>{
 let frame:Frame|undefined;
 await expect.poll(async()=>{frame=body(page);if(!frame)return false;try{return await frame.evaluate(m=>document.body?.innerText.includes(m)??false,MARK);}catch{return false;}},{timeout:30000}).toBe(true);
 return frame!;
}
type Scroll={st:number;frac:number;top:number;h:number;ch:number};
async function scrollState(page:Page):Promise<Scroll>{
 const frame=await bodyReady(page);
 return frame.evaluate(()=>{const se=document.scrollingElement!;const ps=[...document.querySelectorAll('p')];
  // The first paragraph whose bottom is below the viewport top.
  const top=ps.find(p=>p.getBoundingClientRect().bottom>2);const n=/T79P(\d+)/.exec(top?.textContent??'');
  return {st:se.scrollTop,frac:se.scrollTop/Math.max(1,se.scrollHeight-se.clientHeight),top:n?Number(n[1]):-1,h:se.scrollHeight,ch:se.clientHeight};});
}
const levelOf=async(page:Page):Promise<number>=>{
 const box=page.locator('[data-testid="reader-zoom-box"]').filter({visible:true}).first();
 if(await box.count())return Number(await box.getAttribute('data-reader-zoom'));
 const label=await reader(page).getByTestId('reader-zoom-level').textContent().catch(()=>null);return Number(/(\d+)%/.exec(label??'')?.[1]??100);
};
async function control(page:Page,which:'in'|'out'|'reset'):Promise<void>{
 const inline=reader(page).getByTestId(which==='reset'?'reader-zoom-level':`reader-zoom-${which}`);
 if(await inline.isVisible())return inline.click();
 await page.mouse.move(1,1);await page.keyboard.press('Escape');
 await reader(page).getByTestId('reader-more').click();
 await page.getByRole('menuitem',{name:which==='in'?'Zoom in':which==='out'?'Zoom out':/^Reset zoom/}).click();
}
async function resetTo100(page:Page):Promise<void>{if(await levelOf(page)!==100)await control(page,'reset');await expect.poll(()=>levelOf(page)).toBe(100);}

/** Every animation frame in the app document: chrome and body box positions, and the iframe identity. */
async function startSampler(page:Page):Promise<void>{
 await page.evaluate(()=>{
  const w=window as unknown as {t79:{samples:{subject:number;toolbar:number;box:string;level:string;frameId:number;missing:boolean}[];loads:number;stop:boolean}};
  w.t79={samples:[],loads:0,stop:false};
  let ids=0;const id=new WeakMap<Element,number>();
  const visible=(sel:string)=>[...document.querySelectorAll<HTMLElement>(sel)].find(e=>e.offsetParent!==null||e.getClientRects().length>0);
  const seen=new WeakSet<Element>();
  const tick=()=>{if(w.t79.stop)return;
   const root=visible('section.reader');const frame=root?.querySelector<HTMLIFrameElement>('iframe.reader__frame')??null;
   if(frame&&!seen.has(frame)){seen.add(frame);frame.addEventListener('load',()=>{w.t79.loads++;});}
   if(frame&&!id.has(frame))id.set(frame,++ids);
   const slot=(frame?.closest('[data-testid="reader-zoom-box"]')??frame) as HTMLElement|null;const r=slot?.getBoundingClientRect();
   w.t79.samples.push({subject:root?.querySelector('.reader__subject')?.getBoundingClientRect().top??-1,toolbar:root?.querySelector('.reader__toolbar')?.getBoundingClientRect().top??-1,box:r?`${Math.round(r.left)},${Math.round(r.top)},${Math.round(r.width)},${Math.round(r.height)}`:'none',level:root?.querySelector('[data-testid="reader-zoom-level"]')?.textContent??'',frameId:frame?id.get(frame)!:0,missing:!frame});
   requestAnimationFrame(tick);};
  requestAnimationFrame(tick);
 });
}
async function stopSampler(page:Page){return page.evaluate(()=>{const w=window as unknown as {t79:{samples:{subject:number;toolbar:number;box:string;level:string;frameId:number;missing:boolean}[];loads:number;stop:boolean}};w.t79.stop=true;return w.t79;});}

/** Fraction of "ink" pixels (luma < 150) in a PNG, decoded in a scratch page. */
async function ink(decoder:Page,png:Buffer):Promise<number>{
 return decoder.evaluate(async b64=>{const bytes=Uint8Array.from(atob(b64),c=>c.charCodeAt(0));const bmp=await createImageBitmap(new Blob([bytes],{type:'image/png'}));
  const c=new OffscreenCanvas(bmp.width,bmp.height);const g=c.getContext('2d')!;g.drawImage(bmp,0,0);const d=g.getImageData(0,0,bmp.width,bmp.height).data;
  let n=0;for(let i=0;i<d.length;i+=4){if(0.299*d[i]!+0.587*d[i+1]!+0.114*d[i+2]!<150)n++;}return n/(d.length/4);},png.toString('base64'));
}

type Leg={trace?:string[];reloads:number;elementLoads:number;blank:number;shots:number;moved:number;replaced:number;missing:number;samples:number;scroll:{level:number;frac:number;top:number}[];refetch:number;imageScale:number|null;notInDom:number};

/** Step through a fixed ladder with the buttons, measuring everything the leg asserts. */
async function zoomLeg(page:Page,decoder:Page,info:TestInfo,label:string):Promise<Leg>{
 const dir=shotDir(info);
 await open(page);
 await resetTo100(page);
 // Remote images: grant them where the policy asks, so they are requested once.
 const grant=reader(page).getByRole('button',{name:/^(Load images|Show images)$/}).first();
 if(await grant.isVisible({timeout:3000}).catch(()=>false))await grant.click();
 // Let the grant settle first: the remote images fail (unroutable test address),
 // and the bar and toast then change state once, which can re-render the body
 // once in Firefox a few seconds later (seen 2 of ~20 legs, not zoom related).
 // Start only after 4 s with no frame navigation (at most 30 s).
 {let last=Date.now();const mark=(f:Frame)=>{if(f!==page.mainFrame())last=Date.now();};page.on('framenavigated',mark);
  const until=Date.now()+30000;while(Date.now()-last<4000&&Date.now()<until)await page.waitForTimeout(250);page.off('framenavigated',mark);}
 let frame=await bodyReady(page);
 // Scroll about a third of the way into the body.
 await frame.evaluate(()=>{const se=document.scrollingElement!;se.scrollTop=Math.round((se.scrollHeight-se.clientHeight)*0.33);});
 await page.waitForTimeout(300);
 const start=await scrollState(page);
 const inline100=await (await bodyReady(page)).evaluate(()=>document.querySelector<HTMLImageElement>('img[alt="inline square"]')?.getBoundingClientRect().width??0);

 let images=0;const onRequest=(r:{url:()=>string})=>{if(/\/api\/rc\/(i|p)\//.test(new URL(r.url()).pathname))images++;};
 let reloads=0;const t0=Date.now();const trace:string[]=[];const onNav=(f:Frame)=>{if(f!==page.mainFrame()){reloads++;trace.push(`${Date.now()-t0}ms nav ${f.url().slice(0,40)}`);}};
 page.on('request',onRequest);page.on('framenavigated',onNav);page.on('frameattached',onNav);
 await startSampler(page);
 const box=await (page.locator('[data-testid="reader-zoom-box"]').filter({visible:true}).first().count())?page.locator('[data-testid="reader-zoom-box"]').filter({visible:true}).first():reader(page).locator('iframe.reader__frame').first();
 const clip=(await box.boundingBox())!;
 const ladder:('in'|'out'|'reset')[]=['in','in','in','in','in','out','out','out','reset','out','out','in','in'];
 let blank=0,shots=0,notInDom=0;const scroll:Leg['scroll']=[];let imageScale:number|null=null;
 for(const [k,step] of ladder.entries()){
  trace.push(`${Date.now()-t0}ms step ${k} ${step}`);await control(page,step);
  // Right after the step: back-to-back screenshots of the body, and the DOM check.
  for(let s=0;s<4;s++){
   const png=await page.screenshot({clip:{x:clip.x+2,y:clip.y+2,width:Math.min(clip.width-4,700),height:Math.min(clip.height-4,500)},animations:'allow',caret:'initial'});
   shots++;const v=await ink(decoder,png);
   if(v<0.002){blank++;fs.writeFileSync(path.join(dir,`${label}-blank-${k}-${s}.png`),png);}
   if(s===0)fs.writeFileSync(path.join(dir,`${label}-step${String(k).padStart(2,'0')}-${step}.png`),png);
   const f=body(page);let ok=false;try{ok=f?await f.evaluate(m=>document.body?.innerText.includes(m)??false,MARK):false;}catch{ok=false;}if(!ok)notInDom++;
  }
  await page.waitForTimeout(250);
  const level=await levelOf(page);
  const now=await scrollState(page);
  scroll.push({level,frac:+now.frac.toFixed(3),top:now.top});
  if(level===200&&imageScale===null){
   frame=await bodyReady(page);
   const inner=await frame.evaluate(()=>document.querySelector<HTMLImageElement>('img[alt="inline square"]')?.getBoundingClientRect().width??0);
   // On-screen width: the inner box times the outer frame's visual scale.
   const visual=await reader(page).locator('iframe.reader__frame').first().evaluate(e=>e.getBoundingClientRect().width/(e as HTMLElement).offsetWidth);
   imageScale=inline100?inner*visual/inline100:null;
  }
 }
 const sampled=await stopSampler(page);
 page.off('request',onRequest);page.off('framenavigated',onNav);page.off('frameattached',onNav);
 const s0=sampled.samples.find(s=>!s.missing);
 const moved=sampled.samples.filter(s=>s0&&!s.missing&&(Math.abs(s.subject-s0.subject)>1||Math.abs(s.toolbar-s0.toolbar)>1||s.box.split(',').some((v,i)=>Math.abs(Number(v)-Number(s0.box.split(',')[i]))>1))).length;
 const ids=new Set(sampled.samples.map(s=>s.frameId).filter(Boolean));
 const leg:Leg={trace:reloads?trace:[],reloads,elementLoads:sampled.loads,blank,shots,moved,replaced:Math.max(0,ids.size-1),missing:sampled.samples.filter(s=>s.missing).length,samples:sampled.samples.length,scroll:[{level:100,frac:+start.frac.toFixed(3),top:start.top},...scroll],refetch:images,imageScale,notInDom};
 info.annotations.push({type:`t79 ${label}`,description:JSON.stringify(leg)});
 fs.writeFileSync(path.join(dir,`${label}-measure.json`),JSON.stringify(leg,null,1));
 await resetTo100(page);
 return leg;
}
function assertLeg(leg:Leg,label:string):void{
 expect(leg.reloads,`${label}: frame loads during zoom`).toBe(0);
 expect(leg.elementLoads,`${label}: load events on the reader iframe`).toBe(0);
 expect(leg.blank,`${label}: blank body frames`).toBe(0);
 expect(leg.notInDom,`${label}: samples without the message text`).toBe(0);
 expect(leg.replaced,`${label}: iframe element replaced`).toBe(0);
 expect(leg.missing,`${label}: frames with no iframe`).toBe(0);
 expect(leg.moved,`${label}: frames where the chrome or the body box moved`).toBe(0);
 expect(leg.refetch,`${label}: image requests during zoom`).toBe(0);
 const start=leg.scroll[0]!;
 for(const s of leg.scroll){
  expect(Math.abs(s.frac-start.frac),`${label}: scroll fraction at ${s.level}%`).toBeLessThanOrEqual(0.1);
  expect(Math.abs(s.top-start.top),`${label}: top paragraph at ${s.level}%`).toBeLessThanOrEqual(2);
 }
 if(leg.imageScale!==null)expect(leg.imageScale,`${label}: inline image scale at 200%`).toBeCloseTo(2,1);
}

for(const width of [1440,390]){
 test(`zoom steps change the scale in place at ${width}px`,async({page,context},info)=>{
  await page.setViewportSize({width,height:900});
  const decoder=await context.newPage();
  await signIn(page);
  const leg=await zoomLeg(page,decoder,info,`${info.project.name}-${width}`);
  // Reduced motion: no transition on the box or the frame either way.
  await page.emulateMedia({reducedMotion:'reduce'});
  const motion=await page.evaluate(()=>{const f=document.querySelector<HTMLElement>('section.reader iframe.reader__frame');const b=f?.parentElement;const t=(e:Element|null|undefined)=>{if(!e)return null;const c=getComputedStyle(e);return `${c.transitionProperty}|${c.transitionDuration}`;};return [t(f),t(b)];});
  info.annotations.push({type:'transition',description:JSON.stringify(motion)});
  // Nothing that the zoom changes (transform, width, height) may be animated.
  for(const d of motion){if(d===null)continue;const [props,durs]=d.split('|') as [string,string];const ps=props.split(', ');const ds=durs.split(', ');
   ps.forEach((p,i)=>{const dur=ds[i%ds.length]!;if(['all','transform','width','height','scale'].includes(p))expect(parseFloat(dur),`transition on ${p}`).toBeLessThanOrEqual(0.001);});}
  assertLeg(leg,`${width}`);

  // Ctrl+wheel burst over the body: coalesced, no reload, few preference writes.
  await page.emulateMedia({reducedMotion:'no-preference'});
  let puts=0;let reloads=0;
  const onReq=(r:{url:()=>string;method:()=>string})=>{if(r.method()==='PUT'&&r.url().includes('reader.zoom'))puts++;};
  const onNav=(f:Frame)=>{if(f!==page.mainFrame())reloads++;};
  page.on('request',onReq);page.on('framenavigated',onNav);page.on('frameattached',onNav);
  const fb=(await reader(page).locator('iframe.reader__frame').first().boundingBox())!;
  await page.mouse.move(fb.x+fb.width/2,fb.y+Math.min(fb.height/2,150));
  await page.keyboard.down('Control');for(let i=0;i<12;i++)await page.mouse.wheel(0,-60);await page.keyboard.up('Control');
  await expect.poll(()=>levelOf(page)).toBe(200);
  await page.waitForTimeout(1500);
  await page.keyboard.down('Control');for(let i=0;i<12;i++)await page.mouse.wheel(0,60);await page.keyboard.up('Control');
  await expect.poll(()=>levelOf(page)).toBe(50);
  await page.waitForTimeout(1500);
  page.off('request',onReq);page.off('framenavigated',onNav);page.off('frameattached',onNav);
  info.annotations.push({type:'wheel',description:JSON.stringify({puts,reloads})});
  expect(reloads,'frame loads during the Ctrl+wheel bursts').toBe(0);
  // 14 level changes in all (100->200, 200->50); one write per change before t79.
  expect(puts,'preference writes for 2 x 12 wheel notches (14 level changes)').toBeLessThanOrEqual(6);
  await resetTo100(page);
  await decoder.close();
 });
}

test('zoom steps change the scale in place in a detached reader window',async({page,context},info)=>{
 await page.setViewportSize({width:1440,height:900});
 const decoder=await context.newPage();
 await signIn(page);
 await open(page);
 await resetTo100(page);
 const row=page.locator('.list__row').filter({hasText:subject}).first();
 await row.dblclick();
 const tab=page.getByRole('tab',{name:subject,exact:true});
 await expect(tab).toBeVisible();
 const pending=page.waitForEvent('popup');
 await tab.press('Shift+F10');await page.getByRole('menuitem',{name:`Open ${subject} in a new window`,exact:true}).click();
 const popup=await pending;
 await popup.setViewportSize({width:1100,height:860}).catch(()=>undefined);
 await expect(popup.locator('.reader__subject').filter({visible:true}).first()).toHaveText(subject,{timeout:60000});
 // The popup is rendered by the main window: its frames belong to the popup page.
 const leg=await zoomLeg(popup as Page,decoder,info,`${info.project.name}-detached`).catch(async e=>{throw e;});
 assertLeg(leg,'detached');
 const closed=popup.isClosed();
 expect(closed,'the detached window stays open').toBe(false);
 await popup.close();
 await resetTo100(page);
 await decoder.close();
});
