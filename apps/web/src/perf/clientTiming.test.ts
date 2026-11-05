import {describe,expect,it,vi} from 'vitest';
import {ClientTiming,collectLoadMetrics,MAX_MS,type Deps,type TimingPayload} from './clientTiming.ts';

function deps(over:Partial<Deps>={}):{deps:Deps;sent:TimingPayload[];settle:()=>void;hide:()=>void;later:Array<()=>void>;measures:Array<{name:string;ms:number}>} {
 const sent:TimingPayload[]=[];const later:Array<()=>void>=[];const measures:Array<{name:string;ms:number}>=[];
 let settle=()=>{};let hide=()=>{};
 const entries=(type:string):PerformanceEntry[]=>{
  if(type==='navigation')return [{responseStart:41.6,domContentLoadedEventEnd:900.2,loadEventEnd:1500.9,name:'https://mail.example/inbox?token=secret'} as unknown as PerformanceEntry];
  if(type==='paint')return [{name:'first-paint',startTime:300},{name:'first-contentful-paint',startTime:350.4}] as PerformanceEntry[];
  return [];
 };
 const d:Deps={
  now:()=>1234.4,entries,supported:()=>true,random:()=>0.5,privacySignal:()=>false,device:()=>'desktop',
  whenSettled:fn=>{settle=fn;},onHide:fn=>{hide=fn;},
  policy:async()=>({enabled:true,sampleRatePercent:100,folderLoads:true,messageLoads:true}),
  kindEnabled:()=>true,afterPaint:fn=>{fn();},later:fn=>{later.push(fn);},measure:(n,a,b)=>{measures.push({name:n,ms:b-a});},
  send:async p=>{sent.push(p);},lcp:()=>2100.7,...over,
 };
 return {deps:d,sent,settle:()=>settle(),hide:()=>hide(),later,measures};
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
 it('reports no legacy folderOpen figure any more', async()=>{
  let t=1000;const h=deps({now:()=>t});const c=new ClientTiming(h.deps);
  await armed(c,h);
  t=5000;c.noteFolderSelected();c.noteList(true,10);t=5400;c.noteList(false,3);
  h.later.forEach(fn=>fn());await flush();
  expect(JSON.stringify(h.sent)).not.toContain('folderOpen');
 });
});

async function armed(c:ClientTiming,h:ReturnType<typeof deps>):Promise<void>{
 c.noteList(false,10);await flush();
 // Drain the page-load beacon so interaction beacons stand alone.
 h.settle();await flush();h.sent.length=0;
}

describe('folder loads',()=>{
 it('measures a cold folder from the pick to the painted list, every time',async()=>{
  let t=1000;const h=deps({now:()=>t});const c=new ClientTiming(h.deps);await armed(c,h);
  t=5000;c.noteFolderSelected();c.noteList(true,10);t=5400;c.noteList(false,3);
  t=7000;c.noteFolderSelected();c.noteList(true,3);t=7900;c.noteList(false,8);
  expect(h.sent).toEqual([]);// batched, not one beacon per open
  h.later.forEach(fn=>fn());await flush();
  expect(h.sent).toEqual([{device:'desktop',metrics:[{name:'folderLoadCold',ms:400},{name:'folderLoadCold',ms:900}]}]);
  expect(h.measures.map(m=>m.name)).toEqual(['mw:folder','mw:folder']);
 });
 it('calls a list that was already there when the pick settled cached',async()=>{
  let t=1000;const h=deps({now:()=>t});const c=new ClientTiming(h.deps);await armed(c,h);
  t=5000;c.noteFolderSelected();t=5016;c.noteList(false,25);
  h.later.forEach(fn=>fn());await flush();
  expect(h.sent[0]!.metrics).toEqual([{name:'folderLoadCached',ms:16}]);
 });
 it('ignores empty folders, and a pick before the page was usable',async()=>{
  let t=1000;const h=deps({now:()=>t});const c=new ClientTiming(h.deps);
  c.noteFolderSelected();c.noteList(true,0);c.noteList(false,5);// not armed yet: this is the page load
  await armed(c,h);
  t=5000;c.noteFolderSelected();c.noteList(false,0);
  h.later.forEach(fn=>fn());await flush();
  expect(h.sent).toEqual([]);
 });
 it('sends nothing when the folder feature is off server side or for this person',async()=>{
  let t=1000;
  const server=deps({now:()=>t,policy:async()=>({enabled:true,sampleRatePercent:100,folderLoads:false,messageLoads:true})});
  const a=new ClientTiming(server.deps);await armed(a,server);
  t=5000;a.noteFolderSelected();a.noteList(false,5);server.later.forEach(fn=>fn());await flush();
  expect(server.sent).toEqual([]);
  let on=true;
  const user=deps({now:()=>t,kindEnabled:k=>k!=='folder'||on});
  const b=new ClientTiming(user.deps);await armed(b,user);
  on=false;// opted out in Settings after load: no reload needed
  t=6000;b.noteFolderSelected();b.noteList(false,5);user.later.forEach(fn=>fn());await flush();
  expect(user.sent).toEqual([]);
 });
 it('a policy from an older server (no flags) records no interactions',async()=>{
  const old=deps({policy:async()=>({enabled:true,sampleRatePercent:100})});const c=new ClientTiming(old.deps);await armed(c,old);
  c.noteFolderSelected();c.noteList(false,5);const m=c.beginMessage();m.done(true);
  old.later.forEach(fn=>fn());await flush();
  expect(old.sent).toEqual([]);
 });
});

