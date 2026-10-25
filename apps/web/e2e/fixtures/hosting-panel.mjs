import https from 'node:https';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import {fileURLToPath,URLSearchParams} from 'node:url';
// DEV/CI fixture only. Fixed fake credentials; binds exclusively to loopback.
const repo=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../../..');
fs.mkdirSync(path.join(repo,'.orchestration/reports'),{recursive:true});const statePath=path.join(repo,'.orchestration/reports/codex-hosting-panel-state.json');
const domains=new Map();const suspendedDomains=new Set();const commands=[];
function save(){fs.writeFileSync(statePath,JSON.stringify({commands,domains:[...domains.keys()]}));}save();
https.createServer({cert:fs.readFileSync(path.join(repo,'crates/mw-server/tests/fixtures/tls/localhost-cert.pem')),key:fs.readFileSync(path.join(repo,'crates/mw-server/tests/fixtures/tls/localhost-key.pem'))},(req,res)=>{
if(req.method!=='POST'||req.url!=='/api/'){res.writeHead(404);res.end();return;}let body='';req.on('data',chunk=>{body+=chunk;if(body.length>16384)req.destroy();});req.on('end',()=>{
const form=new URLSearchParams(body);if(form.get('hash')!=='test-access:panel-fixture-secret'){res.writeHead(403);res.end('refused');return;}
const command=form.get('cmd');commands.push(command);const domain=form.get('arg2');let result='0';
switch(command){case 'v-list-mail-domains':result=JSON.stringify(Object.fromEntries([...domains].map(([name])=>[name,{SUSPENDED:suspendedDomains.has(name)?'yes':'no',PASSWORD:'never-return-provider-hash'}])));break;
case 'v-add-mail-domain':domains.set(domain,new Map());break;
case 'v-delete-mail-domain':domains.delete(domain);suspendedDomains.delete(domain);break;
case 'v-suspend-mail-domain':suspendedDomains.add(domain);break;
case 'v-unsuspend-mail-domain':suspendedDomains.delete(domain);break;
case 'v-list-mail-accounts':result=JSON.stringify(Object.fromEntries(domains.get(domain)??[]));break;
case 'v-add-mail-account':domains.get(domain)?.set(form.get('arg3'),{SUSPENDED:'no',QUOTA:form.get('arg5')??'unlimited',ALIAS:''});break;
case 'v-change-mail-account-quota':{const account=domains.get(domain)?.get(form.get('arg3'));if(account)account.QUOTA=form.get('arg4');break;}
case 'v-change-mail-account-password':break; // Never retain the replacement password.
case 'v-suspend-mail-account':case 'v-unsuspend-mail-account':{const account=domains.get(domain)?.get(form.get('arg3'));if(account)account.SUSPENDED=command==='v-suspend-mail-account'?'yes':'no';break;}
case 'v-add-mail-account-alias':case 'v-delete-mail-account-alias':{const account=domains.get(domain)?.get(form.get('arg3'));if(account){const aliases=new Set(account.ALIAS.split(',').filter(Boolean));if(command==='v-add-mail-account-alias')aliases.add(form.get('arg4'));else aliases.delete(form.get('arg4'));account.ALIAS=[...aliases].join(',');}break;}
case 'v-delete-mail-account':domains.get(domain)?.delete(form.get('arg3'));break;
default:res.writeHead(422);res.end('unsupported-fixture-command');save();return;}
save();res.writeHead(200,{'Content-Type':'application/json','Connection':'close'});res.end(result);});
}).listen(Number(process.env.MW_TEST_HOSTING_PORT||42421),'127.0.0.1');
