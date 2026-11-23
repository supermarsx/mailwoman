import {defineConfig} from 'vitepress';
import {readdirSync,readFileSync} from 'node:fs';
import {resolve,relative,basename} from 'node:path';
import {fileURLToPath} from 'node:url';

const root=fileURLToPath(new URL('../',import.meta.url));
const base=process.env.DOCS_BASE??'/';
const repo='https://github.com/supermarsx/mailwoman';

// Sidebar: one group per top-level section, generated from the files on disk so
// a new page appears without editing this file. Section order and labels are
// set here; inside a section the overview page (index.md / README.md) comes
// first, then pages and sub-folders alphabetically by their title.
const sections=[
 ['Using Mailwoman',[['user']]],
 ['Installation & operations',[['deploy']]],
 ['Administration',[['admin']]],
 ['Security & privacy',[['security']]],
 ['Mail providers',[['bridges'],['protocols','Protocols']]],
 ['Integrations & plugins',[['integrations','Integrations'],['plugins','Plugins'],['export','Import & export']]],
 ['Engineering',[['testing','Testing'],['perf','Performance']]],
];
const skip=new Set(['node_modules','public','vendor']);
const title=file=>{
 const text=readFileSync(file,'utf8');
 const front=text.match(/^---\r?\n[\s\S]*?^title:\s*(.+)$/m)?.[1];
 return (front??text.match(/^#\s+(.+)$/m)?.[1]??basename(file,'.md').replaceAll('-',' ')).replace(/[`*_]/g,'').replace(/^['"]|['"]$/g,'').trim();
};
const isOverview=name=>/^(index|readme)\.md$/i.test(name);
const link=file=>'/'+relative(root,file).replaceAll('\\','/').replace(/\.md$/,'');
function items(dir){
 const entries=readdirSync(dir,{withFileTypes:true}).filter(e=>!e.name.startsWith('.')&&!skip.has(e.name));
 const overview=entries.filter(e=>e.isFile()&&isOverview(e.name)).map(e=>({text:'Overview',link:link(resolve(dir,e.name))})).slice(0,1);
 const rest=entries.flatMap(e=>{
  const file=resolve(dir,e.name);
  if(e.isDirectory()){const sub=items(file);return sub.length?[{text:e.name.replaceAll('-',' ').replace(/^./,c=>c.toUpperCase()),collapsed:true,items:sub}]:[];}
  if(!e.name.endsWith('.md')||isOverview(e.name))return [];
  return [{text:title(file),link:link(file)}];
 }).sort((a,b)=>a.text.localeCompare(b.text));
 return [...overview,...rest];
}
function section([text,dirs],i){
 const list=dirs.length===1&&!dirs[0][1]
  ?items(resolve(root,dirs[0][0]))
  :dirs.map(([dir,label])=>({text:label,collapsed:true,items:items(resolve(root,dir))}));
 return {text,collapsed:i>0,items:list};
}
const engineeringRoot=readdirSync(root,{withFileTypes:true})
 .filter(e=>e.isFile()&&e.name.endsWith('.md')&&!/^(index|readme)\.md$/i.test(e.name))
 .map(e=>({text:title(resolve(root,e.name)),link:link(resolve(root,e.name))}))
 .sort((a,b)=>a.text.localeCompare(b.text));
const sidebar=[
 {text:'Start here',items:[{text:'Welcome',link:'/'},{text:'Documentation map',link:'/README'}]},
 ...sections.map(section),
 {text:'Reference & project notes',collapsed:true,items:engineeringRoot},
];

export default defineConfig({
 title:'Mailwoman',
 titleTemplate:':title · Mailwoman manual',
 description:'The Mailwoman user and administrator manual.',
 lang:'en-US',
 base,appearance:'dark',cleanUrls:true,lastUpdated:true,
 srcExclude:['node_modules/**','**/vendor/**'],
 head:[
  ['link',{rel:'icon',type:'image/svg+xml',href:`${base}favicon.svg`}],
  ['link',{rel:'apple-touch-icon',href:`${base}apple-touch-icon.png`}],
  ['meta',{name:'theme-color',content:'#111111'}],
 ],
 // Operator guides contain literal placeholders such as <accountId>. Render
 // those as text while preserving the explicit anchors used by settings links.
 markdown:{html:false,image:{lazyLoading:true},config(md){
  const renderText=md.renderer.rules.text;
  md.renderer.rules.text=(tokens,index,options,env,self)=>renderText(tokens,index,options,env,self).replace(/&lt;a id=(?:&quot;|")([A-Za-z0-9_.:-]+)(?:&quot;|")&gt;&lt;\/a&gt;/g,'<a id="$1"></a>');
 }},
 themeConfig:{
  logo:{src:'/logo.svg',alt:''},
  siteTitle:'Mailwoman',
  nav:[
   {text:'Use',link:'/user/sync-offline',activeMatch:'^/user/'},
   {text:'Deploy',link:'/deploy/README',activeMatch:'^/deploy/'},
   {text:'Administer',link:'/admin/index',activeMatch:'^/admin/'},
   {text:'Security',link:'/security/README',activeMatch:'^/security/'},
   {text:'Project',items:[
    {text:'Documentation map',link:'/README'},
    {text:'Engineering practices',link:'/engineering-practices'},
    {text:'Specification (SPEC.md)',link:`${repo}/blob/master/SPEC.md`},
    {text:'Versioning & releases',link:`${repo}/blob/master/VERSIONING.md`},
    {text:'Security policy',link:`${repo}/blob/master/SECURITY.md`},
   ]},
  ],
  sidebar,
  search:{provider:'local',options:{detailedView:true,translations:{button:{buttonText:'Search the manual',buttonAriaLabel:'Search the manual'}}}},
  outline:{level:[2,3],label:'On this page'},
  socialLinks:[{icon:'github',link:repo,ariaLabel:'Mailwoman on GitHub'}],
  editLink:{pattern:`${repo}/edit/master/docs/:path`,text:'Edit this page on GitHub'},
  lastUpdated:{text:'Last updated',formatOptions:{dateStyle:'medium'}},
  docFooter:{prev:'Previous',next:'Next'},
  returnToTopLabel:'Back to top',
  sidebarMenuLabel:'Menu',
  darkModeSwitchLabel:'Theme',
  lightModeSwitchTitle:'Switch to light theme',
  darkModeSwitchTitle:'Switch to dark theme',
  externalLinkIcon:true,
  notFound:{title:'PAGE NOT FOUND',quote:'This page is not in the manual. It may have moved; try the search or start from the welcome page.',linkLabel:'Go to the welcome page',linkText:'Back to the manual'},
  footer:{message:'Mailwoman is released under the MIT license.',copyright:'Mailwoman documentation'},
 },
});
