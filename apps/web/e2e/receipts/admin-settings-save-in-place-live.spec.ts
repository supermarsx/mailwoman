import {test,expect} from '../csrf.ts';
import {mkdirSync} from 'node:fs';
import type {Browser,Page,TestInfo} from '@playwright/test';

// t75: Save on an admin setting's page keeps the page. After a write the page
// reloads the settings list, the setting and its domain/per-user rows; reading
// those while they reloaded suspended the console's page boundary, so for a few
// frames the whole Settings page was an empty box (the page skeleton) and came
// back as new nodes. Here every animation frame from the click on Save until
// the reloads have settled is sampled: the console's <main>, the Settings page,
// the setting's form and its Save button must stay the SAME DOM nodes, and no
// frame may be empty (no page, no form, or the page skeleton). The saved value
// are shown in place: the Save button says "Saving…" while the write is in
// flight, and the result ("Saved. …") is the console's toast.
// The settings list reload is held 400 ms (pass-through) so the in-between
// state is long enough to see. Stack: node scripts/e2e-receipts-stack.mjs.
// Chromium and Firefox; 1440 and 390 are set here. Shots go to MW_SCREENSHOT_DIR.
const KEY='mail.focusedInbox';
const SHOTS=process.env['MW_SCREENSHOT_DIR'];
const SIZES=[{name:'1440',width:1440,height:900},{name:'390',width:390,height:844}] as const;

const shot=(info:TestInfo,name:string)=>{
  if(!SHOTS)return info.outputPath(`${name}.png`);
  mkdirSync(SHOTS,{recursive:true});
  return `${SHOTS}/${info.project.name}-${name}.png`;
};

async function adminPage(browser:Browser,info:TestInfo,size:{width:number;height:number}):Promise<Page>{
  const context=await browser.newContext({baseURL:String(info.project.use.baseURL),viewport:size,serviceWorkers:'block',locale:'en-US'});
  const page=await context.newPage();
  await page.goto('/admin');
  await page.getByLabel('Admin username').fill('root');
  await page.getByLabel('Password',{exact:true}).fill('hunter2');
  await page.waitForTimeout(3100);
  const login=page.waitForResponse(r=>new URL(r.url()).pathname==='/admin/login'&&r.request().method()==='POST');
  await page.getByRole('button',{name:'Sign in',exact:true}).click();
  expect((await login).status()).toBe(200);
  await page.goto(`/admin#/settings/${KEY}`);
  await expect(page.getByTestId('setting-detail')).toBeVisible({timeout:30000});
  await expect(page.getByTestId('setting-detail').getByText(`Key: ${KEY}`,{exact:true})).toBeVisible();
  // The domain and per-user tables have answered (their first load is not under test).
  await expect(page.getByTestId('setting-detail').getByTestId('table-empty').or(page.getByTestId('setting-detail').locator('table')).first()).toBeVisible();
  return page;
}

/** Back to the shipped default (no stored value, unlocked), through the admin API. */
async function resetKey(playwright:{request:{newContext:(o:{baseURL:string})=>Promise<import('@playwright/test').APIRequestContext>}},info:TestInfo):Promise<void>{
  const api=await playwright.request.newContext({baseURL:String(info.project.use.baseURL)});
  try{
    expect((await api.post('/admin/login',{data:{username:'root',password:'hunter2'}})).status()).toBe(200);
    const etag=(await api.get(`/admin/settings/${KEY}`)).headers()['etag'];
    const res=await api.delete(`/admin/settings/${KEY}`,{headers:etag?{'if-match':etag}:{}});
    expect([200,204,404]).toContain(res.status());
  }finally{await api.dispose();}
}

interface Frame{t:number;main:boolean;page:boolean;form:boolean;save:boolean;skeleton:boolean;empty:boolean;saving:boolean;saved:boolean;checked:boolean|null;height:number}

