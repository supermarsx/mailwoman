import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor,within} from '@solidjs/testing-library';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import {startNotificationCenter,notificationNotices,notificationUnread,changeCenterNotice,refreshNotificationCenter,publishAvailableUpdate,type CenterNotice} from '../../state/notificationCenter.ts';
import {settingsRequest} from '../../state/settingsRequest.ts';
import Notifications from './index.tsx';
import {toast} from '../../components/toast/toast.ts';
vi.mock('../../components/toast/toast.ts',()=>({toast:{success:vi.fn(),error:vi.fn()},registerToastObstacle:()=>()=>undefined}));
const fixture=vi.hoisted(()=>({rows:[] as CenterNotice[],fetch:vi.fn(),stateChange:undefined as undefined|((change:{changed:Record<string,Record<string,string>>})=>void),connection:undefined as undefined|((value:string)=>void)}));
vi.mock('../../realtime/context.ts',async()=>{const {createSignal}=await import('solid-js');const [state,setState]=createSignal('idle');fixture.connection=setState;return {useRealtime:()=>({connection:{state},onStateChange:(fn:typeof fixture.stateChange)=>{fixture.stateChange=fn;return ()=>{fixture.stateChange=undefined;};}})};});
vi.mock('../../api/http.ts',()=>({apiFetch:(...args:unknown[])=>fixture.fetch(...args)}));
let stop:(()=>void)|undefined;
beforeEach(()=>{
 vi.mocked(toast.success).mockClear();vi.mocked(toast.error).mockClear();fixture.connection?.('idle');
 fixture.rows=[{id:'security-one',ts:100,kind:'session',event:'fingerprintMismatch',severity:'warning',state:'active',read:false},{id:'login-one',ts:90,kind:'login',event:'success',severity:'info',state:'active',read:false}];
 fixture.fetch.mockReset().mockImplementation(async(_url:string,init:RequestInit)=>{
  expect(new Headers(init.headers).get('x-mailwoman-storage-authority')).toBe('notification_test_authority');
  if(init.method==='POST'){
   const request=JSON.parse(String(init.body)) as {id:string;action:string};let row=fixture.rows.find(row=>row.id===request.id);
   if(!row&&request.action==='publish-update'){row={id:request.id,ts:110,kind:'update',event:'newVersion',severity:'info',state:'active',read:false};fixture.rows.push(row);}
   if(!row)return new Response('{}',{status:404});
   if(['dismiss','archive','restore'].includes(request.action))row.state=request.action==='dismiss'?'dismissed':request.action==='archive'?'archived':'active';
   if(['dismiss','archive','read'].includes(request.action))row.read=true;
   return new Response(JSON.stringify({notice:row}));
  }
  return new Response(JSON.stringify({notices:fixture.rows}));
 });
 activateAccountPreferences('account','notification_test_authority');stop=startNotificationCenter();
});
afterEach(()=>{cleanup();stop?.();stop=undefined;suspendAccountPreferences();});
it('ranges and toggles notices and archives only the selection',async()=>{
 render(()=><Notifications/>);await screen.findByText('Successful sign-in');
 const rows=[...document.querySelectorAll<HTMLElement>('[data-selection-id]')];
 fireEvent.click(rows[0]!,{ctrlKey:true});fireEvent.click(rows[1]!,{shiftKey:true});fireEvent.click(rows[0]!,{metaKey:true});
 expect(rows.map(row=>row.dataset.bulkSelected)).toEqual(['false','true']);
 fireEvent.click(within(screen.getByRole('toolbar',{name:'Selection actions'})).getByRole('button',{name:'Archive'}));
 await waitFor(()=>expect(notificationNotices().find(row=>row.id==='login-one')?.state).toBe('archived'));
 expect(notificationNotices().find(row=>row.id==='security-one')?.state).toBe('active');
});
it('dismisses, archives and restores notifications across a refresh and opens the relevant security panel',async()=>{
 render(()=><Notifications/>);await screen.findByText('Successful sign-in');expect(notificationUnread()).toBe(2);
 const warning=screen.getByText(/does not match the one that signed in/).closest('li')!;
 fireEvent.click(within(warning).getByRole('button',{name:'Open related action'}));expect(settingsRequest()).toMatchObject({section:'security',card:'session-alerts-card'});await waitFor(()=>expect(notificationUnread()).toBe(1));
 fireEvent.click(within(screen.getByText(/does not match the one that signed in/).closest('li')!).getByRole('button',{name:'Dismiss'}));await waitFor(()=>expect(notificationNotices().find(n=>n.id==='security-one')?.state).toBe('dismissed'));
 fireEvent.click(screen.getByRole('button',{name:'Dismissed'}));const dismissed=screen.getByText(/does not match/).closest('li')!;fireEvent.click(within(dismissed).getByRole('button',{name:'Archive'}));await waitFor(()=>expect(notificationNotices().find(n=>n.id==='security-one')?.state).toBe('archived'));
 await refreshNotificationCenter();fireEvent.click(screen.getByRole('button',{name:'Archived'}));fireEvent.click(within(screen.getByText(/does not match/).closest('li')!).getByRole('button',{name:'Restore'}));await waitFor(()=>expect(notificationNotices().find(n=>n.id==='security-one')?.state).toBe('active'));
});
it('publishes an actual available build once and preserves a dismissed update on repeated checks',async()=>{
 await waitFor(()=>expect(notificationNotices()).toHaveLength(2));publishAvailableUpdate('26.20+new-build');await waitFor(()=>expect(notificationNotices()).toHaveLength(3));await changeCenterNotice('update:26.20+new-build','dismiss');publishAvailableUpdate('26.20+new-build');await waitFor(()=>expect(fixture.fetch).toHaveBeenCalledTimes(4));expect(notificationNotices().find(n=>n.kind==='update')?.state).toBe('dismissed');
});
it('discards a delayed response after logout and clears private notifications immediately',async()=>{
 await waitFor(()=>expect(notificationNotices()).toHaveLength(2));let release!:(value:Response)=>void;fixture.fetch.mockImplementationOnce(()=>new Promise(resolve=>{release=resolve;}));const pending=refreshNotificationCenter();suspendAccountPreferences();expect(notificationNotices()).toEqual([]);release(new Response(JSON.stringify({notices:fixture.rows})));await pending;expect(notificationNotices()).toEqual([]);
});

it('confirms notification actions and keeps search fields accessible without a manual refresh button',async()=>{
 render(()=><Notifications/>);await screen.findByText('Successful sign-in');
 expect(screen.getByRole('searchbox',{name:'Search'})).toBeInTheDocument();expect(screen.queryByRole('button',{name:'Refresh'})).toBeNull();
 const row=screen.getByText('Successful sign-in').closest('li')!;fireEvent.click(within(row).getByRole('button',{name:'Dismiss'}));
 await waitFor(()=>expect(toast.success).toHaveBeenCalledWith('Dismissed · 1'));
});
it('refreshes the current principal on notification pushes keyed by the sign-in parent and after reconnecting',async()=>{
 await waitFor(()=>expect(notificationNotices()).toHaveLength(2));
 fixture.rows.push({...fixture.rows[0]!,id:'pushed-notice'});fixture.stateChange?.({changed:{parent:{Notification:'n2'}}});
 await waitFor(()=>expect(notificationNotices()).toHaveLength(3));
 fixture.rows.push({...fixture.rows[0]!,id:'offline-notice'});fixture.connection?.('connecting');fixture.connection?.('online');
 await waitFor(()=>expect(notificationNotices()).toHaveLength(4));
});
