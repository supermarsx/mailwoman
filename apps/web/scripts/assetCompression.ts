import {brotliCompressSync,constants,gzipSync} from 'node:zlib';
import {constants as fsConstants,closeSync,fstatSync,lstatSync,openSync,readSync,readdirSync,renameSync,unlinkSync,writeFileSync} from 'node:fs';
import {randomUUID} from 'node:crypto';
import {join,resolve} from 'node:path';
import type {Plugin} from 'vite';
const MAX_FILES=8192,MAX_FILE_BYTES=16*1024*1024,MAX_SOURCE_BYTES=256*1024*1024,MAX_ENTRIES=20000;
function readExact(path:string,length:number):Buffer {
 const fd=openSync(path,fsConstants.O_RDONLY|(fsConstants.O_NOFOLLOW??0));
 try{
  const stat=fstatSync(fd);if(!stat.isFile()||stat.size!==length)throw new Error('Static compression source changed during build');
  const buffer=Buffer.alloc(length+1);let at=0;
  while(at<buffer.length){const count=readSync(fd,buffer,at,buffer.length-at,null);if(count===0)break;at+=count;}
  if(at!==length)throw new Error('Static compression source changed during build');
  return buffer.subarray(0,at);
 }finally{closeSync(fd);}
}
function unchanged(path:string,bytes:Buffer):boolean {
 try{
  const stat=lstatSync(path);
  return stat.isFile()&&!stat.isSymbolicLink()&&stat.size===bytes.length&&readExact(path,bytes.length).equals(bytes);
 }catch(error){if((error as NodeJS.ErrnoException).code==='ENOENT')return false;throw error;}
}
/** Called AFTER raw SRI/service-worker digests; compressed sidecars are selected
 * at the SAME public URL by the server, never extra SW precache requests. */
export function writeCompressedAssets(outDir:string):{files:number;sourceBytes:number;gzipBytes:number;brotliBytes:number}{
 let entries=0,sourceBytes=0;const files:{path:string;length:number}[]=[];
 const visit=(directory:string):void=>{
  for(const item of readdirSync(directory,{withFileTypes:true})){
   if(++entries>MAX_ENTRIES)throw new Error('Static compression entry bound exceeded');
   if(item.isSymbolicLink())throw new Error('Static compression refuses symlinks');
   const path=join(directory,item.name);
   if(item.isDirectory()){visit(path);continue;}
   if(!item.isFile())throw new Error('Static compression requires regular files');
   if(!/\.(?:m?js|css|ftl|json|svg|wasm)$/.test(item.name)||/\.map\./.test(item.name))continue;
   const length=lstatSync(path).size;
   if(length<1024||length>MAX_FILE_BYTES)continue;
   sourceBytes+=length;if(sourceBytes>MAX_SOURCE_BYTES||files.length>=MAX_FILES)throw new Error('Static compression source bound exceeded');
   files.push({path,length});
  }
 };visit(outDir);
 let gzipBytes=0,brotliBytes=0;
 for(const {path,length} of files){
  // Exact-size bounded reads: a growing output file cannot allocate beyond the
  // declared per-file cap before the consistency check rejects it.
  const input=readExact(path,length);
  const limit=Math.min(Math.floor(input.length*.95),input.length-128);
  for(const [suffix,bytes]of [['.gz',gzipSync(input,{level:6})],['.br',brotliCompressSync(input,{params:{[constants.BROTLI_PARAM_QUALITY]:5,[constants.BROTLI_PARAM_LGWIN]:22}})]] as const){
   if(bytes.length>limit)continue;
   const destination=path+suffix,temporary=destination+'.tmp-'+randomUUID();
   if(!unchanged(destination,bytes)){
    try{writeFileSync(temporary,bytes,{flag:'wx',mode:0o644});renameSync(temporary,destination);}finally{try{unlinkSync(temporary);}catch(error){if((error as NodeJS.ErrnoException).code!=='ENOENT')throw error;}}
   }
   if(suffix==='.gz')gzipBytes+=bytes.length;else brotliBytes+=bytes.length;
  }
 }
 return {files:files.length,sourceBytes,gzipBytes,brotliBytes};
}
export function assetCompression():Plugin{
 let outDir='',written=false;
 return {name:'mw-static-compression',apply:'build',enforce:'post',
  configResolved(config){outDir=resolve(config.root,config.build.outDir);},
  writeBundle(){written=true;},
  closeBundle:{order:'post',sequential:true,handler(){if(written)writeCompressedAssets(outDir);}},
 };
}
