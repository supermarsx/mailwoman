import {defineConfig} from 'vitepress';
import {readdirSync,readFileSync} from 'node:fs';
import {resolve,relative,basename} from 'node:path';
import {fileURLToPath} from 'node:url';
const root=fileURLToPath(new URL('../',import.meta.url));
const labels={user:'Using Mailwoman',deploy:'Installation & operations',admin:'Administration',security:'Security & privacy',bridges:'Mail providers',integrations:'Integrations',export:'Import & export',plugins:'Plugins',protocols:'Protocols',testing:'Testing',perf:'Performance'};
const title=file=>readFileSync(file,'utf8').match(/^#\s+(.+)$/m)?.[1]?.replace(/[`*_]/g,'').trim()??basename(file,'.md').replaceAll('-',' ');
function items(dir){return readdirSync(dir,{withFileTypes:true}).filter(entry=>!entry.name.startsWith('.')&&entry.name!=='node_modules').flatMap(entry=>{
 const file=resolve(dir,entry.name);
 if(entry.isDirectory())return [{text:labels[entry.name]??entry.name.replaceAll('-',' '),collapsed:true,items:items(file)}];
 if(!entry.name.endsWith('.md'))return [];
 return [{text:title(file),link:'/'+relative(root,file).replaceAll('\\','/').replace(/\.md$/,'')}];
}).sort((a,b)=>a.text.localeCompare(b.text));}
const sections=Object.keys(labels).map(dir=>({text:labels[dir],collapsed:dir!=='user',items:items(resolve(root,dir))}));
export default defineConfig({
 title:'Mailwoman',description:'The Mailwoman user and administrator manual.',
 base:process.env.DOCS_BASE??'/',appearance:'dark',cleanUrls:true,lastUpdated:true,
 srcExclude:['node_modules/**','**/vendor/**'],
 // Operator guides contain literal placeholders such as <accountId>. Render
 // those as text while preserving the explicit anchors used by settings links.
 markdown:{html:false,config(md){
  const renderText=md.renderer.rules.text;
  md.renderer.rules.text=(tokens,index,options,env,self)=>renderText(tokens,index,options,env,self).replace(/&lt;a id=(?:&quot;|")([A-Za-z0-9_.:-]+)(?:&quot;|")&gt;&lt;\/a&gt;/g,'<a id="$1"></a>');
 }},

 themeConfig:{
  siteTitle:'MAILWOMAN / MANUAL',
  nav:[{text:'Start here',link:'/'},{text:'User guide',link:'/user/sync-offline'},{text:'Deployment',link:'/deploy/README'},{text:'Administration',link:'/admin/settings'}],
  sidebar:[{text:'Start here',link:'/'},...sections,{text:'Engineering & reference',collapsed:true,items:items(root).filter(item=>!item.items&&item.link!=='/index'&&item.link!=='/README')}],
  search:{provider:'local',options:{detailedView:true}},
  outline:{level:[2,3],label:'On this page'},
  socialLinks:[{icon:'github',link:'https://github.com/supermarsx/mailwoman'}],
  editLink:{pattern:'https://github.com/supermarsx/mailwoman/edit/master/docs/:path',text:'Edit this page'},
  footer:{message:'Mailwoman documentation'},
 },
});
