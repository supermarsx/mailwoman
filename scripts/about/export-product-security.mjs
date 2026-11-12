// Export the files from the exact Cargo build-script invocation. No globbing
// across old target directories and no network fetches of advisory data.
import {spawnSync} from 'node:child_process';
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {resolve,join} from 'node:path';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';
export function validateInventory(metadata,bom,lock){
 if(bom.bomFormat!=='CycloneDX'||bom.specVersion!=='1.6')throw new Error('Unexpected SBOM format');
 const hash=createHash('sha256').update(lock).digest('hex');
 if(metadata.cargoLockSha256!==hash)throw new Error('SBOM does not match the current Cargo.lock');
 if(bom.metadata?.properties?.find(p=>p.name==='mailwoman:cargo-lock-sha256')?.value!==hash)throw new Error('SBOM source hash mismatch');
 const refs=new Set(bom.components.map(c=>c['bom-ref']));
 if(refs.size!==bom.components.length||metadata.componentCount!==refs.size)throw new Error('Duplicate or missing components');
 for(const edge of bom.dependencies??[])for(const ref of [edge.ref,...edge.dependsOn])if(!refs.has(ref))throw new Error('Unresolved component reference');
}
export function main(args=process.argv.slice(2)){
 const at=args.indexOf('--out');if(at<0||!args[at+1])throw new Error('Use --out <artifact-directory>');
 const output=resolve(args[at+1]);
 const meta=spawnSync('cargo',['metadata','--no-deps','--locked','--offline','--format-version','1'],{encoding:'utf8',maxBuffer:16*1024*1024});if(meta.status!==0)throw new Error('Cargo metadata failed');
 const id=JSON.parse(meta.stdout).packages.find(p=>p.name==='mw-server')?.id;if(!id)throw new Error('Server package missing');
 const run=spawnSync('cargo',['check','-p','mw-server','--lib','--locked','--offline','--message-format=json'],{encoding:'utf8',maxBuffer:64*1024*1024});
 if(run.status!==0){process.stderr.write(run.stderr??'');throw new Error('Product inventory build failed');}
 const messages=run.stdout.split(/\r?\n/).filter(Boolean).map(line=>JSON.parse(line));
 const built=messages.findLast(m=>m.reason==='build-script-executed'&&m.package_id===id);if(!built?.out_dir)throw new Error('Exact build output directory missing');
 const raw=readFileSync(join(built.out_dir,'software-bom.cdx.json'));
 const metadata=JSON.parse(readFileSync(join(built.out_dir,'product-security.json'),'utf8'));
 const bom=JSON.parse(raw);validateInventory(metadata,bom,readFileSync('Cargo.lock'));
 if(metadata.sbomSha256!==createHash('sha256').update(raw).digest('hex'))throw new Error('Inventory digest mismatch');
 mkdirSync(output,{recursive:true});writeFileSync(join(output,'software-bom.cdx.json'),raw);writeFileSync(join(output,'product-security.json'),JSON.stringify(metadata,null,2)+'\n');
 writeFileSync(join(output,'SECURITY.md'),readFileSync('SECURITY.md'));
 writeFileSync(join(output,'SHA256SUMS'),['software-bom.cdx.json','product-security.json','SECURITY.md'].map(file=>`${createHash('sha256').update(readFileSync(join(output,file))).digest('hex')}  ${file}`).join('\n')+'\n');
 process.stdout.write(`Exported ${metadata.componentCount} workspace components and build facts.\n`);
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href){try{main();}catch(error){process.stderr.write(`${error.message}\n`);process.exitCode=1;}}
