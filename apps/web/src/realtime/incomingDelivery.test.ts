import {afterEach,expect,it,vi} from 'vitest';
import {createIncomingDelivery} from './incomingDelivery.ts';
import type {IncomingMail} from './incomingMail.ts';
const batch={owner:{id:'one',generation:1,storageAuthority:'authority_one'},accountId:'one',messages:[{id:'new'}]} as IncomingMail;
afterEach(()=>vi.useRealTimers());
it('retries transient notification-policy failures without another delivery event',async()=>{
 vi.useFakeTimers();const deliver=vi.fn().mockRejectedValueOnce({status:503}).mockResolvedValue(undefined);const queue=createIncomingDelivery({deliver,current:()=>true});queue.send(batch);queue.send(batch);await vi.advanceTimersByTimeAsync(1000);expect(deliver).toHaveBeenCalledTimes(2);queue.clear();
});
it('honours retry-after and clears queued alerts when their authority ends',async()=>{
 vi.useFakeTimers();let current=true;const deliver=vi.fn().mockRejectedValue({status:429,retryAfterSecs:10});const queue=createIncomingDelivery({deliver,current:()=>current});queue.send(batch);await vi.advanceTimersByTimeAsync(1000);expect(deliver).toHaveBeenCalledOnce();current=false;await vi.advanceTimersByTimeAsync(10000);expect(deliver).toHaveBeenCalledOnce();queue.clear();
});
it('does not retry a policy refusal or a stale mailbox request',async()=>{
 vi.useFakeTimers();const deliver=vi.fn().mockRejectedValue({status:403});const queue=createIncomingDelivery({deliver,current:()=>true});queue.send(batch);await vi.advanceTimersByTimeAsync(60000);expect(deliver).toHaveBeenCalledOnce();queue.clear();
});
