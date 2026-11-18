import {afterEach,expect,it,vi} from 'vitest';
import {createConnectionStability} from './connectionStability.ts';
import {createConnection} from './connection.ts';
afterEach(()=>vi.useRealTimers());
it('counts established losses, ignores startup and duplicate outage reports, and waits for stable recovery',()=>{
 vi.useFakeTimers();const changed=vi.fn(),tracker=createConnectionStability(changed);
 tracker.observe(false);tracker.observe(false);expect(changed).not.toHaveBeenCalled();
 for(let i=0;i<3;i++){tracker.observe(true);tracker.observe(false);tracker.observe(false);vi.advanceTimersByTime(1000);}
 expect(changed.mock.calls).toEqual([[true]]);
 tracker.observe(true);vi.advanceTimersByTime(59_999);expect(changed.mock.calls).toEqual([[true]]);
 tracker.observe(false);tracker.observe(true);vi.advanceTimersByTime(60_000);expect(changed.mock.calls).toEqual([[true],[false]]);tracker.reset();
});
it('does not accumulate isolated drops and resets across sessions',()=>{
 vi.useFakeTimers();const changed=vi.fn(),tracker=createConnectionStability(changed);
 for(let i=0;i<4;i++){tracker.observe(true);tracker.observe(false);vi.advanceTimersByTime(61_000);}
 expect(changed).not.toHaveBeenCalled();tracker.reset();
 for(let i=0;i<3;i++){tracker.observe(true);tracker.observe(false);}
 expect(changed).toHaveBeenLastCalledWith(true);tracker.observe(true);tracker.reset();expect(changed).toHaveBeenLastCalledWith(false);
 vi.advanceTimersByTime(120_000);expect(changed).toHaveBeenCalledTimes(2);
});
it('detects live socket reconnect loops and clears on logout without hiding authentication expiry',()=>{
 vi.useFakeTimers();const connection=createConnection();
 for(let i=0;i<3;i++){connection.report('open','ws');connection.report('reconnecting','ws');}
 expect(connection.unstable()).toBe(true);expect(connection.state()).toBe('connecting');
 connection.report('open','ws');expect(connection.unstable()).toBe(true);
 connection.setAuthExpired();expect(connection.unstable()).toBe(false);expect(connection.state()).toBe('auth-expired');
 connection.report('closed','offline');expect(connection.state()).toBe('idle');expect(vi.getTimerCount()).toBe(0);
});
