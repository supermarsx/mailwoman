// The build plugin fills a dictionary of exact per-locale file lists. Catalog
// text and asset URLs remain lazy; equal lists share only manifest metadata.
export interface CatalogManifest { lists: string[][]; locales: [string, number][] }
const manifest: CatalogManifest = /* @mw-catalog-manifest */ { lists: [], locales: [] };
type Loaders = Record<string, () => Promise<string>>;
export function catalogAssetSources(
 source: CatalogManifest,
 importUrls: () => Promise<{ default: Loaders }> = () => import('./catalogAssetUrls.ts'),
): Loaders {
 const keys = source.locales.flatMap(([locale, index]) => {
  const files = source.lists[index];
  if (!files) throw new Error('Missing translation catalog manifest list');
  return files.map(file => `../../locales/${locale}/${file}`);
 }).sort();
 return Object.fromEntries(keys.map(key => [key, async () => {
  const { default: loaders } = await importUrls();
  const loader = loaders[key];
  if (!loader) throw new Error(`Missing translation catalog: ${key}`);
  return loader();
 }]));
}
export default catalogAssetSources(manifest);
