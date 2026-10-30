import {test,expect,withCsrf} from '../csrf.ts';
import { ADMIN } from '../admin/admin-path.ts';
import {mkdirSync} from 'node:fs';
import type {Browser,Page,Response,TestInfo} from '@playwright/test';

// t90: times under 1 ms are shown in μs (or ns), never as "0 ms", on the admin
// screens that show timings: the dashboard health card (database ping), the
// statistics page (database query time tile, response-time charts) and the
// diagnostics checks table. Each value on screen is compared with the number
// the server sent for it. Stack: node scripts/e2e-receipts-stack.mjs.
// Chromium and Firefox at 1440. Shots go to MW_SCREENSHOT_DIR.
const SHOTS=process.env['MW_SCREENSHOT_DIR'];
const SIZE={width:1440,height:900};
// A zero shown in ms, seconds or μs ("0 ms", "0.0 ms", "0,0 s"); "<1 ms" and "10 ms" do not match.
const ZERO=/(^|[^\d.,<])0(?:[.,]0+)?\s?(?:ms|s|[μµ]s)\b/;
const MICRO=/\d\s?[μµ]s\b/;

test.beforeEach(({browserName},info)=>{void browserName;test.skip(info.project.name==='phone','desktop width only (1440)');});

const shot=(info:TestInfo,name:string)=>{
  if(!SHOTS)return info.outputPath(`${name}.png`);
  mkdirSync(SHOTS,{recursive:true});
  return `${SHOTS}/${info.project.name}-1440-${name}.png`;
};

const json=async<T,>(r:Promise<Response>):Promise<T>=>(await (await r).json()) as T;
const at=(path:string)=>(r:Response)=>new URL(r.url()).pathname===`${ADMIN}${path}`&&r.status()===200;

async function signIn(browser:Browser,info:TestInfo):Promise<Page>{
  const context=await browser.newContext({baseURL:String(info.project.use.baseURL),viewport:SIZE,serviceWorkers:'block',locale:'en-US'});
  withCsrf(context.request);
  const page=await context.newPage();
  await page.goto(ADMIN);
  await page.getByLabel('Admin username').fill('root');
  await page.getByLabel('Password',{exact:true}).fill('hunter2');
  await page.waitForTimeout(3100);
  const login=page.waitForResponse(r=>new URL(r.url()).pathname===`${ADMIN}/login`&&r.request().method()==='POST');
  await page.getByRole('button',{name:'Sign in',exact:true}).click();
  expect((await login).status()).toBe(200);
  return page;
}

/** The text the console shows for `ms`, checked against the rule for its size. */
function checkShown(text:string,ms:number,what:string):void{
  expect(text,what).not.toMatch(/NaN|undefined/);
  if(ms>0)expect(text,`${what}: ${ms} ms is not zero`).not.toMatch(ZERO);
  if(ms>=0.001&&ms<0.9995)expect(text,`${what}: ${ms} ms reads in μs`).toMatch(MICRO);
}

test('dashboard database ping reads in its own unit',async({browser},info)=>{
  const page=await signIn(browser,info);
  const dash=json<{health?:{components:{database:{metrics:{pingMs:number}}}}}>(page.waitForResponse(at('/dashboard')));
  await page.goto(`${ADMIN}#/dashboard`);
  const ping=(await dash).health?.components.database.metrics.pingMs;
  expect(typeof ping).toBe('number');
  const row=page.getByTestId('health-database-stats').locator('dd').first();
  await expect(row).toBeVisible({timeout:30000});
  const text=(await row.textContent())!;
  checkShown(text,ping!,'dashboard ping');
  await page.screenshot({path:shot(info,'dashboard'),fullPage:true});
  await page.context().close();
});

