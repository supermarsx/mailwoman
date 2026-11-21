// @vitest-environment node
import {afterEach,expect,it} from 'vitest';
import {mkdtempSync,mkdirSync,readFileSync,writeFileSync,rmSync,existsSync,truncateSync,symlinkSync,statSync,utimesSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {brotliDecompressSync,gzipSync,gunzipSync} from 'node:zlib';
import {writeCompressedAssets} from '../../scripts/assetCompression.ts';
import {assetIntegrity} from '../../vite.config.ts';
const owned:string[]=[];
afterEach(()=>{for(const directory of owned.splice(0))rmSync(directory,{recursive:true,force:true});});
function fixture(){const root=mkdtempSync(join(tmpdir(),'mw-public-encoding-'));owned.push(root);const dist=join(root,'dist');mkdirSync(join(dist,'assets'),{recursive:true});return {root,dist};}
it('emits deterministic smaller sidecars whose decoded bytes preserve source/SRI payloads',()=>{
 const {dist}=fixture();const path=join(dist,'assets','public-ABC12345.js');const raw=Buffer.from('export const fixture="public-static";\n'.repeat(400));writeFileSync(path,raw);
 const result=writeCompressedAssets(dist);expect(result.files).toBe(1);expect(gunzipSync(readFileSync(path+'.gz'))).toEqual(raw);expect(brotliDecompressSync(readFileSync(path+'.br'))).toEqual(raw);
 const first=readFileSync(path+'.gz');utimesSync(path+'.gz',1,1);const stamp=statSync(path+'.gz').mtimeMs;
 writeCompressedAssets(dist);expect(readFileSync(path+'.gz')).toEqual(first);expect(statSync(path+'.gz').mtimeMs).toBe(stamp);expect(readFileSync(path)).toEqual(raw);
});
it('skips private HTML, source maps, already compressed/binary assets and tiny payloads',()=>{
 const {dist}=fixture();for(const name of ['console.html','private.js.map','font.woff2','photo.png','archive.gz'])writeFileSync(join(dist,'assets',name),'fixture'.repeat(1000));writeFileSync(join(dist,'assets','small.js'),'x');
 expect(writeCompressedAssets(dist).files).toBe(0);for(const name of ['console.html','private.js.map','font.woff2','photo.png','archive.gz','small.js'])expect(existsSync(join(dist,'assets',name+'.br'))).toBe(false);
});
it('runs after real raw SRI/SW digest generation and never adds sidecars to precache',()=>{
 const {root,dist}=fixture();const path=join(dist,'assets','public-ABC12345.js');writeFileSync(path,'export const fixture="public-static";\n'.repeat(400));
 writeFileSync(join(dist,'index.html'),'<script type="module" src="./assets/public-ABC12345.js"></script>');
 writeFileSync(join(dist,'sw.js'),'const digests=/* @mw-asset-digests */ null;const offline=/* @mw-offline-assets */ [];/* @mw-offline-background */');
 // A retained/custom outDir may already contain sidecars from an older build.
 writeFileSync(path+'.gz',gzipSync(readFileSync(path)));
 const integrity=assetIntegrity();(integrity.configResolved as (c:unknown)=>void)({command:'build',root,build:{outDir:'dist'}});(integrity.writeBundle as ()=>void)();(integrity.closeBundle as ()=>void)();
 const sw=readFileSync(join(dist,'sw.js'));const html=readFileSync(join(dist,'index.html'));expect(html.toString()).toContain(' integrity="sha384-');
 writeCompressedAssets(dist);expect(readFileSync(join(dist,'sw.js'))).toEqual(sw);expect(readFileSync(join(dist,'index.html'))).toEqual(html);expect(sw.toString()).not.toContain('.js.gz');expect(sw.toString()).not.toContain('.js.br');expect(gunzipSync(readFileSync(path+'.gz'))).toEqual(readFileSync(path));
});

it('bounds eligible source size and refuses escaped filesystem input before producing sidecars',()=>{
 const {dist}=fixture();const huge=join(dist,'assets','large.js');writeFileSync(huge,'');truncateSync(huge,16*1024*1024+1);
 expect(writeCompressedAssets(dist).files).toBe(0);expect(existsSync(huge+'.gz')).toBe(false);
 // Windows symlink creation needs an OS privilege; the actual symlink branch is
 // exercised by the Linux build gate, while the sparse-size contract is portable.
 if(process.platform!=='win32'){
  const outside=join(dist,'outside.js');writeFileSync(outside,'export const publicFixture=1;'.repeat(100));
  symlinkSync(outside,join(dist,'assets','escape.js'));
  expect(()=>writeCompressedAssets(dist)).toThrow(/symlink/);expect(existsSync(outside+'.gz')).toBe(false);
 }
});
