import {expect,it,vi} from 'vitest';
import {catalogAssetLoaders} from './catalogAssetLoaders.ts';
it('loads only the requested catalog and preserves Unicode text',async()=>{
 const request=vi.fn(async()=>new Response('hello = Olá 🙂'));
 const loaders=catalogAssetLoaders({'../../locales/pt-PT/mail.ftl':'/assets/mail-hash.ftl','../../locales/de/mail.ftl':'/assets/de-hash.ftl'},request);
 expect(request).not.toHaveBeenCalled();expect(await loaders['../../locales/pt-PT/mail.ftl']!()).toBe('hello = Olá 🙂');expect(request).toHaveBeenCalledExactlyOnceWith('/assets/mail-hash.ftl');
});
it('propagates missing and offline catalog failures so the caller can retry',async()=>{
 const request=vi.fn().mockResolvedValueOnce(new Response('missing',{status:404})).mockRejectedValueOnce(new TypeError('offline')).mockResolvedValueOnce(new Response('ready = Ready'));
 const loaders=catalogAssetLoaders({catalog:'/assets/catalog.ftl'},request);
 await expect(loaders.catalog!()).rejects.toThrow('404');await expect(loaders.catalog!()).rejects.toThrow('offline');await expect(loaders.catalog!()).resolves.toBe('ready = Ready');
});
