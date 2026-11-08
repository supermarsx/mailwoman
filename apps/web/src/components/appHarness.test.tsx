import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {renderWithApp} from './appHarness.tsx';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';

beforeEach(()=>{
 suspendAccountPreferences();
 vi.stubGlobal('fetch',vi.fn(async()=>new Response(JSON.stringify({}),{headers:{'content-type':'application/json'}})));
});
afterEach(()=>{suspendAccountPreferences();vi.unstubAllGlobals();});

it('releases the store auth subscriptions when its rendered fixture unmounts',()=>{
 const {app,result}=renderWithApp(()=>null);
 // A mounted store observes authority replacement and clears account selection.
 app.selectContact('mounted-selection');
 activateAccountPreferences('first-fixture-account');
 expect(app.selectedContactId()).toBeNull();
 // A disposed store must stop reacting to the next test's account changes.
 app.selectContact('disposed-selection');
 result.unmount();
 activateAccountPreferences('next-fixture-account');
 expect(app.selectedContactId()).toBe('disposed-selection');
});
