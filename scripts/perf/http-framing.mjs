// Small, isolated HTTP/1 framing corpus. This does not generate malformed TCP/IP.
import net from 'node:net';
import {writeFile,mkdir} from 'node:fs/promises';
import {dirname,resolve} from 'node:path';

const args=Object.fromEntries(process.argv.slice(2).map(arg=>{const at=arg.indexOf('=');if(at<0)throw new Error('Use --name=value');return[arg.slice(2,at),arg.slice(at+1)];}));
for(const name of Object.keys(args))if(!['base','timeout','slow-header','output'].includes(name))throw new Error(`Unknown option: ${name}`);
const base=new URL(args.base??'http://127.0.0.1:42214');
if(base.protocol!=='http:'||!['127.0.0.1','[::1]'].includes(base.hostname)||base.username||base.password||base.pathname!=='/'||base.search||base.hash)throw new Error('Framing tests require an explicit isolated loopback HTTP origin');
const timeout=Number(args.timeout??5000);if(!Number.isFinite(timeout)||timeout<100||timeout>120000)throw new Error('Invalid timeout');
const common='Host: localhost\r\nConnection: close\r\n';
const corpus=[
 ['invalid-method',`G\0ET /healthz HTTP/1.1\r\n${common}\r\n`],
 ['conflicting-lengths',`GET /healthz HTTP/1.1\r\n${common}Content-Length: 1\r\nContent-Length: 2\r\n\r\nxx`],
 ['invalid-length',`GET /healthz HTTP/1.1\r\n${common}Content-Length: xyz\r\n\r\n`],
 ['invalid-header-name',`GET /healthz HTTP/1.1\r\n${common}Bad\0Name: value\r\n\r\n`],
 ['oversized-header',`GET /healthz HTTP/1.1\r\n${common}X-Oversized: ${'x'.repeat(96*1024)}\r\n\r\n`],
];
if(args['slow-header']==='true')corpus.push(['incomplete-header','GET /healthz HTTP/1.1\r\nHost: localhost\r\nX-Incomplete: ']);
async function probe(name,wire){return new Promise(resolve=>{
 const started=performance.now();let finished=false,received=Buffer.alloc(0);
 const socket=net.connect({host:base.hostname.replace(/^\[|\]$/g,''),port:Number(base.port||80)});
 const timer=setTimeout(()=>finish(true),timeout);
 function finish(timedOut=false,error){if(finished)return;finished=true;clearTimeout(timer);socket.destroy();const match=/^HTTP\/1\.[01] (\d{3})/.exec(received.toString('latin1'));const status=match?Number(match[1]):null;resolve({name,status,elapsedMs:performance.now()-started,timedOut,socketError:error??null,refused:!timedOut&&((status===null&&received.length===0&&(!error||['ECONNRESET','EPIPE'].includes(error)))||[400,408,413,414,431,501,505].includes(status))});}
 socket.on('connect',()=>socket.write(wire));socket.on('data',chunk=>{received=Buffer.concat([received,chunk]).subarray(0,8192);if(received.includes('\r\n\r\n'))finish();});socket.on('end',()=>finish());socket.on('close',()=>finish());socket.on('error',error=>finish(false,error.code));
});}
async function healthy(){const response=await fetch(new URL('/healthz',base),{redirect:'error',signal:AbortSignal.timeout(timeout)});await response.body?.cancel();return response.status===200;}
const report={generatedAt:new Date().toISOString(),target:base.origin,healthyBefore:await healthy(),cases:[]};
if(!report.healthyBefore)throw new Error('Control health request failed');
for(const [name,wire]of corpus){const result=await probe(name,wire);report.cases.push(result);console.log(`${name}: status=${result.status??'closed'}, refused=${result.refused}, timeout=${result.timedOut}, elapsed=${Math.ceil(result.elapsedMs)}ms`);}
report.healthyAfter=await healthy();const output=resolve(args.output??'.orchestration/reports/http-framing.json');await mkdir(dirname(output),{recursive:true});await writeFile(output,JSON.stringify(report,null,2)+'\n');
if(!report.healthyAfter||report.cases.some(result=>!result.refused))process.exitCode=1;
