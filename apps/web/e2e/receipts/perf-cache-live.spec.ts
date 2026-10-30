import {test,expect} from '../csrf.ts';
import type {Page,Request} from '@playwright/test';

// t99: warm navigation reuses what the app already read. After the mailbox,
// Settings, Calendar and Contacts have each been opened once, going round them
// again must not ask the server again for the resources the client keeps
// (the JMAP session, the public config, the settings map, the lazy chunks), and
// each warm step must stay inside a time budget set from the t99 measurements
// (debug server on a seeded receipts stack; the budgets are about 3x what was
// measured, so a slow CI runner still passes and a regression to "refetch
// everything" does not).
// Stack: node scripts/e2e-receipts-stack.mjs. Chromium and Firefox at 1440.
const BUDGET={
  /** Reload of the mail list with a warm HTTP cache (measured 0.9 s). */
  mailReload:3000,
  /** Opening a surface already opened once (measured 0.05-0.11 s). */
  warmSurface:1000,
  /** Settings section already shown once (measured 0.06-0.25 s). */
  warmSection:1000,
};
/** Requests the client keeps between navigations (t99 sessionCache, settings, publicConfig). */
const KEPT=[/\/jmap\/session$/,/\/api\/public-config$/,/\/api\/settings$/];

function path(r:Request):string{return new URL(r.url()).pathname;}

function record(page:Page){
  const seen:Request[]=[];
  const on=(r:Request)=>{seen.push(r);};
  page.on('request',on);
  return {take(){const out=[...seen];seen.length=0;return out;},stop(){page.off('request',on);}};
}

async function signIn(page:Page){
  await page.goto('/');
  await page.getByLabel('Email or username',{exact:true}).fill('testuser');
  await page.getByLabel('Password',{exact:true}).fill('testpass');
  await page.getByRole('button',{name:'Sign in',exact:true}).click();
  await expect(page.locator('.list__scroll .list__row').first()).toBeVisible({timeout:60000});
}

const inbox=(page:Page)=>page.locator('#shell-nav .sidebar__nav').getByRole('button',{name:/^Inbox/}).first();
const sections=(page:Page)=>page.getByRole('navigation',{name:'Settings sections'});
const noPending=(page:Page)=>page.waitForFunction(()=>![...document.querySelectorAll('[data-testid="async-pending"]')].some(e=>e.getClientRects().length>0),null,{timeout:30000,polling:'raf'});

/** One round: Settings (two sections), Calendar, Contacts, back to the list. */
async function round(page:Page):Promise<Record<string,number>>{
  const times:Record<string,number>={};
  const time=async(name:string,act:()=>Promise<unknown>,ready:()=>Promise<unknown>)=>{const t0=Date.now();await act();await ready();times[name]=Date.now()-t0;};
  await time('settings',()=>page.locator('.sidebar__settings').first().click(),async()=>{await expect(sections(page)).toBeVisible();await noPending(page);});
  for(const name of ['Appearance','Notifications'])
    await time(`section ${name}`,()=>sections(page).getByRole('button',{name,exact:true}).click(),async()=>{await expect(sections(page).getByRole('button',{name,exact:true})).toHaveAttribute('aria-current','page');await noPending(page);});
  await inbox(page).click();
  await expect(page.locator('.list__scroll .list__row').first()).toBeVisible();
  await time('calendar',()=>page.getByTestId('nav-calendar').click(),()=>expect(page.getByRole('button',{name:'New event',exact:true})).toBeVisible());
  await time('contacts',()=>page.getByTestId('nav-contacts').click(),()=>expect(page.getByRole('searchbox',{name:'Search contacts'})).toBeVisible());
  await time('inbox',()=>inbox(page).click(),()=>expect(page.locator('.list__scroll .list__row').first()).toBeVisible());
  return times;
}

test.describe('t99 warm navigation',()=>{
  test.beforeEach(({},info)=>{test.skip(info.project.name==='phone','desktop layout; the phone drawer is covered elsewhere');});

  test('a second round over Settings, Calendar and Contacts refetches nothing the client keeps, inside the budgets',async({page},info)=>{
    test.setTimeout(240000);
    await page.setViewportSize({width:1440,height:900});
    await signIn(page);
    const log=record(page);
    const cold=await round(page);
    const coldRequests=log.take();
    // Let any background refresh started by the first round finish.
    await page.waitForTimeout(1500);
    log.take();
    const warm=await round(page);
    const warmRequests=log.take();
    log.stop();
    const kept=warmRequests.filter(r=>KEPT.some(re=>re.test(path(r))));
    const chunks=warmRequests.filter(r=>/\/assets\/.+\.js$/.test(path(r)));
    await info.attach('t99-perf',{contentType:'application/json',body:JSON.stringify({cold,warm,
      coldRequests:coldRequests.map(r=>`${r.method()} ${path(r)}`),warmRequests:warmRequests.map(r=>`${r.method()} ${path(r)}`)},null,1)});
    expect(kept.map(r=>path(r)),'kept resources were fetched again on a warm round').toEqual([]);
    expect(chunks.map(r=>path(r)),'lazy chunks were fetched again on a warm round').toEqual([]);
    for(const [name,ms] of Object.entries(warm))
      expect(ms,`${name} warm`).toBeLessThan(name.startsWith('section')?BUDGET.warmSection:BUDGET.warmSurface);
  });

  test('a reload asks /jmap/session at most three times and shows the list inside the budget',async({page},info)=>{
    test.setTimeout(180000);
    await page.setViewportSize({width:1440,height:900});
    await signIn(page);
    await page.waitForTimeout(3000);
    const log=record(page);
    const t0=Date.now();
    await page.reload();
    await expect(page.locator('.list__scroll .list__row').first()).toBeVisible({timeout:30000});
    const shown=Date.now()-t0;
    await page.waitForTimeout(2000);
    const requests=log.take();
    log.stop();
    const sessions=requests.filter(r=>/\/jmap\/session$/.test(path(r)));
    const publicConfig=requests.filter(r=>/\/api\/public-config$/.test(path(r)));
    await info.attach('t99-reload',{contentType:'application/json',body:JSON.stringify({shown,requests:requests.map(r=>`${r.method()} ${path(r)}`)},null,1)});
    // Before t99: 4-7 session reads and 2 public-config reads per load.
    expect(sessions.length).toBeLessThanOrEqual(3);
    expect(publicConfig.length).toBeLessThanOrEqual(1);
    expect(shown).toBeLessThan(BUDGET.mailReload);
  });
});
