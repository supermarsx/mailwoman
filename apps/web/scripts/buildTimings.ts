import {performance} from 'node:perf_hooks';
import {mkdirSync,writeFileSync} from 'node:fs';
import {join} from 'node:path';
import type {Plugin} from 'vite';
import type {TransformCacheStats} from './cachedSolid.ts';
/** Opt-in phase measurements without changing the shipped assets. */
export function buildTimings(cache:TransformCacheStats):Plugin{
 const enabled=process.env['MW_BUILD_PROFILE']==='1';
 const start=Number(process.env['MW_BUILD_PROFILE_STARTED_AT'])||performance.now();
 let directory='',configAt=0,buildAt=0,transformedAt=0,writtenAt=0,modules=0,chunks=0,catalogs=0;
 return {name:'mw-build-timings',apply:'build',enforce:'post',
  configResolved(config){if(!enabled)return;directory=join(config.root,'node_modules/.cache/mailwoman-build');configAt=performance.now();},
  buildStart(){if(enabled)buildAt=performance.now();},
  buildEnd(){if(enabled){transformedAt=performance.now();modules=[...this.getModuleIds()].length;}},
  generateBundle(_options,bundle){if(!enabled)return;chunks=Object.values(bundle).filter(item=>item.type==='chunk').length;catalogs=Object.keys(bundle).filter(name=>name.endsWith('.ftl')).length;},
  writeBundle(){if(enabled)writtenAt=performance.now();},
  closeBundle(){if(!enabled||!writtenAt)return;const end=performance.now();const result={totalMs:Math.round(end-start),configurationMs:Math.round(configAt-start),startupMs:Math.round(buildAt-configAt),transformMs:Math.round(transformedAt-buildAt),renderAndWriteMs:Math.round(writtenAt-transformedAt),finalizationMs:Math.round(end-writtenAt),modules,jsChunks:chunks,catalogAssets:catalogs,jsxCache:{...cache}};mkdirSync(directory,{recursive:true});writeFileSync(join(directory,'profile.json'),JSON.stringify(result,null,2)+'\n');console.info('Mailwoman build profile:',JSON.stringify(result));},
 };
}