describe('message loads',()=>{
 it('measures an opened message to its painted body, cold and cached, in one batch',async()=>{
  let t=1000;const h=deps({now:()=>t});const c=new ClientTiming(h.deps);await armed(c,h);
  t=2000;const cold=c.beginMessage();t=2700;cold.done(false);
  t=3000;const warm=c.beginMessage();t=3012;warm.done(true);
  h.later.forEach(fn=>fn());await flush();
  expect(h.sent).toEqual([{device:'desktop',metrics:[{name:'messageLoadCold',ms:700},{name:'messageLoadCached',ms:12}]}]);
  expect(h.measures.map(m=>m.name)).toEqual(['mw:message','mw:message']);
 });
 it('a superseded open is dropped, and done twice counts once',async()=>{
  let t=1000;const h=deps({now:()=>t});const c=new ClientTiming(h.deps);await armed(c,h);
  const first=c.beginMessage();t=1100;const second=c.beginMessage();t=1500;
  first.done(false);second.done(false);second.done(false);
  h.later.forEach(fn=>fn());await flush();
  expect(h.sent[0]!.metrics).toEqual([{name:'messageLoadCold',ms:400}]);
 });
 it('carries nothing that identifies a message or folder',async()=>{
  const h=deps();const c=new ClientTiming(h.deps);await armed(c,h);
  const m=c.beginMessage();m.done(false);c.noteFolderSelected();c.noteList(false,3);
  h.later.forEach(fn=>fn());await flush();
  const keys=new Set(h.sent.flatMap(b=>[...Object.keys(b),...b.metrics.flatMap(m=>Object.keys(m))]));
  expect([...keys].sort()).toEqual(['device','metrics','ms','name']);
 });
 it('is inert when the message feature is off or the page was never usable',async()=>{
  const off=deps({kindEnabled:k=>k!=='message'});const c=new ClientTiming(off.deps);await armed(c,off);
  c.beginMessage().done(false);off.later.forEach(fn=>fn());await flush();
  expect(off.sent).toEqual([]);
  const fresh=deps();const d=new ClientTiming(fresh.deps);
  expect(()=>d.beginMessage().done(true)).not.toThrow();
  fresh.later.forEach(fn=>fn());await flush();
  expect(fresh.sent).toEqual([]);
 });
 it('holds at most one beacon of samples',async()=>{
  const h=deps();const c=new ClientTiming(h.deps);await armed(c,h);
  for(let i=0;i<40;i++)c.beginMessage().done(false);
  h.later.forEach(fn=>fn());await flush();
  expect(h.sent[0]!.metrics.length).toBeLessThanOrEqual(12);
 });
});
