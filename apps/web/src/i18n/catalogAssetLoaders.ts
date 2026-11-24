/** Production wire format: one JSON pack per locale and kind (`app`, `admin`), `{namespace: ftl text}`. */
export type PackKind = 'app' | 'admin';
export interface CatalogEntry { locale: string; namespace: string; kind: PackKind }
/** `<locale>/<kind>` -> URL of the hashed pack, relative to the page. */
export type PackTable = Record<string, string>;

/** Id of the JSON block the build writes into index.html (scripts/catalogAssets.ts). */
export const PACK_TABLE_ID = 'mw-catalog-packs';

/** The table the build embeds in the page, or undefined outside a production page. */
export function readPackTable(doc: Pick<Document, 'getElementById'> | undefined = typeof document === 'undefined' ? undefined : document): PackTable | undefined {
 const text = doc?.getElementById(PACK_TABLE_ID)?.textContent;
 if (!text) return undefined;
 try {
  const parsed: unknown = JSON.parse(text);
  return parsed !== null && typeof parsed === 'object' ? (parsed as PackTable) : undefined;
 } catch { return undefined; }
}

const key = (locale: string, namespace: string): string => `../../locales/${locale}/${namespace}.ftl`;

/**
 * Loaders keyed like the development glob (`../../locales/<locale>/<namespace>.ftl`).
 * A pack is fetched on the first request for any of its namespaces and shared by
 * the rest; a failed fetch is forgotten so the next request retries it. Each
 * namespace is handed back as text, parsed by the caller only when first used.
 */
// eslint-disable-next-line mw/no-global-fetch -- Public hashed text assets; no account API requests or writes.
export function catalogPackLoaders(entries: readonly CatalogEntry[], table: () => PackTable | undefined = readPackTable, request: typeof fetch = (...args) => fetch(...args), baseUri: () => string | undefined = () => (typeof document === 'undefined' ? undefined : document.baseURI)): Record<string, () => Promise<string>> {
 const packs = new Map<string, Promise<Record<string, string>>>();
 const fetchPack = async (locale: string, kind: PackKind): Promise<Record<string, string>> => {
  // The table sits in index.html; give a pack requested during startup one macrotask per retry to see it.
  let urls = table();
  for (let wait = 0; urls === undefined && wait < 3; wait++) { await new Promise((resolve) => setTimeout(resolve, 0)); urls = table(); }
  const relative = urls?.[`${locale}/${kind}`];
  if (relative === undefined) throw new Error(`Translation pack missing: ${locale}/${kind}`);
  const base = baseUri();
  const response = await request(base === undefined ? relative : new URL(relative, base).href);
  if (!response.ok) throw new Error(`Translation catalog unavailable (${response.status})`);
  return (await response.json()) as Record<string, string>;
 };
 return Object.fromEntries(entries.map(({locale, namespace, kind}) => [key(locale, namespace), async () => {
  const id = `${locale}/${kind}`;
  let pack = packs.get(id);
  if (pack === undefined) {
   pack = fetchPack(locale, kind);
   packs.set(id, pack);
   pack.catch(() => { if (packs.get(id) === pack) packs.delete(id); });
  }
  const text = (await pack)[namespace];
  if (text === undefined) throw new Error(`Missing translation catalog: ${key(locale, namespace)}`);
  return text;
 }]));
}
