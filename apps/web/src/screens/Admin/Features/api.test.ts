import {beforeEach,expect,it,vi} from 'vitest';
import {featuresApi,type FeatureCatalogue} from './api.ts';
const {request}=vi.hoisted(()=>({request:vi.fn()}));
vi.mock('../shell/adminApi.ts',()=>({adminRequest:request}));
const catalogue:FeatureCatalogue={revision:'rev',profiles:[],groups:[],assignments:[],memberships:{},nativeTargets:[],domains:['canonical.test'],catalogue:[]};
beforeEach(()=>request.mockReset());

it('adds visible routing aliases without changing the feature write revision or canonical domains',async()=>{
 request.mockImplementation(async(path:string)=>path==='/features'?{data:catalogue,etag:'feature-tag'}:{data:[{name:'canonical.test',aliases:['alias.test','second.test','canonical.test']},{name:'unavailable.test',aliases:['hidden.test']}],etag:'domain-tag'});
 const result=await featuresApi.load();
 expect(result.etag).toBe('feature-tag');expect(result.data.domains).toEqual(['canonical.test']);expect(result.data.domainAliases).toEqual({'alias.test':'canonical.test','second.test':'canonical.test'});
 expect(request).toHaveBeenCalledWith('/domains');
});

it('keeps the feature catalogue usable when separately permissioned domain metadata is refused',async()=>{
 request.mockImplementation(async(path:string)=>{if(path==='/domains')throw new Error('403');return {data:catalogue,etag:'feature-tag'};});
 expect(await featuresApi.load()).toEqual({data:catalogue,etag:'feature-tag'});
});

it('propagates a feature read failure rather than treating domain metadata as a catalogue',async()=>{
 request.mockImplementation(async(path:string)=>{if(path==='/features')throw new Error('feature unavailable');return {data:[],etag:'domain-tag'};});
 await expect(featuresApi.load()).rejects.toThrow('feature unavailable');
});
