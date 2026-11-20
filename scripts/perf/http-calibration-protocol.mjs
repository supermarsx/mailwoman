// Synthetic transport calibration only. No Mailwoman authentication or data.
export const JSON_PAYLOAD = Object.freeze({calibration:'constant-response',padding:'x'.repeat(4096)});
export const JSON_BYTES = Buffer.from(JSON.stringify(JSON_PAYLOAD));
export const BINARY_BYTES = Buffer.alloc(65536);
for(let i=0;i<BINARY_BYTES.length;i++)BINARY_BYTES[i]=[65,66,67,68][i%4];

export function calibrationHandoff(value,now=Date.now()){
 const base=new URL(value?.base??'invalid:');
 if(base.protocol!=='http:'||base.hostname!=='127.0.0.1'||!base.port||base.pathname!=='/'||base.search||base.hash||base.username||base.password||value?.schema!==1||value?.kind!=='synthetic-transport-calibration'||typeof value.token!=='string'||!/^[a-f0-9]{64}$/.test(value.token)||!Number.isSafeInteger(value.expiresAtMs)||value.expiresAtMs<=now||value.expiresAtMs>now+901000)throw new Error('Invalid private calibration handoff');
 return {base:base.origin,token:value.token,expiresAtMs:value.expiresAtMs};
}

export function calibrationOptions(args){
 if(Object.keys(args).some(key=>!['handoff','output','seconds','rates','concurrency'].includes(key))||typeof args.handoff!=='string'||!args.handoff||typeof args.output!=='string'||!args.output)throw new Error('Explicit private handoff and output paths required');
 const seconds=Number(args.seconds??3),concurrency=Number(args.concurrency??256),rates=String(args.rates??'10000,20000,40000,60000,80000').split(',').map(Number);
 if(!Number.isFinite(seconds)||seconds<.1||seconds>30||!Number.isInteger(concurrency)||concurrency<1||concurrency>1024||!rates.length||rates.length>5||new Set(rates).size!==rates.length||rates.some(rate=>!Number.isInteger(rate)||rate<100||rate>80000))throw new Error('Calibration options exceed bounded limits');
 return {seconds,concurrency,rates};
}

export function calibrationRequests(){
 return [
  {name:'synthetic-json-4k',path:'/json',validate:value=>value?.calibration===JSON_PAYLOAD.calibration&&value.padding===JSON_PAYLOAD.padding},
  {name:'synthetic-binary-64k',path:'/binary',validateBytes:bytes=>bytes.length===BINARY_BYTES.length&&bytes.equals(BINARY_BYTES)},
 ];
}

export function calibrationReport(options,stages){
 return {schema:1,kind:'synthetic-transport-calibration',applicationCapacityMeasured:false,securityProof:false,mailwomanCapacityClaim:false,distinctPhysicalGeneratorHostsMeasured:false,interpretation:'Generator and constant-response transport only; not Mailwoman capacity, authentication, policies, mail operations or upstream performance.',runtime:process.version,options,payloadBytes:{json:JSON_BYTES.length,binary:BINARY_BYTES.length},stages};
}
