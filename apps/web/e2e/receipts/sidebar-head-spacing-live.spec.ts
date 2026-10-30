import {test,expect} from '../csrf.ts';
import {mkdirSync} from 'node:fs';
import type {Page} from '@playwright/test';

// t89: the sidebar head. The product title and the account selector share the
// folder list's inline inset (the title's text, the selector's avatar or
// address and the folder icons start on one line), the vertical rhythm on the
// spacing scale is title -> selector 8px (space 3), selector -> Compose 12px
// and Compose -> first folder 12px (space 4), and the selector's content sits
// centred in its box. Long titles and addresses end in an ellipsis with the full
// text as a tooltip; under dir=rtl the same measurements hold from the right.
// Stack: node scripts/e2e-receipts-stack.mjs. Chromium and Firefox; 1440 and
// 390 (the phone drawer), light and dark, are set here. Screenshots go to
// MW_T89_SHOTS when it is set.
const SHOTS=process.env['MW_T89_SHOTS'];
const SIZES=[{name:'1440',width:1440,height:900},{name:'390',width:390,height:844}] as const;
const TITLE_TO_SELECTOR=8,HEAD_TO_COMPOSE=12,COMPOSE_TO_FOLDERS=12;

type Box={l:number;r:number;t:number;b:number};
type Head={
  rtl:boolean;sidebar:Box;brand:Box|null;account:Box;trigger:Box|null;lead:Box|null;text:Box|null;chevron:Box|null;
  compose:Box;scroll:Box;folder:Box;folderIcon:Box;
};

async function measure(page:Page):Promise<Head>{
  return page.evaluate(()=>{
    const box=(el:Element|null|undefined)=>{if(!el)return null;const r=el.getBoundingClientRect();return r.width===0&&r.height===0?null:{l:r.left,r:r.right,t:r.top,b:r.bottom};};
    const nav=document.querySelector('#shell-nav')!;
    const brand=nav.querySelector('.sidebar__brand');
    // The brand's own text (a logo, when there is one, comes first and is the start).
    let brandBox=box(brand);
    if(brand&&getComputedStyle(brand).display!=='none'){
      // Inline start from the text itself, block extent from the title's box.
      const range=document.createRange();range.selectNodeContents(brand);const r=range.getBoundingClientRect(),e=brand.getBoundingClientRect();
      brandBox={l:Math.max(r.left,e.left),r:Math.min(r.right,e.right),t:e.top,b:e.bottom};
    }else brandBox=null;
    const account=nav.querySelector('.sidebar__account')!;
    const trigger=account.querySelector('[data-account-indicator="picker"], [data-account-indicator="plain"], [data-account-indicator]');
    const lead=trigger?.querySelector('.person-avatar')??null;
    const text=trigger?.querySelector('[data-mailbox-label]')??trigger?.querySelector('span span')??null;
    // The chevron is the picker button's last child.
    const chevron=trigger?.matches('button[data-account-indicator="picker"]')?trigger.lastElementChild:null;
    const folder=nav.querySelector('.sidebar__nav .sidebar__box')!;
    return {
      rtl:getComputedStyle(nav).direction==='rtl',sidebar:box(nav)!,brand:brandBox,account:box(account)!,
      trigger:box(trigger),lead:box(lead),text:box(text),chevron:box(chevron),
      compose:box(nav.querySelector('.sidebar__compose'))!,scroll:box(nav.querySelector('.sidebar__scroll'))!,
      folder:box(folder)!,folderIcon:box(folder.querySelector('svg'))!,
    };
  });
}
/** Inline-start inset from the sidebar edge (left in LTR, right in RTL). */
const inset=(h:Head,b:Box)=>h.rtl?h.sidebar.r-b.r:b.l-h.sidebar.l;
const mid=(b:Box)=>(b.t+b.b)/2;
const round=(n:number)=>Math.round(n*10)/10;