test('statistics: database tile and response-time charts',async({browser},info)=>{
  const page=await signIn(browser,info);
  // A few requests so the history has latency samples.
  for(let i=0;i<5;i++)await page.request.get(`${ADMIN}/dashboard`);
  const dash=json<{health?:{dbPingMs:number}}>(page.waitForResponse(at('/dashboard')));
  const history=json<{series:{p50Ms:number|null;p95Ms:number|null;p50ResolutionMs?:number|null}[]}>(page.waitForResponse(at('/metrics/history')));
  await page.goto(`${ADMIN}#/observability/statistics`);
  const ping=(await dash).health!.dbPingMs;
  const tile=page.locator('figure').filter({hasText:'Database query time'});
  await expect(tile).toBeVisible({timeout:30000});
  checkShown((await tile.textContent())!.replace('Database query time',''),ping,'database tile');

  const series=(await history).series;
  // t106: median and 95th percentile share one Response time card; column 2 is the median.
  const region=page.getByRole('region',{name:'Response time',exact:true});
  await expect(region).toBeVisible();
  const cells=await region.locator('table tbody tr td:nth-child(2)').allTextContents();
  expect(cells.length).toBe(series.length);
  const known=series.map(p=>p.p50Ms).filter((v):v is number=>typeof v==='number');
  for(const [i,p] of series.entries()){
    if(p.p50Ms===null)continue;
    expect(cells[i],`p50 bucket ${i}`).not.toMatch(/NaN/);
    if(p.p50Ms>0)expect(cells[i],`p50 bucket ${i} (${p.p50Ms} ms)`).not.toMatch(ZERO);
  }
  // One unit for the whole chart: every measured cell ends in the same unit.
  const units=new Set(cells.filter((c,i)=>series[i]!.p50Ms!==null&&!c.startsWith('<')).map(c=>c.replace(/^[-\d.,\s]+/,'')));
  expect(units.size,`units ${[...units].join(', ')}`).toBeLessThanOrEqual(1);
  if(known.length&&Math.max(...known)<0.9995&&Math.max(...known)>=0.001)expect([...units][0]).toMatch(/^[μµ]s$/);
  info.annotations.push({type:'p50',description:`${known.length} buckets, max ${Math.max(0,...known)} ms -> ${[...units].join(',')}; tile ${ping} ms`});
  await page.screenshot({path:shot(info,'statistics'),fullPage:true});
  await page.context().close();
});

test('diagnostics: each check time matches the server and sub-ms reads in μs',async({browser},info)=>{
  const page=await signIn(browser,info);
  // Before t105 the page runs the server suite from 'Run server checks' and shows
  // the table in place; after t105 a check card on #/diagnostics/server runs it
  // ('Run again' on 'Database connection') and the table is on #/diagnostics/history.
  await page.goto(`${ADMIN}#/diagnostics`);
  const legacy=page.getByRole('button',{name:'Run server checks'});
  const all=page.getByRole('button',{name:'Run all checks'});
  await expect(legacy.or(all).first()).toBeVisible({timeout:30000});
  const redesigned=!(await legacy.isVisible());
  const answer=json<{run:{results:{summary:string;status:string;durationMs:number;durationUs?:number}[]}}>(page.waitForResponse(r=>new URL(r.url()).pathname===`${ADMIN}/diag/run`&&r.request().method()==='POST'));
  if(redesigned){
    await page.goto(`${ADMIN}#/diagnostics/server`);
    const card=page.getByRole('row',{name:'Database connection',exact:true});
    await card.getByRole('button',{name:/^Run(?: again)?$/}).click();
  }else{
    await expect(legacy).toBeEnabled();
    await legacy.click();
  }
  const results=(await answer).run.results;
  expect(results.length).toBeGreaterThan(0);
  if(redesigned)await page.goto(`${ADMIN}#/diagnostics/history`);
  const table=page.locator('table').filter({has:page.getByText('Check results')}).filter({has:page.getByRole('columnheader',{name:'Duration'})});
  await expect(table.locator('tbody tr').first()).toBeVisible({timeout:30000});
  const rows=table.locator('tbody tr');
  expect(await rows.count()).toBe(results.length);
  let micro=0;
  for(const [i,r] of results.entries()){
    expect(typeof r.durationUs,`durationUs on ${r.summary}`).toBe('number');
    expect(r.durationMs).toBe(Math.floor(r.durationUs!/1000));
    const text=(await rows.nth(i).locator('td').nth(1).textContent())!;
    // A skipped check never ran: no time is shown rather than a measured zero.
    if(r.status==='skipped'){expect(text).toBe('—');continue;}
    checkShown(text,r.durationUs!/1000,`check "${r.summary}" (${r.durationUs} μs) shown "${text}"`);
    if(r.durationUs!>0&&r.durationUs!<1000)micro++;
  }
  info.annotations.push({type:'diagnostics',description:`${results.length} checks, ${micro} under 1 ms: ${results.map(r=>`${r.durationUs}us`).join(' ')}`});
  // The stack's local checks (database, scratch write) finish well under 1 ms.
  expect(micro,'at least one sub-ms check to look at').toBeGreaterThan(0);
  await expect(table).not.toContainText(/(^|[^\d.,<])0 ms/);
  await table.scrollIntoViewIfNeeded();
  await page.screenshot({path:shot(info,'diagnostics'),fullPage:true});
  await page.context().close();
});
