import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,screen,waitFor,within} from '@solidjs/testing-library';
import IncidentsTab from './IncidentsTab.tsx';
import {renderInShell,holding} from '../shell/testkit.tsx';
const service=vi.hoisted(()=>({request:vi.fn()}));
vi.mock('../shell/adminApi.ts',()=>({adminRequest:service.request}));
vi.mock('../../../api/admin/settings.ts',async original=>({...await original<typeof import('../../../api/admin/settings.ts')>(),listAdminSettings:vi.fn(async()=>({kind:'ok',data:[]}))}));
afterEach(()=>{cleanup();service.request.mockReset();});
const item={id:'incident-1',updatedAt:1800000000,version:'version-1',deadlines:{notification:1800259200},incident:{title:'Mail service disruption',category:'service-disruption',status:'recovering',significance:'significant',source:'health',component:'mail',detectedAt:1800000000,awareAt:1800000000,firstHealthyAt:1800000060,verifiedAt:null,owner:'Operations',impact:'Delayed mail',rootCause:'',measures:'',recoveryPlan:'',recoveryEvidence:'',recoveryTimeMinutes:null,recoveryPointMinutes:null,auditReference:'',events:[{at:1800000060,kind:'recovery-observed',note:'Two healthy samples observed; operator verification is still required.',actor:'system'}],reports:[]}};
function setup(){service.request.mockImplementation(async(_path:string,options?:{method?:string})=>({data:options?.method?{ok:true}:{items:[item],automatic:true}}));}
it('distinguishes observed recovery and records an actual authority submission',async()=>{
 setup();renderInShell(()=><IncidentsTab/>);const title=await screen.findByText('Mail service disruption');fireEvent.click(title.closest('summary')!);
 expect(screen.getByText(/operator verification is still required/)).toBeVisible();
 const record=screen.getByRole('button',{name:'Record authority submission'});await waitFor(()=>expect(record).toBeEnabled());fireEvent.click(record);
 const dialog=screen.getByTestId('incident-report');fireEvent.input(within(dialog).getByLabelText('Authority reference'),{target:{value:'Authority-123'}});fireEvent.click(within(dialog).getByRole('button',{name:'Save'}));
 await waitFor(()=>expect(service.request).toHaveBeenCalledWith('/incidents/incident-1/reports',expect.objectContaining({method:'POST',body:expect.objectContaining({phase:'early-warning',reference:'Authority-123'})})));
});
it('sends a source-bound version when editing and does not silently verify recovery',async()=>{
 setup();renderInShell(()=><IncidentsTab/>);await screen.findByText('Mail service disruption');const edit=screen.getByRole('button',{name:'Edit'});await waitFor(()=>expect(edit).toBeEnabled());fireEvent.click(edit);
 const dialog=screen.getByTestId('incident-editor');fireEvent.input(within(dialog).getByLabelText('Recovery evidence'),{target:{value:'Delivery test completed'}});fireEvent.click(within(dialog).getByRole('button',{name:'Save'}));
 await waitFor(()=>expect(service.request).toHaveBeenCalledWith('/incidents/incident-1',expect.objectContaining({method:'PUT',body:expect.objectContaining({version:'version-1',status:'recovering',recoveryEvidence:'Delivery test completed'})})));
});
it('keeps incident mutation controls disabled for read-only administrators',async()=>{
 setup();renderInShell(()=><IncidentsTab/>,{loadMe:holding(['privacy.read'])});await screen.findByText('Mail service disruption');expect(screen.getByRole('button',{name:'Edit'})).toBeDisabled();expect(screen.getByRole('button',{name:'Add incident'})).toBeDisabled();expect(screen.getByRole('button',{name:'Check signals now'})).toBeDisabled();
});
