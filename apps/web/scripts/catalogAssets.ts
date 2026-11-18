import {fileURLToPath} from 'node:url';
import {readFileSync,readdirSync} from 'node:fs';
import {join} from 'node:path';
import {normalizePath,type Plugin} from 'vite';
/** Tests use raw imports; production avoids executable translation chunks. */
export function catalogAssets():Plugin{
 const catalog=normalizePath(fileURLToPath(new URL('../src/i18n/catalog.ts',import.meta.url)));
 const assets=fileURLToPath(new URL('../src/i18n/catalogAssetSources.ts',import.meta.url));
 const locales=fileURLToPath(new URL('../locales/',import.meta.url));
 return {name:'mw-catalog-assets',apply:'build',enforce:'pre',
  resolveId(source,importer){
   if(process.env['MW_CATALOG_ASSETS']!=='0'&&source==='./catalogSources.ts'&&importer&&normalizePath(importer)===catalog)return assets;
  },
  load(id){
   if(normalizePath(id)!==normalizePath(assets))return;
   const keys=readdirSync(locales,{withFileTypes:true}).filter(entry=>entry.isDirectory()).flatMap(locale=>
    readdirSync(join(locales,locale.name)).filter(name=>name.endsWith('.ftl')).map(name=>`../../locales/${locale.name}/${name}`)
   ).sort();
   const source=readFileSync(assets,'utf8'),marker='/* @mw-catalog-keys */ []';
   if(!source.includes(marker))throw new Error('Catalog manifest marker missing');
   return source.replace(marker,JSON.stringify(keys));
  },
 };
}
