import {createHash} from 'node:crypto';
import {mkdirSync,readFileSync,writeFileSync} from 'node:fs';
import {dirname,join,relative,resolve} from 'node:path';
import type {Plugin} from 'vite';
/** Build-only proof of rendered module placement; never emitted as a web asset. */
export function bundleGraph():Plugin{
 let root='',outDir='',destination='';
 return {name:'mw-bundle-graph',apply:'build',enforce:'post',
  configResolved(config){root=config.root;outDir=resolve(root,config.build.outDir);destination=join(root,'node_modules/.cache/mailwoman-build/bundle-graph.json');},
  writeBundle(_options,bundle){
   const chunks=Object.fromEntries(Object.values(bundle).filter(item=>item.type==='chunk').map(chunk=>{
    const bytes=readFileSync(join(outDir,chunk.fileName));
    return [chunk.fileName,{imports:chunk.imports,dynamicImports:chunk.dynamicImports,entry:chunk.isEntry,byteLength:bytes.length,sha256:createHash('sha256').update(bytes).digest('hex'),modules:Object.fromEntries(Object.entries(chunk.modules).map(([id,info])=>[relative(root,id.replace(/^\0/, '').split('?')[0]!).replaceAll('\\','/'),info.renderedLength]))}];
   }));
   mkdirSync(dirname(destination),{recursive:true});
   writeFileSync(destination,JSON.stringify({schema:1,outDir,chunks},null,2)+'\n');
  },
 };
}
