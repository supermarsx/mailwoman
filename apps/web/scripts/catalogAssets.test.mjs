import {test,afterEach} from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,readFileSync,rmSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {catalogAssets,packTable} from './catalogAssets.ts';
import {packAssetName,readLocaleSources} from './catalogPacks.ts';
import {catalogAssetSources} from '../src/i18n/catalogAssetSources.ts';
const roots=[];
const originalFlag=process.env.MW_CATALOG_ASSETS;
afterEach(()=>{for(const root of roots.splice(0))rmSync(root,{recursive:true,force:true});if(originalFlag===undefined)delete process.env.MW_CATALOG_ASSETS;else process.env.MW_CATALOG_ASSETS=originalFlag;});
const sourcePath=fileURLToPath(new URL('../src/i18n/catalogAssetSources.ts',import.meta.url));
function fixture(){
 const root=mkdtempSync(join(tmpdir(),'mw-catalog-dictionary-'));roots.push(root);
 mkdirSync(join(root,'src/i18n'),{recursive:true});writeFileSync(join(root,'src/i18n/catalogAssetSources.ts'),readFileSync(sourcePath));
 const files={en:{'a.ftl':'a = A en\nb = B en','admin-cache.ftl':'x = X en','only-en.ftl':'oe = OE'},zz:{'a.ftl':'a = A zz'},empty:{}};
 for(const [locale,list] of Object.entries(files)){
  mkdirSync(join(root,'locales',locale),{recursive:true});for(const [file,text] of Object.entries(list))writeFileSync(join(root,'locales',locale,file),text);
 }
 writeFileSync(join(root,'locales/zz/ignored.txt'),'not a catalog');writeFileSync(join(root,'locales/TRANSLATION-STATUS.json'),'{}');return root;
}
function load(root){
 const plugin=catalogAssets(root);const code=plugin.load(join(root,'src/i18n/catalogAssetSources.ts'));
 assert.equal(typeof code,'string','Actual plugin must emit catalog source');
 const match=code.match(/const manifest: CatalogManifest = (.*);/);assert.ok(match?.[1],'Actual generated manifest required');
 return {code,manifest:JSON.parse(match[1]),plugin};
}
test('manifest lists every namespace of every locale in its pack and shares equal lists',()=>{
 const {manifest,code}=load(fixture());
 assert.deepEqual(manifest.locales,[['empty',0],['en',0],['zz',0]]);
 assert.deepEqual(manifest.lists,[{app:['a','only-en'],admin:['admin-cache']}]);
 assert.deepEqual(Object.keys(catalogAssetSources(manifest)),['empty','en','zz'].flatMap(locale=>['a','only-en','admin-cache'].map(name=>`../../locales/${locale}/${name}.ftl`)).sort());
 assert.ok(!/\.json|assets\//.test(JSON.stringify(manifest)),'no hashed file name in the manifest');
 assert.ok(!code.includes('catalogAssetUrls'));
});
test('buildStart emits one asset per locale and kind with the English fallback merged',()=>{
 const {plugin}=load(fixture());const emitted=[];
 plugin.buildStart.call({emitFile:file=>emitted.push(file)});
 assert.deepEqual(emitted.map(file=>file.name),['empty','en','zz'].flatMap(locale=>['app','admin'].map(kind=>packAssetName(locale,kind))));
 const pack=name=>JSON.parse(emitted.find(file=>file.name===name).source);
 assert.deepEqual(pack('catalog.zz.app.json'),{a:'a = A zz\n\nb = B en\n','only-en':'\noe = OE\n'});
 assert.deepEqual(pack('catalog.zz.admin.json'),{'admin-cache':'\nx = X en\n'});
 assert.deepEqual(pack('catalog.en.app.json'),{a:'a = A en\nb = B en','only-en':'oe = OE'});
});
test('the table goes into index.html, not into a script, and a missing pack fails the build',()=>{
 const {plugin}=load(fixture());plugin.buildStart.call({emitFile(){}});
 const bundle={};for(const locale of ['empty','en','zz'])for(const kind of ['app','admin'])bundle[`assets/c-${locale}-${kind}.json`]={type:'asset',fileName:`assets/catalog.${locale}.${kind}-H${locale}${kind}.json`,names:[packAssetName(locale,kind)]};
 bundle['assets/index.js']={type:'chunk',fileName:'assets/index.js'};
 const html=plugin.transformIndexHtml.handler('<html><head><title>x</title></head><body></body></html>',{bundle});
 const block=html.match(/<script type="application\/json" id="mw-catalog-packs">(.*)<\/script>/);assert.ok(block);
 const table=JSON.parse(block[1]);
 assert.deepEqual(Object.keys(table),['empty','en','zz'].flatMap(locale=>['app','admin'].map(kind=>`${locale}/${kind}`)));
 assert.equal(table['zz/app'],'./assets/catalog.zz.app-Hzzapp.json');
 assert.ok(html.indexOf('mw-catalog-packs')<html.indexOf('</head>'));
 delete bundle['assets/c-zz-admin.json'];
 assert.throws(()=>packTable(bundle,['zz']),/zz\/admin was not emitted/);
 assert.equal(plugin.transformIndexHtml.handler('<head></head>',{}),undefined);
});
test('worker builds and the raw-import flag emit nothing and leave the source alone',()=>{
 const root=fixture();const worker=catalogAssets(root,{emit:false});const emitted=[];
 worker.buildStart.call({emitFile:file=>emitted.push(file)});assert.deepEqual(emitted,[]);
 assert.equal(worker.transformIndexHtml.handler('<head></head>',{bundle:{}}),undefined);
 process.env.MW_CATALOG_ASSETS='0';const plain=catalogAssets(root);plain.buildStart.call({emitFile:file=>emitted.push(file)});assert.deepEqual(emitted,[]);
 assert.equal(plain.transformIndexHtml.handler('<head></head>',{bundle:{}}),undefined);
});
test('invalid dictionary references and a missing injection marker fail visibly',()=>{
 assert.throws(()=>catalogAssetSources({lists:[],locales:[['aa',0]]}),/Missing translation catalog manifest list/);
 const root=fixture();writeFileSync(join(root,'src/i18n/catalogAssetSources.ts'),'export default {};');assert.throws(()=>load(root),/Catalog manifest marker missing/);
});
test('production flag and exact source resolution behavior remain unchanged',()=>{
 const root=fixture();const plugin=catalogAssets(root);const importer=join(root,'src/i18n/catalog.ts');
 process.env.MW_CATALOG_ASSETS='0';assert.equal(plugin.resolveId('./catalogSources.ts',importer),undefined);
 process.env.MW_CATALOG_ASSETS='1';assert.equal(plugin.resolveId('./catalogSources.ts',importer),join(root,'src/i18n/catalogAssetSources.ts'));
 assert.equal(plugin.resolveId('./catalogSources.ts',join(root,'src/another.ts')),undefined);
});
test('actual repository manifest has a key for every namespace of every locale',()=>{
 const root=fileURLToPath(new URL('../',import.meta.url));const {manifest}=load(root);
 const sources=readLocaleSources(join(root,'locales'));const names=new Set([...sources.values()].flatMap(files=>[...files.keys()]));
 const actual=Object.keys(catalogAssetSources(manifest));
 assert.equal(actual.length,sources.size*names.size);assert.equal(manifest.lists.length,1);assert.equal(manifest.locales.length,sources.size);
 assert.ok(actual.includes('../../locales/pt-BR/mail.ftl')&&actual.includes('../../locales/ja/admin-cache.ftl'));
});
