import {describe,it,expect,vi} from 'vitest';
import {screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {EgressDestinations} from './EgressDestinations.tsx';
import {renderInShell} from './shell/testkit.tsx';
import type {EgressDestinationsApi} from '../../api/admin/egressDestinations.ts';
const row={id:'analytics-only',trafficClass:'analytics',destination:'stats.corp.test:443',note:'Counts',createdBy:'root',createdAt:1};
function service():EgressDestinationsApi{return {list:vi.fn(async()=>({destinations:[row],eligibleClasses:['analytics','plugins']})),create:vi.fn(async()=>{}),remove:vi.fn(async()=>{})};}
describe('class-specific internal destinations',()=>{
  it('opens a dedicated create form with no inherited analytics permission',async()=>{
    const api=service();renderInShell(()=><EgressDestinations api={api} editor="new" readOnly={false} onNavigate={()=>{}} onBack={()=>{}}/>);
    const select=await screen.findByLabelText(/^Traffic class/);expect(select).toHaveValue('');
    expect(screen.queryByText(row.destination)).not.toBeInTheDocument();
    fireEvent.change(select,{target:{value:'plugins'}});fireEvent.input(screen.getByLabelText(/^Destination/),{target:{value:'ews.corp.test:443'}});
    fireEvent.click(screen.getByRole('button',{name:'Save'}));
    await waitFor(()=>expect(api.create).toHaveBeenCalledWith({trafficClass:'plugins',destination:'ews.corp.test:443',note:''}));
  });
  it('requires explicit revocation confirmation and removes only the selected id',async()=>{
    const api=service();renderInShell(()=><EgressDestinations api={api} readOnly={false} onNavigate={()=>{}} onBack={()=>{}}/>);
    fireEvent.click(await screen.findByRole('button',{name:'Revoke permission'}));expect(api.remove).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button',{name:'Confirm revocation'}));
    await waitFor(()=>expect(api.remove).toHaveBeenCalledWith('analytics-only'));
  });
  it('disables readonly controls',async()=>{
    const api=service();vi.mocked(api.create).mockRejectedValue(new Error('refused'));
    renderInShell(()=><EgressDestinations api={api} editor="new" readOnly onNavigate={()=>{}} onBack={()=>{}}/>);
    expect(await screen.findByLabelText(/^Traffic class/)).toBeDisabled();
    expect(screen.getByRole('button',{name:'Save'})).toBeDisabled();expect(api.create).not.toHaveBeenCalled();
  });
});
