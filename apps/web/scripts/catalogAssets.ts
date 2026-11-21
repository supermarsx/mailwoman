import {fileURLToPath} from 'node:url';
import {readFileSync,readdirSync} from 'node:fs';
import {join} from 'node:path';
import {normalizePath,type Plugin} from 'vite';
import type {CatalogManifest} from '../src/i18n/catalogAssetSources.ts';
/** Tests use raw imports; production avoids executable translation chunks. */
export function catalogAssets(root=fileURLToPath(new URL('../',import.meta.url))):Plugin{
 const catalog=normalizePath(join(root,'src/i18n/catalog.ts'));
 const assets=join(root,'src/i18n/catalogAssetSources.ts');
 const locales=join(root,'locales');
 return {name:'mw-catalog-assets',apply:'build',enforce:'pre',
  resolveId(source,importer){
   if(process.env['MW_CATALOG_ASSETS']!=='0'&&source==='./catalogSources.ts'&&importer&&normalizePath(importer)===catalog)return assets;
  },
  load(id){
   if(normalizePath(id)!==normalizePath(assets))return;
   const manifest:CatalogManifest={lists:[],locales:[]};
   const indices=new Map<string,number>();
   for(const locale of readdirSync(locales,{withFileTypes:true}).filter(entry=>entry.isDirectory()).map(entry=>entry.name).sort()){
    const files=readdirSync(join(locales,locale)).filter(name=>name.endsWith('.ftl')).sort();
    const signature=JSON.stringify(files);
    let index=indices.get(signature);
    if(index===undefined){index=manifest.lists.length;indices.set(signature,index);manifest.lists.push(files);}
    manifest.locales.push([locale,index]);
   }
   const source=readFileSync(assets,'utf8'),marker='/* @mw-catalog-manifest */ { lists: [], locales: [] }';
   if(!source.includes(marker))throw new Error('Catalog manifest marker missing');
   return source.replace(marker,JSON.stringify(manifest));
  },
 };
}
