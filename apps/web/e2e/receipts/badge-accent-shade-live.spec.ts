// t85: the "Lighter accent" / "Darker accent" unread badge colours against a
// live server built from the tree. Each is the effective accent (named,
// custom or the theme's own) with its OKLCH lightness 0.2 higher or lower,
// clamped, with black or white count text at 4.5:1 or better. The colour is
// recomputed when the accent or theme changes, without a reload, and an
// administrator can make either the locked default.
// Runs at 1440 and 390 wide in the chromium and firefox projects, light and
// dark theme. Screenshots go to MW_T85_SHOTS.
import {test,expect} from '../csrf.ts';
import { ADMIN } from '../admin/admin-path.ts';
import type {Page,TestInfo,APIRequestContext} from '@playwright/test';
import path from 'node:path';

const KEY='appearance.badgeColor';
test.describe.configure({timeout:300000});
const skipPhone=(info:TestInfo)=>test.skip(info.project.name==='phone','the 390 legs run in the chromium and firefox projects');
const shot=(info:TestInfo,name:string)=>{const dir=process.env['MW_T85_SHOTS'];return dir?path.join(dir,`${info.project.name}-${name}.png`):info.outputPath(`${name}.png`);};

type Rgb=[number,number,number];
function parseRgb(css:string):Rgb{const m=/rgba?\((\d+(?:\.\d+)?)[ ,]+(\d+(?:\.\d+)?)[ ,]+(\d+(?:\.\d+)?)/.exec(css);if(!m)throw new Error(`not rgb: ${css}`);return [Number(m[1]),Number(m[2]),Number(m[3])];}
const lin=(v:number)=>{const s=v/255;return s<=0.04045?s/12.92:Math.pow((s+0.055)/1.055,2.4);};
function oklabL([r,g,b]:Rgb):number{
 const [R,G,B]=[lin(r),lin(g),lin(b)];
 const l=Math.cbrt(0.4122214708*R+0.5363325363*G+0.0514459929*B);
 const m=Math.cbrt(0.2119034982*R+0.6806995451*G+0.1073969566*B);
 const s=Math.cbrt(0.0883024619*R+0.2817188376*G+0.6299787005*B);
 return 0.2104542553*l+0.793617785*m-0.0040720468*s;
}
function contrast(a:Rgb,b:Rgb):number{const Y=([r,g,bb]:Rgb)=>0.2126*lin(r)+0.7152*lin(g)+0.0722*lin(bb);const [x,y]=[Y(a),Y(b)].sort((p,q)=>q-p) as [number,number];return (x+0.05)/(y+0.05);}

async function signIn(page:Page):Promise<void>{
 await page.goto('/');await page.getByLabel('Email or username',{exact:true}).fill('testuser');await page.getByLabel('Password',{exact:true}).fill('testpass');await page.getByRole('button',{name:'Sign in',exact:true}).click();
 await expect(page.getByLabel('Email or username',{exact:true})).toHaveCount(0,{timeout:60000});
 await expect(page.locator('.mail-pane')).toBeVisible({timeout:60000});
}
async function drawer(page:Page):Promise<void>{const menu=page.locator('header.shell__bar button[aria-controls="shell-nav"]').first();if(await menu.isVisible()&&(await menu.getAttribute('aria-expanded'))!=='true')await menu.click();}
async function openAppearance(page:Page):Promise<void>{
 const nav=page.getByRole('navigation',{name:'Settings sections'});
 if(!await nav.isVisible()){
  const settings=page.locator('.sidebar, #shell-nav').getByRole('button',{name:'Settings',exact:true}).first();
  if(!await settings.isVisible())await drawer(page);
  await settings.click();
 }
 await nav.getByRole('button',{name:'Appearance',exact:true}).click();
 await expect(page.getByRole('group',{name:'Unread badge colour',exact:true})).toBeVisible();
}
async function closeSettings(page:Page):Promise<void>{await page.getByRole('button',{name:'Close settings',exact:true}).click();await expect(page.getByRole('navigation',{name:'Settings sections'})).toHaveCount(0);}

/** What the page paints right now: the accent, and the badge colour and text, all as computed rgb(). */
async function measure(page:Page){
 return page.evaluate(()=>{
  const probe=(cls:string,bg:string)=>{const el=document.createElement('span');el.className=cls;if(bg)el.style.background=bg;document.body.append(el);const cs=getComputedStyle(el);const out={bg:cs.backgroundColor,fg:cs.color};el.remove();return out;};
  const real=document.querySelector('.sidebar__badge');
  const badge=real?{bg:getComputedStyle(real).backgroundColor,fg:getComputedStyle(real).color}:probe('sidebar__badge','');
  return {accent:probe('','var(--accent)').bg,badge,real:real!==null,attr:document.documentElement.getAttribute('data-badge'),appearance:document.documentElement.getAttribute('data-appearance')};
 });
}
async function expectShade(page:Page,which:'lighter'|'darker',label:string):Promise<{accent:string;badge:string}>{
 let last='';
 await expect.poll(async()=>{
  const m=await measure(page);
  const a=oklabL(parseRgb(m.accent));const b=oklabL(parseRgb(m.badge.bg));
  const want=Math.min(1,Math.max(0,a+(which==='lighter'?0.2:-0.2)));
  last=`${label}: accent ${m.accent} L=${a.toFixed(3)} badge ${m.badge.bg} L=${b.toFixed(3)} want ${want.toFixed(3)} text ${m.badge.fg} attr ${m.attr}`;
  return m.attr===`accent-${which}`&&Math.abs(b-want)<0.012&&contrast(parseRgb(m.badge.bg),parseRgb(m.badge.fg))>=4.5;
 }).toBe(true).catch((error:unknown)=>{throw new Error(`${label}: shade never settled; last ${last}`,{cause:error});});
 console.log('T85_SHADE',last);
 const m=await measure(page);return {accent:m.accent,badge:m.badge.bg};
}

async function adminApi(playwright:{request:{newContext:(o:{baseURL:string})=>Promise<APIRequestContext>}},info:TestInfo):Promise<APIRequestContext>{
 const api=await playwright.request.newContext({baseURL:String(info.project.use.baseURL)});
 await new Promise(r=>setTimeout(r,3100));
 expect((await api.post(`${ADMIN}/login`,{data:{username:'root',password:'hunter2'}})).status()).toBe(200);
 return api;
}
async function resetAdmin(api:APIRequestContext):Promise<void>{
 const etag=(await api.get(`${ADMIN}/settings/${KEY}`)).headers()['etag'];
 const res=await api.delete(`${ADMIN}/settings/${KEY}`,{headers:etag?{'if-match':etag}:{}});
 expect([200,204,404]).toContain(res.status());
}

for(const width of [1440,390]){
 test.describe(`badge accent shades at ${width}px`,()=>{
  test('lighter and darker follow the accent and the theme live, in light and dark',async({page,playwright},info)=>{
   skipPhone(info);
   await page.setViewportSize({width,height:900});
   const api=await adminApi(playwright,info);
   try{
    await resetAdmin(api);
    await signIn(page);
    await expect(page.locator('.list__row').first()).toBeVisible({timeout:60000});
    await openAppearance(page);
    const group=page.getByRole('group',{name:'Unread badge colour',exact:true});
    const accents=page.getByRole('group',{name:'Accent',exact:true});
    for(const theme of ['Light','Dark'] as const){
     await page.getByRole('button',{name:theme,exact:true}).first().click();
     await expect.poll(()=>page.evaluate(()=>document.documentElement.getAttribute('data-appearance'))).toBe(theme.toLowerCase());
     await accents.getByRole('button',{name:'Blue',exact:true}).click();
     const t=theme.toLowerCase();
     // Swatches show the colour the badge will take.
     const swatchBg=(name:string)=>group.getByRole('button',{name,exact:true}).evaluate(el=>getComputedStyle(el).backgroundColor);
     await group.getByRole('button',{name:'Lighter accent',exact:true}).click();
     const lighter=await expectShade(page,'lighter',`${t} blue lighter`);
     await expect.poll(()=>swatchBg('Lighter accent')).toBe(lighter.badge);
     await group.screenshot({path:shot(info,`${t}-picker-lighter-${width}`)});
     await group.getByRole('button',{name:'Darker accent',exact:true}).click();
     const darker=await expectShade(page,'darker',`${t} blue darker`);
     await expect.poll(()=>swatchBg('Darker accent')).toBe(darker.badge);
     expect(darker.badge).not.toBe(lighter.badge);
     // Change the accent: the badge follows without a reload.
     const nav=await page.evaluate(()=>performance.getEntriesByType('navigation').length+':'+String(performance.timeOrigin));
     await accents.getByRole('button',{name:'Magenta',exact:true}).click();
     const after=await expectShade(page,'darker',`${t} magenta darker`);
     expect(after.accent).not.toBe(darker.accent);
     expect(after.badge).not.toBe(darker.badge);
     await expect.poll(()=>swatchBg('Darker accent')).toBe(after.badge);
     expect(await page.evaluate(()=>performance.getEntriesByType('navigation').length+':'+String(performance.timeOrigin))).toBe(nav);
     await group.screenshot({path:shot(info,`${t}-picker-darker-magenta-${width}`)});
     await group.getByRole('button',{name:'Lighter accent',exact:true}).click();
     await expectShade(page,'lighter',`${t} magenta lighter`);
     await closeSettings(page);
     if(width<800)await drawer(page);
     const real=await measure(page);expect(real.real,'a folder badge is on screen').toBe(true);
     await expectShade(page,'lighter',`${t} magenta lighter (sidebar)`);
     await page.screenshot({path:shot(info,`${t}-sidebar-lighter-${width}`)});
     await page.keyboard.press('Escape');
     if(width<800){const menu=page.locator('header.shell__bar button[aria-controls="shell-nav"]').first();if((await menu.getAttribute('aria-expanded'))==='true')await menu.click();await expect(menu).toHaveAttribute('aria-expanded','false');await page.waitForTimeout(400);}
     await openAppearance(page);
    }
    // Theme switch alone repaints it (dark -> light, same option, same accent).
    const before=(await measure(page)).badge.bg;
    await page.getByRole('button',{name:'Light',exact:true}).first().click();
    const light=await expectShade(page,'lighter','theme switch to light');
    expect(light.badge).not.toBe(before);
    // Persisted: a reload keeps the option.
    await page.reload();await expect(page.getByRole('button',{name:'Compose',exact:true}).first()).toBeVisible({timeout:60000});
    await expectShade(page,'lighter','after reload');
    // Back to the default for the next test.
    await openAppearance(page);
    await page.getByRole('group',{name:'Unread badge colour',exact:true}).getByRole('button').first().click();
    await expect.poll(()=>page.evaluate(()=>document.documentElement.getAttribute('data-badge'))).toBe(null);
    expect(await page.evaluate(()=>document.documentElement.style.getPropertyValue('--mw-badge-derived'))).toBe('');
    await accents.getByRole('button',{name:'Theme default',exact:true}).click().catch(()=>{});
   }finally{await api.dispose();}
  });

  test('an administrator lock holds every user to Darker accent',async({page,browser,playwright},info)=>{
   skipPhone(info);
   await page.setViewportSize({width,height:900});
   const api=await adminApi(playwright,info);
   try{
    const etag=(await api.get(`${ADMIN}/settings/${KEY}`)).headers()['etag'];
    const put=await api.put(`${ADMIN}/settings/${KEY}`,{data:{globalValue:'accent-darker',locked:true},headers:etag?{'if-match':etag}:{}});
    expect(put.status(),await put.text()).toBe(200);
    // The registry refuses a value outside the enum.
    const etag2=(await api.get(`${ADMIN}/settings/${KEY}`)).headers()['etag'];
    const bad=await api.put(`${ADMIN}/settings/${KEY}`,{data:{globalValue:'accent-brighter',locked:true},headers:etag2?{'if-match':etag2}:{}});
    expect(bad.status()).toBeGreaterThanOrEqual(400);
    await signIn(page);
    await expect(page.locator('.list__row').first()).toBeVisible({timeout:60000});
    await openAppearance(page);
    const group=page.getByRole('group',{name:'Unread badge colour',exact:true});
    await expect(group.getByRole('button',{name:'Darker accent',exact:true})).toHaveAttribute('aria-pressed','true');
    for(const b of await group.getByRole('button').all())await expect(b).toBeDisabled();
    await expect(group).toHaveAttribute('aria-describedby','settings-badge-color-lock');
    await expectShade(page,'darker','admin lock');
    expect((await (await page.request.get('/api/settings')).json() as {settings:Record<string,{value:unknown;locked?:boolean}>}).settings[KEY]).toMatchObject({value:'accent-darker',locked:true});
    await page.locator('[data-settings-section="appearance"]').screenshot({path:shot(info,`locked-settings-${width}`)}).catch(()=>page.screenshot({path:shot(info,`locked-settings-${width}`)}));
    await closeSettings(page);if(width<800)await drawer(page);
    await expectShade(page,'darker','admin lock (sidebar)');
    await page.screenshot({path:shot(info,`locked-sidebar-${width}`)});
    // The console lists both new values for the setting.
    const context=await browser.newContext({baseURL:String(info.project.use.baseURL),viewport:{width,height:900},serviceWorkers:'block'});
    try{
     const admin=await context.newPage();
     await admin.goto(ADMIN);await admin.getByLabel('Admin username').fill('root');await admin.getByLabel('Password',{exact:true}).fill('hunter2');await admin.waitForTimeout(3100);
     await admin.getByRole('button',{name:'Sign in',exact:true}).click();
     await admin.goto(`${ADMIN}#/settings/${KEY}`);
     const detail=admin.getByTestId('setting-detail');await expect(detail).toBeVisible({timeout:30000});
     const form=detail.locator('form[aria-labelledby="setting-global-heading"]');
     const select=form.locator('select').first();
     await expect(select).toHaveValue('accent-darker');
     const options=await select.locator('option').evaluateAll(els=>els.map(e=>(e as HTMLOptionElement).value));
     expect(options).toEqual(expect.arrayContaining(['accent-lighter','accent-darker']));
     await expect(form.getByRole('switch')).toHaveAttribute('aria-checked','true');
     await admin.screenshot({path:shot(info,`admin-locked-${width}`),fullPage:true});
    }finally{await context.close();}
   }finally{await resetAdmin(api).catch(()=>{});await api.dispose();}
  });
 });
}
