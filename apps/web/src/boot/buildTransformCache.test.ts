// @vitest-environment node
import {mkdtempSync,writeFileSync,readFileSync,readdirSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,dirname,basename} from 'node:path';
import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import type {Plugin,ResolvedConfig} from 'vite';
import {cacheSolidTransforms} from '../../scripts/cachedSolid.ts';
const roots:string[]=[];
beforeEach(()=>vi.stubEnv('MW_BUILD_CACHE','1'));
afterEach(()=>{vi.unstubAllEnvs();for(const root of roots.splice(0)){if(dirname(root)!==tmpdir()||!basename(root).startsWith('mw-solid-cache-'))throw new Error('Invalid fixture path');rmSync(root,{recursive:true,force:true});}});
async function fixture(command='build'){
 const root=mkdtempSync(join(tmpdir(),'mw-solid-cache-'));roots.push(root);
 const lock=join(root,'pnpm-lock.yaml');writeFileSync(lock,'first lock');
 const compile=vi.fn(async(source:string)=>({code:`compiled:${source}`,map:{version:3,sources:['App.tsx'],names:[],mappings:''}}));
 const stats={hits:0,misses:0};
 const plugin=cacheSolidTransforms({name:'solid',transform:compile} as Plugin,{directory:join(root,'cache'),stats});
 const config={command,mode:'production',root,cacheDir:root,configFileDependencies:[],define:{FLAG:'1'},env:{},build:{target:'es2022',sourcemap:false}} as unknown as ResolvedConfig;
 const configure=()=>Reflect.apply(plugin.configResolved as CallableFunction,{},[config]);await configure();
 const run=(source='first',ssr=false)=>Reflect.apply(plugin.transform as CallableFunction,{},[source,join(root,'App.tsx'),{ssr}]);
 return {root,lock,compile,stats,config,configure,run};
}
it('reuses byte-identical code and source maps and invalidates changed source or dependencies',async()=>{
 const f=await fixture();const initial=await f.run();expect(await f.run()).toEqual(initial);expect(f.compile).toHaveBeenCalledOnce();
 await f.run('changed');expect(f.compile).toHaveBeenCalledTimes(2);
 writeFileSync(f.lock,'new compiler dependency');await f.configure();await f.run('changed');expect(f.compile).toHaveBeenCalledTimes(3);
 expect(f.stats.hits).toBe(1);
});
it('invalidates changed build defines and repairs corrupt cache records',async()=>{
 const f=await fixture();await f.run();f.config.define!['FLAG']='2';await f.configure();await f.run();expect(f.compile).toHaveBeenCalledTimes(2);
 const file=join(f.root,'cache',readdirSync(join(f.root,'cache'))[0]!);const record=JSON.parse(readFileSync(file,'utf8'));record.result.code='corrupt';writeFileSync(file,JSON.stringify(record));
 expect((await f.run()).code).toBe('compiled:first');expect(f.compile).toHaveBeenCalledTimes(3);
});
it('bypasses development, SSR and explicitly disabled caching',async()=>{
 const dev=await fixture('serve');await dev.run();await dev.run();expect(dev.compile).toHaveBeenCalledTimes(2);
 const ssr=await fixture();await ssr.run('first',true);await ssr.run('first',true);expect(ssr.compile).toHaveBeenCalledTimes(2);
 const disabled=await fixture();vi.stubEnv('MW_BUILD_CACHE','0');await disabled.configure();await disabled.run();await disabled.run();expect(disabled.compile).toHaveBeenCalledTimes(2);
});
it('still builds when cache storage is unavailable',async()=>{
 const f=await fixture();writeFileSync(join(f.root,'cache'),'not a directory');
 expect((await f.run()).code).toBe('compiled:first');expect((await f.run()).code).toBe('compiled:first');expect(f.compile).toHaveBeenCalledTimes(2);
});
