import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import {LinkedMailboxFactor} from './LinkedMailboxFactor.tsx';
import {assertPasskey} from './webauthn.ts';
vi.mock('../../i18n/index.ts',()=>({t:(key:string)=>key}));
vi.mock('./webauthn.ts',()=>({assertPasskey:vi.fn(),passkeySupported:()=>true}));
beforeEach(()=>activateAccountPreferences('1','msa1_primary_verified'));
afterEach(()=>{cleanup();suspendAccountPreferences();vi.resetAllMocks();});
describe('linked mailbox second factor',()=>{
  it('sends only the selected proof to the dedicated confirmation callback',async()=>{
    const confirm=vi.fn(async()=>undefined);
    render(()=><LinkedMailboxFactor challenge={{twofaRequired:true,challengeId:'private-link-challenge',factors:['totp','recovery']}} onConfirm={confirm} onCancel={()=>undefined} errorText={()=>'Refused'}/>);
    fireEvent.change(screen.getByRole('combobox'),{target:{value:'recovery'}});
    fireEvent.input(screen.getByLabelText('settings-2fa-recovery-code-label'),{target:{value:' recovery-code '}});
    fireEvent.click(screen.getByRole('button',{name:'settings-2fa-challenge-verify'}));
    await waitFor(()=>expect(confirm).toHaveBeenCalledExactlyOnceWith({method:'recovery',code:'recovery-code'}));
    expect(screen.getByLabelText('settings-2fa-recovery-code-label')).toHaveValue('');
  });
  it('does not dispatch a delayed passkey proof after a realm change',async()=>{
    const confirm=vi.fn(async()=>undefined);
    let release!:(value:Awaited<ReturnType<typeof assertPasskey>>)=>void;
    vi.mocked(assertPasskey).mockImplementation(()=>new Promise(resolve=>{release=resolve;}));
    render(()=><LinkedMailboxFactor challenge={{twofaRequired:true,challengeId:'challenge',factors:['webauthn'],webauthn:{challenge:'x',credentialIds:[],rpId:'localhost',userVerification:'preferred'}}} onConfirm={confirm} onCancel={()=>undefined} errorText={()=>'Refused'}/>);
    fireEvent.click(screen.getByRole('button',{name:'settings-2fa-challenge-use-passkey'}));
    activateAccountPreferences('1','msa1_other_verified');
    release({credentialId:'id',clientDataJson:'data',authenticatorData:'auth',signature:'signature',userHandle:null});
    await waitFor(()=>expect(screen.getByRole('button',{name:'linked-accounts-cancel'})).not.toBeDisabled());
    expect(confirm).not.toHaveBeenCalled();
  });
});
