import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {apiFetch,setNativeTransport} from './http.ts';
import {installLinkedMailbox} from './linkedMailboxSession.ts';
import {activateAccountPreferences,suspendAccountPreferences} from './accountPreferenceLifecycle.ts';
vi.mock('../components/StepUpDialog.tsx',()=>({requestStepUp:async(options:{send:(proof:unknown,begin:boolean)=>Promise<Response>})=>(await options.send({password:'parent-proof'},false)).ok}));
const child={linkId:'child-link',accountId:'same',storageAuthority:'msa1_child_verified'};
const header=(init?:RequestInit)=>new Headers(init?.headers).get('x-mailwoman-link');
const ok=()=>Response.json({ok:true});
beforeEach(()=>{activateAccountPreferences('same',child.storageAuthority);installLinkedMailbox(child);document.cookie='mw_csrf=token; Path=/';});
afterEach(()=>{installLinkedMailbox(null);suspendAccountPreferences();setNativeTransport(null);vi.unstubAllGlobals();document.cookie='mw_csrf=; Max-Age=0; Path=/';});
describe('linked mailbox transport authority',()=>{
  it('selects audited mailbox routes but never parent management or public config',async()=>{
    const send=vi.fn(async(_url:string,_init?:RequestInit)=>ok());vi.stubGlobal('fetch',send);
    await apiFetch('/jmap/api',{method:'POST'});await apiFetch('/api/accounts');await apiFetch('/api/public-config');
    expect(header(send.mock.calls[0]?.[1])).toBe(child.linkId);
    expect(header(send.mock.calls[1]?.[1])).toBeNull();expect(header(send.mock.calls[2]?.[1])).toBeNull();
  });
  it('refuses unsupported child operations without contacting the primary',async()=>{
    const send=vi.fn();vi.stubGlobal('fetch',send);
    await expect(apiFetch('/api/masked')).rejects.toMatchObject({name:'LinkedContextUnsupportedError'});
    expect(send).not.toHaveBeenCalled();
  });
  it('uses explicit discovery probes without promoting the active selection',async()=>{
    const send=vi.fn(async(_url:string,_init?:RequestInit)=>ok());vi.stubGlobal('fetch',send);
    await apiFetch('/api/me',{mailboxContext:null});await apiFetch('/jmap/session',{mailboxContext:'other-link'});await apiFetch('/api/me');
    expect(header(send.mock.calls[0]?.[1])).toBeNull();expect(header(send.mock.calls[1]?.[1])).toBe('other-link');expect(header(send.mock.calls[2]?.[1])).toBe(child.linkId);
    expect(send.mock.calls[1]?.[1]).not.toHaveProperty('mailboxContext');
  });
  it('keeps CSRF refresh primary and retries under the original child selector',async()=>{
    const send=vi.fn(async(_url:string,_init?:RequestInit)=>ok());
    send.mockResolvedValueOnce(Response.json({error:'missing or invalid CSRF token'},{status:403}));
    send.mockResolvedValueOnce(Response.json({csrfToken:'renewed'}));vi.stubGlobal('fetch',send);
    await apiFetch('/jmap/api',{method:'POST'});
    expect(send.mock.calls[1]?.[0]).toBe('/api/me');expect(header(send.mock.calls[1]?.[1])).toBeNull();expect(header(send.mock.calls[2]?.[1])).toBe(child.linkId);
  });
  it.each([['/api/accounts/link',null],['/api/account/signatures',child.linkId]])('keeps step-up in the authority of %s',async(path,expected)=>{
    const send=vi.fn(async(_url:string,_init?:RequestInit)=>ok());
    send.mockResolvedValueOnce(Response.json({code:'step_up_required',methods:['password']},{status:403}));vi.stubGlobal('fetch',send);
    await apiFetch(path,{method:'POST'});
    expect(send.mock.calls[1]?.[0]).toBe('/api/security/step-up');expect(header(send.mock.calls[1]?.[1])).toBe(expected);
  });
  it('drops responses after context changes and never sends a selector to another origin',async()=>{
    let release!:(response:Response)=>void;
    const send=vi.fn((_url:string,_init?:RequestInit)=>new Promise<Response>(resolve=>{release=resolve;}));vi.stubGlobal('fetch',send);
    const pending=apiFetch('/api/me');const rejected=expect(pending).rejects.toMatchObject({name:'AbortError'});
    installLinkedMailbox(null);release(ok());await rejected;
    installLinkedMailbox(child);send.mockResolvedValue(ok());await apiFetch('https://outside.example/api/me');expect(header(send.mock.calls[1]?.[1])).toBeNull();
  });
  it('converts only a matching download query selector into an XHR header',async()=>{
    const send=vi.fn(async(_url:string,_init?:RequestInit)=>ok());vi.stubGlobal('fetch',send);
    await apiFetch('/jmap/download/same/blob/file?mwLink=child-link&type=text');
    expect(send.mock.calls[0]?.[0]).toBe('/jmap/download/same/blob/file?type=text');expect(header(send.mock.calls[0]?.[1])).toBe(child.linkId);
    await expect(apiFetch('/jmap/download/same/blob/file?mwLink=foreign')).rejects.toThrow();
    expect(send).toHaveBeenCalledOnce();
  });
});
