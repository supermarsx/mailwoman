import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,render,waitFor} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {toast,toasts,resetToasts} from './toast/toast.ts';
const fixture=vi.hoisted(()=>({map:()=>({}) as Record<string,{value:unknown}>}));
vi.mock('../api/settings.ts',()=>({settings:()=>fixture.map()}));
import {StoragePersistencePrompt} from './StoragePersistencePrompt.tsx';
afterEach(()=>{cleanup();suspendAccountPreferences();resetToasts();vi.restoreAllMocks();localStorage.clear();});
it('offers persistence after opt-in caching, waits for the gesture, and respects admin denial',async()=>{
 const persist=vi.fn(async()=>true);Object.defineProperty(navigator,'storage',{configurable:true,value:{persist,persisted:async()=>false,estimate:async()=>({usage:512,quota:2**30})}});
 activateAccountPreferences('a','storage_prompt_test_authority');const [map,setMap]=createSignal<Record<string,{value:unknown}>>({'offline.enabled':{value:false},'offline.persistentStorageAllowed':{value:true},'offline.requestPersistence':{value:true}});fixture.map=map;
 render(()=><StoragePersistencePrompt/>);await Promise.resolve();expect(toasts()).toHaveLength(0);expect(persist).not.toHaveBeenCalled();
 setMap(previous=>({...previous,'offline.enabled':{value:true}}));await waitFor(()=>expect(toasts().some(row=>row.id==='storage-persistence')).toBe(true));expect(persist).not.toHaveBeenCalled();
 const request=toasts().find(row=>row.id==='storage-persistence')!.action!;setMap(previous=>({...previous,'offline.persistentStorageAllowed':{value:false}}));await request.run();expect(persist).not.toHaveBeenCalled();
 toast.dismiss('storage-persistence');
});

it('recommends persistence on storage pressure without requesting it automatically',async()=>{
 const persist=vi.fn(async()=>true);Object.defineProperty(navigator,'storage',{configurable:true,value:{persist,persisted:async()=>false,estimate:async()=>({usage:950_000_000,quota:1_000_000_000})}});
 activateAccountPreferences('a','storage_pressure_test_authority');fixture.map=()=>({'offline.persistentStorageAllowed':{value:true}});
 render(()=><StoragePersistencePrompt/>);
 await waitFor(()=>expect(toasts().some(row=>row.id==='storage-pressure')).toBe(true));expect(persist).not.toHaveBeenCalled();
 await toasts().find(row=>row.id==='storage-pressure')!.action!.run();expect(persist).toHaveBeenCalledTimes(1);
});
