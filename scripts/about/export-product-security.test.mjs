import {test} from 'node:test';import assert from 'node:assert/strict';import {createHash} from 'node:crypto';import {validateInventory,validatePublicSources} from './export-product-security.mjs';
const lock=Buffer.from('version = 4');const digest=createHash('sha256').update(lock).digest('hex');
function sample(){return {bom:{bomFormat:'CycloneDX',specVersion:'1.6',metadata:{properties:[{name:'mailwoman:cargo-lock-sha256',value:digest}]},components:[{'bom-ref':'a'}],dependencies:[{ref:'a',dependsOn:[]}]},metadata:{cargoLockSha256:digest,componentCount:1}};}
test('accepts a source-bound inventory with resolved dependencies',()=>{const s=sample();validateInventory(s.metadata,s.bom,lock);});
test('refuses stale source, duplicate components and dangling references',()=>{let s=sample();assert.throws(()=>validateInventory(s.metadata,s.bom,Buffer.from('changed')));s=sample();s.bom.components.push({'bom-ref':'a'});assert.throws(()=>validateInventory(s.metadata,s.bom,lock));s=sample();s.bom.dependencies[0].dependsOn=['missing'];assert.throws(()=>validateInventory(s.metadata,s.bom,lock));});
test('exports only the exact compiled policy and current frontend lock hashes',()=>{
 const policy=Buffer.from('# Security\n'),pnpm=Buffer.from('lockfileVersion: 9\n');
 const sha=value=>createHash('sha256').update(value).digest('hex');
 const metadata={securityPolicySha256:sha(policy),pnpmLockSha256:sha(pnpm)};
 validatePublicSources(metadata,policy,pnpm);
 assert.throws(()=>validatePublicSources(metadata,Buffer.from('different policy'),pnpm));
 assert.throws(()=>validatePublicSources(metadata,policy,Buffer.from('different lock')));
 assert.throws(()=>validatePublicSources(metadata,policy,null));
 validatePublicSources({...metadata,pnpmLockSha256:null},policy,null);
});
