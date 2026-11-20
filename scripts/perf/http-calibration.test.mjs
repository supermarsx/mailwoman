import test from 'node:test';
import assert from 'node:assert/strict';
import {calibrationHandoff,calibrationOptions,calibrationRequests,calibrationReport,JSON_BYTES,BINARY_BYTES} from './http-calibration-protocol.mjs';
import {calibrationHandler} from './http-calibration-server.mjs';

const token='a'.repeat(64);
test('calibration accepts only bounded private owned-loopback handoffs',()=>{
 const valid={schema:1,kind:'synthetic-transport-calibration',base:'http://127.0.0.1:12345',token,expiresAtMs:10000};
 assert.equal(calibrationHandoff(valid,0).base,'http://127.0.0.1:12345');
 for(const patch of [{base:'http://localhost:12345'},{base:'http://example.test:12345'},{base:'http://127.0.0.1:12345/path'},{base:'http://user:secret@127.0.0.1:12345'},{base:'https://127.0.0.1:12345'},{expiresAtMs:0},{expiresAtMs:1000000},{token:'bad'},{kind:'mailwoman'}])assert.throws(()=>calibrationHandoff({...valid,...patch},0));
});
test('calibration phases, rates and outstanding requests stay bounded',()=>{
 const base={handoff:'private.json',output:'report.json'};
 assert.deepEqual(calibrationOptions(base),{seconds:3,concurrency:256,rates:[10000,20000,40000,60000,80000]});
 for(const patch of [{seconds:31},{seconds:0},{rates:'0'},{rates:'100001'},{rates:'100,100'},{rates:'100,200,300,400,500,600'},{concurrency:1025},{base:'http://arbitrary.test'}])assert.throws(()=>calibrationOptions({...base,...patch}));
});
test('synthetic validators require exact JSON contents and binary bytes',()=>{
 const [json,binary]=calibrationRequests();
 assert.equal(json.validate(JSON.parse(JSON_BYTES)),true);
 assert.equal(json.validate({calibration:'constant-response',padding:'wrong'}),false);
 assert.equal(binary.validateBytes(BINARY_BYTES),true);
 const corrupted=Buffer.from(BINARY_BYTES);corrupted[65535]=0;
 assert.equal(binary.validateBytes(corrupted),false);
 assert.equal(binary.validateBytes(BINARY_BYTES.subarray(0,65535)),false);
});
test('server authority, control and payload counters are separate and disclose no authority',()=>{
 let requests=0,bytes=0,resets=0;
 const handler=calibrationHandler({token,reset:()=>{resets++;},snapshot:()=>({requests,bytes}),record:size=>{requests++;bytes+=size;}});
 const invoke=(url,method='GET',authority=token)=>{
  const response={status:0,headers:{},body:null,writeHead(status,headers={}){this.status=status;this.headers=headers;},end(body){this.body=body;}};
  handler({url,method,headers:{'x-calibration-token':authority}},response);return response;
 };
 assert.equal(invoke('/json','GET','wrong').status,404);
 assert.equal(requests,0);
 assert.equal(invoke('/json').body.length,JSON_BYTES.length);
 assert.equal(invoke('/binary').body.length,65536);
 assert.equal(invoke('/binary','POST').status,404);
 assert.equal(invoke('/__reset','POST').status,204);
 assert.equal(resets,1);assert.equal(requests,2);
 const metrics=invoke('/__metrics');assert.deepEqual(JSON.parse(metrics.body),{requests:2,bytes:JSON_BYTES.length+65536});
 assert.ok(!metrics.body.includes(token));
});
test('calibration reports cannot establish application or security capacity',()=>{
 const report=calibrationReport({seconds:1,concurrency:1,rates:[100]},[{requests:100,accepted:100}]);
 assert.equal(report.applicationCapacityMeasured,false);assert.equal(report.securityProof,false);assert.equal(report.mailwomanCapacityClaim,false);
 assert.ok(!JSON.stringify(report).includes(token));
});
