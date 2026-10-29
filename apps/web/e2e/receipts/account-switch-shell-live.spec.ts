import {test,expect} from '../csrf.ts';
import {connect} from 'node:net';
import {mkdirSync} from 'node:fs';
import type {Page} from '@playwright/test';

// t71: switching between linked mailboxes keeps the mailbox shell. The tab
// strip, the top bar, the sidebar and the message-list frame stay the SAME DOM
// nodes in the same place for every animation frame of the switch; only the
// account's own content (folder names, rows, counts) goes to placeholders that
// sit where that content goes.
// Stack: node scripts/e2e-receipts-stack.mjs (greenmail without sign-in checks,
// mailbox linking on). Chromium and Firefox; 1440 and 390 are set here.
const SMTP=process.env['MW_RECEIPTS_SMTP']??'127.0.0.1:42225';
const SHOTS=process.env['MW_SCREENSHOT_DIR'];
const SIZES=[{name:'1440',width:1440,height:900},{name:'390',width:390,height:844}] as const;

async function deliver(to:string,subject:string):Promise<void>{
  const [host,port]=SMTP.split(':');
  await new Promise<void>((resolve,reject)=>{
    const socket=connect(Number(port),host!);
    const steps=['EHLO switch.test','MAIL FROM:<sender@switch.test>',`RCPT TO:<${to}>`,'DATA',
      `From: Switch Sender <sender@switch.test>\r\nTo: ${to}\r\nSubject: ${subject}\r\nMessage-ID: <${subject.replace(/\W/g,'-')}@switch.test>\r\n\r\nShell stays.\r\n.`,'QUIT'];
    let buffer='';
    socket.setEncoding('utf8');
    socket.on('data',chunk=>{
      buffer+=chunk;
      const lines=buffer.split('\r\n');buffer=lines.pop()!;
      for(const line of lines){
        if(/^\d{3}-/.test(line))continue;
        if(!/^[23]/.test(line)){socket.destroy();reject(new Error(`smtp: ${line}`));return;}
        const next=steps.shift();
        if(next===undefined){socket.end();resolve();return;}
        socket.write(`${next}\r\n`);
      }
    });
    socket.on('error',reject);
  });
}
async function drawer(page:Page){
  const open=page.getByRole('button',{name:'Open folders and apps',exact:true});
  if(await open.isVisible()&&await open.getAttribute('aria-expanded')!=='true')await open.click();
}
async function openMenu(page:Page){
  const trigger=page.locator('button[data-account-indicator="picker"]');
  if(await page.getByRole('button',{name:'Open folders and apps',exact:true}).isVisible()){await drawer(page);await page.waitForTimeout(400);}
  if(await trigger.getAttribute('aria-expanded')!=='true')await trigger.click();
  const menu=page.getByRole('menu',{name:'Mailbox account',exact:true});
  await expect(menu).toBeVisible();
  return menu;
}
/** Back to the mail list (a restored Settings tab may be in front). */
async function inbox(page:Page){
  const box=page.locator('#shell-nav .sidebar__nav').getByRole('button',{name:/^Inbox/}).first();
  if(!await box.isVisible())await drawer(page);
  await box.click();
  // Choosing a folder closes the phone drawer; wait until it has slid away.
  await expect(page.locator('#shell-nav')).not.toHaveClass(/sidebar--open/);
  await page.waitForTimeout(400);
}
async function signIn(page:Page){
  await page.goto('/');
  await page.getByLabel('Email or username',{exact:true}).fill('parent@example.org');
  await page.getByLabel('Password',{exact:true}).fill('parent-pass');
  await page.getByRole('button',{name:'Sign in',exact:true}).click();
  await expect(page.getByLabel('Email or username',{exact:true})).toHaveCount(0);
  await expect(page.locator('[data-account-indicator]').first()).toBeAttached();
}
/** The child link (made through Settings › Linked mailboxes when it is missing). */
async function ensureLink(page:Page):Promise<string>{
  const listing=async()=>(await (await page.request.get('/api/accounts')).json() as {accounts:{id:string;address?:string;status:string}[]}).accounts;
  const found=(await listing()).find(account=>account.address==='child@child.test'&&account.status==='ready');
  if(found)return found.id;
  await drawer(page);
  await page.locator('.sidebar, #shell-nav').getByRole('button',{name:'Settings',exact:true}).first().click();
  await page.getByRole('navigation',{name:'Settings sections'}).getByRole('button',{name:'Linked mailboxes',exact:true}).click();
  await page.getByRole('button',{name:'Link a mailbox',exact:true}).click();
  const editor=page.getByRole('region',{name:'Link a mailbox',exact:true});
  await editor.getByLabel('Username or email address',{exact:true}).fill('child@child.test');
  await editor.getByLabel('Password',{exact:true}).fill('child-pass');
  await editor.getByRole('button',{name:'Verify and link',exact:true}).click();
  // The step-up dialog appears unless a recent confirmation still holds.
  const proof=page.getByRole('dialog',{name:'Confirm it is you'});
  const done=page.getByRole('heading',{name:'Linked mailboxes',exact:true});
  await expect(proof.or(done).first()).toBeVisible();
  if(await proof.isVisible()){
    await proof.getByLabel('Password',{exact:true}).fill('parent-pass');
    await proof.getByRole('button',{name:'Confirm and continue',exact:true}).click();
  }
  await expect.poll(async()=>(await listing()).find(account=>account.address==='child@child.test')?.status).toBe('ready');
  const id=(await listing()).find(account=>account.address==='child@child.test')!.id;
  await page.reload();
  return id;
}

