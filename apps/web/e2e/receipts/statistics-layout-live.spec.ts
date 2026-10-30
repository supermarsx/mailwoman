import {test,expect,withCsrf} from '../csrf.ts';
import type {Browser,Page,TestInfo} from '@playwright/test';
import {ADMIN} from '../admin/admin-path.ts';

// t106: Statistics and the Dashboard's traffic panel lay figures, charts and
// tables out in separate sections, and one period control updates all of them
// in place. Live against a server built from the tree (engine mode over
// Greenmail) with real traffic sent to it:
//   node apps/web/e2e/receipts/linked-live-fixture.mjs
//   (send some requests to the server for a few minutes)
//   MW_RECEIPTS_BASE_URL=... npx playwright test -c apps/web/e2e/receipts/playwright.config.ts statistics-layout-live --project chromium --project firefox
// Each browser runs 1440 and 390 px wide, light and dark. MW_SCREENSHOT_DIR keeps
// a copy of every screenshot.

const ADMIN_PASSWORD=process.env['MW_FIXTURE_ADMIN_PASSWORD']??process.env['MW_ADMIN_PASSWORD']??'linked-fixture-admin';

async function signIn(browser:Browser,info:TestInfo,width:number,scheme:'light'|'dark'):Promise<{page:Page;errors:string[]}>{
  const context=await browser.newContext({colorScheme:scheme,locale:'en-US',timezoneId:'UTC',viewport:{width,height:900},serviceWorkers:'block',...(info.project.use.baseURL?{baseURL:info.project.use.baseURL}:{})});
  const page=await context.newPage();
  withCsrf(context.request);
  page.setDefaultTimeout(20000);
  await page.emulateMedia({colorScheme:scheme});
  const errors:string[]=[];
  page.on('pageerror',e=>errors.push(`pageerror: ${e.message}`));
  page.on('console',m=>{if(m.type()==='error')errors.push(`console: ${m.text()} (${m.location().url})`);});
  await page.goto(ADMIN);
  await page.getByLabel('Admin username').fill('root');
  await page.getByLabel('Password',{exact:true}).fill(ADMIN_PASSWORD);
  await page.waitForTimeout(3100);
  const login=page.waitForResponse(r=>r.url().endsWith(`${ADMIN}/login`)&&r.request().method()==='POST');
  await page.getByRole('button',{name:'Sign in',exact:true}).click();
  const result=await login;expect(result.status(),await result.text()).toBe(200);
  await expect(page.locator('#admin-main')).toBeVisible();
  // Let the initial dashboard's lazy catalog settle before navigating away;
  // Firefox reports a cancelled dynamic import as a page error otherwise.
  await expect(page.getByTestId('request-history').getByLabel('Period',{exact:true})).toBeVisible();
  // The signed-out bootstrap intentionally gets 401 from /admin/me. These
  // layout checks observe errors in the authenticated statistics page.
  errors.length=0;
  // The console keeps its own appearance per admin (Dark unless chosen): pick this leg's pack.
  const prefs=await page.request.get(`${ADMIN}/me/preferences`);
  expect(prefs.status()).toBe(200);
  const put=await page.request.put(`${ADMIN}/me/preferences`,{headers:{'if-match':prefs.headers()['etag']??'*'},data:{...(await prefs.json() as object),appearance:{theme:scheme,accent:'',mode:scheme}}});
  expect(put.status(),await put.text()).toBe(200);
  return {page,errors};
}

async function shot(page:Page,info:TestInfo,name:string):Promise<void>{
  const size=page.viewportSize()!;
  const height=await page.evaluate(()=>{const m=document.querySelector('#admin-main');return Math.max(document.documentElement.scrollHeight,(m?.scrollHeight??0)+200);});
  await page.setViewportSize({width:size.width,height:Math.min(12000,height)});
  await page.screenshot({path:info.outputPath(name),fullPage:true});
  const dir=process.env['MW_SCREENSHOT_DIR'];
  if(dir)await page.screenshot({path:`${dir}/after-${info.project.name}-${name}`,fullPage:true});
  await page.setViewportSize(size);
}

/** Nothing wider than the viewport: the page, the console's scroll region and every card. */
async function noOverflow(page:Page):Promise<void>{
  const found=await page.evaluate(()=>{
    const out:string[]=[];
    const vw=document.documentElement.clientWidth;
    if(document.documentElement.scrollWidth>vw+1)out.push(`document ${document.documentElement.scrollWidth}>${vw}`);
    const main=document.querySelector('#admin-main');
    if(main&&main.scrollWidth>main.clientWidth+1)out.push(`#admin-main ${main.scrollWidth}>${main.clientWidth}`);
    for(const el of document.querySelectorAll('[data-chart-card],[data-table-card],figure')){
      const r=el.getBoundingClientRect();
      if(r.right>vw+1||r.left<-1)out.push(`${el.getAttribute('data-testid')??el.tagName} ${Math.round(r.left)}..${Math.round(r.right)} of ${vw}`);
    }
    return out;
  });
  expect(found).toEqual([]);
}

