import {afterEach,expect,it,vi} from 'vitest';
import type {PushClientOptions,PushStatus} from './pushClient.ts';
import type {PushTransport} from '../contracts/push.ts';
const fake=vi.hoisted(()=>({options:undefined as PushClientOptions|undefined}));
vi.mock('../worker/networkProxy.ts',()=>({createSharedPushClient:(options:PushClientOptions)=>{fake.options=options;return {connect:vi.fn(),close:vi.fn(),reconnect:vi.fn(),onStateChange:()=>()=>{},onMailboxChange:()=>()=>{}};}}));
import {createRealtimeController} from './controller.ts';
afterEach(()=>vi.useRealTimers());
const report=(status:PushStatus,transport:PushTransport)=>fake.options!.onStatus!(status,transport);
it('poll fallback catches up the active account even without any initial StateChange',()=>{
 vi.useFakeTimers();const catchUp=vi.fn();const controller=createRealtimeController({onCatchUp:catchUp});controller.start();
 report('degraded','poll');expect(catchUp).toHaveBeenCalledTimes(1);
 vi.advanceTimersByTime(30000);expect(catchUp).toHaveBeenCalledTimes(2);
 report('degraded','poll');vi.advanceTimersByTime(30000);expect(catchUp).toHaveBeenCalledTimes(3);
 report('open','ws');expect(catchUp).toHaveBeenCalledTimes(4);vi.advanceTimersByTime(60000);expect(catchUp).toHaveBeenCalledTimes(4);controller.stop();
});
it('stops catch-up on logout or an expired session and catches a reconnect without known state',()=>{
 vi.useFakeTimers();const catchUp=vi.fn();const controller=createRealtimeController({onCatchUp:catchUp});controller.start();
 report('degraded','poll');report('unauthorized','offline');vi.advanceTimersByTime(60000);expect(catchUp).toHaveBeenCalledTimes(1);
 report('reconnecting','ws');report('open','ws');expect(catchUp).toHaveBeenCalledTimes(2);
 report('degraded','poll');controller.stop();const count=catchUp.mock.calls.length;vi.advanceTimersByTime(60000);expect(catchUp).toHaveBeenCalledTimes(count);
});
