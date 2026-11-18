import {createHash,randomUUID} from 'node:crypto';
import {readFile,writeFile,mkdir,rename,unlink} from 'node:fs/promises';
import {readFileSync} from 'node:fs';
import {join} from 'node:path';
import solid from 'vite-plugin-solid';
import type {Plugin,ResolvedConfig,TransformResult} from 'vite';
type SourceDescription=Extract<TransformResult,{code:string}>;

export interface TransformCacheStats {hits:number;misses:number}
interface CacheOptions {directory?:string;stats?:TransformCacheStats}
const digest=(text:string)=>createHash('sha256').update(text).digest('hex');
/** Cache the pure production JSX compiler step. Resolution, CSS extraction,
 * bundling, SBOM generation and integrity checks still run on every build. */
export function cacheSolidTransforms(plugin:Plugin,options:CacheOptions={}):Plugin{
 const original=plugin.transform,resolveConfig=plugin.configResolved;
 if(typeof original!=='function')throw new Error('Solid transform hook changed; update the build cache adapter');
 let enabled=false,directory='',seed='';
 const stats=options.stats??{hits:0,misses:0};
 plugin.configResolved=async function(config:ResolvedConfig){
  if(typeof resolveConfig==='function')await Reflect.apply(resolveConfig,this,[config]);
  enabled=config.command==='build'&&config.mode!=='test'&&process.env['MW_BUILD_CACHE']!=='0';
  directory=options.directory??join(config.cacheDir,'mailwoman-solid');
  const hash=createHash('sha256');
  for(const file of [...new Set([...config.configFileDependencies,join(config.root,'pnpm-lock.yaml'),join(config.root,'package.json')])].sort()){
   hash.update(file);try{hash.update(readFileSync(file));}catch{hash.update('missing');}
  }
  hash.update(JSON.stringify({schema:1,node:process.version,root:config.root,mode:config.mode,env:config.env,define:config.define,target:config.build.target,sourcemap:config.build.sourcemap}));
  seed=hash.digest('hex');
 };
 plugin.transform=async function(source,id,transformOptions){
  if(!enabled||transformOptions?.ssr||! /\.[mc]?[jt]sx(?:\?|$)/i.test(id))return Reflect.apply(original,this,[source,id,transformOptions]);
  const key=digest(`${seed}\0${id}\0${source}`),file=join(directory,`${digest(id)}.json`);
  try{
   const stored=JSON.parse(await readFile(file,'utf8')) as {schema:number;key:string;checksum:string;result:SourceDescription};
   if(stored.schema===1&&stored.key===key&&typeof stored.result?.code==='string'&&stored.checksum===digest(JSON.stringify(stored.result))){stats.hits++;return stored.result;}
  }catch{/* Missing, corrupt or inaccessible caches are harmless misses. */}
  stats.misses++;
  const result=await Reflect.apply(original,this,[source,id,transformOptions]) as SourceDescription|null;
  if(result&&typeof result==='object'&&typeof result.code==='string'){
   const temporary=`${file}.${process.pid}.${randomUUID()}.tmp`;
   try{
    await mkdir(directory,{recursive:true});
    await writeFile(temporary,JSON.stringify({schema:1,key,checksum:digest(JSON.stringify(result)),result}));
    await rename(temporary,file);
   }catch{/* Cache writes must never make a valid build fail. */}
   finally{await unlink(temporary).catch(()=>undefined);}
  }
  return result;
 };
 return plugin;
}
export function cachedSolid(options:CacheOptions={}):Plugin{return cacheSolidTransforms(solid(),options);}
