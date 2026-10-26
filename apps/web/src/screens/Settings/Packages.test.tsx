import {it,expect,vi} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {Packages} from './Packages.tsx';
import {createSettingsNavigation} from './navigation.tsx';
import type {PackagesApi,PackagesView} from '../../api/packages.ts';
function fixture(enabled=false) {
  let state:PackagesView={accountId:'one',revision:'r1',available:true,refreshSecs:900,state:{enabled,packages:[]}};
  const api:PackagesApi={load:vi.fn(async()=>structuredClone(state)),preference:vi.fn(async(value)=>{state={...state,state:{...state.state,enabled:value}};}),save:vi.fn(async()=>{}),action:vi.fn(async()=>{})};
  const navigation=createSettingsNavigation({section:'packages'});
  render(()=><Packages accountId="one" api={api} navigation={navigation}/>);
  return {api,navigation};
}
it('is off until explicitly enabled and never starts tracking during load',async()=>{
  const {api}=fixture();
  const checkbox=await screen.findByRole('checkbox',{name:'Enable package tracking for my account'});
  expect(checkbox).not.toBeChecked();expect(screen.getByRole('button',{name:'Add package'})).toBeDisabled();
  expect(api.action).not.toHaveBeenCalled();
  fireEvent.click(checkbox);await waitFor(()=>expect(api.preference).toHaveBeenCalledWith(true,'r1'));
  await waitFor(()=>expect(screen.getByRole('button',{name:'Add package'})).toBeEnabled());
});
it('uses a dedicated editor with private local save instead of inline controls',async()=>{
  const {api}=fixture(true);
  await screen.findByRole('table');expect(screen.queryByLabelText('Tracking number')).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole('button',{name:'Add package'}));
  await screen.findByLabelText('Tracking number');expect(screen.queryByRole('table')).not.toBeInTheDocument();
  fireEvent.input(screen.getByLabelText('Tracking number'),{target:{value:'RA123456789PT'}});
  fireEvent.input(screen.getByLabelText('Carrier code'),{target:{value:'portugal-ctt'}});
  fireEvent.click(screen.getByRole('button',{name:'Save package'}));
  await waitFor(()=>expect(api.save).toHaveBeenCalledWith({label:'',trackingNumber:'RA123456789PT',carrier:'portugal-ctt'},'r1',undefined));
  expect(api.action).not.toHaveBeenCalled();await screen.findByRole('table');
});
