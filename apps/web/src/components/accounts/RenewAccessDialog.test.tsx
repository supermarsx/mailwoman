import {cleanup,fireEvent,render,screen} from '@solidjs/testing-library';
import {afterEach,describe,expect,it,vi} from 'vitest';
import {ApiError} from '../../api/client.ts';
import type {LinkedMailbox} from '../../api/linkedAccounts.ts';
import {RenewAccessDialog} from './RenewAccessDialog.tsx';
import {linkedAccountsRevision} from '../../api/linkedMailboxSession.ts';
import {toast,toasts} from '../toast/index.ts';

vi.mock('../../i18n/index.ts',()=>({t:(key:string,args?:Record<string,string>)=>args?.name!==undefined?`${key}:${args.name}`:key,isolate:(value:string)=>value,loadCatalog:async()=>undefined}));
vi.mock('../../api/publicConfig.ts',()=>({publicConfig:async()=>({antiBot:{mailbox:false}})}));
vi.mock('../../api/accountPreferenceLifecycle.ts',()=>({preferenceAccount:()=>({storageAuthority:'msa1_owner'}),preferenceAccountIsCurrent:()=>true}));
afterEach(cleanup);

const expired:LinkedMailbox={id:'child',accountId:'1',address:'child@example.org',displayName:'Child',status:'reauthRequired',revision:'v1',renewal:'signIn',renewalReason:'credentialRejected'};
const flush=async()=>{for(let i=0;i<6;i++)await Promise.resolve();};

