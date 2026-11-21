import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {parseAst} from 'vite';
import {bundleGraph} from './bundleGraph.ts';
import {inspectSolidTemplates,inspectStandaloneAsset,solidTemplateGraphProof,solidModuleSources,solidTemplateAssetCoverage} from './solidTemplateContract.mjs';

const inspect = code => inspectSolidTemplates(code,parseAst(code));
const factory = html => `import {template as t} from 'solid-js/web';const view=t(${JSON.stringify(html)});`;
function graph(code) {
  const sha256='a'.repeat(64);
  return {schema:1,chunks:{'assets/app.js':{sha256,byteLength:10,modules:{'src/App.tsx':50,'src/unused.ts':0}}},solidTemplates:{schema:1,chunks:{'assets/app.js':{sha256,byteLength:10,modules:{'src/App.tsx':{...inspect(code),renderedLength:50,sourceId:'app'},'src/unused.ts':{status:'zero-rendered',renderedLength:0,sourceId:'unused'}}}},sourceClosure:{app:{dependencies:['unused'],result:inspect(code)},unused:{dependencies:[],result:inspect('export const unused=1')}},assets:{}}};
}

test('actual parsed shell factory literal is attested, ordinary generated email HTML is ignored',()=>{
  const code=factory('<main class=shell><div></div></main>')+`export const email='<p style="color:red">mail</p>';export const build=()=>'<div style='+email+'>';`;
  const result=solidTemplateGraphProof(graph(code));
  assert.equal(result.templates,1);assert.deepEqual(result.violations,[]);
});
test('quoted, unquoted, uppercase and nested template-content styles are all DOM attributes',()=>{
  for(const html of ['<div style="color:red"></div>',"<div style='color:red'></div>",'<div STYLE=color:red></div>','<template><span style=color:red></span></template>']){
    assert.equal(solidTemplateGraphProof(graph(factory(html))).violations[0].styleAttributes,1);
  }
});
test('SVG/math flags and normal compiler import aliases use same literal inspection',()=>{
  for(const flags of ['',',true',',false,true',',false,false,true']) {
    assert.equal(inspect(`import {template as _$tmpl} from 'solid-js/web';const x=_$tmpl('<svg><path></path></svg>'${flags});`).templates.length,1);
  }
  // Exact pinned jsx-dom-expressions appendTemplates factory shape: static
  // TemplateLiteral plus three boolean flags (not a dynamic HTML expression).
  assert.equal(inspect('import {template as _$template} from "solid-js/web";var _tmpl$=/*#__PURE__*/_$template(`<div style=color:red>`, false, false, false);').templates[0].styleAttributes,1);
});
test('unexpected calls, binding escapes/shadowing, namespace and dynamic imports fail closed',()=>{
  for(const suffix of ['t(html)','t(`<div>${value}</div>`)','t("<div>",1)','t?.("<div>")','const other=t','t.call(null,"<div>")','function nested(t){return t("<div>")}']) {
    assert.throws(()=>inspect(`import {template as t} from 'solid-js/web';${suffix};`));
  }
  for(const code of ["import * as s from 'solid-js/web';s.template('<div>')","const s=import('solid-js/web')","const s=require('solid-js/web')","export {template} from 'solid-js/web'","export * from 'solid-js/web'"])assert.throws(()=>inspect(code));
});
test('malformed HTML/JS and missing AST fail rather than receiving a clean attestation',()=>{
  assert.throws(()=>inspect(factory('<div style="unterminated>')));
  assert.throws(()=>inspect('const x = ;'));
  assert.throws(()=>inspectSolidTemplates('const x=1',null));
});
test('normal imports/object keys do not masquerade as template variable references',()=>{
  assert.equal(inspect("import {template as t, render} from 'solid-js/web';const x={t:1};x.t;render(()=>null,root);").templates.length,0);
  assert.equal(inspect("export {isServer,render} from 'solid-js/web'").templates.length,0);
});
test('missing or incompatible graph proof and every chunk/module/hash coverage mismatch deny',()=>{
  const original=graph(factory('<div></div>'));
  for(const mutate of [g=>delete g.solidTemplates,g=>g.solidTemplates.schema=2,g=>delete g.solidTemplates.chunks['assets/app.js'],g=>g.solidTemplates.chunks['assets/app.js'].sha256='b'.repeat(64),g=>g.solidTemplates.chunks['assets/app.js'].byteLength++,g=>delete g.solidTemplates.chunks['assets/app.js'].modules['src/App.tsx'],g=>g.solidTemplates.chunks['assets/app.js'].modules.extra={status:'zero-rendered'},g=>g.solidTemplates.chunks['assets/app.js'].modules['src/unused.ts'].status='unknown',g=>g.chunks['assets/app.js'].modules['src/App.tsx']=51]) {
    const value=structuredClone(original);mutate(value);assert.throws(()=>solidTemplateGraphProof(value));
  }
});
test('rendered unknown/synthetic modules cannot use zero-rendered exemption',()=>{
  const value=graph(factory('<div></div>'));
  value.solidTemplates.chunks['assets/app.js'].modules['src/App.tsx']={status:'zero-rendered',renderedLength:50};
  assert.throws(()=>solidTemplateGraphProof(value));
});
test('bounded code/templates reject excessive inputs before HTML parse',()=>{
  assert.throws(()=>inspectSolidTemplates(' '.repeat(8*1024*1024+1),{type:'Program',body:[]}));
  assert.throws(()=>inspect(factory('<div>'+ 'a'.repeat(256*1024)+'</div>')));
});
test('resolved aliases/direct web entry roots cannot hide factory calls or re-export barrels',()=>{
  const code="import {template as t} from '@web-runtime';t('<div style=color:red>');";
  assert.deepEqual(solidModuleSources(parseAst(code)),['@web-runtime']);
  assert.equal(inspectSolidTemplates(code,parseAst(code),['@web-runtime']).templates[0].styleAttributes,1);
  const barrel="export * from '@web-runtime'";
  assert.throws(()=>inspectSolidTemplates(barrel,parseAst(barrel),['@web-runtime']));
  assert.equal(inspect("import {template as t} from 'solid-js/web/dist/web.js';t('<div style=color:red>')").templates[0].styleAttributes,1);
});
test('a retained module conservatively checks factories in dead exports too',()=>{
  const code=factory('<div></div>')+"export function unused(){return t('<div style=color:red>')}export const retained=1;";
  assert.equal(solidTemplateGraphProof(graph(code)).violations.length,1);
});

