import {beforeAll,expect,it,vi} from 'vitest';
import {fireEvent,screen,waitFor} from '@solidjs/testing-library';
import type {AdminSetting} from '../../api/admin/settings.ts';
import {loadCatalog} from '../../i18n/index.ts';
import {AdminPrivacy,type PrivacyStats} from './Privacy.tsx';
import {IMAGE_CONCURRENCY_KEY,type ImageConcurrencyRuntime} from './AdminImageConcurrency.tsx';
import {holding,renderInShell} from './shell/testkit.tsx';
beforeAll(async()=>{await Promise.all([loadCatalog('privacy'),loadCatalog('admin-settings'),loadCatalog('admin-shell')]);});
const setting:AdminSetting={key:IMAGE_CONCURRENCY_KEY,family:false,kind:{type:'enum',values:['auto','conservative','balanced','throughput','manual']},default:'auto',nullable:false,scopes:'global',userOverride:false,category:'privacy',mapsTo:'',env:null,dedicatedRoute:null,scopeId:'account',hot:true,ceiling:false,userVisible:false,globalValue:null,allowUserOverride:false,globalLocked:false,updatedBy:null,updatedAt:null,effective:{value:'auto',source:'default'},envState:null,overrides:{},etag:'"mode1"'};
const runtime:ImageConcurrencyRuntime={mode:'auto',adaptive:true,cpuCores:4,memoryLimitBytes:4*1024**3,memoryAvailableBytes:2*1024**3,ceilings:{perMessage:30,perAccount:128,global:128,pipeline:128,decoders:10},effective:{perMessage:12,perAccount:32,global:32,pipeline:32,decoders:4},active:{network:2,pipeline:3,decoders:1,queued:7}};
const stats:PrivacyStats={cacheEntries:3,cacheBytes:1024,trackersReplaced:0,traffic:[],imageConcurrency:runtime};
const api=(rows:AdminSetting[]=[setting])=>({settings:vi.fn(async()=>({kind:'ok' as const,data:rows,etag:null})),stats:vi.fn(async()=>stats),saveMode:vi.fn(async(mode:string,_etag:string):Promise<AdminSetting>=>({...setting,globalValue:mode,effective:{value:mode,source:'global'},etag:'"mode2"'}))});
it('shows fractional container CPU capacity without treating it as unavailable',async()=>{
 const service=api();service.stats.mockResolvedValue({...stats,imageConcurrency:{...runtime,cpuCores:1.5}});renderInShell(()=><AdminPrivacy service={service}/>);
 expect(await screen.findByTestId('image-concurrency-runtime')).toHaveTextContent('CPU cores: 1.5');
});
it('saves a themed preset with one enum write and shows real effective capacity separately from ceilings',async()=>{
 const service=api();renderInShell(()=><AdminPrivacy service={service}/>);
 const trigger=await screen.findByRole('button',{name:'Image concurrency profile'});await waitFor(()=>expect(trigger).toBeEnabled());expect(trigger).toHaveTextContent('Automatic');expect(screen.getByTestId('image-concurrency-runtime')).toHaveTextContent('CPU cores: 4');expect(screen.getByTestId('image-concurrency-runtime')).toHaveTextContent('12 effective / 30 ceiling');expect(screen.getByTestId('image-concurrency-runtime')).toHaveTextContent('2 network · 3 pipelines · 1 decoders · 7 queued');
 fireEvent.click(trigger);fireEvent.click(screen.getByRole('menuitemradio',{name:'High throughput'}));expect(service.saveMode).not.toHaveBeenCalled();fireEvent.click(screen.getByRole('button',{name:'Apply profile'}));
 await waitFor(()=>expect(service.saveMode).toHaveBeenCalledExactlyOnceWith('throughput','"mode1"'));await waitFor(()=>expect(service.stats).toHaveBeenCalledTimes(2));expect(trigger).toHaveTextContent('High throughput');
});
it('preserves an unsaved choice after a refused write and retries with the refreshed setting ETag',async()=>{
 const service=api();service.settings.mockResolvedValueOnce({kind:'ok',data:[setting],etag:null}).mockResolvedValue({kind:'ok',data:[{...setting,etag:'"fresh"'}],etag:null});service.saveMode.mockRejectedValueOnce({status:412});
 renderInShell(()=><AdminPrivacy service={service}/>);const trigger=await screen.findByRole('button',{name:'Image concurrency profile'});await waitFor(()=>expect(trigger).toBeEnabled());fireEvent.click(trigger);fireEvent.click(screen.getByRole('menuitemradio',{name:'Balanced'}));fireEvent.click(screen.getByRole('button',{name:'Apply profile'}));
 await waitFor(()=>expect(service.settings).toHaveBeenCalledTimes(2));expect(trigger).toHaveTextContent('Balanced');await waitFor(()=>expect(screen.getByRole('button',{name:'Apply profile'})).toBeEnabled());fireEvent.click(screen.getByRole('button',{name:'Apply profile'}));await waitFor(()=>expect(service.saveMode).toHaveBeenLastCalledWith('balanced','"fresh"'));expect(service.saveMode).toHaveBeenCalledTimes(2);
});
it.each(['capability','environment'] as const)('does not offer a preset mutation when blocked by %s',async blocked=>{
 const service=api(blocked==='environment'?[{...setting,envState:{var:'MW_IMAGE_PROFILE',precedence:'pin',set:true,value:'auto'}}]:[setting]);
 renderInShell(()=><AdminPrivacy service={service}/>,blocked==='capability'?{loadMe:holding(['settings.read','egress.read'])}:{});const trigger=await screen.findByRole('button',{name:'Image concurrency profile'});await waitFor(()=>expect(trigger).toBeDisabled());expect(screen.getByRole('button',{name:'Apply profile'})).toBeDisabled();fireEvent.click(trigger);expect(screen.queryByRole('menu')).toBeNull();expect(service.saveMode).not.toHaveBeenCalled();
});
it('does not fabricate resource values when runtime information is missing',async()=>{
 const service=api();service.stats.mockResolvedValue({cacheEntries:3,cacheBytes:1024,trackersReplaced:0,traffic:[]} as typeof stats);renderInShell(()=><AdminPrivacy service={service}/>);await screen.findByRole('button',{name:'Image concurrency profile'});expect(screen.getByText('Unavailable')).toBeVisible();expect(screen.queryByTestId('image-concurrency-runtime')).toBeNull();
});
