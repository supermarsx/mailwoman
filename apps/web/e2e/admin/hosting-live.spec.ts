import { readFileSync } from 'node:fs';
import { ADMIN } from './admin-path.ts';
import { createServer } from 'node:https';
import type { Duplex } from 'node:stream';
import { test, expect } from '../csrf.ts';
const certificate=readFileSync(new URL('../../../../crates/mw-server/tests/fixtures/tls/localhost-cert.pem',import.meta.url),'utf8');
const privateKey=readFileSync(new URL('../../../../crates/mw-server/tests/fixtures/tls/localhost-key.pem',import.meta.url),'utf8');
// The server runs in a container; it reaches this in-process panel on the runner host through
// host.docker.internal (docker-compose.dev.yml maps it to host-gateway on Linux). MW_E2E_FIXTURE_HOST overrides it.
const fixtureHost=process.env['MW_E2E_FIXTURE_HOST']??'host.docker.internal';

/** A HestiaCP `/api/` stand-in holding mail domains and accounts in memory: it answers the
 * positional `v-*` commands the server sends (hosting_operations.rs) with Hestia's JSON
 * listings, or with the numeric return code for a mutation. */
async function hestiaFixture(){
  type Account={quota:string;suspended:boolean;aliases:string[];password:string};
  const domains=new Map<string,{suspended:boolean;accounts:Map<string,Account>}>();const commands:string[]=[];const sockets=new Set<Duplex>();
  const server=createServer({cert:certificate,key:privateKey},(request,response)=>{
    const chunks:Buffer[]=[];request.on('data',chunk=>chunks.push(chunk as Buffer));request.on('end',()=>{
      const form=new URLSearchParams(Buffer.concat(chunks).toString('utf8'));const cmd=form.get('cmd')??'';const arg=(n:number)=>form.get(`arg${n}`)??'';
      const reply=(body:string)=>{response.writeHead(200,{'content-type':'application/json'});response.end(body);};
      if(request.method!=='POST'||request.url!=='/api/'||form.get('hash')!=='test-access:panel-fixture-secret'||arg(1)!=='fixture'){reply('10');return;}
      commands.push(cmd);const domain=domains.get(arg(2));const account=domain?.accounts.get(arg(3));const done=(ok:boolean)=>reply(ok?'0':'3');
      switch(cmd){
        case 'v-list-mail-domains':reply(JSON.stringify(Object.fromEntries([...domains].map(([name,row])=>[name,{SUSPENDED:row.suspended?'yes':'no',ACCOUNTS:String(row.accounts.size)}]))));return;
        case 'v-list-mail-accounts':reply(JSON.stringify(Object.fromEntries([...(domain?.accounts??[])].map(([name,row])=>[name,{QUOTA:row.quota,SUSPENDED:row.suspended?'yes':'no',ALIAS:row.aliases.join(','),MD5:'fixture-hash-never-shown'}]))));return;
        case 'v-add-mail-domain':if(domain||!arg(2)){done(false);return;}domains.set(arg(2),{suspended:false,accounts:new Map()});done(true);return;
        case 'v-delete-mail-domain':done(domains.delete(arg(2)));return;
        case 'v-suspend-mail-domain':case 'v-unsuspend-mail-domain':if(domain)domain.suspended=cmd==='v-suspend-mail-domain';done(Boolean(domain));return;
        case 'v-add-mail-account':if(!domain||account){done(false);return;}domain.accounts.set(arg(3),{password:arg(4),quota:arg(5)||'unlimited',suspended:false,aliases:[]});done(true);return;
        case 'v-change-mail-account-quota':if(account)account.quota=arg(4);done(Boolean(account));return;
        case 'v-change-mail-account-password':if(account)account.password=arg(4);done(Boolean(account));return;
        case 'v-suspend-mail-account':case 'v-unsuspend-mail-account':if(account)account.suspended=cmd==='v-suspend-mail-account';done(Boolean(account));return;
        case 'v-delete-mail-account':done(Boolean(domain?.accounts.delete(arg(3))));return;
        case 'v-add-mail-account-alias':if(account)account.aliases.push(arg(4));done(Boolean(account));return;
        case 'v-delete-mail-account-alias':if(account)account.aliases=account.aliases.filter(alias=>alias!==arg(4));done(Boolean(account));return;
        default:reply('1');
      }
    });
  });
  server.on('connection',socket=>{sockets.add(socket);socket.on('close',()=>sockets.delete(socket));});
  await new Promise<void>(done=>server.listen(0,'0.0.0.0',done));
  return {commands,origin:`https://${fixtureHost}:${(server.address() as {port:number}).port}`,close:async()=>{for(const socket of sockets)socket.destroy();await new Promise<void>(done=>server.close(()=>done()));}};
}

