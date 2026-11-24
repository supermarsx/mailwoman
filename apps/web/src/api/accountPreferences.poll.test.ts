import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {activateAccountPreferences,suspendAccountPreferences} from './accountPreferenceLifecycle.ts';
import {startAccountUiPreferences,type AccountPreferencesApi} from './accountPreferences.ts';

const stream=vi.hoisted(()=>({up:false}));
vi.mock('../realtime/context.ts',()=>({pushStreamUp:()=>stream.up}));

let stop:(()=>void)|undefined;
beforeEach(()=>{vi.useFakeTimers();vi.spyOn(document,'visibilityState','get').mockReturnValue('visible');});
afterEach(()=>{stop?.();stop=undefined;suspendAccountPreferences();vi.useRealTimers();vi.restoreAllMocks();stream.up=false;});

function transport(){
  const load=vi.fn(async()=>({preferences:{},etag:'"0"'}));
  const api:AccountPreferencesApi={load,save:vi.fn(async()=>({preferences:{},etag:'"0"'}))};
  return {api,load};
}

it('sends no timed preference read while the push stream is up, and reads every 30 s while it is not (t52w)',async()=>{
  const {api,load}=transport();
  activateAccountPreferences('poll-a','authority-a');stop=startAccountUiPreferences(api);
  await vi.advanceTimersByTimeAsync(0);
  const afterBind=load.mock.calls.length;
  stream.up=true;await vi.advanceTimersByTimeAsync(30_000*6);
  expect(load.mock.calls.length).toBe(afterBind);
  stream.up=false;await vi.advanceTimersByTimeAsync(30_000);
  expect(load.mock.calls.length).toBe(afterBind+1);
  await vi.advanceTimersByTimeAsync(30_000);
  expect(load.mock.calls.length).toBe(afterBind+2);
});

it('does not read on the timer while signed out, as before',async()=>{
  const {api,load}=transport();
  stop=startAccountUiPreferences(api);
  await vi.advanceTimersByTimeAsync(30_000*2);
  expect(load).not.toHaveBeenCalled();
});
