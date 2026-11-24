import {join,dirname} from 'node:path';
import {MAX_WORKERS_PER_HOST} from './http-load.mjs';
import {parseAllowList} from './http-targets.mjs';
import {fileURLToPath} from 'node:url';

export function generatorWorkers(value=1){
 const workers=Number(value);
 if(!Number.isInteger(workers)||workers<1||workers>MAX_WORKERS_PER_HOST)throw new Error(`HTTP generator workers must be 1–${MAX_WORKERS_PER_HOST}`);
 return workers;
}
// Only private handoff/report file paths become arguments, never credentials.
export function httpGeneratorLaunch({directory,ready,workers=1,allowTargets}){
 workers=generatorWorkers(workers);
 if(allowTargets&&workers===1)throw new Error('Private rig targets need the multiprocess controller');
 const scripts=dirname(fileURLToPath(import.meta.url)),handoff=join(directory,ready.handoffFile),output=join(directory,ready.outputFile);
 return workers===1
  ?{script:join(scripts,'http-real-fixture.mjs'),args:[handoff,output],workers}
  :{script:join(scripts,'http-multi-controller.mjs'),args:[`--handoff=${handoff}`,`--output=${output}`,`--workers=${workers}`,...(allowTargets?[`--allow-targets=${parseAllowList(allowTargets).map(origin=>origin.slice('http://'.length)).join(',')}`]:[])],workers};
}
