import {afterEach, expect, it, vi} from 'vitest';
import {createAccountPreferencesSync, PreferenceConflict, type AccountUiPreferences, type AccountPreferencesApi} from './accountPreferences.ts';
function deferred<T>() {let resolve!:(value:T)=>void;const promise=new Promise<T>(r=>{resolve=r;});return {promise,resolve};}
const owner={id:'a',generation:1};
afterEach(()=>vi.useRealTimers());
it('ignores old account loads and acknowledgements after rebinding',async()=>{
 const save=vi.fn();const publish=vi.fn();
 const old=deferred<{preferences:AccountUiPreferences;etag:string}>();let calls=0;
 const next=createAccountPreferencesSync({load:()=>++calls===1?old.promise:Promise.resolve({preferences:{direction:'rtl'},etag:'"2"'}),save},publish,vi.fn(),()=>true);
 next.bind(owner);next.bind({id:'b',generation:2});await Promise.resolve();old.resolve({preferences:{folderPinned:['private-a']},etag:'"1"'});await Promise.resolve();
 expect(publish.mock.calls.at(-1)?.[0]).toEqual({direction:'rtl'});next.dispose();
});
it('drops queued writes synchronously on signout',async()=>{
 vi.useFakeTimers();const save=vi.fn();const sync=createAccountPreferencesSync({load:async()=>({preferences:{},etag:'"0"'}),save},vi.fn(),vi.fn(),()=>true);
 sync.bind(owner);await Promise.resolve();sync.update({markRead:'never'});sync.bind(null);await vi.advanceTimersByTimeAsync(1000);expect(save).not.toHaveBeenCalled();sync.dispose();
});
it('merges another device fields after a CAS conflict, keeping only local edits',async()=>{
 vi.useFakeTimers();let loads=0;const saved:AccountUiPreferences[]=[];
 const api:AccountPreferencesApi={load:async()=>({preferences:++loads===1?{direction:'ltr'}:{direction:'rtl',workspaceLayout:'stack'},etag:`"${loads}"`}),save:async(value,etag)=>{saved.push(value);if(etag==='"1"')throw new PreferenceConflict();return {preferences:value,etag:'"3"'};}};
 const publish=vi.fn();const sync=createAccountPreferencesSync(api,publish,vi.fn(),()=>true);sync.bind(owner);await Promise.resolve();sync.update({markRead:'never'});await sync.flush();await vi.advanceTimersByTimeAsync(500);expect(saved.at(-1)).toEqual({direction:'rtl',workspaceLayout:'stack',markRead:'never'});expect(publish.mock.calls.at(-1)?.[0]).toEqual(saved.at(-1));sync.dispose();
});
it('preserves an edit made while saving and ignores a late old-account acknowledgement',async()=>{
 const pending=deferred<{preferences:AccountUiPreferences;etag:string}>();const publish=vi.fn();const sync=createAccountPreferencesSync({load:async()=>({preferences:{},etag:'"0"'}),save:()=>pending.promise},publish,vi.fn(),()=>true);
 sync.bind(owner);await Promise.resolve();sync.update({direction:'ltr'});const first=sync.flush();sync.update({direction:'rtl'});pending.resolve({preferences:{direction:'ltr'},etag:'"1"'});await first;expect(publish.mock.calls.at(-1)?.[0]).toEqual({direction:'rtl'});sync.bind(null);expect(publish.mock.calls.at(-1)?.[0]).toEqual({});sync.dispose();
});
it('does not spin retry writes after a network failure',async()=>{
 vi.useFakeTimers();const save=vi.fn(async()=>{throw new Error('offline');});const sync=createAccountPreferencesSync({load:async()=>({preferences:{},etag:'"0"'}),save},vi.fn(),vi.fn(),()=>true);sync.bind(owner);await Promise.resolve();sync.update({direction:'rtl'});await vi.advanceTimersByTimeAsync(5000);expect(save).toHaveBeenCalledOnce();sync.dispose();
});
it('does not publish a saved response from a different authenticated generation', async () => {
 const pending=deferred<{preferences:AccountUiPreferences;etag:string}>();const publish=vi.fn();
 const sync=createAccountPreferencesSync({load:async()=>({preferences:{},etag:'"0"'}),save:()=>pending.promise},publish,vi.fn(),()=>true);
 sync.bind(owner);await Promise.resolve();sync.update({folderPinned:['a-private']});const write=sync.flush();
 sync.bind({id:'b',generation:2});await Promise.resolve();pending.resolve({preferences:{folderPinned:['a-private']},etag:'"1"'});await write;
 expect(publish.mock.calls.at(-1)?.[0]).toEqual({});sync.dispose();
});
it('merges different-folder icon edits across devices and preserves explicit automatic resets',async()=>{
 vi.useFakeTimers();let loads=0;const writes:AccountUiPreferences[]=[];
 const sync=createAccountPreferencesSync({load:async()=>({preferences:{folderIcons:++loads===1?{a:'star',b:'flag'}:{a:'star',b:'clock',c:'lock'}},etag:`"${loads}"`}),save:async(value,etag)=>{writes.push(value);if(etag==='"1"')throw new PreferenceConflict();return {preferences:value,etag:'"3"'};}},vi.fn(),vi.fn(),()=>true);
 sync.bind(owner);await Promise.resolve();sync.update({folderIcons:{b:'flag'}});await sync.flush();await vi.advanceTimersByTimeAsync(500);
 expect(writes.at(-1)?.folderIcons).toEqual({b:'clock',c:'lock'});sync.dispose();
});
it('merges separate tag edits after conflicts and drops old-account edits',async()=>{
 vi.useFakeTimers();let loads=0;const writes:AccountUiPreferences[]=[];const label=(name:string)=>({name,color:'#112233',icon:'tag' as const});
 const sync=createAccountPreferencesSync({load:async()=>({preferences:{tags:++loads===1?{a:label('A'),b:label('B')}:{a:label('A'),b:label('Other device'),c:label('C')}},etag:`"${loads}"`}),save:async(value,etag)=>{writes.push(value);if(etag==='"1"')throw new PreferenceConflict();return {preferences:value,etag:'"3"'};}},vi.fn(),vi.fn(),()=>true);
 sync.bind(owner);await Promise.resolve();sync.update({tags:{a:label('Edited A'),b:label('B')}});await sync.flush();await vi.advanceTimersByTimeAsync(500);
 expect(writes.at(-1)?.tags).toEqual({a:label('Edited A'),b:label('Other device'),c:label('C')});
 sync.update({tags:{a:label('Private A')}});sync.bind({id:'other',generation:2});await Promise.resolve();await vi.advanceTimersByTimeAsync(500);expect(writes).toHaveLength(2);sync.dispose();
});