test('hosting pages configure a sealed panel and manage real HTTPS fixture inventory @phone',async({page})=>{
 const panel=await hestiaFixture();try{
 await page.goto(ADMIN);await page.getByLabel('Admin username').fill('root');await page.getByLabel('Password',{exact:true}).fill('hunter2');await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect(page.getByRole('button',{name:'Sign in',exact:true})).toHaveCount(0);
 await page.goto(`${ADMIN}#/integrations/hosting`);await page.getByRole('button',{name:'New hosting connection'}).click();
 const name=`Panel ${test.info().project.name} ${Date.now()}`;await page.getByLabel('Connection name').fill(name);await page.getByRole('combobox',{name:'Provider',exact:true}).selectOption('hestia');await page.getByLabel('HTTPS panel origin').fill(panel.origin);await page.getByLabel('Hestia account owner').fill('fixture');await page.getByLabel('Hestia access key').fill('test-access');await page.getByLabel('Enable this connection').check();
 await page.getByRole('combobox',{name:'Saved credential',exact:true}).selectOption('set');await page.getByLabel('Hestia secret key').fill('panel-fixture-secret');await page.getByLabel('Allow this configured destination on a private or loopback address').check();await page.getByLabel('TLS server name override').fill('localhost');await page.getByLabel('Additional trusted CA certificates (PEM)').fill(certificate);
 expect((await page.getByLabel('Additional trusted CA certificates (PEM)').boundingBox())!.width).toBeGreaterThan(250);expect((await page.getByLabel('Enable this connection').boundingBox())!.width).toBeLessThan(30);expect((await page.getByRole('button',{name:'Save connection'}).boundingBox())!.width).toBeLessThan(300);expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);await page.screenshot({path:test.info().outputPath('hosting-connection.png'),fullPage:true,animations:'disabled'});
 const created=page.waitForResponse(r=>r.url().endsWith('/integrations/hosting/connections')&&r.request().method()==='POST'&&r.ok());await page.getByRole('button',{name:'Save connection'}).click();const stepup=page.getByRole('dialog',{name:'Confirm it is you'});await stepup.getByRole('textbox',{name:'Password',exact:true}).fill('hunter2');await stepup.getByRole('button',{name:'Confirm and continue'}).click();const response=await created;const text=await response.text();expect(text).not.toContain('panel-fixture-secret');const saved=JSON.parse(text) as {id:string};await expect(page).toHaveURL(new RegExp(`/hosting/${saved.id}/edit$`));await page.reload();await expect(page.getByLabel('Connection name')).toHaveValue(name);await expect(page.getByLabel('Hestia secret key')).toHaveCount(0);
 await page.goto(`${ADMIN}#/integrations/hosting/${saved.id}/manage`);await page.getByRole('button',{name:'Create domain',exact:true}).click();const domain=`browser-${test.info().project.name}-${Date.now()}.example.org`;await page.getByLabel('Domain name',{exact:true}).fill(domain);await page.getByRole('button',{name:'Create domain',exact:true}).click();await page.getByRole('button',{name:domain,exact:true}).click();await page.getByRole('button',{name:'Create mailbox',exact:true}).click();await page.getByLabel('Panel mailbox account name').fill('alice');await page.getByLabel('New password').fill('one-time-mailbox-secret');await page.getByLabel('Quota (MiB)').fill('64');await page.getByRole('button',{name:'Create mailbox',exact:true}).click();await expect(page.getByRole('rowheader',{name:`alice@${domain} alice`,exact:true})).toBeVisible();await page.getByRole('button',{name:'Edit mailbox',exact:true}).click();await expect(page.getByLabel('Quota (MiB)')).toHaveValue('64');await page.getByLabel('Quota (MiB)').fill('128');await page.getByRole('button',{name:'Change mailbox quota',exact:true}).last().click();await expect(page.getByRole('cell',{name:'128',exact:true})).toBeVisible();expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);await page.screenshot({path:test.info().outputPath('hosting-mailboxes.png'),fullPage:true,animations:'disabled'});
 await page.getByRole('button',{name:'Edit mailbox',exact:true}).click();await page.getByRole('button',{name:'Reset mailbox password',exact:true}).click();await page.getByLabel('New password').fill('replaced-mailbox-secret');await page.getByRole('button',{name:'Reset mailbox password',exact:true}).last().click();await expect(page).toHaveURL(new RegExp(`/domains/${domain}$`));
 await page.getByRole('button',{name:'Edit mailbox',exact:true}).click();await page.getByRole('button',{name:'Add mailbox alias',exact:true}).click();await page.getByLabel('Same-domain alias local part').fill('sales');await page.getByRole('button',{name:'Add mailbox alias',exact:true}).last().click();await expect(page).toHaveURL(new RegExp(`/domains/${domain}$`));await page.getByRole('button',{name:'Edit mailbox',exact:true}).click();await expect(page.getByRole('button',{name:'Remove alias',exact:true})).toBeVisible();await page.getByRole('button',{name:'Remove alias',exact:true}).click();let change=page.getByRole('alertdialog');await change.getByRole('textbox').fill(`sales@${domain}`);await change.getByRole('button',{name:'Remove alias',exact:true}).click();await expect(change).toHaveCount(0);await expect(page.getByRole('button',{name:'Remove alias',exact:true})).toHaveCount(0);
 for(const label of ['Disable','Enable']){await page.getByRole('button',{name:label,exact:true}).click();change=page.getByRole('alertdialog');await change.getByRole('textbox').fill(`alice@${domain}`);await change.getByRole('button',{name:label,exact:true}).click();await expect(change).toHaveCount(0);}
 await page.getByRole('button',{name:'Back to panel inventory',exact:true}).click();
 await page.getByRole('button',{name:'Edit mailbox',exact:true}).click();await page.getByRole('button',{name:'Delete mailbox',exact:true}).click();const deletion=page.getByRole('alertdialog');await expect(deletion.getByRole('button',{name:'Delete mailbox',exact:true})).toBeDisabled();await deletion.getByRole('textbox').fill(`${domain}/alice`);await deletion.getByRole('button',{name:'Delete mailbox',exact:true}).click();await expect(deletion).toHaveCount(0);await expect(page).toHaveURL(new RegExp(`/domains/${domain}$`));await expect(page.getByRole('rowheader',{name:`alice@${domain} alice`,exact:true})).toHaveCount(0);
 await page.getByRole('button',{name:'Delete domain',exact:true}).click();await deletion.getByRole('textbox').fill(domain);await deletion.getByRole('button',{name:'Delete domain',exact:true}).click();await expect(deletion).toHaveCount(0);await expect(page).toHaveURL(/\/manage$/);await expect(page.getByRole('button',{name:domain,exact:true})).toHaveCount(0);
 expect(panel.commands).toContain('v-delete-mail-domain');
 }finally{await panel.close();}
});
