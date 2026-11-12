import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,screen,waitFor,within} from '@solidjs/testing-library';
import NotificationsTab from './NotificationsTab.tsx';
import {renderInShell,holding} from '../shell/testkit.tsx';
const service=vi.hoisted(()=>({request:vi.fn(),status:'uncertain'}));
vi.mock('../shell/adminApi.ts',async original=>({...await original<typeof import('../shell/adminApi.ts')>(),adminRequest:service.request}));
vi.mock('../../../api/admin/settings.ts',async original=>({...await original<typeof import('../../../api/admin/settings.ts')>(),listAdminSettings:vi.fn(async()=>({kind:'ok',data:[]}))}));
afterEach(()=>{cleanup();service.request.mockReset();service.status='uncertain';});
function setup():void{service.request.mockImplementation(async(_path:string,options?:{method?:string})=>{
 if(options?.method==='POST'){service.status='pending';return {data:{status:'pending'}};}
 return {data:{passwordConfigured:false,items:[{id:'notice',event:'dsr-admin:request-1',status:service.status,at:1800000000,attempts:1,error:'Acceptance unknown'}]}};
});}

it('requires confirmation before retrying an uncertain delivery and sends explicit duplicate-risk acknowledgement',async()=>{
 setup();renderInShell(()=><NotificationsTab/>);
 const retry=await screen.findByRole('button',{name:'Retry'});await waitFor(()=>expect(retry).toBeEnabled());fireEvent.click(retry);
 const dialog=screen.getByRole('alertdialog');expect(within(dialog).getByText(/may deliver a duplicate/)).toBeVisible();
 expect(service.request).not.toHaveBeenCalledWith('/compliance/mail/notice/resolve',expect.anything());
 fireEvent.click(within(dialog).getByRole('button',{name:'Retry'}));
 await waitFor(()=>expect(service.request).toHaveBeenCalledWith('/compliance/mail/notice/resolve',{method:'POST',body:{action:'retry',acceptDuplicateRisk:true}}));
 expect(await screen.findByText('Queued')).toBeVisible();
});

it('keeps delivery resolution and test sending unavailable to read-only administrators',async()=>{
 setup();renderInShell(()=><NotificationsTab/>,{loadMe:holding(['settings.read'])});
 expect(await screen.findByText('Acceptance uncertain')).toBeVisible();
 expect(screen.queryByRole('button',{name:'Retry'})).toBeNull();
 expect(screen.getByRole('button',{name:'Send test email'})).toBeDisabled();
});
