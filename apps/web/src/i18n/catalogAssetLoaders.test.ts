import {expect,it,vi} from 'vitest';
import {catalogPackLoaders, readPackTable, type CatalogEntry} from './catalogAssetLoaders.ts';

const entries: CatalogEntry[] = [
 {locale: 'pt-PT', namespace: 'mail', kind: 'app'}, {locale: 'pt-PT', namespace: 'compose', kind: 'app'},
 {locale: 'pt-PT', namespace: 'admin-users', kind: 'admin'}, {locale: 'de', namespace: 'mail', kind: 'app'},
];
const table = {'pt-PT/app': './assets/catalog.pt-PT.app-h1.json', 'pt-PT/admin': './assets/catalog.pt-PT.admin-h2.json', 'de/app': './assets/catalog.de.app-h3.json'};
const pack = (files: Record<string, string>) => new Response(JSON.stringify(files));

it('fetches a pack once for all its namespaces, keeps Unicode text and fetches nothing before the first request', async () => {
 const request = vi.fn(async () => pack({mail: 'hello = Olá 🙂', compose: 'c = Escrever'}));
 const loaders = catalogPackLoaders(entries, () => table, request, () => 'https://mail.example/app/');
 expect(request).not.toHaveBeenCalled();
 expect(await loaders['../../locales/pt-PT/mail.ftl']!()).toBe('hello = Olá 🙂');
 expect(await loaders['../../locales/pt-PT/compose.ftl']!()).toBe('c = Escrever');
 expect(request).toHaveBeenCalledExactlyOnceWith('https://mail.example/app/assets/catalog.pt-PT.app-h1.json');
});
it('keeps one pack per locale and kind', async () => {
 const request = vi.fn(async (url: RequestInfo | URL) => pack(String(url).includes('admin') ? {'admin-users': 'u = U'} : {mail: 'm = M'}));
 const loaders = catalogPackLoaders(entries, () => table, request as typeof fetch, () => undefined);
 await loaders['../../locales/de/mail.ftl']!(); await loaders['../../locales/pt-PT/mail.ftl']!(); await loaders['../../locales/pt-PT/admin-users.ftl']!();
 expect(request.mock.calls.map(([url]) => String(url)).sort()).toEqual(['./assets/catalog.de.app-h3.json', './assets/catalog.pt-PT.admin-h2.json', './assets/catalog.pt-PT.app-h1.json']);
});
it('propagates missing, offline and absent-namespace failures and retries the pack afterwards', async () => {
 const request = vi.fn().mockResolvedValueOnce(new Response('missing', {status: 404})).mockRejectedValueOnce(new TypeError('offline')).mockResolvedValueOnce(pack({mail: 'ready = Ready'}));
 const loaders = catalogPackLoaders(entries, () => table, request, () => undefined);
 const mail = loaders['../../locales/pt-PT/mail.ftl']!;
 await expect(mail()).rejects.toThrow('404'); await expect(mail()).rejects.toThrow('offline'); await expect(mail()).resolves.toBe('ready = Ready');
 await expect(loaders['../../locales/pt-PT/compose.ftl']!()).rejects.toThrow('Missing translation catalog');
 expect(request).toHaveBeenCalledTimes(3);
});
it('reports a pack the page table does not name', async () => {
 const loaders = catalogPackLoaders(entries, () => ({}), vi.fn(), () => undefined);
 await expect(loaders['../../locales/de/mail.ftl']!()).rejects.toThrow('Translation pack missing: de/app');
});
it('waits briefly for the table during startup', async () => {
 let reads = 0;
 const loaders = catalogPackLoaders(entries, () => (++reads < 3 ? undefined : table), async () => pack({mail: 'x = y'}), () => undefined);
 await expect(loaders['../../locales/de/mail.ftl']!()).resolves.toBe('x = y');
});
it('reads the page table from the JSON block and ignores a missing or damaged one', () => {
 const doc = (text: string | null) => ({getElementById: () => (text === null ? null : ({textContent: text} as HTMLElement))});
 expect(readPackTable(doc('{"en/app":"./assets/a.json"}'))).toEqual({'en/app': './assets/a.json'});
 expect(readPackTable(doc(null))).toBeUndefined(); expect(readPackTable(doc('{'))).toBeUndefined(); expect(readPackTable(doc('null'))).toBeUndefined();
 expect(readPackTable(undefined)).toBeUndefined();
});