function summary(h:Head){
  return {
    rtl:h.rtl,
    titleInset:h.brand?round(inset(h,h.brand)):null,folderInset:round(inset(h,h.folderIcon)),
    selectorBoxInset:round(inset(h,h.account)),folderBoxInset:round(inset(h,h.folder)),
    selectorLeadInset:h.lead?round(inset(h,h.lead)):h.text?round(inset(h,h.text)):null,
    titleToSelector:h.brand?round(h.account.t-h.brand.b):null,headToCompose:round(h.compose.t-h.account.b),composeToFolders:round(h.folder.t-h.compose.b),
    triggerHeight:h.trigger?round(h.trigger.b-h.trigger.t):null,
    centreOffsets:h.trigger?[h.lead,h.text,h.chevron].filter((b):b is Box=>!!b).map(b=>round(mid(b)-mid(h.trigger!))):[],
    endInset:h.trigger?round(h.rtl?h.trigger.l-h.sidebar.l:h.sidebar.r-h.trigger.r):null,
  };
}

function check(h:Head,where:string,compact:boolean){
  const s=summary(h);
  if(!compact){
    expect.soft(Math.abs(s.titleInset!-s.folderInset),`${where}: title inset ${s.titleInset} = folder inset ${s.folderInset}`).toBeLessThanOrEqual(1);
    expect.soft(Math.abs(s.selectorBoxInset-s.folderBoxInset),`${where}: selector box ${s.selectorBoxInset} on the folder row edge ${s.folderBoxInset}`).toBeLessThanOrEqual(1);
    expect.soft(Math.abs(s.selectorLeadInset!-s.folderInset),`${where}: selector avatar/address ${s.selectorLeadInset} on the folder icon line ${s.folderInset}`).toBeLessThanOrEqual(1);
    expect.soft(Math.abs(s.titleToSelector!-TITLE_TO_SELECTOR),`${where}: title -> selector ${s.titleToSelector}`).toBeLessThanOrEqual(1);
    // The selector's far edge stays inside the sidebar padding (no overflow).
    expect.soft(s.endInset!,`${where}: selector end inset`).toBeGreaterThanOrEqual(0);
  }
  expect.soft(Math.abs(s.headToCompose-HEAD_TO_COMPOSE),`${where}: selector -> Compose ${s.headToCompose}`).toBeLessThanOrEqual(1);
  expect.soft(Math.abs(s.composeToFolders-COMPOSE_TO_FOLDERS),`${where}: Compose -> first folder ${s.composeToFolders}`).toBeLessThanOrEqual(1);
  for(const offset of s.centreOffsets)expect.soft(Math.abs(offset),`${where}: selector content centred (${s.centreOffsets.join(',')})`).toBeLessThanOrEqual(1);
  return s;
}

