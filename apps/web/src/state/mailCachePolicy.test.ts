import {afterEach,expect,it,vi} from 'vitest';
import {readMailCachePolicy} from './mailCachePolicy.ts';
const state=vi.hoisted(()=>({ready:true,values:{} as Record<string,{value:unknown}>}));
vi.mock('../api/settings.ts',()=>({settings:()=>state.values,settingsLoadStatus:()=>state.ready?'ready':'loading'}));
afterEach(()=>{state.ready=true;state.values={};});
it('rejects invalid limits and never prefetches before authenticated settings are ready',()=>{state.values={'mail.cache.maxConcurrency':{value:999},'mail.cache.prefetchEnabled':{value:true}};state.ready=false;expect(readMailCachePolicy().maxConcurrency).toBe(1);expect(readMailCachePolicy().prefetchEnabled).toBe(false);state.ready=true;expect(readMailCachePolicy().prefetchEnabled).toBe(true);});
it('clamps freshness to maximum age and honors explicit exclusions',()=>{state.values={'mail.cache.freshSeconds':{value:100},'mail.cache.maxAgeSeconds':{value:5},'mail.cache.excludedFolders':{value:['id:private']}};const policy=readMailCachePolicy();expect(policy.freshSeconds).toBe(5);expect(policy.excludedFolders).toEqual(['id:private']);});
