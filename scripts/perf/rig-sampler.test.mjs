import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,writeFile,readFile,rm} from 'node:fs/promises';
import {join} from 'node:path';
import os from 'node:os';
import {discover,sampleOnce,parseArgs,run} from './rig-sampler.mjs';
import {parseSamples} from './rig-cgroup.mjs';

const ID=n=>String(n).repeat(64).slice(0,64);
const stat=usage=>`usage_usec ${usage}\nuser_usec ${usage}\nsystem_usec 0\nnr_periods 0\nnr_throttled 0\nthrottled_usec 0\n`;

async function tree(){
 const root=await mkdtemp(join(os.tmpdir(),'rig-sampler-'));
 // cgroupfs layout (docker/<id>) and systemd layout (system.slice/docker-<id>.scope).
 await mkdir(join(root,'docker',ID(1)),{recursive:true});await writeFile(join(root,'docker',ID(1),'cpu.stat'),stat(100));
 await mkdir(join(root,'system.slice',`docker-${ID(2)}.scope`),{recursive:true});await writeFile(join(root,'system.slice',`docker-${ID(2)}.scope`,'cpu.stat'),stat(200));
 await mkdir(join(root,'docker',ID(3)),{recursive:true});await writeFile(join(root,'docker',ID(3),'cpu.stat'),stat(300));
 await mkdir(join(root,'user.slice'),{recursive:true});
 await writeFile(join(root,'proc-stat'),'cpu  1 0 1 10 0 0 0 0 0 0\ncpu0 1 0 1 10 0 0 0 0 0 0\n');
 return root;
}

test('discovery finds container cgroups in both layouts; a sample separates this run from other containers',async()=>{
 const root=await tree();
 try{
  const found=await discover(root);
  assert.deepEqual([...found.keys()].sort(),[ID(1),ID(2),ID(3)]);
  const sample=await sampleOnce({labels:{app:ID(1),generator:ID(2),database:ID(9)},discovered:found,procStatPath:join(root,'proc-stat'),now:()=>42});
  assert.equal(sample.tMs,42);assert.equal(sample.containers.app.cpu.usage_usec,100);assert.equal(sample.containers.generator.cpu.usage_usec,200);
  assert.equal(sample.containers.database.error,'cgroup not found');
  assert.deepEqual(sample.others,{[ID(3).slice(0,12)]:{usage_usec:300}});
  assert.deepEqual(sample.procStat.cpus,{0:[2,12]});
 }finally{await rm(root,{recursive:true,force:true});}
});

test('the loop writes parseable JSON lines and stops; arguments are strict',async()=>{
 const root=await tree();
 try{
  const out=join(root,'samples.jsonl');let n=0;
  const lines=await run({root,out,procStatPath:join(root,'proc-stat'),intervalMs:50,maxSeconds:10,rediscoverMs:100,labels:{app:ID(1)}},{stopped:()=>++n>3});
  const {samples,malformed}=parseSamples(await readFile(out,'utf8'));
  assert.equal(malformed,0);assert.equal(samples.length,lines);assert.ok(samples.length>=3);
  assert.ok(samples.every(s=>s.containers.app.cpu.usage_usec===100));
 }finally{await rm(root,{recursive:true,force:true});}
 assert.deepEqual(parseArgs(['--root=/r','--out=/o',`--label=app=${ID(1)}`]).labels,{app:ID(1)});
 for(const bad of [['--root=/r','--out=/o'],['--root=/r','--out=/o','--label=app=xyz'],['--root=/r','--out=/o',`--label=app=${ID(1)}`,'--interval-ms=10'],['--bogus=1']])assert.throws(()=>parseArgs(bad));
});
