import {test,afterEach} from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,readFileSync,readdirSync,rmSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {gzipSync} from 'node:zlib';
import {catalogAssets} from './catalogAssets.ts';
import {catalogAssetSources} from '../src/i18n/catalogAssetSources.ts';
const roots=[];
const originalFlag=process.env.MW_CATALOG_ASSETS;
afterEach(()=>{for(const root of roots.splice(0))rmSync(root,{recursive:true,force:true});if(originalFlag===undefined)delete process.env.MW_CATALOG_ASSETS;else process.env.MW_CATALOG_ASSETS=originalFlag;});
const sourcePath=fileURLToPath(new URL('../src/i18n/catalogAssetSources.ts',import.meta.url));
function fixture(){
 const root=mkdtempSync(join(tmpdir(),'mw-catalog-dictionary-'));roots.push(root);
 mkdirSync(join(root,'src/i18n'),{recursive:true});writeFileSync(join(root,'src/i18n/catalogAssetSources.ts'),readFileSync(sourcePath));
 for(const [locale,files] of Object.entries({zz:['z.ftl','a.ftl'],aa:['a.ftl','z.ftl'],bb:['a.ftl'],empty:[]})){
  mkdirSync(join(root,'locales',locale),{recursive:true});for(const file of files)writeFileSync(join(root,'locales',locale,file),'public = fixture');
 }
 writeFileSync(join(root,'locales/bb/ignored.txt'),'not a catalog');return root;
}
function load(root){
 const plugin=catalogAssets(root);const code=plugin.load(join(root,'src/i18n/catalogAssetSources.ts'));
 assert.equal(typeof code,'string','Actual plugin must emit catalog source');
 const match=code.match(/const manifest: CatalogManifest = (.*);/);assert.ok(match?.[1],'Actual generated manifest required');
 return {code,manifest:JSON.parse(match[1])};
}
test('actual plugin deduplicates equal lists while retaining unequal, empty and sorted exact keys',()=>{
 const {manifest,code}=load(fixture());
 assert.deepEqual(manifest.locales,[['aa',0],['bb',1],['empty',2],['zz',0]]);
 assert.deepEqual(manifest.lists,[['a.ftl','z.ftl'],['a.ftl'],[]]);
 assert.deepEqual(Object.keys(catalogAssetSources(manifest)),['../../locales/aa/a.ftl','../../locales/aa/z.ftl','../../locales/bb/a.ftl','../../locales/zz/a.ftl','../../locales/zz/z.ftl']);
 assert.ok(code.includes("() => import('./catalogAssetUrls.ts')"));
});
test('runtime retains lazy URL loading, exact keys, Unicode results and retryable original errors',async()=>{
 const {manifest}=load(fixture());let imports=0,calls=0;
 const urls={default:{'../../locales/bb/a.ftl':async()=>{calls++;if(calls===2)throw new Error('offline');return calls===1?'Olá 🙂':'retry';}}};
 const sources=catalogAssetSources(manifest,async()=>{imports++;return urls;});assert.equal(imports,0);assert.equal(calls,0);
 const selected=sources['../../locales/bb/a.ftl'];assert.equal(typeof selected,'function');
 assert.equal(await selected(),'Olá 🙂');await assert.rejects(selected(),/offline/);assert.equal(await selected(),'retry');
 assert.equal(imports,3);assert.equal(calls,3);assert.equal(sources['../../locales/bb/z.ftl'],undefined);
 await assert.rejects(sources['../../locales/aa/a.ftl'](),/Missing translation catalog: ..\/..\/locales\/aa\/a.ftl/);
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
test('actual repository manifest reconstructs every original key and reduces metadata without fetching catalogs',()=>{
 const root=fileURLToPath(new URL('../',import.meta.url));const {manifest}=load(root);
 const expected=readdirSync(join(root,'locales'),{withFileTypes:true}).filter(x=>x.isDirectory()).flatMap(locale=>readdirSync(join(root,'locales',locale.name)).filter(file=>file.endsWith('.ftl')).map(file=>`../../locales/${locale.name}/${file}`)).sort();
 let imports=0;const actual=Object.keys(catalogAssetSources(manifest,async()=>{imports++;return {default:{}};}));assert.deepEqual(actual,expected);assert.ok(actual.length>1000);assert.equal(imports,0);
 assert.ok(gzipSync(JSON.stringify(manifest)).length<gzipSync(JSON.stringify(expected)).length);
});
