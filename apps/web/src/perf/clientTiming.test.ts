import {describe,expect,it,vi} from 'vitest';
import {ClientTiming,collectLoadMetrics,MAX_MS,type Deps,type TimingPayload} from './clientTiming.ts';

function deps(over:Partial<Deps>={}):{deps:Deps;sent:TimingPayload[];settle:()=>void;hide:()=>void} {
 const sent:TimingPayload[]=[];
 let settle=()=>{};let hide=()=>{};
 const entries=(type:string):PerformanceEntry[]=>{
  if(type==='navigation')return [{responseStart:41.6,domContentLoadedEventEnd:900.2,loadEventEnd:1500.9,name:'https://mail.example/inbox?token=secret'} as unknown as PerformanceEntry];
  if(type==='paint')return [{name:'first-paint',startTime:300},{name:'first-contentful-paint',startTime:350.4}] as PerformanceEntry[];
  return [];
 };
 const d:Deps={
  now:()=>1234.4,entries,supported:()=>true,random:()=>0.5,privacySignal:()=>false,device:()=>'desktop',
  whenSettled:fn=>{settle=fn;},onHide:fn=>{hide=fn;},
  policy:async()=>({enabled:true,sampleRatePercent:100}),
  send:async p=>{sent.push(p);},lcp:()=>2100.7,...over,
 };
 return {deps:d,sent,settle:()=>settle(),hide:()=>hide()};
}
const flush=()=>new Promise(r=>setTimeout(r,0));

describe('collectLoadMetrics',()=>{
 it('reports whitelisted names with integer milliseconds only',()=>{
  const {deps:d}=deps();
  expect(collectLoadMetrics(d,1234)).toEqual([
   {name:'timeToUsable',ms:1234},{name:'ttfb',ms:42},{name:'dcl',ms:900},{name:'load',ms:1501},{name:'fcp',ms:350},{name:'lcp',ms:2101},
  ]);
 });
 it('clamps and drops values that are not finite or are negative',()=>{
  const {deps:d}=deps({entries:()=>[],lcp:()=>9e9});
  expect(collectLoadMetrics(d,Number.NaN)).toEqual([{name:'lcp',ms:MAX_MS}]);
  expect(collectLoadMetrics(d,-5)).toEqual([{name:'lcp',ms:MAX_MS}]);
 });
 it('does not throw when the performance entries are unavailable',()=>{
  const {deps:d}=deps({entries:()=>{throw new Error('unsupported');},lcp:()=>null});
  expect(collectLoadMetrics(d,10)).toEqual([{name:'timeToUsable',ms:10}]);
 });
});

describe('ClientTiming',()=>{
 it('sends one beacon with only a device class and metrics once the page settles',async()=>{
  const h=deps();const c=new ClientTiming(h.deps);
  c.noteList(true,0);c.noteList(false,25);
  expect(h.sent).toEqual([]);
  h.settle();await flush();h.hide();h.settle();await flush();
  expect(h.sent).toHaveLength(1);
  const body=h.sent[0]!;
  expect(Object.keys(body).sort()).toEqual(['device','metrics']);
  for(const m of body.metrics){expect(Object.keys(m).sort()).toEqual(['ms','name']);expect(Number.isInteger(m.ms)).toBe(true);}
  expect(JSON.stringify(body)).not.toMatch(/https?:|secret|token|example/);
 });
 it('sends on pagehide when the page never settles',async()=>{
  const h=deps();const c=new ClientTiming(h.deps);
  c.markUsable();h.hide();await flush();
  expect(h.sent).toHaveLength(1);
 });
 it('sends nothing before the list has rows (login and anonymous screens never mark)',async()=>{
  const h=deps();const c=new ClientTiming(h.deps);
  c.noteList(false,0);c.noteList(true,0);h.settle();h.hide();await flush();
  expect(h.sent).toEqual([]);
 });
 it('sends nothing when the server turned collection off',async()=>{
  const h=deps({policy:async()=>({enabled:false,sampleRatePercent:100})});const c=new ClientTiming(h.deps);
  c.markUsable();h.settle();await flush();
  expect(h.sent).toEqual([]);
 });
 it('sends nothing when the session is not in the sample',async()=>{
  const h=deps({policy:async()=>({enabled:true,sampleRatePercent:10}),random:()=>0.5});const c=new ClientTiming(h.deps);
  c.markUsable();h.settle();await flush();
  expect(h.sent).toEqual([]);
  const yes=deps({policy:async()=>({enabled:true,sampleRatePercent:60}),random:()=>0.5});
  new ClientTiming(yes.deps).markUsable();yes.settle();await flush();
  expect(yes.sent).toHaveLength(1);
 });
 it('honours Do Not Track and Global Privacy Control',async()=>{
  const policy=vi.fn(async()=>({enabled:true,sampleRatePercent:100}));
  const h=deps({privacySignal:()=>true,policy});const c=new ClientTiming(h.deps);
  c.markUsable();h.settle();h.hide();await flush();
  expect(h.sent).toEqual([]);expect(policy).not.toHaveBeenCalled();
 });
 it('sends nothing and does not throw when the browser lacks the APIs or the policy or send fails',async()=>{
  const none=deps({supported:()=>false});
  expect(()=>new ClientTiming(none.deps).markUsable()).not.toThrow();
  const broken=deps({policy:async()=>{throw new Error('401');}});
  new ClientTiming(broken.deps).markUsable();broken.settle();await flush();
  expect(broken.sent).toEqual([]);
  const failing=deps({send:async()=>{throw new Error('offline');}});
  new ClientTiming(failing.deps).markUsable();failing.settle();await flush();
  const throwing=deps({now:()=>{throw new Error('x');}});
  expect(()=>new ClientTiming(throwing.deps).noteList(false,3)).not.toThrow();
 });
 it('omits the device class when it cannot be told',async()=>{
  const h=deps({device:()=>undefined});new ClientTiming(h.deps).markUsable();h.settle();await flush();
  expect(h.sent[0]).not.toHaveProperty('device');
 });
 it('reports the first folder open after the page is usable, once',async()=>{
  let t=1000;const h=deps({now:()=>t});const c=new ClientTiming(h.deps);
  c.noteList(false,10);t=5000;
  c.noteFolderSelected();c.noteList(true,10);t=5400;c.noteList(false,3);
  await flush();
  expect(h.sent).toEqual([{device:'desktop',metrics:[{name:'folderOpen',ms:400}]}]);
  c.noteFolderSelected();c.noteList(true,3);t=9000;c.noteList(false,3);await flush();
  expect(h.sent).toHaveLength(1);
 });
});
