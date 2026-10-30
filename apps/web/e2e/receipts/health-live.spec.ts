import {test,expect} from '../csrf.ts';
import type {Page} from '@playwright/test';
import {ADMIN} from '../admin/admin-path.ts';

// t103: the admin Health panel gathers real numbers. Live against a server built
// from the tree, in engine mode over Greenmail, with the mw-render worker beside
// the binary. Stack: node apps/web/e2e/receipts/linked-live-fixture.mjs, then
//   MW_RECEIPTS_BASE_URL=... npx playwright test -c apps/web/e2e/receipts/playwright.config.ts health-live
// Every component must show measured numbers or a disabled reason with how to
// enable it, never "the server did not report it". MW_SCREENSHOT_DIR keeps a
// copy of every screenshot.

const COMPONENTS=['database','migrations','sync','outbox','search','render','plugins'] as const;
const ADMIN_PASSWORD=process.env['MW_FIXTURE_ADMIN_PASSWORD']??'linked-fixture-admin';

type Component={status:string;reason:string;enable?:string;metrics:Record<string,unknown>};
type Health={checkedAt:number;components:Record<string,Component>;history:unknown[]};

async function shot(page:Page,info:{outputPath(name:string):string;project:{name:string}},name:string){
  // The console scrolls inside its main region: bring the panel to the top and
  // make the viewport tall enough to hold all of it.
  const panel=page.getByTestId('dash-health');
  const size=page.viewportSize()!;
  const height=Math.ceil((await panel.boundingBox())!.height)+160;
  await page.setViewportSize({width:size.width,height:Math.max(size.height,height)});
  await panel.evaluate(el=>el.scrollIntoView({block:'start'}));
  await page.screenshot({path:info.outputPath(name)});
  const dir=process.env['MW_SCREENSHOT_DIR'];
  if(dir)await page.screenshot({path:`${dir}/${info.project.name}-${name}`});
  await page.setViewportSize(size);
}

async function adminSignIn(page:Page){
  await page.goto(ADMIN);
  await page.getByLabel('Admin username').fill('root');
  await page.getByLabel('Password',{exact:true}).fill(ADMIN_PASSWORD);
  await page.waitForTimeout(3100);
  const login=page.waitForResponse(r=>r.url().endsWith(`${ADMIN}/login`)&&r.request().method()==='POST');
  await page.getByRole('button',{name:'Sign in',exact:true}).click();
  const result=await login;expect(result.status(),await result.text()).toBe(200);
  await expect(page.locator('#admin-main')).toBeVisible();
}

for(const scheme of ['light','dark'] as const){
  test(`health panel shows measured numbers or a precise disabled reason for every component (${scheme})`,async({page,browser},info)=>{
    page.setDefaultTimeout(20000);
    // A signed-in mailbox registers in the engine, so mail sync has a row to report.
    await page.goto('/');
    await page.getByLabel('Email or username',{exact:true}).fill('parent@example.org');
    await page.getByLabel('Password',{exact:true}).fill('parent-pass');
    await page.getByRole('button',{name:'Sign in',exact:true}).click();
    await expect(page.getByLabel('Email or username',{exact:true})).toHaveCount(0);

    const context=await browser.newContext({colorScheme:scheme,locale:'en-US',viewport:page.viewportSize(),...(info.project.use.baseURL?{baseURL:info.project.use.baseURL}:{})});
    const admin=await context.newPage();
    admin.setDefaultTimeout(20000);
    const errors:string[]=[];admin.on('pageerror',e=>errors.push(e.message));
    try{
      await adminSignIn(admin);

      // The API: a status from the four for each, numbers or a reason to enable.
      let health:Health|undefined;
      await expect.poll(async()=>{
        const r=await admin.request.get(`${ADMIN}/dashboard/health?fresh=1`);
        expect(r.status(),await r.text()).toBe(200);
        health=await r.json() as Health;
        return Number(health.components['sync']?.metrics['accounts']??0);
      },{timeout:60000,message:'the signed-in mailbox registers'}).toBeGreaterThanOrEqual(1);
      const h=health!;
      for(const id of COMPONENTS){
        const c=h.components[id]!;
        expect(['ok','warn','fail','disabled'],`${id}`).toContain(c.status);
        expect(c.reason,`${id}`).not.toMatch(/did not report/i);
        if(c.status==='disabled')expect(c.enable,`${id} says how to enable it`).toBeTruthy();
        else expect(Object.keys(c.metrics).length,`${id} has numbers`).toBeGreaterThan(0);
      }
      expect(h.components['database']!.metrics['pingMs']).toEqual(expect.any(Number));
      expect(h.components['migrations']!.metrics['pending']).toEqual([]);
      expect(h.components['search']!.metrics['present']).toBe(true);
      expect(h.components['search']!.metrics['documents']).toEqual(expect.any(Number));
      expect(h.components['outbox']!.metrics['queued']).toEqual(expect.any(Number));
      const render=h.components['render']!;
      expect(render.metrics['workerConfigured'],render.reason).toBe(true);
      expect(render.status,render.reason).toBe('ok');
      expect(render.metrics['version']).toEqual(expect.any(String));
      expect(Number(render.metrics['pingMs'])).toBeGreaterThan(0);
      // No plugin is enabled on the fixture: disabled, with how to enable it.
      expect(h.components['plugins']!.status).toBe('disabled');

      // The panel.
      await admin.goto(`${ADMIN}#/dashboard`);
      const panel=admin.getByTestId('dash-health');
      await expect(panel).toBeVisible();
      await expect(panel).not.toContainText('did not report');
      for(const id of COMPONENTS){
        const row=admin.getByTestId(`health-${id}`);
        await expect(row.getByTestId(`health-${id}-status`)).toHaveText(/OK|Warning|Failing|Disabled/);
        const status=await row.getAttribute('data-status');
        if(status==='disabled')await expect(row.getByTestId(`health-${id}-enable`)).toContainText('To turn it on');
        else await expect(row.getByTestId(`health-${id}-stats`)).toContainText(/\d/);
      }
      await expect(admin.getByTestId('health-database-stats')).toContainText(/μs|ms/);
      await expect(admin.getByTestId('health-sync-stats')).toContainText('registered');
      await expect(admin.getByTestId('health-render-stats')).toContainText('Worker version');

      // "Check again" reloads the panel in place: same document, a fresh probe.
      await admin.evaluate(()=>{(window as unknown as {t103:number}).t103=1;});
      const fresh=admin.waitForResponse(r=>r.url().includes(`${ADMIN}/dashboard/health?fresh=1`));
      await admin.getByTestId('dash-health-refresh').click();
      expect((await fresh).status()).toBe(200);
      expect(await admin.evaluate(()=>(window as unknown as {t103?:number}).t103)).toBe(1);
      await expect(admin.getByTestId('dash-health-refresh')).toBeEnabled();
      const WORD:Record<string,string>={ok:'OK',warn:'Warning',fail:'Failing',disabled:'Disabled'};
      for(const id of COMPONENTS){
        const row=admin.getByTestId(`health-${id}`);
        const status=(await row.getAttribute('data-status'))!;
        await expect(row.getByTestId(`health-${id}-status`),`${id} word follows its status`).toHaveText(WORD[status]!);
      }

      expect(await admin.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
      await shot(admin,info,`health-${scheme}.png`);
      expect(errors).toEqual([]);
    }finally{
      await context.close();
    }
  });
}