async function drawer(page:Page){
  const open=page.getByRole('button',{name:'Open folders and apps',exact:true});
  if(await open.isVisible()&&await open.getAttribute('aria-expanded')!=='true'){await open.click();await page.waitForTimeout(400);}
}
async function signIn(page:Page,user:string,pass:string){
  await page.goto('/');
  await page.getByLabel('Email or username',{exact:true}).fill(user);
  await page.getByLabel('Password',{exact:true}).fill(pass);
  await page.getByRole('button',{name:'Sign in',exact:true}).click();
  await expect(page.getByLabel('Email or username',{exact:true})).toHaveCount(0);
  await expect(page.locator('#shell-nav .sidebar__account [data-account-indicator]').first()).toBeAttached();
  await expect(page.locator('#shell-nav .sidebar__nav .sidebar__box').first()).toBeVisible({timeout:30000});
}
/** Each shard establishes its own picker fixture; no preceding spec is required. */
async function ensurePicker(page:Page):Promise<void>{
  const meResponse=await page.request.get('/api/me');
  expect(meResponse.ok(),'read authenticated picker owner').toBe(true);
  const me=await meResponse.json() as {storageAuthority:string};
  expect(me.storageAuthority).toBeTruthy();
  const headers={'x-mailwoman-storage-authority':me.storageAuthority};
  const listing=async()=>{
    const response=await page.request.get('/api/accounts',{headers});
    expect(response.ok(),'read picker links').toBe(true);
    return (await response.json() as {accounts:{id:string;address?:string;status:string;revision:string}[]}).accounts;
  };
  if((await listing()).some(account=>account.status==='ready'))return;
  expect((await page.request.post('/api/security/step-up',{data:{password:'parent-pass'}})).ok()).toBe(true);
  const stale=(await listing()).find(account=>account.address==='child@child.test');
  if(stale)expect((await page.request.delete(`/api/accounts/${stale.id}`,{headers:{...headers,'if-match':JSON.stringify(stale.revision)}})).ok()).toBe(true);
  const linked=await page.request.post('/api/accounts/link',{headers,data:{username:'child@child.test',password:'child-pass',displayName:'Child'}});
  expect(linked.status(),'create the independent sidebar picker fixture').toBe(201);
  await expect.poll(async()=>(await listing()).some(account=>account.status==='ready')).toBe(true);
  await page.reload();
}
async function folderMode(page:Page,mode:'expanded'|'compact'){
  const button=page.getByRole('button',{name:'Compact folder icons',exact:true}).first();
  const pressed=await button.getAttribute('aria-pressed')==='true';
  if((mode==='compact')!==pressed)await button.click();
  await expect(page.locator('.workspace-shell')).toHaveAttribute('data-folder-mode',mode);
  await page.waitForTimeout(200);
}
/** Light or dark through Settings > Appearance (the default theme is not the system's). */
async function theme(page:Page,name:'Light'|'Dark'){
  const nav=page.getByRole('navigation',{name:'Settings sections'});
  if(!await nav.isVisible()){
    const settings=page.locator('#shell-nav').getByRole('button',{name:'Settings',exact:true}).first();
    if(!await settings.isVisible())await drawer(page);
    await settings.click();
  }
  await nav.getByRole('button',{name:'Appearance',exact:true}).click();
  await page.getByRole('button',{name,exact:true}).first().click();
  await expect.poll(()=>page.evaluate(()=>document.documentElement.getAttribute('data-appearance'))).toBe(name.toLowerCase());
}
async function shoot(page:Page,name:string){
  if(!SHOTS)return;
  const nav=page.locator('#shell-nav');
  const b=await nav.boundingBox();
  const path=`${SHOTS}/${name}.png`;
  await page.screenshot(b?{path,clip:{x:Math.max(0,b.x),y:0,width:Math.min(b.width,400),height:Math.min(320,b.height)}}:{path});
}