/** The sections inside `root` come in the order tiles, charts, tables, and do not mix. */
async function separated(page:Page,root:string):Promise<void>{
  const layout=await page.evaluate(sel=>{
    const scope=document.querySelector(sel)!;
    return [...scope.querySelectorAll('[data-stats-section]')].map(s=>({
      kind:s.getAttribute('data-stats-section'),
      top:s.getBoundingClientRect().top,
      cards:s.querySelectorAll('[data-chart-card]').length,
      figures:s.querySelectorAll('figure').length,
    }));
  },root);
  expect(layout.map(s=>s.kind)).toEqual(['tiles','charts','tables']);
  const [tiles,charts,tables]=layout as [typeof layout[0],typeof layout[0],typeof layout[0]];
  expect(tiles.top).toBeLessThan(charts.top);
  expect(charts.top).toBeLessThan(tables.top);
  expect(tiles.cards,'no chart in the tiles section').toBe(0);
  expect(tiles.figures,'tiles in the tiles section').toBeGreaterThanOrEqual(4);
  expect(charts.figures,'no tile in the charts section').toBe(0);
  expect(charts.cards,'chart cards in the charts section').toBeGreaterThanOrEqual(4);
}

/** Picking a period reloads every figure, chart and table in place: the same nodes stay. */
async function periodUpdatesInPlace(page:Page,scope:string):Promise<void>{
  const root=page.locator(scope);
  const card=root.getByTestId('spark-requests');
  await expect(card).toBeVisible();
  await card.evaluate(el=>el.setAttribute('data-t106-probe','kept'));
  const before=await root.getByText(/seconds per bucket/).textContent();
  const history=page.waitForResponse(r=>r.url().includes('/metrics/history')&&r.url().includes('from='));
  const paths=page.waitForResponse(r=>r.url().includes('/metrics/not-found'));
  await root.getByLabel('Period',{exact:true}).selectOption('21600');
  const url=new URL((await history).url());
  expect(Number(url.searchParams.get('to'))-Number(url.searchParams.get('from'))).toBe(21600);
  await paths;
  await expect(root.locator('[aria-busy="true"]')).toHaveCount(0);
  await expect(root.getByText(/seconds per bucket/)).not.toHaveText(before!);
  // In place: the card that was there is still the same element, no page skeleton.
  await expect(root.locator('[data-t106-probe="kept"]')).toHaveCount(1);
  await expect(page.locator('[data-skeleton]')).toHaveCount(0);
  await expect(root.getByTestId('tile-requests')).toBeVisible();
}

for(const width of [1440,390])for(const scheme of ['light','dark'] as const){
  test(`statistics: tiles, charts and tables apart; one period control updates all in place (${width} ${scheme})`,async({browser},info)=>{
    test.skip(info.project.name==='phone','each browser runs 1440 and 390 itself');
    const {page,errors}=await signIn(browser,info,width,scheme);
    await page.goto(`${ADMIN}#/observability/statistics`);
    const stats=page.getByTestId('statistics');
    await expect(stats.getByTestId('spark-requests')).toBeVisible();
    await expect(stats.getByTestId('not-found-paths')).toBeVisible();
    await expect(stats).toHaveAttribute('aria-busy','false');
    await expect(stats.getByLabel('Period',{exact:true})).toHaveCount(1);
    await separated(page,'[data-testid="statistics"]');
    // Measured traffic: the request total is a number, not "Not available".
    await expect(stats.getByTestId('tile-requests')).not.toContainText('Not available');
    // Each chart is a named region with a summary and a table view.
    const latency=stats.getByRole('region',{name:'Response time',exact:true});
    await expect(latency.getByRole('list',{name:'Legend'})).toContainText('Median');
    await stats.getByTestId('spark-requests').getByRole('button',{name:'View as table'}).click();
    await expect(stats.getByTestId('spark-requests').locator('[data-chart-table="shown"] tbody tr').first()).toBeVisible();
    await stats.getByTestId('spark-requests').getByRole('button',{name:'View as table'}).click();
    await noOverflow(page);
    await shot(page,info,`statistics-${width}-${scheme}.png`);
    await periodUpdatesInPlace(page,'[data-testid="statistics"]');
    await noOverflow(page);
    expect(errors).toEqual([]);
    await page.context().close();
  });

  test(`dashboard: the traffic panel keeps tiles, charts and tables apart and updates in place (${width} ${scheme})`,async({browser},info)=>{
    test.skip(info.project.name==='phone','each browser runs 1440 and 390 itself');
    const {page,errors}=await signIn(browser,info,width,scheme);
    await page.goto(ADMIN);
    const panel=page.getByTestId('request-history');
    await expect(panel.getByTestId('spark-requests')).toBeVisible();
    await expect(panel).toHaveAttribute('aria-busy','false');
    await separated(page,'[data-testid="request-history"]');
    // The rest of the Dashboard keeps its figures and links.
    await expect(page.getByTestId('tile-users')).toBeVisible();
    await noOverflow(page);
    await shot(page,info,`dashboard-${width}-${scheme}.png`);
    await periodUpdatesInPlace(page,'[data-testid="request-history"]');
    await noOverflow(page);
    expect(errors).toEqual([]);
    await page.context().close();
  });
}
