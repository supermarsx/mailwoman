import {beforeEach,expect,it,vi} from 'vitest';
import {Show} from 'solid-js';
import {fireEvent,screen,waitFor} from '@solidjs/testing-library';
import {PublicAssetCacheCard} from './PublicAssetCacheCard.tsx';
import {loadPublicAssetCache,clearPublicAssetCache,type PublicAssetCacheStatus} from '../../api/admin/publicAssets.ts';
import type {AdminSetting} from '../../api/admin/settings.ts';
import {renderInShell,holding} from './shell/testkit.tsx';
import {useShell} from './shell/context.ts';
vi.mock('../../api/admin/publicAssets.ts',()=>({loadPublicAssetCache:vi.fn(),clearPublicAssetCache:vi.fn()}));
vi.mock('./Settings/InlineSettingEditor.tsx',()=>({InlineSettingEditor:(props:{settingKey:string})=><div data-testid={props.settingKey}/> }));
const report:PublicAssetCacheStatus={scope:'public-static-representations-only',config:{enabled:true,budgetMiB:64,maxFileMiB:4,maxEntries:1024,revalidateSeconds:1,preload:'shell'},effectivePreload:'shell',generation:1,residentOwnedBytes:1024,entries:2,effectiveMaxFileBytes:4*1024*1024,pendingUnique:3,runningLoads:2,queuedUnique:1,maxQueuedUnique:16,maxRunningLoads:2,configurationStale:false,lastConfigAt:1700000000,counters:{hits:8,misses:3,loads:2,coalesced:5,evictions:0,expired:0,oversize:1,busy:0,loadFailures:0,staleInserts:0,clears:0,preloadLoaded:1,preloadSkipped:2},hardRssLimit:false,embeddedRawBytesDuplicated:false};
beforeEach(()=>{vi.mocked(loadPublicAssetCache).mockReset().mockResolvedValue(report);vi.mocked(clearPublicAssetCache).mockReset().mockResolvedValue({...report,generation:2,entries:0,residentOwnedBytes:0});});
it('renders actual resident and queue status and clears through the maintenance API',async()=>{
 const settings=[{key:'web.assets.cache.enabled'},{key:'maintenance.database.enabled'}] as AdminSetting[];
 renderInShell(()=><PublicAssetCacheCard settings={settings} onChanged={()=>{}}/>);
 expect(await screen.findByText('8 / 3 / 5')).toBeVisible();expect(screen.getByText('2 / 2; 1 / 16')).toBeVisible();
 expect(screen.getByTestId('web.assets.cache.enabled')).toBeVisible();expect(screen.queryByTestId('maintenance.database.enabled')).toBeNull();
 expect(screen.getByText(/not in-flight responses or process RSS/)).toBeVisible();
 fireEvent.click(screen.getByRole('button',{name:'Clear static cache'}));
 await waitFor(()=>expect(clearPublicAssetCache).toHaveBeenCalledOnce());await waitFor(()=>expect(screen.getByText('0 / 1,024')).toBeVisible());
 expect(vi.mocked(clearPublicAssetCache).mock.calls[0]?.[0]).toBeInstanceOf(AbortSignal);
});
it('loads after capabilities resolve and preserves a read-only operator',async()=>{
 const Ready=()=>{const shell=useShell();return <Show when={shell.can.grants().kind==='listed'}><PublicAssetCacheCard settings={undefined} onChanged={()=>{}}/></Show>;};
 renderInShell(()=><Ready/>,{loadMe:holding(['maintenance.read'])});
 expect(await screen.findByText('8 / 3 / 5')).toBeVisible();expect(screen.getByRole('button',{name:'Clear static cache'})).toBeDisabled();expect(clearPublicAssetCache).not.toHaveBeenCalled();
});
it('keeps existing status and exposes a failed clear instead of pretending reclamation',async()=>{
 vi.mocked(clearPublicAssetCache).mockRejectedValue(new Error('owned clear failed'));
 renderInShell(()=><PublicAssetCacheCard settings={undefined} onChanged={()=>{}}/>);
 await screen.findByText('8 / 3 / 5');fireEvent.click(screen.getByRole('button',{name:'Clear static cache'}));
 expect(await screen.findByRole('alert')).toBeVisible();expect(screen.getByText('2 / 1,024')).toBeVisible();
});
it('aborts outstanding observation on unmount and does not dispatch without read permission',async()=>{
 let signal:AbortSignal|undefined;vi.mocked(loadPublicAssetCache).mockImplementation(async value=>{signal=value;return await new Promise<PublicAssetCacheStatus>(()=>{});});
 const rendered=renderInShell(()=><PublicAssetCacheCard settings={undefined} onChanged={()=>{}}/>);
 await waitFor(()=>expect(signal).toBeDefined());rendered.unmount();expect(signal?.aborted).toBe(true);
 vi.mocked(loadPublicAssetCache).mockClear();const Ready=()=>{const shell=useShell();return <Show when={shell.can.grants().kind==='listed'}><PublicAssetCacheCard settings={undefined} onChanged={()=>{}}/></Show>;};
 renderInShell(()=><Ready/>,{loadMe:holding([])});await screen.findByRole('heading',{name:'Public static asset RAM cache'});expect(loadPublicAssetCache).not.toHaveBeenCalled();
});

it('displays encoded resident payloads and verified/generated load counts with mechanical controls',async()=>{
 vi.mocked(loadPublicAssetCache).mockResolvedValue({...report,encodedResidentBytes:2048,ownedEncodedResidentBytes:1024,counters:{...report.counters,validatedSidecars:7,generatedEncodings:2}});
 const settings=[{key:'web.assets.cache.compression.enabled'},{key:'web.assets.cache.compression.brotli'},{key:'web.assets.cache.compression.minBytes'}] as AdminSetting[];
 renderInShell(()=><PublicAssetCacheCard settings={settings} onChanged={()=>{}}/>);
 expect(await screen.findByText('7 / 2')).toBeVisible();
 for(const setting of settings)expect(screen.getByTestId(setting.key)).toBeVisible();
});