for(const [user,pass,kind] of [['parent@example.org','parent-pass','picker'],['testuser','testpass','plain']] as const){
  test(`sidebar head spacing: title and ${kind} account selector on the folder inset and the spacing scale`,async({page},info)=>{
    test.skip(info.project.name==='phone','the spec sets 1440 and 390 itself');
    test.setTimeout(240000);
    if(SHOTS)mkdirSync(SHOTS,{recursive:true});
    const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
    const csp:string[]=[];page.on('console',m=>{if(/Content.Security.Policy|CSP/i.test(m.text()))csp.push(m.text());});
    await signIn(page,user,pass);
    if(kind==='picker'){
      await ensurePicker(page);
      await expect(page.locator('#shell-nav button[data-account-indicator="picker"]')).toBeAttached();
    }
    const results:Record<string,unknown>={};
    const initial=await page.evaluate(()=>document.documentElement.getAttribute('data-appearance'));
    for(const scheme of ['light','dark'] as const){
      await page.setViewportSize(SIZES[0]);
      await theme(page,scheme==='light'?'Light':'Dark');
      for(const size of SIZES){
        await page.setViewportSize(size);
        await page.waitForTimeout(300);
        const phone=size.width<600;
        const modes:('expanded'|'compact')[]=phone?['expanded']:['expanded','compact'];
        for(const mode of modes){
          if(!phone)await folderMode(page,mode);
          if(phone)await drawer(page);
          const name=`${info.project.name}-${kind}-${size.name}-${scheme}-${mode}`;
          for(const dir of ['ltr','rtl'] as const){
            await page.evaluate(d=>document.documentElement.setAttribute('dir',d),dir);
            await page.waitForTimeout(150);
            const h=await measure(page);
            results[`${name}-${dir}`]=check(h,`${name}-${dir}`,mode==='compact');
            await shoot(page,`${name}-${dir}`);
          }
          await page.evaluate(()=>document.documentElement.setAttribute('dir','ltr'));
          if(mode==='compact'){
            // Compact: the one-letter selector is centred in the 64px rail.
            const h=await measure(page);
            const trigger=h.trigger??h.account;
            expect.soft(Math.abs((trigger.l+trigger.r)/2-(h.sidebar.l+h.sidebar.r)/2),`${name}: compact selector centred in the rail`).toBeLessThanOrEqual(1.5);
            await folderMode(page,'expanded');
          }
        }
        if(phone){await page.keyboard.press('Escape');await page.waitForTimeout(300);}
      }
    }
    // Long title and address: an ellipsis inside the head, the full text in a tooltip.
    await page.setViewportSize(SIZES[0]);
    if(initial==='light')await theme(page,'Light');
    await folderMode(page,'expanded');
    await page.mouse.move(700,450);
    const long=await page.evaluate(()=>{
      const nav=document.querySelector('#shell-nav')!;
      const brand=nav.querySelector<HTMLElement>('.sidebar__brand')!;
      const label=nav.querySelector<HTMLElement>('.sidebar__account [data-mailbox-label], .sidebar__account [data-account-indicator="plain"] span')!;
      const text=brand.lastChild as Text;const before=text.data;text.data='A very long product name for this webmail service';
      const labelBefore=label.textContent;label.textContent='a.very.long.mailbox.address.for.testing@subdomain.example.org';
      const head=nav.querySelector('.sidebar__head')!.getBoundingClientRect(),side=nav.getBoundingClientRect();
      const out={
        brandEllipsis:getComputedStyle(brand).textOverflow,brandClipped:brand.scrollWidth>brand.clientWidth,brandTitle:brand.getAttribute('title')??brand.getAttribute('data-tooltip-title'),
        labelEllipsis:getComputedStyle(label).textOverflow,labelClipped:label.scrollWidth>label.clientWidth,
        // The tooltip layer adopts a native title while hovered (data-tooltip-title).
        selectorTitle:(()=>{const el=label.closest('[title],[data-tooltip],[data-tooltip-title]');return el?.getAttribute('title')??el?.getAttribute('data-tooltip')??el?.getAttribute('data-tooltip-title')??null;})(),
        headInside:head.left>=side.left-0.5&&head.right<=side.right+0.5,
        headScroll:nav.querySelector('.sidebar__head')!.scrollWidth<=nav.querySelector<HTMLElement>('.sidebar__head')!.clientWidth+1,
      };
      text.data=before;label.textContent=labelBefore;
      return out;
    });
    results['long']=long;
    expect.soft(long.brandEllipsis,'long title ends in an ellipsis').toBe('ellipsis');
    expect.soft(long.brandClipped,'long title is clipped, not wrapped').toBe(true);
    expect.soft(long.brandTitle,'title has the full name as a tooltip').toBeTruthy();
    expect.soft(long.labelEllipsis,'long address ends in an ellipsis').toBe('ellipsis');
    expect.soft(long.selectorTitle,'selector has the full address as a tooltip').toBeTruthy();
    expect.soft(long.headInside&&long.headScroll,'head never overflows the sidebar').toBe(true);
    // Focus-ring room: the selector's ring (the theme's ring width plus the 2px
    // offset of a11y.focusable) stays inside the sidebar on both sides.
    const ring=await page.evaluate(()=>{
      const nav=document.querySelector('#shell-nav')!;const el=nav.querySelector('.sidebar__account [data-account-indicator]')!;
      const width=parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--focus-ring-width'))||3;
      const out=width+2,r=el.getBoundingClientRect(),s=nav.getBoundingClientRect();
      return {out,start:r.left-out-s.left,end:s.right-(r.right+out)};
    });
    results['focus']=ring;
    expect.soft(ring.start,'focus ring room before the selector').toBeGreaterThanOrEqual(0);
    expect.soft(ring.end,'focus ring room after the selector').toBeGreaterThanOrEqual(0);
    await info.attach('sidebar-head.json',{body:JSON.stringify(results,null,1),contentType:'application/json'});
    console.log(`T89 ${info.project.name} ${kind} ${JSON.stringify(results)}`);
    expect(errors,'no page errors').toEqual([]);
    expect(csp,'no CSP reports').toEqual([]);
  });
}
