import test from 'node:test';
import assert from 'node:assert/strict';
import {parseCpuset,formatCpuset,validatePlan,allocate,dockerCpuArgs,hostEnv} from './rig-cpuset.mjs';
import {hostFacts} from './http-goal.mjs';

test('cpusets parse and format strictly',()=>{
 assert.deepEqual(parseCpuset('0-3,8,10-11'),[0,1,2,3,8,10,11]);
 assert.equal(formatCpuset([11,10,8,3,2,1,0]),'0-3,8,10-11');
 assert.equal(formatCpuset([5]),'5');
 for(const bad of ['','a','1-','3-1','1,1','2,1','0-1,1-2','1024','1,,2',' 1',undefined,7])assert.throws(()=>parseCpuset(bad),String(bad));
 assert.throws(()=>formatCpuset([]));assert.throws(()=>formatCpuset([1,1]));
});

test('a plan is disjoint, inside the VM, and names the CPUs outside the run',()=>{
 const plan=validatePlan({app:'1-4',generator:'5-8',rig:'9'},{vcpus:26});
 assert.equal(plan.roles.app.cpus,4);assert.deepEqual(plan.usedCpus,[1,2,3,4,5,6,7,8,9]);
 assert.equal(plan.outsideCpuset,'0,10-25');assert.equal(plan.outsideCpus.length,17);
 assert.throws(()=>validatePlan({app:'1-4',generator:'4-8'},{vcpus:26}),/CPU 4 is in both app and generator/);
 assert.throws(()=>validatePlan({app:'1-4',generator:'25-26'},{vcpus:26}),/outside the 26 VM vCPUs/);
 assert.throws(()=>validatePlan({app:'1-4'},{vcpus:26}),/app and generator/);
 assert.throws(()=>validatePlan({app:'1',generator:'2',gpu:'3'},{vcpus:26}),/Unknown cpuset role/);
});

test('allocation is consecutive from CPU 1 in role order and refuses an oversized request',()=>{
 const plan=allocate({app:4,database:4,generator:4,rig:1},{vcpus:26});
 assert.deepEqual(Object.fromEntries(Object.entries(plan.roles).map(([role,v])=>[role,v.cpuset])),{app:'1-4',database:'5-8',generator:'9-12',rig:'13'});
 assert.equal(allocate({app:56,generator:16,rig:1},{vcpus:76}).roles.rig.cpuset,'73');
 assert.throws(()=>allocate({app:56,generator:16,rig:4},{vcpus:76}),/Not enough VM vCPUs/);
 assert.throws(()=>allocate({app:-1,generator:1},{vcpus:8}),/Invalid app CPU count/);
});

test('docker flags and MW_LOAD_HOST declarations match the plan and are what the report records',()=>{
 const plan=allocate({app:4,database:2,generator:4,upstream:1,rig:1},{vcpus:26});
 assert.deepEqual(dockerCpuArgs(plan,'app'),['--cpuset-cpus','1-4','--cpus','4']);
 assert.throws(()=>dockerCpuArgs(allocate({app:1,generator:1},{vcpus:4}),'database'),/No cpuset/);
 const env=hostEnv(plan);
 assert.deepEqual(env,{MW_LOAD_HOST_DOCKER_VCPUS:'26',MW_LOAD_HOST_CPUSET_APP:'1-4',MW_LOAD_HOST_CPUSET_GENERATOR:'7-10',MW_LOAD_HOST_CPUSET_DB:'5-6',MW_LOAD_HOST_CPUSET_UPSTREAM:'11',MW_LOAD_HOST_CPUSET_RIG:'12'});
 const facts=hostFacts(env,{availableParallelism:()=>80,cpus:()=>[],platform:()=>'linux',totalmem:()=>1});
 assert.equal(facts.dockerVmVcpus,26);
 assert.deepEqual(facts.cpusets,{app:'1-4',database:'5-6',generator:'7-10',upstream:'11',rig:'12'});
 // Without the new declarations the recorded object is unchanged.
 assert.deepEqual(hostFacts({},{availableParallelism:()=>1,cpus:()=>[],platform:()=>'linux',totalmem:()=>1}).cpusets,{app:null,database:null,generator:null});
});