test('zero-rendered or absent forwarding barrels are included in source-closure proof',()=>{
  for(const code of ["export * from 'solid-js/web'", "export {template as t} from 'solid-js/web'"])assert.throws(()=>inspect(code));
  const value=graph("export const live=1");
  value.solidTemplates.sourceClosure.unused.dependencies=['forwarder'];
  assert.throws(()=>solidTemplateGraphProof(value),'missing forwarding source cannot be considered clean');
  value.solidTemplates.sourceClosure.forwarder={dependencies:[],result:{status:'unknown'}};
  assert.throws(()=>solidTemplateGraphProof(value));
});
test('every emitted JS asset requires exact parsed-output or chunk proof, without filename exemptions',()=>{
  const value=graph(factory('<div></div>'));
  const asset={...inspect('self.onmessage=()=>{}'),sha256:'b'.repeat(64),byteLength:30};
  value.solidTemplates.assets['assets/offline-background.js']=asset;
  assert.deepEqual(solidTemplateAssetCoverage(value,['assets/app.js','assets/offline-background.js']),[]);
  assert.throws(()=>solidTemplateAssetCoverage(value,['assets/app.js','assets/offline-background.js','assets/missing-lazy.js']));
  assert.throws(()=>solidTemplateAssetCoverage(value,['assets/app.js']));
  delete value.chunks['assets/app.js'];delete value.solidTemplates.chunks['assets/app.js'];
  assert.throws(()=>solidTemplateAssetCoverage(value,['assets/app.js','assets/offline-background.js']),'omitting both maps cannot hide real lazy JS');
  asset.status='unknown';
  assert.throws(()=>solidTemplateAssetCoverage(value,['assets/offline-background.js']));
});
test('standalone output with erased factory bindings or unresolved imports fails closed',()=>{
  const simple='self.onmessage=()=>self.postMessage({done:true})';
  assert.equal(inspectStandaloneAsset(simple,parseAst(simple)).templates.length,0);
  for(const code of ['function t(html){const el=document.createElement("template");el.innerHTML=html;return el}t("<div style=color:red>")',"import {a} from './vendor.js';a('<div style=color:red>')"]){
    assert.throws(()=>inspectStandaloneAsset(code,parseAst(code)));
  }
});

for(const includeZeroBarrel of [false,true])for(const opaque of [false,true])test(`actual producer rejects ${opaque?'opaque':'parsed'} forwarding barrel ${includeZeroBarrel?'with zero rendered bytes':'absent from chunk.modules'}`,async()=>{
  const root=mkdtempSync(join(tmpdir(),'mw-template-proof-'));
  try{
    mkdirSync(join(root,'dist/assets'),{recursive:true});
    writeFileSync(join(root,'dist/assets/app.js'),'const rendered=1;');
    const entry=join(root,'src/App.js');const barrel=join(root,'src/barrel.js');
    const runtime=join(root,'node_modules/solid-js/web/dist/web.js');
    const consumer="import {template as t} from './barrel.js';export const view=t('<div style=color:red>');";
    const forwarder="export * from 'solid-js/web'";
    const sources=new Map([[entry,{code:consumer,ast:parseAst(consumer),importedIds:[barrel],dynamicallyImportedIds:[]}],[barrel,{code:opaque?null:forwarder,ast:opaque?null:parseAst(forwarder),importedIds:[runtime],dynamicallyImportedIds:[]}]]);
    const plugin=bundleGraph();
    plugin.configResolved({root,build:{outDir:'dist'}});
    const modules={[entry]:{renderedLength:20},...(includeZeroBarrel?{[barrel]:{renderedLength:0}}:{})};
    await assert.rejects(plugin.writeBundle.call({getModuleInfo:id=>sources.get(id),resolve:async source=>({id:source==='solid-js/web'?runtime:barrel})},{},{'assets/app.js':{type:'chunk',fileName:'assets/app.js',imports:[],dynamicImports:[],isEntry:true,modules}}),opaque?/no inspectable code/:/Solid re-export/);
  }finally{rmSync(root,{recursive:true,force:true});}
});
