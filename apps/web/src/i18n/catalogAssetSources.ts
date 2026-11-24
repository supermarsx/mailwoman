// The build plugin fills a dictionary of exact per-locale namespace lists (app
// and admin packs). Equal lists share only manifest metadata; pack text and URLs
// are fetched lazily (catalogAssetLoaders.ts). No hashed name appears here, so a
// translation edit does not rename the chunk that holds this module.
import {catalogPackLoaders, type CatalogEntry, type PackKind} from './catalogAssetLoaders.ts';
export interface CatalogManifest { lists: Array<Record<PackKind, string[]>>; locales: [string, number][] }
const manifest: CatalogManifest = /* @mw-catalog-manifest */ { lists: [], locales: [] };
type Loaders = Record<string, () => Promise<string>>;
/** Every pack already holds the English text for ids its locale lacks. */
export const mergedFallback = true;
export function catalogAssetSources(source: CatalogManifest, ...rest: [table?: Parameters<typeof catalogPackLoaders>[1], request?: Parameters<typeof catalogPackLoaders>[2], baseUri?: Parameters<typeof catalogPackLoaders>[3]]): Loaders {
 const entries: CatalogEntry[] = source.locales.flatMap(([locale, index]) => {
  const list = source.lists[index];
  if (!list) throw new Error('Missing translation catalog manifest list');
  return (['app', 'admin'] as const).flatMap((kind) => list[kind].map((namespace) => ({locale, namespace, kind})));
 }).sort((a, b) => { const x = `${a.locale}/${a.namespace}`, y = `${b.locale}/${b.namespace}`; return x < y ? -1 : x > y ? 1 : 0; });
 return catalogPackLoaders(entries, ...rest);
}
export default catalogAssetSources(manifest);
