import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import {LinkMailboxEditor} from './LinkMailboxEditor.tsx';
vi.mock('../../i18n/index.ts',()=>({t:(key:string)=>key}));
beforeEach(()=>activateAccountPreferences('1','msa1_parent_verified'));
afterEach(()=>{cleanup();suspendAccountPreferences();});
function mount(verify=vi.fn(async()=>undefined)){
  render(()=><LinkMailboxEditor enabled onVerify={verify} onCancel={()=>undefined} errorText={()=>'Safe error'}/>);
  fireEvent.input(screen.getByLabelText('linked-accounts-address'),{target:{value:'child@example.org'}});
  fireEvent.input(screen.getByLabelText('linked-accounts-password'),{target:{value:'never-store-me'}});
  return verify;
}
describe('dedicated link editor',()=>{
  it('passes credentials only to verification and clears the password afterward',async()=>{
    const verify=mount();
    fireEvent.click(screen.getByRole('button',{name:'linked-accounts-submit'}));
    await waitFor(()=>expect(verify).toHaveBeenCalledExactlyOnceWith({username:'child@example.org',password:'never-store-me'}));
    await waitFor(()=>expect(screen.getByLabelText('linked-accounts-password')).toHaveValue(''));
  });
  it('clears passwords after refusal and shows only mapped error text',async()=>{
    mount(vi.fn(async()=>{throw new Error('upstream password diagnostic');}));
    fireEvent.click(screen.getByRole('button',{name:'linked-accounts-submit'}));
    expect(await screen.findByRole('alert')).toHaveTextContent('Safe error');
    expect(screen.getByLabelText('linked-accounts-password')).toHaveValue('');
    expect(screen.queryByText('upstream password diagnostic')).not.toBeInTheDocument();
  });
  it('refuses submission after a same-ID realm change',()=>{
    const verify=mount();
    activateAccountPreferences('1','msa1_other_verified');
    fireEvent.click(screen.getByRole('button',{name:'linked-accounts-submit'}));
    expect(verify).not.toHaveBeenCalled();
    expect(screen.getByRole('alert')).toHaveTextContent('linked-accounts-changed');
  });
});
