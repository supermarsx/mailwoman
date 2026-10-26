import {describe,it,expect,vi} from 'vitest';
import {fireEvent,screen,within,waitFor} from '@solidjs/testing-library';
import {MaintenanceMode} from './MaintenanceMode.tsx';
import {renderInShell} from './shell/testkit.tsx';
import type {MaintenanceConfig} from '../../api/admin/maintenanceMode.ts';
const initial:MaintenanceConfig={enabled:false,message:'',retryAfterSecs:300};
describe('maintenance mode',()=>{
  it('allows an invalid stored configuration to be repaired without sending response metadata',async()=>{
    const replacement={...initial,enabled:true};
    const save=vi.fn(async(data:MaintenanceConfig)=>({data,etag:'"fixed"',status:200}));
    renderInShell(()=><MaintenanceMode readOnly={false} api={{read:async()=>({data:{...replacement,configurationInvalid:true},etag:'"invalid"',status:200}),save}}/>);
    await screen.findByText(/stored configuration could not be read/);
    fireEvent.click(screen.getByRole('button',{name:/^Save$/}));
    fireEvent.click(within(await screen.findByRole('alertdialog')).getByRole('button',{name:/^Save$/}));
    await waitFor(()=>expect(save).toHaveBeenCalledWith(replacement,'"invalid"'));
  });
  it('requires confirmation and preserves the read revision on save',async()=>{
    const save=vi.fn(async(data:MaintenanceConfig)=>({data,etag:'"two"',status:200}));
    renderInShell(()=><MaintenanceMode readOnly={false} api={{read:async()=>({data:initial,etag:'"one"',status:200}),save}}/>);
    fireEvent.click(await screen.findByLabelText('Pause user access'));
    fireEvent.click(screen.getByRole('button',{name:/^Save$/}));
    expect(save).not.toHaveBeenCalled();
    fireEvent.click(within(await screen.findByRole('alertdialog')).getByRole('button',{name:/^Save$/}));
    await waitFor(()=>expect(save).toHaveBeenCalledWith({...initial,enabled:true},'"one"'));
    expect(await screen.findByText('Maintenance mode is active')).toBeVisible();
  });
  it('does not allow read-only operators to change maintenance state',async()=>{
    const save=vi.fn();
    renderInShell(()=><MaintenanceMode readOnly api={{read:async()=>({data:initial,etag:'"one"',status:200}),save}}/>);
    expect(await screen.findByLabelText('Pause user access')).toBeDisabled();
    expect(screen.getByRole('button',{name:/^Save$/})).toBeDisabled();
    expect(save).not.toHaveBeenCalled();
  });
});