/** Starts a rAF sampler that compares every frame with the nodes on screen now. */
async function startSampler(page:Page):Promise<void>{
  await page.evaluate(()=>{
    const q=(s:string)=>document.querySelector(s);
    const form=()=>q('[data-testid="setting-detail"] form[aria-labelledby="setting-global-heading"]');
    const refs={main:q('#admin-main'),page:q('[data-testid="admin-settings"]'),form:form(),save:form()?.querySelector('button[type="submit"]')??null};
    const frames:unknown[]=[];
    let on=true;
    const t0=performance.now();
    const tick=()=>{
      const f=form();
      const save=f?.querySelector('button[type="submit"]')??null;
      const pageNode=q('[data-testid="admin-settings"]');
      const text=(pageNode?.textContent??'').trim();
      const box=f?.querySelector('input[type="checkbox"]') as HTMLInputElement|null|undefined;
      frames.push({
        t:Math.round(performance.now()-t0),
        main:q('#admin-main')===refs.main&&refs.main!==null,
        page:pageNode===refs.page&&refs.page!==null&&pageNode!.isConnected,
        form:f===refs.form&&refs.form!==null,
        save:save===refs.save&&refs.save!==null,
        skeleton:q('[data-testid="admin-page-skeleton"]')!==null,
        empty:pageNode===null||f===null||text.length<40||(pageNode as HTMLElement).getBoundingClientRect().height<100,
        saving:/Saving…/.test(save?.textContent??''),
        // A success result is a toast in the console's toast region (Feedback kind success).
        saved:/Saved\./.test(q('[data-testid="toast-region"]')?.textContent??'')||/Saved\./.test(q('[data-testid="setting-detail"]')?.textContent??''),
        checked:box?box.checked:null,
        height:Math.round((pageNode as HTMLElement|null)?.getBoundingClientRect().height??0),
      });
      if(on)requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
    (window as unknown as {__t75:{stop:()=>unknown[]}}).__t75={stop:()=>{on=false;return frames;}};
  });
}
const stopSampler=(page:Page)=>page.evaluate(()=>(window as unknown as {__t75:{stop:()=>unknown[]}}).__t75.stop()) as Promise<Frame[]>;

test.describe.configure({timeout:240000});
test.beforeEach(({browserName},info)=>{void browserName;test.skip(info.project.name==='phone','the 390 legs run in the chromium and firefox projects');});

for(const size of SIZES){
  test(`Save keeps the Settings page mounted, with no empty frame, at ${size.name}`,async({browser,playwright},info)=>{
    await resetKey(playwright,info);
    const page=await adminPage(browser,info,size);
    try{
      // Hold every reload of the settings list (after a write) for 400 ms.
      await page.route(url=>url.pathname==='/admin/settings',async route=>{
        if(route.request().method()==='GET')await new Promise(r=>setTimeout(r,400));
        await route.continue();
      });
      const form=page.getByTestId('setting-detail').locator('form[aria-labelledby="setting-global-heading"]');
      const box=form.locator('input[type="checkbox"]').first();
      await expect(box).not.toBeChecked();
      await page.screenshot({path:shot(info,`${size.name}-before`)});
      for(const [round,value] of [[1,true],[2,false]] as const){
        await box.click();
        await startSampler(page);
        const put=page.waitForResponse(r=>new URL(r.url()).pathname===`/admin/settings/${KEY}`&&r.request().method()==='PUT');
        const reload=page.waitForResponse(r=>new URL(r.url()).pathname==='/admin/settings'&&r.request().method()==='GET');
        await form.getByRole('button',{name:'Save',exact:true}).click();
        expect((await put).status()).toBe(200);
        // In the middle of the held list reload.
        await page.waitForTimeout(150);
        await page.screenshot({path:shot(info,`${size.name}-round${round}-during`)});
        expect((await reload).status()).toBe(200);
        await expect(page.getByTestId('toast-region').getByText(/^Saved\./).or(form.getByText(/^Saved\./)).first()).toBeVisible();
        await page.waitForTimeout(500);
        const frames=await stopSampler(page);
        await page.screenshot({path:shot(info,`${size.name}-round${round}-after`)});
        const bad=frames.filter(f=>!f.main||!f.page||!f.form||!f.save||f.skeleton||f.empty);
        const summary=`${frames.length} frames; replaced/empty: ${bad.length}; first bad ${JSON.stringify(bad[0]??null)}`;
        console.log(`[t75] ${info.project.name} ${size.name} round ${round}: ${summary}; saving frames ${frames.filter(f=>f.saving).length}; saved frames ${frames.filter(f=>f.saved).length}; heights ${[...new Set(frames.map(f=>f.height))].join(',')}`);
        expect(frames.length,'frames sampled').toBeGreaterThan(20);
        expect(bad,summary).toEqual([]);
        // The saved value stays on screen in every frame, and the page says it saved.
        expect(frames.every(f=>f.checked===value),'saved value shown in every frame').toBe(true);
        expect(frames.some(f=>f.saved)).toBe(true);
        await expect(box).toBeChecked({checked:value});
        await expect(page.getByTestId('setting-detail').getByTestId('source-badge').first()).toHaveAttribute('data-source',value?'stored':/stored|default/);
      }
    }finally{
      await page.context().close();
      await resetKey(playwright,info);
    }
  });
}
