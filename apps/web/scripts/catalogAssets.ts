import {fileURLToPath} from 'node:url';
import {readFileSync} from 'node:fs';
import {join} from 'node:path';
import {normalizePath,type Plugin} from 'vite';
import {PACK_TABLE_ID} from '../src/i18n/catalogAssetLoaders.ts';
import type {CatalogManifest} from '../src/i18n/catalogAssetSources.ts';
import {buildPacks, packAssetName, packKind, packNamespaces, readLocaleSources, serializePack, type PackKind} from './catalogPacks.ts';


/** `<locale>/<kind>` -> `./assets/<file>` from the emitted assets, as the page resolves them. */
export function packTable(bundle: Record<string, {type: string; fileName: string; names?: string[]; name?: string}>, locales: readonly string[]): Record<string, string> {
  const byName = new Map<string, string>();
  for (const item of Object.values(bundle)) {
    if (item.type !== 'asset') continue;
    for (const name of item.names ?? (item.name === undefined ? [] : [item.name])) byName.set(name, item.fileName);
  }
  const table: Record<string, string> = {};
  for (const locale of locales) {
    for (const kind of ['app', 'admin'] as const) {
      const file = byName.get(packAssetName(locale, kind));
      if (file === undefined) throw new Error(`Translation pack ${locale}/${kind} was not emitted`);
      table[`${locale}/${kind}`] = `./${file}`;
    }
  }
  return table;
}

/**
 * Tests use raw imports; production ships two JSON packs per locale (app, admin)
 * and writes their hashed URLs into index.html. The table is not part of any
 * JavaScript chunk, so a translation change renames the pack and the page shell
 * only. `emit: false` (worker builds) substitutes the loader but emits nothing.
 */
export function catalogAssets(root=fileURLToPath(new URL('../',import.meta.url)),options:{emit?:boolean}={}):Plugin{
 const emit=options.emit??true;
 const catalog=normalizePath(join(root,'src/i18n/catalog.ts'));
 const assets=join(root,'src/i18n/catalogAssetSources.ts');
 const locales=join(root,'locales');
 const enabled=():boolean=>process.env['MW_CATALOG_ASSETS']!=='0';
 let localeNames:string[]=[];
 return {name:'mw-catalog-assets',apply:'build',enforce:'pre',
  resolveId(source,importer){
   if(enabled()&&source==='./catalogSources.ts'&&importer&&normalizePath(importer)===catalog)return assets;
  },
  load(id){
   if(normalizePath(id)!==normalizePath(assets))return;
   const sources=readLocaleSources(locales);
   const manifest:CatalogManifest={lists:[],locales:[]};
   const indices=new Map<string,number>();
   for(const locale of sources.keys()){
    const list:Record<PackKind,string[]>={app:[],admin:[]};
    for(const namespace of packNamespaces(sources,locale))list[packKind(namespace)].push(namespace);
    const signature=JSON.stringify(list);
    let index=indices.get(signature);
    if(index===undefined){index=manifest.lists.length;indices.set(signature,index);manifest.lists.push(list);}
    manifest.locales.push([locale,index]);
   }
   const source=readFileSync(assets,'utf8'),marker='/* @mw-catalog-manifest */ { lists: [], locales: [] }';
   if(!source.includes(marker))throw new Error('Catalog manifest marker missing');
   return source.replace(marker,JSON.stringify(manifest));
  },
  buildStart(){
   if(!emit||!enabled())return;
   const packs=buildPacks(readLocaleSources(locales));
   localeNames=[...new Set(packs.map(pack=>pack.locale))];
   for(const pack of packs)this.emitFile({type:'asset',name:packAssetName(pack.locale,pack.kind),source:serializePack(pack)});
  },
  transformIndexHtml:{order:'post',handler(html,context){
   if(!emit||!enabled()||!context.bundle)return;
   const table=JSON.stringify(packTable(context.bundle as never,localeNames));
   if(!html.includes('</head>'))throw new Error('index.html has no </head> for the translation pack table');
   return html.replace('</head>',()=>`    <script type="application/json" id="${PACK_TABLE_ID}">${table}</script>\n  </head>`);
  }},
 };
}