/** The frame parts that must survive a switch, by selector. */
const FRAME={
  shell:'.workspace-shell',
  tabs:'.subtab-strip',
  topbar:'.workspace-bar',
  sidebar:'#shell-nav',
  listPane:'.mail-pane',
  list:'section.list',
  phoneBar:'header.shell__bar',
  // The account switcher's own box and the first row under the folders: the
  // folder placeholders keep it where the folders put it.
  account:'.sidebar__account',
  belowFolders:'#shell-nav .sidebar__nav > button[aria-label="Attachments"]',
} as const;

type Sample={t:number;missing:string[];replaced:string[];rects:Record<string,[number,number,number,number]|null>;pending:number;fullPending:boolean;animated:number};

/** Start an every-frame sampler: node identity and the boxes of each part. */
async function startSampler(page:Page){
  await page.evaluate((frame)=>{
    const w=window as unknown as {__t71?:{nodes:Record<string,Element|null>;samples:unknown[];stop:boolean}};
    const nodes:Record<string,Element|null>={};
    for(const [name,selector] of Object.entries(frame))nodes[name]=document.querySelector(selector);
    const state={nodes,samples:[] as unknown[],stop:false};w.__t71=state;
    const start=performance.now();
    const tick=()=>{
      if(state.stop)return;
      const missing:string[]=[],replaced:string[]=[];const rects:Record<string,[number,number,number,number]|null>={};
      for(const [name,selector] of Object.entries(frame)){
        const kept=state.nodes[name];if(!kept)continue;
        if(!kept.isConnected)missing.push(name);
        if(document.querySelector(selector)!==kept)replaced.push(name);
        const box=kept.isConnected&&!(kept as HTMLElement).hidden?kept.getBoundingClientRect():null;
        rects[name]=box?[Math.round(box.x),Math.round(box.y),Math.round(box.width),Math.round(box.height)]:null;
      }
      const pendings=[...document.querySelectorAll('[data-testid="async-pending"]')];
      // A pending state that is not inside the list or the sidebar is the
      // route-level fallback standing in for the whole screen.
      const fullPending=pendings.some(el=>!el.closest('section.list')&&!el.closest('#shell-nav')&&!el.closest('.workspace-pane'));
      // Placeholder shapes that are animating (none under reduced motion).
      const animated=[...document.querySelectorAll('[data-skeleton] span, [data-skeleton] [data-folder-skeleton] span')]
        .filter(el=>getComputedStyle(el).animationName!=='none').length;
      state.samples.push({t:Math.round(performance.now()-start),missing,replaced,rects,pending:pendings.length,fullPending,animated});
      requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  },FRAME);
}
async function stopSampler(page:Page):Promise<Sample[]>{
  return page.evaluate(()=>{const w=window as unknown as {__t71:{stop:boolean;samples:Sample[]}};w.__t71.stop=true;return w.__t71.samples;});
}

/** Each switch request is held back a little (passed through, not answered by
 *  the test) so the in-between state is on screen long enough to see. */
async function slowSwitch(page:Page,on:boolean){
  const pattern=/\/api\/accounts\/[^/]+\/authorize|\/api\/me|\/jmap\/session|\/jmap\/api/;
  if(on)await page.route(pattern,async route=>{await new Promise(r=>setTimeout(r,350));await route.continue().catch(()=>undefined);});
  else await page.unrouteAll({behavior:'wait'});
}

test('switching mailboxes keeps the shell: same nodes, same boxes, placeholders only for account content',async({page},info)=>{
  test.skip(info.project.name==='phone','the spec sets 1440 and 390 itself');
  test.setTimeout(240000);
  page.setDefaultTimeout(20000);
  const errors:string[]=[];page.on('pageerror',error=>errors.push(error.message));
  if(SHOTS)mkdirSync(SHOTS,{recursive:true});
  const tag=`${info.project.name}-${Date.now()}`;
  await deliver('child@child.test',`Child only ${tag}`);
  await deliver('parent@example.org',`Parent only ${tag}`);
  await signIn(page);
  const link=await ensureLink(page);

  for(const size of SIZES){
    await page.setViewportSize(size);
    // The phone leg runs with reduced motion: the placeholders must be still.
    await page.emulateMedia({reducedMotion:size.width<600?'reduce':'no-preference'});
    await page.reload();
    await expect(page.locator('[data-account-indicator]').first()).toBeAttached();
    // Start in the parent mailbox, its list loaded.
    await expect(page.locator('button[data-account-indicator="picker"]')).toBeAttached();
    await inbox(page);
    await expect(page.locator('section.list').getByText("Parent only").first()).toBeVisible({timeout:30000}).catch(async()=>{
      // Opened in the child mailbox (the last session's choice): go back first.
      const menu=await openMenu(page);await menu.locator('[data-mailbox-item][data-link=""]').click();
      await inbox(page);
      await expect(page.locator('section.list').getByText("Parent only").first()).toBeVisible({timeout:30000});
    });

    for(const [target,expectText,label] of [[link,"Child only","to-child"],[null,"Parent only","to-parent"]] as const){
      const menu=await openMenu(page);
      const item=menu.locator(target===null?'[data-mailbox-item][data-link=""]':`[data-mailbox-item][data-link="${target}"]`).first();
      if(SHOTS)await page.screenshot({path:`${SHOTS}/${info.project.name}-${size.name}-${label}-00-before.png`});
      await slowSwitch(page,true);
      // Close the drawer at phone width first so its slide is not in the samples.
      await startSampler(page);
      await item.click();
      // A frame sequence of the switch, as fast as screenshots go.
      let settled=false;
      const frames=(async()=>{
        for(let step=1;SHOTS&&!settled&&step<=60;step++)
          await page.screenshot({path:`${SHOTS}/${info.project.name}-${size.name}-${label}-${String(step).padStart(2,'0')}-during.png`}).catch(()=>undefined);
      })();
      await expect(page.locator('section.list').getByText(expectText).first()).toBeVisible({timeout:30000});
      await expect(page.locator('[data-account-picker]').first()).not.toHaveAttribute('aria-busy','true');
      await page.waitForTimeout(400);
      settled=true;await frames;
      const samples=await stopSampler(page);
      await slowSwitch(page,false);
      if(SHOTS)await page.screenshot({path:`${SHOTS}/${info.project.name}-${size.name}-${label}-99-after.png`});
      const first=samples[0]!;
      const moved:string[]=[];
      // The sidebar is a drawer at 390: choosing closes it (it slides), so only
      // the identity of it and what is in it is held there.
      const inDrawer=['sidebar','account','belowFolders'];
      const boxed=Object.keys(first.rects).filter(name=>!(size.width<600&&inDrawer.includes(name)));
      for(const sample of samples)for(const name of boxed){
        const a=first.rects[name]??null,b=sample.rects[name]??null;
        if(JSON.stringify(a)!==JSON.stringify(b)&&(a===null||b===null||a.some((v,i)=>Math.abs(v-b[i]!)>1)))moved.push(`${name}@${sample.t}ms ${JSON.stringify(a)}->${JSON.stringify(b)}`);
      }
      const report={project:info.project.name,size:size.name,label,frames:samples.length,
        missing:[...new Set(samples.flatMap(s=>s.missing))],replaced:[...new Set(samples.flatMap(s=>s.replaced))],
        fullPendingFrames:samples.filter(s=>s.fullPending).length,pendingFrames:samples.filter(s=>s.pending>0).length,animatedFrames:samples.filter(s=>s.animated>0).length,moved:moved.slice(0,8),movedCount:moved.length};
      console.log(`T71 ${JSON.stringify(report)}`);
      await info.attach(`switch-${size.name}-${label}.json`,{body:JSON.stringify({report,samples}),contentType:'application/json'});
      expect.soft(report.missing,'frame parts never leave the document').toEqual([]);
      expect.soft(report.replaced,'frame parts are the same nodes').toEqual([]);
      expect.soft(report.fullPendingFrames,'no whole-screen loading state').toBe(0);
      expect.soft(moved,'frame parts do not move').toEqual([]);
      if(size.width<600)expect.soft(report.animatedFrames,'reduced motion: still placeholders').toBe(0);
      else expect.soft(report.animatedFrames,'placeholders pulse without reduced motion (sampler sees them)').toBeGreaterThan(0);
      expect(samples.length,'the sampler ran through the switch').toBeGreaterThan(5);
    }
  }
  expect(errors).toEqual([]);
});
