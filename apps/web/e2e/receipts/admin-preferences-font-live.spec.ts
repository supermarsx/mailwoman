// t101: the admin console's "My preferences" page against a live server built
// from the tree. Each Interface font option changes the computed font-family of
// the console (frame, main pane, sidebar) at once and again after a reload, the
// bundled faces load under the console CSP, and the other appearance choices on
// the page (density, theme, accent) apply too. No CSP violation is logged.
// Runs at 1440 and 390 wide in the chromium and firefox projects. Screenshots go
// to MW_T101_SHOTS.
import {test,expect} from '../csrf.ts';
import {ADMIN} from '../admin/admin-path.ts';
import {adminLogin} from '../admin/admin-login.ts';
import type {Page,TestInfo,ConsoleMessage} from '@playwright/test';
import path from 'node:path';

test.describe.configure({timeout:300000});
const shot=(info:TestInfo,name:string)=>{const dir=process.env['MW_T101_SHOTS'];return dir?path.join(dir,`${info.project.name}-${name}.png`):info.outputPath(`${name}.png`);};

// The option value, and the first family the computed font-family must start with.
const FONTS:[string,string][]=[
 ['mono','"JetBrains Mono Variable"'],['serif','"Newsreader Variable"'],['system','system-ui'],
 ['inter','"Inter Variable"'],['roboto','"Roboto Variable"'],['open-sans','"Open Sans Variable"'],['jost','"Jost"'],['default','"Jost"'],
];
const firstFamily=(css:string)=>css.split(',')[0]!.trim().replace(/["']/g,'');

async function fonts(page:Page){
 return page.evaluate(()=>{
  const ff=(sel:string)=>{const el=document.querySelector(sel);return el?getComputedStyle(el).fontFamily:'';};
  return {main:ff('#admin-main'),heading:ff('[data-testid="admin-personal-preferences"] h2'),select:ff('[data-testid="admin-personal-preferences"] select'),nav:ff('nav a, nav button')};
 });
}
async function open(page:Page):Promise<void>{
 await page.goto(`${ADMIN}#/preferences`);
 await expect(page.getByTestId('admin-personal-preferences')).toBeVisible({timeout:60000});
 await expect(page.getByText('Preferences saved to your account.')).toBeVisible();
}
async function saved(page:Page,act:()=>Promise<unknown>):Promise<void>{
 const put=page.waitForResponse(r=>r.url().endsWith('/me/preferences')&&r.request().method()==='PUT');
 await act();const res=await put;expect(res.status(),await res.text()).toBeLessThan(300);
 await expect(page.getByText('Preferences saved to your account.')).toBeVisible();
}

test('admin My preferences: font, density, theme and accent apply live and after reload',async({page},info)=>{
 test.skip(info.project.name==='phone','the 390 legs run in the chromium and firefox projects');
 const csp:string[]=[];
 page.on('console',(m:ConsoleMessage)=>{if(/content[- ]security[- ]policy|font-src|refused to load/i.test(m.text()))csp.push(`${m.type()}: ${m.text()}`);});
 await page.setViewportSize({width:1440,height:900});
 await adminLogin(page);
 for(const width of [1440,390]){
  await page.setViewportSize({width,height:width>800?900:844});
  await open(page);
  const font=page.getByRole('combobox',{name:'Interface font',exact:true});
  await page.screenshot({path:shot(info,`${width}-before`),fullPage:false});
  for(const [value,family] of FONTS){
   await saved(page,()=>font.selectOption(value));
   if(value==='mono'||value==='serif')await page.screenshot({path:shot(info,`${width}-font-${value}`),fullPage:false});
   const now=await fonts(page);
   for(const [where,css] of Object.entries(now))if(css)expect(firstFamily(css),`${value} ${where} live`).toBe(family.replace(/"/g,''));
   if(family.startsWith('"')){
    const name=family.slice(1,-1);
    const loaded=await page.evaluate(async(n)=>{await document.fonts.load(`16px "${n}"`);return document.fonts.check(`16px "${n}"`)&&[...document.fonts].some(f=>f.family.replace(/"/g,'')===n&&f.status==='loaded');},name);
    expect(loaded,`${name} face loaded`).toBe(true);
   }
  }
  // Survives a reload.
  await saved(page,()=>font.selectOption('serif'));
  await page.reload();await expect(page.getByTestId('admin-personal-preferences')).toBeVisible({timeout:60000});
  await expect(page.getByRole('combobox',{name:'Interface font',exact:true})).toHaveValue('serif');
  await expect.poll(async()=>firstFamily((await fonts(page)).main)).toBe('Newsreader Variable');
  await page.screenshot({path:shot(info,`${width}-after-reload-serif`),fullPage:false});

  // Density: the root attribute, and the block padding of the console's data tables.
  for(const [value,px] of [['compact','4px'],['relaxed','12px'],['cozy','8px']] as const){
   await saved(page,()=>page.getByRole('combobox',{name:'Interface density',exact:true}).selectOption(value));
   await expect(page.locator('html')).toHaveAttribute('data-density',value);
   await page.goto(`${ADMIN}#/logs`);
   const th=page.locator('#admin-main table th').first();await expect(th).toBeVisible({timeout:60000});
   await expect.poll(()=>th.evaluate(el=>getComputedStyle(el).paddingTop),{message:`density ${value}`}).toBe(px);
   if(value!=='cozy')await page.screenshot({path:shot(info,`${width}-density-${value}-logs`),fullPage:false});
   await open(page);
  }
  // Theme: a light pack and back to dark, painted on the frame.
  const bg=()=>page.evaluate(()=>getComputedStyle(document.body).backgroundColor);
  const before=await bg();
  await saved(page,()=>page.locator('[data-theme-option="paper"]').click());
  await expect(page.locator('html')).toHaveAttribute('data-theme','paper');
  expect(await bg()).not.toBe(before);
  // Accent: a named accent sets data-accent; "theme default" clears it.
  const accent=page.locator('[data-accent-option]').nth(2);const accentId=await accent.getAttribute('data-accent-option');
  await saved(page,()=>accent.click());
  await expect(page.locator('html')).toHaveAttribute('data-accent',accentId!);
  await page.reload();await expect(page.getByTestId('admin-personal-preferences')).toBeVisible({timeout:60000});
  await expect(page.locator('html')).toHaveAttribute('data-theme','paper');
  await expect(page.locator('html')).toHaveAttribute('data-accent',accentId!);
  await expect.poll(async()=>firstFamily((await fonts(page)).main)).toBe('Newsreader Variable');
  await page.screenshot({path:shot(info,`${width}-after-reload-paper-accent`),fullPage:false});
  // Put back what the next leg and spec expect.
  await saved(page,()=>page.getByRole('button',{name:'Reset appearance',exact:true}).click());
  await saved(page,()=>page.getByRole('combobox',{name:'Interface font',exact:true}).selectOption('default'));
 }
 expect(csp,'CSP console messages').toEqual([]);
});
