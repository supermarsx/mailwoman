// The manual's sidebar. Groups and page order are set here (task flow, not
// alphabetical); labels are each page's own `# heading`. A page that is not
// listed still shows up in the catch-all "More" group of its area, so a new
// page is never lost. Stubs with `redirect:` front matter are always left out.
import {existsSync,readdirSync,readFileSync} from 'node:fs';
import {resolve,relative,basename} from 'node:path';
import {fileURLToPath} from 'node:url';

export const root=fileURLToPath(new URL('../',import.meta.url));
const skip=new Set(['node_modules','public','vendor','screenshots']);
const frontMatter=file=>readFileSync(file,'utf8').replace(/^\uFEFF/,'').match(/^---\r?\n([\s\S]*?)\r?\n---(\r?\n|$)/)?.[1]??'';
export const isStub=file=>/^redirect:\s*\S/m.test(frontMatter(file));
const clean=t=>t.replace(/[`*_]/g,'').replace(/^['"]|['"]$/g,'').trim();
const title=file=>{
 const text=readFileSync(file,'utf8');
 const h1=text.replace(/^---\r?\n[\s\S]*?\r?\n---\r?\n/,'').match(/^#\s+(.+)$/m)?.[1];
 const front=frontMatter(file).match(/^title:\s*(.+)$/m)?.[1];
 return clean(h1??front??basename(file,'.md').replaceAll('-',' '));
};
const rel=file=>relative(root,file).replaceAll('\\','/');
const linkOf=file=>'/'+rel(file).replace(/\.md$/,'');
const used=new Set();
const fileOf=p=>resolve(root,p+'.md');
function page(p){
 const f=fileOf(p);
 if(!existsSync(f)||isStub(f))return null;
 used.add(rel(f));
 return {text:title(f),link:linkOf(f)};
}
const isOverview=name=>/^(index|readme)\.md$/i.test(name);
// Every live page below a folder: overview first, then pages by title, then
// sub-folders. `except` lists folders (relative to docs/) to leave out.
function dirItems(dir,except=[]){
 const abs=resolve(root,dir);
 if(!existsSync(abs))return [];
 const entries=readdirSync(abs,{withFileTypes:true}).filter(e=>!e.name.startsWith('.')&&!skip.has(e.name));
 const pages=entries.filter(e=>e.isFile()&&e.name.endsWith('.md')).map(e=>resolve(abs,e.name)).filter(f=>!isStub(f));
 const ov=pages.filter(f=>isOverview(basename(f)));
 const rest=pages.filter(f=>!isOverview(basename(f))).map(f=>({text:title(f),link:linkOf(f),f})).sort((a,b)=>a.text.localeCompare(b.text));
 const subs=entries.filter(e=>e.isDirectory()&&!except.includes(`${dir}/${e.name}`)).map(e=>{
  const sub=dirItems(`${dir}/${e.name}`,except);
  return sub.length?{text:e.name==='iis'?'IIS':e.name.replaceAll('-',' ').replace(/^./,c=>c.toUpperCase()),collapsed:true,items:sub}:null;
 }).filter(Boolean).sort((a,b)=>a.text.localeCompare(b.text));
 [...ov,...rest.map(r=>r.f)].forEach(f=>used.add(rel(f)));
 return [...ov.map(f=>({text:title(f),link:linkOf(f)})),...rest.map(({text,link})=>({text,link})),...subs];
}
const list=paths=>paths.flatMap(p=>(typeof p==='string'?[page(p)].filter(Boolean):[p]));
const group=(text,items,collapsed=true)=>({text,collapsed,items});

// Using Mailwoman
const using=()=>group('Using Mailwoman',[
 ...list(['user/index']),
 group('Mail',list(['user/mailbox/reading','user/mailbox/composing','user/mailbox/search','user/mailbox/folders-labels',
  'user/mailbox/attachments','user/mailbox/bulk-actions','user/mailbox/templates','user/snooze','user/unified-mail',
  'user/mailbox/linked-accounts','user/mailbox/moving-between-accounts','user/mailbox/sharing'])),
 ...list(['user/rules-filters','user/contacts','user/calendar','user/tasks','user/notes','user/notifications','user/packages',
  'user/assist','user/encryption-keys','user/keyboard-shortcuts','user/offline-sync','user/apps-desktop-mobile']),
 group('Settings',list(['user/settings/index','user/settings/account','user/settings/mail','user/settings/interface',
  'user/settings/integrations','user/settings/about'])),
],false);

// Administering
const administering=()=>{
 const proxy=dirItems('deploy/proxy');
 const upgrade=dirItems('deploy/upgrade-notes');
 const groups=[
  group('The admin console',list(['admin/index','admin/client-access','admin/server-metadata','admin/hosting'])),
  group('Install and upgrade',[...list(['deploy/README','deploy/script','deploy/packaging','deploy/compose-formats','deploy/reverse-proxy']),
   ...(proxy.length?[group('Reverse-proxy recipes',proxy)]:[]),
   ...list(['deploy/acme','deploy/postgres','deploy/cache','deploy/push','deploy/websocket','deploy/backups','deploy/shells','deploy/desktop','deploy/mobile-android']),
   ...(upgrade.length?[group('Upgrade notes',upgrade)]:[])]),
  group('Accounts and domains',list(['admin/domains','admin/guided-domain-setup','admin/domain-providers','admin/preprovisioned-users',
   'admin/user-access-lifecycle','admin/directory','deploy/ldap','deploy/kerberos','admin/identity-provider','admin/sso','admin/oauth-clients',
   'deploy/sending-identities','deploy/imap-pop3','deploy/caldav-carddav'])),
  group('Security policy',list(['admin/security-policy','admin/security-center','admin/sessions','admin/signin-locks','admin/signin-geography',
   'admin/ip-bans','admin/inbound-allowlists','admin/request-protection','admin/protocol-guard','admin/crawler-identification',
   'admin/smtp-transport-policy','admin/smtp-internationalization','admin/authenticator','admin/encryption','admin/deployment-security',
   'admin/product-security','admin/ssh-tunnels','deploy/crypto-security','deploy/connection-limits','deploy/egress-proxy','deploy/fail2ban','deploy/hardening'])),
  group('Privacy and compliance',list(['admin/privacy','admin/consent','admin/cookies','admin/data-requests','admin/breach-register',
   'admin/retention','admin/compliance','assist','admin/country-database','admin/feature-governance'])),
  group('Mail features',list(['admin/composer-features','deploy/composer-ribbon','admin/outbox-recall','admin/package-tracking','admin/notifications',
   'admin/webhooks','admin/ai-assistant','admin/jmap','deploy/read-receipts','deploy/delivery-status','deploy/mailing-list-actions',
   'admin/outgoing-header-privacy','deploy/outgoing-header-privacy','deploy/attachment-preview','deploy/account-drafts','deploy/mail-import-export',
   'deploy/sieve-runtime','deploy/scheduler-runtime','admin/plugins','admin/ui-plugins'])),
  group('Appearance',list(['admin/branding','deploy/fonts'])),
  group('Operations',list(['admin/observability','admin/latency-metrics','admin/metrics-persistence','admin/logs','admin/diagnostics',
   'admin/analytics','admin/maintenance','admin/database-maintenance','admin/database-startup','admin/postgresql-tuning','admin/performance-modes','deploy/online-sqlite-maintenance',
   'admin/operational-incidents','admin/configuration-transfer','deploy/telemetry','deploy/scaling','deploy/load-testing','deploy/websocket-load-testing'])),
  group('Troubleshooting',list(['admin/troubleshooting'])),
 ];
 // Reference pages live under Reference, engineering pages under Engineering notes.
 const elsewhere=/^\/(admin|deploy)\/(settings|configuration|config-catalogue|erasure-coverage|scan-path-catalog|load-baseline-2026-10-26)$/;
 const placed=new Set(used);
 const more=[...dirItems('admin').filter(i=>!i.items),...dirItems('deploy',['deploy/proxy','deploy/upgrade-notes']).filter(i=>!i.items)]
  .filter(i=>!elsewhere.test(i.link)&&!placed.has(i.link.slice(1)+'.md'));
 return group('Administering',[...groups,...(more.length?[group('More',more)]:[])]);
};

const reference=()=>group('Reference',list(['reference/cli','reference/environment','admin/configuration','admin/config-catalogue',
 'reference/settings','admin/settings','reference/glossary','protocols/imap-extensions','admin/erasure-coverage','admin/scan-path-catalog']));
const integrations=()=>group('Integrations and protocols',[...list(['integrations/nextcloud']),
 ...[['Mail bridges','bridges'],['Plugins','plugins'],['Import and export','export']].flatMap(([t,d])=>{const i=dirItems(d);return i.length?[group(t,i)]:[];})]);
const security=()=>group('Security model',dirItems('security',['security/audit-prep']));
const changelog=()=>group('Changelog',dirItems('changelog').sort((a,b)=>b.text.localeCompare(a.text,undefined,{numeric:true})));

// Engineering notes (collapsed, last).
const engineering=()=>{
 const loose=readdirSync(root,{withFileTypes:true}).filter(e=>e.isFile()&&e.name.endsWith('.md')&&!isOverview(e.name))
  .map(e=>resolve(root,e.name)).filter(f=>!isStub(f)&&!used.has(rel(f)))
  .map(f=>({text:title(f),link:linkOf(f)})).sort((a,b)=>a.text.localeCompare(b.text));
 const sub=(t,d,except)=>{const i=dirItems(d,except);return i.length?[group(t,i)]:[];};
 return group('Engineering notes',[
  ...dirItems('engineering'),...loose,...list(['deploy/load-baseline-2026-10-26']),
  ...sub('Testing','testing'),...sub('Performance','perf'),...sub('Security audit preparation','security/audit-prep'),
 ]);
};

export function buildSidebar(){
 used.clear();
 const start=group('Start here',[{text:'Welcome',link:'/'},
  ...list(['getting-started/what-is-mailwoman','getting-started/install-quick','getting-started/first-sign-in']),
  ...dirItems('getting-started').filter(i=>!used.has(i.link.slice(1)+'.md')),{text:'Documentation map',link:'/README'}],false);
 used.add('README.md');used.add('index.md');
 // Groups that claim pages run before the catch-alls (Administering's More, Engineering).
 const u=using(),a=administering(),s=security(),i=integrations(),r=reference(),c=changelog(),e=engineering();
 return [start,u,a,s,i,r,c,e].filter(g=>g.items.length);
}
