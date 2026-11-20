import {join,dirname} from 'node:path';
import {fileURLToPath} from 'node:url';

export function generatorWorkers(value=1){
 const workers=Number(value);
 if(!Number.isInteger(workers)||workers<1||workers>16)throw new Error('HTTP generator workers must be 1–16');
 return workers;
}
// Only private handoff/report file paths become arguments, never credentials.
export function httpGeneratorLaunch({directory,ready,workers=1}){
 workers=generatorWorkers(workers);
 const scripts=dirname(fileURLToPath(import.meta.url)),handoff=join(directory,ready.handoffFile),output=join(directory,ready.outputFile);
 return workers===1
  ?{script:join(scripts,'http-real-fixture.mjs'),args:[handoff,output],workers}
  :{script:join(scripts,'http-multi-controller.mjs'),args:[`--handoff=${handoff}`,`--output=${output}`,`--workers=${workers}`],workers};
}