describe('Renew access dialog (t48)',()=>{
  it('signs the same link in again with only the password and reports success',async()=>{
    const renewed:LinkedMailbox={...expired,status:'ready',renewal:'automatic',renewalReason:null};
    const api={renew:vi.fn(async()=>({account:renewed})),confirmRenew:vi.fn()};
    const onRenewed=vi.fn();
    render(()=><RenewAccessDialog account={expired} label="Child" api={api} onRenewed={onRenewed} onCancel={()=>undefined}/>);
    const dialog=screen.getByRole('dialog');
    expect(dialog).toHaveAccessibleName('linked-accounts-renew-title:Child');
    expect(dialog).toHaveTextContent('linked-accounts-renew-credential-rejected');
    expect(dialog.textContent).not.toMatch(/unlink/i);
    // The address is known: only the password is asked for.
    expect(screen.queryByLabelText('linked-accounts-address')).toBeNull();
    fireEvent.input(screen.getByLabelText('linked-accounts-password'),{target:{value:'new-secret'}});
    fireEvent.click(screen.getByRole('button',{name:'linked-accounts-renew-submit'}));
    await flush();
    expect(api.renew).toHaveBeenCalledExactlyOnceWith('child',{password:'new-secret'});
    expect(onRenewed).toHaveBeenCalledExactlyOnceWith(renewed);
  });
  it('asks for the sign-in name only for an old link with no known address',async()=>{
    const api={renew:vi.fn(async()=>({account:{...expired,status:'ready' as const}})),confirmRenew:vi.fn()};
    const {address:_drop,...legacy}={...expired,renewalReason:null};
    render(()=><RenewAccessDialog account={legacy} label="Child" api={api} onRenewed={()=>undefined} onCancel={()=>undefined}/>);
    fireEvent.input(screen.getByLabelText('linked-accounts-address'),{target:{value:' child@example.org '}});
    fireEvent.input(screen.getByLabelText('linked-accounts-password'),{target:{value:'pw'}});
    fireEvent.click(screen.getByRole('button',{name:'linked-accounts-renew-submit'}));
    await flush();
    expect(api.renew).toHaveBeenCalledExactlyOnceWith('child',{username:'child@example.org',password:'pw'});
  });
  it('a rejected password keeps the dialog open with a reason and clears the field',async()=>{
    const api={renew:vi.fn(async():Promise<never>=>{throw new ApiError(401,'no');}),confirmRenew:vi.fn()};
    const onRenewed=vi.fn();
    render(()=><RenewAccessDialog account={expired} label="Child" api={api} onRenewed={onRenewed} onCancel={()=>undefined}/>);
    fireEvent.input(screen.getByLabelText('linked-accounts-password'),{target:{value:'wrong'}});
    fireEvent.click(screen.getByRole('button',{name:'linked-accounts-renew-submit'}));
    await flush();
    expect(screen.getByRole('alert')).toHaveTextContent('linked-accounts-renew-password-rejected');
    expect((screen.getByLabelText('linked-accounts-password') as HTMLInputElement).value).toBe('');
    expect(onRenewed).not.toHaveBeenCalled();
  });
  it('a mailbox with two-step verification continues in the same dialog',async()=>{
    const api={
      renew:vi.fn(async()=>({twofaRequired:true as const,challengeId:'c1',factors:['totp']})),
      confirmRenew:vi.fn(async()=>({account:{...expired,status:'ready' as const}})),
    };
    render(()=><RenewAccessDialog account={{...expired,renewalReason:'factorRequired'}} label="Child" api={api as never} onRenewed={()=>undefined} onCancel={()=>undefined}/>);
    expect(screen.getByRole('dialog')).toHaveTextContent('linked-accounts-renew-factor');
    fireEvent.input(screen.getByLabelText('linked-accounts-password'),{target:{value:'pw'}});
    fireEvent.click(screen.getByRole('button',{name:'linked-accounts-renew-submit'}));
    await flush();
    expect(screen.getByRole('dialog')).toHaveTextContent('linked-accounts-factor-title');
  });
  it('says whose password it asks for, and can still take a different sign-in name (t70)',async()=>{
    const api={renew:vi.fn(async()=>({account:{...expired,status:'ready' as const}})),confirmRenew:vi.fn()};
    render(()=><RenewAccessDialog account={expired} label="Child" api={api} onRenewed={()=>undefined} onCancel={()=>undefined}/>);
    expect(screen.getByRole('dialog')).toHaveTextContent('linked-accounts-renew-password-only');
    fireEvent.click(screen.getByRole('button',{name:'linked-accounts-renew-other-name'}));
    expect(screen.queryByRole('button',{name:'linked-accounts-renew-other-name'})).toBeNull();
    expect(screen.getByRole('dialog')).toHaveTextContent('linked-accounts-renew-help');
    fireEvent.input(screen.getByLabelText('linked-accounts-address'),{target:{value:'alias@example.org'}});
    fireEvent.input(screen.getByLabelText('linked-accounts-password'),{target:{value:'pw'}});
    fireEvent.click(screen.getByRole('button',{name:'linked-accounts-renew-submit'}));
    await flush();
    expect(api.renew).toHaveBeenCalledExactlyOnceWith('child',{username:'alias@example.org',password:'pw'});
  });
  it.each([
    [502,'linkedServerUnreachable','linked-accounts-renew-unreachable'],
    [429,'linkedRenewLimit','linked-accounts-renew-limit'],
    [401,'linkedPasswordRejected','linked-accounts-renew-password-rejected'],
  ])('a %i %s refusal is explained (t70)',async(status,code,text)=>{
    const api={renew:vi.fn(async():Promise<never>=>{throw new ApiError(status,code,{code});}),confirmRenew:vi.fn()};
    render(()=><RenewAccessDialog account={expired} label="Child" api={api} onRenewed={()=>undefined} onCancel={()=>undefined}/>);
    fireEvent.input(screen.getByLabelText('linked-accounts-password'),{target:{value:'pw'}});
    fireEvent.click(screen.getByRole('button',{name:'linked-accounts-renew-submit'}));
    await flush();
    expect(screen.getByRole('alert')).toHaveTextContent(text);
    expect((screen.getByLabelText('linked-accounts-password') as HTMLInputElement).value).toBe('');
  });
  it('Escape cancels',()=>{
    const onCancel=vi.fn();
    render(()=><RenewAccessDialog account={expired} label="Child" api={{renew:vi.fn(),confirmRenew:vi.fn()}} onRenewed={()=>undefined} onCancel={onCancel}/>);
    fireEvent.keyDown(screen.getByRole('dialog'),{key:'Escape'});
    expect(onCancel).toHaveBeenCalled();
  });
  // t102/t91: the link was unlinked or relinked elsewhere; the server answers 404 linkedAccountNotFound.
  it('a 404 linkedAccountNotFound closes the dialog, reloads the links and says the mailbox is no longer linked',async()=>{
    toast.clear();
    const goneError=new ApiError(404,'Linked mailbox request failed',{code:'linkedAccountNotFound'});
    const api={renew:vi.fn(async()=>{throw goneError;}),confirmRenew:vi.fn()};
    const onGone=vi.fn(),onCancel=vi.fn(),onRenewed=vi.fn();
    const revision=linkedAccountsRevision();
    render(()=><RenewAccessDialog account={expired} label="Child" api={api} onRenewed={onRenewed} onCancel={onCancel} onGone={onGone}/>);
    fireEvent.input(screen.getByLabelText('linked-accounts-password'),{target:{value:'pw'}});
    fireEvent.click(screen.getByRole('button',{name:'linked-accounts-renew-submit'}));
    await flush();
    expect(api.renew).toHaveBeenCalledTimes(1);
    expect(onGone).toHaveBeenCalledTimes(1);
    expect(onRenewed).not.toHaveBeenCalled();
    expect(linkedAccountsRevision()).not.toBe(revision);
    expect(toasts().map(item=>item.message)).toContain('linked-accounts-renew-gone');
    expect(screen.queryByRole('alert')).toBeNull();
  });
  it('the same 404 on the second-factor step closes the dialog too (onCancel when no onGone)',async()=>{
    toast.clear();
    const challenge={twofaRequired:true as const,challengeId:'c1',factors:['totp']};
    const api={renew:vi.fn(async()=>challenge),confirmRenew:vi.fn(async()=>{throw new ApiError(404,'Linked mailbox request failed',{code:'linkedAccountNotFound'});})};
    const onCancel=vi.fn();
    render(()=><RenewAccessDialog account={expired} label="Child" api={api as never} onRenewed={()=>undefined} onCancel={onCancel}/>);
    fireEvent.input(screen.getByLabelText('linked-accounts-password'),{target:{value:'pw'}});
    fireEvent.click(screen.getByRole('button',{name:'linked-accounts-renew-submit'}));
    await flush();
    const code=screen.getByLabelText('settings-2fa-totp-code-label') as HTMLInputElement;
    expect(code).toBeTruthy();
    fireEvent.input(code!,{target:{value:'123456'}});
    fireEvent.submit(code!.closest('form')!);
    await flush();
    expect(api.confirmRenew).toHaveBeenCalledTimes(1);
    expect(onCancel).toHaveBeenCalledTimes(1);
    expect(toasts().map(item=>item.message)).toContain('linked-accounts-renew-gone');
  });
});
