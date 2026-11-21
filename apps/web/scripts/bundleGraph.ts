import {createHash} from 'node:crypto';
import {mkdirSync,readFileSync,writeFileSync} from 'node:fs';
import {dirname,join,relative,resolve} from 'node:path';
import type {Plugin} from 'vite';
import {inspectSolidTemplates,inspectStandaloneAsset,solidTemplateGraphProof,solidModuleSources,solidTemplateAssetCoverage} from './solidTemplateContract.mjs';
/** Build-only proof of rendered module placement; never emitted as a web asset. */
export function bundleGraph():Plugin{
 let root='',outDir='',destination='';
 return {name:'mw-bundle-graph',apply:'build',enforce:'post',
  configResolved(config){root=config.root;outDir=resolve(root,config.build.outDir);destination=join(root,'node_modules/.cache/mailwoman-build/bundle-graph.json');},
  async writeBundle(_options,bundle){
   const attestations:Record<string,unknown>={};
   let totalCode=0,totalTemplates=0;
   const chunks:Record<string,unknown>={};
   const sourceClosure:Record<string,{dependencies:string[];result:ReturnType<typeof inspectSolidTemplates>}>={};
   const sourceKeys=new Map<string,string>();
   const sourceKey=(id:string)=>createHash('sha256').update(id).digest('hex');
   const inspectSource=async(code:string,ast:unknown,id?:string)=>{
    const solidSources:string[]=[];
    for(const specifier of solidModuleSources(ast)){
     const resolved=await this.resolve(specifier,id);
     const path=resolved?.id.replaceAll('\\','/').split('?')[0]??'';
     if(/\/node_modules\/solid-js\/web\//.test(path))solidSources.push(specifier);
    }
    let result:ReturnType<typeof inspectSolidTemplates>;
    try{result=inspectSolidTemplates(code,ast,solidSources);}
    catch(error){
     const name=id?relative(root,id).replaceAll('\\','/').slice(0,512):'[emitted asset]';
     throw new Error(`Solid template source ${name}: ${(error instanceof Error?error.message:'unknown inspection failure').slice(0,1024)}`,{cause:error});
    }
    totalTemplates+=result.templates.length;
    if(totalTemplates>100000)throw Error('Template count bound');
    return result;
   };
   const pending:string[]=[],scheduled=new Set<string>();
   const enqueue=(id:string)=>{
    if(scheduled.has(id))return;
    if(scheduled.size>=20000)throw Error('Template source closure bound');
    scheduled.add(id);pending.push(id);
   };
   for(const item of Object.values(bundle))if(item.type==='chunk')for(const id of Object.keys(item.modules))enqueue(id);
   while(pending.length){
    const id=pending.pop()!;
    if(sourceKeys.has(id))continue;
    if(sourceKeys.size>=20000)throw Error('Template source closure bound');
    const key=sourceKey(id);
    sourceKeys.set(id,key);
    const source=this.getModuleInfo(id);
    if(!source||source.isExternal)throw Error('Uninspectable source dependency');
    const dependencies=[...source.importedIds,...source.dynamicallyImportedIds];
    if(dependencies.length>4096)throw Error('Template dependency count bound');
    for(const dependency of dependencies)enqueue(dependency);
    if(typeof source.code!=='string')throw Error('Source dependency has no inspectable code');
     const codeBytes=Buffer.byteLength(source.code);
     if(codeBytes>8*1024*1024)throw Error('Template module byte bound');
     totalCode+=codeBytes;
     if(totalCode>128*1024*1024)throw Error('Template proof source bound');
     sourceClosure[key]={dependencies:dependencies.map(sourceKey),result:await inspectSource(source.code,source.ast??this.parse(source.code),id)};
   }
   const assets:Record<string,unknown>={};
   for(const chunk of Object.values(bundle)){
    if(chunk.type!=='chunk')continue;
    const bytes=readFileSync(join(outDir,chunk.fileName));
    const modules:Record<string,number>={};
    const proofs:Record<string,unknown>={};
    for(const [id,info] of Object.entries(chunk.modules)){
     const key=relative(root,id.replace(/^\0/,'').split('?')[0]!).replaceAll('\\','/');
     if(Object.hasOwn(modules,key))throw Error('Ambiguous normalized rendered module identity');
     modules[key]=info.renderedLength;
     const sourceId=sourceKeys.get(id)!;
     if(info.renderedLength===0){proofs[key]={status:'zero-rendered',renderedLength:0,sourceId};continue;}
     const result=sourceClosure[sourceId]!.result;
     if(result.status!=='rendered')throw Error('Rendered module has no inspectable code');
     // Conservative: all factories in a retained module are inspected, including
     // expressions later removed by tree shaking. No expression-liveness claim.
     proofs[key]={status:'rendered',codeSha256:result.codeSha256,sourceId,renderedLength:info.renderedLength};
    }
    attestations[chunk.fileName]={sha256:createHash('sha256').update(bytes).digest('hex'),byteLength:bytes.length,modules:proofs};
    chunks[chunk.fileName]={imports:chunk.imports,dynamicImports:chunk.dynamicImports,entry:chunk.isEntry,byteLength:bytes.length,sha256:createHash('sha256').update(bytes).digest('hex'),modules};
   }
   for(const item of Object.values(bundle)){
    if(item.type!=='asset'||! /\.(?:m?js)$/.test(item.fileName))continue;
    const bytes=readFileSync(join(outDir,item.fileName));
    if(bytes.length>8*1024*1024)throw Error('Standalone JS byte bound');
    totalCode+=bytes.length;
    if(totalCode>128*1024*1024)throw Error('Template asset source bound');
    const code=bytes.toString('utf8');
    assets[item.fileName]={...inspectStandaloneAsset(code,this.parse(code)),sha256:createHash('sha256').update(bytes).digest('hex'),byteLength:bytes.length};
   }
   const graph={schema:1,outDir,chunks,solidTemplates:{schema:1,chunks:attestations,sourceClosure,assets}};
   solidTemplateAssetCoverage(graph,Object.values(bundle).filter(item=>/\.(?:m?js)$/.test(item.fileName)).map(item=>item.fileName));
   solidTemplateGraphProof(graph);
   const encoded=JSON.stringify(graph,null,2)+'\n';
   if(Buffer.byteLength(encoded)>16*1024*1024)throw Error('Template graph byte bound');
   mkdirSync(dirname(destination),{recursive:true});
   writeFileSync(destination,encoded);
  },
 };
}
