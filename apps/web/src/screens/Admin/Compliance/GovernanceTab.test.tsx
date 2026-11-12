import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,screen,waitFor,within} from '@solidjs/testing-library';
import {ContractsTab,ControlsTab,TransfersTab} from './GovernanceTab.tsx';
import {renderInShell,holding} from '../shell/testkit.tsx';

const mock=vi.hoisted(()=>({request:vi.fn(),data:{contract:[] as Record<string,unknown>[],transfer:[],control:[] as Record<string,unknown>[],processor:[{id:'p1',name:'Hosting provider'}],catalog:[{id:'builtin:encryption',name:'Encryption',kind:'control',builtin:true,controlId:'encryption',mappings:'GDPR Art. 32; SOC 2 CC6.1; ISO 27001 A.8.24',status:'unassessed'}]}}));
vi.mock('../shell/adminApi.ts',async importOriginal=>({...await importOriginal<typeof import('../shell/adminApi.ts')>(),adminRequest:mock.request}));
afterEach(()=>{cleanup();mock.request.mockReset();mock.data.contract=[];mock.data.control=[];});
function service():void{mock.request.mockImplementation(async(_path:string,options?:{body?:{name:string;fields:Record<string,string>;kind:string}})=>{
 if(options?.body){const body=options.body;const row={id:'saved',name:body.name,builtin:false,...body.fields};mock.data[body.kind as 'contract'|'control'].push(row);return {data:row};}
 return {data:structuredClone(mock.data)};
});}

it('guides contract entry and saves the linked processor and document evidence',async()=>{
 service();renderInShell(()=><ContractsTab/>);
 const add=await screen.findByRole('button',{name:'Add record'});await waitFor(()=>expect(add).toBeEnabled());fireEvent.click(add);
 const dialog=screen.getByTestId('governance-editor');
 fireEvent.input(within(dialog).getByLabelText('Name'),{target:{value:'Hosting DPA'}});
 fireEvent.input(within(dialog).getByLabelText('Owner'),{target:{value:'Privacy team'}});
 fireEvent.input(within(dialog).getByLabelText('Counterparty'),{target:{value:'Hosting provider'}});
 fireEvent.click(within(dialog).getByRole('button',{name:'Next'}));
 fireEvent.change(within(dialog).getByLabelText('Sub-processor'),{target:{value:'p1'}});
 fireEvent.input(within(dialog).getByLabelText('Contract document URL'),{target:{value:'https://example.org/contracts/dpa-v1'}});
 fireEvent.click(within(dialog).getByRole('button',{name:'Next'}));
 expect(within(dialog).getByText('https://example.org/contracts/dpa-v1')).toBeVisible();
 fireEvent.click(within(dialog).getByRole('button',{name:'Save'}));
 await waitFor(()=>expect(mock.request).toHaveBeenCalledWith('/compliance/records',expect.objectContaining({body:expect.objectContaining({kind:'contract',fields:expect.objectContaining({processorId:'p1',documentUrl:'https://example.org/contracts/dpa-v1'})})})));
 expect(await screen.findByRole('rowheader',{name:'Hosting DPA'})).toBeVisible();
});

it('starts mapped controls unassessed and restricts editing without write permission',async()=>{
 service();renderInShell(()=><ControlsTab/>,{loadMe:holding(['settings.read'])});
 expect(await screen.findByText('Unassessed')).toBeVisible();
 expect(await screen.findByText(/GDPR Art. 32/)).toBeVisible();
 expect(screen.getByRole('button',{name:'Edit'})).toBeDisabled();
 expect(screen.getByRole('button',{name:'Add record'})).toBeDisabled();
});

it('includes both internal and international transfers in the guided form',async()=>{
 service();renderInShell(()=><TransfersTab/>);
 const add=await screen.findByRole('button',{name:'Add record'});await waitFor(()=>expect(add).toBeEnabled());fireEvent.click(add);
 const select=screen.getByLabelText('Transfer type');
 expect(within(select).getByRole('option',{name:'Internal'})).toBeInTheDocument();
 expect(within(select).getByRole('option',{name:'International'})).toBeInTheDocument();
});
