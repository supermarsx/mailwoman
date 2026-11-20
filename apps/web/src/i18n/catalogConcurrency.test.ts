import {afterEach,beforeEach,expect,it,vi} from 'vitest';
const probe=vi.hoisted(()=>({calls:[] as string[],active:0,peak:0,fail:false,held:false,releases:[] as Array<()=>void>}));
vi.mock('./catalogSources.ts',()=>({default:Object.fromEntries(['en/common','en/demo','en/one','en/two','en/three','de/demo','fr/demo','es/demo'].map(key=>[`../../locales/${key}.ftl`,async()=>{
 probe.calls.push(key);probe.active++;probe.peak=Math.max(probe.peak,probe.active);
 try{if(probe.held)await new Promise<void>(resolve=>probe.releases.push(resolve));if(probe.fail){probe.fail=false;throw Error('transient asset failure');}return `probe = ${key}\ncommon-ok = OK`;}
 finally{probe.active--;}
}]))}));
import {loadCatalog,preloadCatalogs,reloadForActiveChain} from './catalog.ts';
import {_resetRegistry,setActiveLocale,t,retainedLocales} from './registry.ts';
import {applyWebPerformance,resetWebPerformance} from '../boot/performancePolicy.ts';
import {FluentBundle} from '@fluent/bundle';
beforeEach(()=>{_resetRegistry();resetWebPerformance();probe.calls=[];probe.active=0;probe.peak=0;probe.fail=false;probe.held=false;probe.releases=[];setActiveLocale('en');});
afterEach(()=>{probe.held=false;for(const release of probe.releases.splice(0))release();vi.restoreAllMocks();resetWebPerformance();});
it('shares concurrent fetch AND Fluent parsing, then reuses the parsed bundle',async()=>{
 const parsed=vi.spyOn(FluentBundle.prototype,'addResource');probe.held=true;
 const requests=Array.from({length:30},()=>loadCatalog('demo'));
 expect(probe.calls).toEqual(['en/demo']);probe.held=false;probe.releases.splice(0).forEach(release=>release());
 await Promise.all(requests);expect(parsed).toHaveBeenCalledTimes(1);expect(t('probe')).toBe('en/demo');
 await loadCatalog('demo');expect(probe.calls).toEqual(['en/demo']);expect(parsed).toHaveBeenCalledTimes(1);
});
it('does not cache a failure and allows a subsequent real load',async()=>{
 probe.fail=true;await expect(loadCatalog('demo')).rejects.toThrow('transient asset failure');
 await loadCatalog('demo');expect(probe.calls).toEqual(['en/demo','en/demo']);expect(t('probe')).toBe('en/demo');
});
it('bounds actual outstanding loads while preserving all requested catalogs',async()=>{
 applyWebPerformance({concurrentLoads:2});probe.held=true;
 const requested=preloadCatalogs(['demo','one','two','three']);expect(probe.active).toBe(2);
 probe.held=false;probe.releases.splice(0).forEach(release=>release());await requested;
 expect(probe.peak).toBe(2);expect(probe.calls.slice().sort()).toEqual(['en/demo','en/one','en/three','en/two']);
});
it('evicts inactive parsed locales and reloads touched modules on a later switch',async()=>{
 applyWebPerformance({retainedLocales:2});setActiveLocale('de');await loadCatalog('demo');
 expect(retainedLocales().sort()).toEqual(['de','en']);setActiveLocale('fr');await reloadForActiveChain();
 expect(retainedLocales().sort()).toEqual(['en','fr']);expect(t('probe')).toBe('fr/demo');
 setActiveLocale('de');await reloadForActiveChain();expect(probe.calls.filter(key=>key==='de/demo')).toHaveLength(2);expect(t('probe')).toBe('de/demo');
});
it('does not resurrect an evicted locale when its old request finishes',async()=>{
 applyWebPerformance({retainedLocales:2});setActiveLocale('de');probe.held=true;
 const old=loadCatalog('demo');setActiveLocale('fr');probe.held=false;probe.releases.splice(0).forEach(release=>release());await old;
 expect(retainedLocales()).not.toContain('de');setActiveLocale('de');await loadCatalog('demo');
 expect(probe.calls.filter(key=>key==='de/demo')).toHaveLength(2);expect(t('probe')).toBe('de/demo');
});
it('never grows the manifest queue for unknown modules',async()=>{
 await Promise.all(Array.from({length:100},(_,index)=>loadCatalog(`unknown-${index}`)));expect(probe.calls).toEqual([]);
});
it('pins active and English bundles when the configured retention shrinks',async()=>{
 setActiveLocale('de');await loadCatalog('demo');setActiveLocale('fr');await loadCatalog('demo');
 applyWebPerformance({retainedLocales:2});expect(retainedLocales().sort()).toEqual(['en','fr']);expect(t('probe')).toBe('fr/demo');
});
