import {afterEach,expect,it,vi} from 'vitest';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {createPreferenceRefresh} from './preferenceSync.ts';

afterEach(()=>{suspendAccountPreferences();vi.useRealTimers();});
it('coalesces bursts and runs one trailing read for changes arriving during a read',async()=>{
  vi.useFakeTimers();activateAccountPreferences('one','authority-one');
  let finish!:()=>void;
  const work=vi.fn().mockImplementationOnce(()=>new Promise<void>(resolve=>{finish=resolve;})).mockResolvedValue(undefined);
  const sync=createPreferenceRefresh(work);
  try{
    sync.changed();sync.changed();sync.changed();await vi.advanceTimersByTimeAsync(80);
    expect(work).toHaveBeenCalledTimes(1);
    sync.changed();sync.changed();finish();await vi.advanceTimersByTimeAsync(80);
    expect(work).toHaveBeenCalledTimes(2);
    await vi.advanceTimersByTimeAsync(1000);expect(work).toHaveBeenCalledTimes(2);
  }finally{sync.dispose();}
});
it('cancels queued work on sign-out or disposal and refreshes after device focus',async()=>{
  vi.useFakeTimers();activateAccountPreferences('one','authority-one');
  const work=vi.fn().mockResolvedValue(undefined),sync=createPreferenceRefresh(work);
  sync.changed();suspendAccountPreferences();await vi.advanceTimersByTimeAsync(80);expect(work).not.toHaveBeenCalled();
  activateAccountPreferences('two','authority-two');window.dispatchEvent(new Event('focus'));
  await vi.advanceTimersByTimeAsync(80);expect(work).toHaveBeenCalledTimes(1);
  sync.changed();sync.dispose();await vi.advanceTimersByTimeAsync(1000);expect(work).toHaveBeenCalledTimes(1);
});
it('skips only the first stream open right after the account became active (t52w)',async()=>{
  vi.useFakeTimers();activateAccountPreferences('one','authority-one');
  const work=vi.fn().mockResolvedValue(undefined),sync=createPreferenceRefresh(work);
  try{
    sync.caughtUp(true);await vi.advanceTimersByTimeAsync(80);expect(work).not.toHaveBeenCalled();
    // A reconnect (or a later open) always refreshes.
    sync.caughtUp(false);await vi.advanceTimersByTimeAsync(80);expect(work).toHaveBeenCalledTimes(1);
    sync.caughtUp(true);await vi.advanceTimersByTimeAsync(80);expect(work).toHaveBeenCalledTimes(2);
  }finally{sync.dispose();}
});
it('refreshes on a first open that comes late, or as a reconnect before any open (t52w)',async()=>{
  vi.useFakeTimers();activateAccountPreferences('one','authority-one');
  const work=vi.fn().mockResolvedValue(undefined),sync=createPreferenceRefresh(work);
  try{
    await vi.advanceTimersByTimeAsync(10_000);sync.caughtUp(true);await vi.advanceTimersByTimeAsync(80);expect(work).toHaveBeenCalledTimes(1);
    activateAccountPreferences('two','authority-two');sync.caughtUp(false);await vi.advanceTimersByTimeAsync(80);expect(work).toHaveBeenCalledTimes(2);
  }finally{sync.dispose();}
});
