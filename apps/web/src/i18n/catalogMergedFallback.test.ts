import {beforeEach,expect,it,vi} from 'vitest';
const calls:string[]=[];
vi.mock('./catalogSources.ts',()=>({mergedFallback:true,default:Object.fromEntries(['en/demo','de/demo','en/other','de/other'].map(key=>[`../../locales/${key}.ftl`,async()=>{calls.push(key);return `${key.replace('/','-')} = ${key}`;}]))}));
import {loadCatalog,reloadForActiveChain} from './catalog.ts';
import {_resetRegistry,setActiveLocale,t} from './registry.ts';
beforeEach(()=>{calls.length=0;_resetRegistry();});
it('loads only the active locale: its pack already holds the English fallback',async()=>{
 setActiveLocale('de');await loadCatalog('demo');
 expect(calls).toEqual(['de/demo']);expect(t('de-demo')).toBe('de/demo');
});
it('does not fetch the default locale for a request made before the locale was chosen',async()=>{
 const early=loadCatalog('other');
 setActiveLocale('de');await reloadForActiveChain();await early;
 expect(calls.filter(key=>key.endsWith('/other'))).toEqual(['de/other']);
});
it('still loads English for a user whose locale is English',async()=>{
 await loadCatalog('demo');expect(calls).toEqual(['en/demo']);
});
