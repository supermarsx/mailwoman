import {defineConfig} from 'vitepress';
import {buildSidebar} from './sidebar.mjs';

const base=process.env.DOCS_BASE??'/';
const repo='https://github.com/supermarsx/mailwoman';

const sidebar=buildSidebar();

export default defineConfig({
 title:'Mailwoman',
 titleTemplate:':title · Mailwoman manual',
 description:'The Mailwoman user and administrator manual.',
 lang:'en-US',
 base,appearance:'dark',cleanUrls:true,lastUpdated:true,
 // Stubs left at moved paths: a no-JS fallback redirect (the theme also
 // redirects in script) and no entry in the search index.
 transformHead({pageData}){
  const to=pageData.frontmatter?.redirect;
  return typeof to==='string'&&to.startsWith('/')?[['meta',{'http-equiv':'refresh',content:`0;url=${base.replace(/\/$/,'')}${to}`}],['meta',{name:'robots',content:'noindex'}]]:[];
 },
 srcExclude:['node_modules/**','**/vendor/**','screenshots/**'],
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
   {text:'Use',link:'/user/',activeMatch:'^/(user|getting-started)/'},
   {text:'Deploy',link:'/deploy/README',activeMatch:'^/deploy/'},
   {text:'Administer',link:'/admin/index',activeMatch:'^/admin/'},
   {text:'Security',link:'/security/README',activeMatch:'^/security/'},
   {text:'Reference',link:'/reference/cli',activeMatch:'^/reference/'},
   {text:'Project',items:[
    {text:'Documentation map',link:'/README'},
    {text:'Engineering practices',link:'/engineering-practices'},
    {text:'Specification (SPEC.md)',link:`${repo}/blob/master/SPEC.md`},
    {text:'Versioning & releases',link:`${repo}/blob/master/VERSIONING.md`},
    {text:'Security policy',link:`${repo}/blob/master/SECURITY.md`},
   ]},
  ],
  sidebar,
  search:{provider:'local',options:{detailedView:true,
   _render:(src,env,md)=>{const html=md.render(src,env);return env.frontmatter?.search===false||env.frontmatter?.redirect?'':html;},translations:{button:{buttonText:'Search the manual',buttonAriaLabel:'Search the manual'}}}},
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
