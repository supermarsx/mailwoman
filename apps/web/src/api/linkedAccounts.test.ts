import {afterEach,describe,expect,it,vi} from 'vitest';
import {apiFetch} from './http.ts';
import {activateAccountPreferences,suspendAccountPreferences} from './accountPreferenceLifecycle.ts';
import {createLinkedAccountsApi,type LinkedMailbox} from './linkedAccounts.ts';
vi.mock('./http.ts',()=>({apiFetch:vi.fn()}));
const row:LinkedMailbox={id:'link/1',accountId:'1',address:'child@example.org',displayName:'Child',storageAuthority:'msa1_child_authority',status:'ready',revision:'revision-1',expiresAt:42};
afterEach(()=>{suspendAccountPreferences();vi.resetAllMocks();});
describe('linked account management transport',()=>{
  it('unlinks by opaque link ID and exact quoted revision without a child selector',async()=>{
    activateAccountPreferences('1','msa1_parent_authority');
    vi.mocked(apiFetch).mockResolvedValue(new Response(null,{status:204}));
    const api=createLinkedAccountsApi();
    try {
      await api.unlink(row);
      const [url,init]=vi.mocked(apiFetch).mock.calls[0]!;
      expect(url).toBe('/api/accounts/link%2F1');
      expect(init?.method).toBe('DELETE');
      expect(new Headers(init?.headers).get('if-match')).toBe('"revision-1"');
      expect(new Headers(init?.headers).has('x-mailwoman-link')).toBe(false);
    } finally {api.dispose();}
  });
  it('aborts an in-flight management operation on same-ID realm change',async()=>{
    activateAccountPreferences('1','msa1_parent_authority');
    let release!:(response:Response)=>void;
    vi.mocked(apiFetch).mockImplementation(()=>new Promise(resolve=>{release=resolve;}));
    const api=createLinkedAccountsApi();
    try {
      const result=api.list();
      const rejected=expect(result).rejects.toMatchObject({name:'AbortError'});
      activateAccountPreferences('1','msa1_other_authority');
      expect(vi.mocked(apiFetch).mock.calls[0]?.[1]?.signal?.aborted).toBe(true);
      release(Response.json({accounts:[row]}));
      await rejected;
    } finally {api.dispose();}
  });
  it('preserves typed policy errors without exposing response diagnostic text',async()=>{
    activateAccountPreferences('1','msa1_parent_authority');
    vi.mocked(apiFetch).mockResolvedValue(Response.json({code:'linkingDisabled',error:'private diagnostic'},{status:403}));
    const api=createLinkedAccountsApi();
    try {await expect(api.list()).rejects.toMatchObject({code:'linkingDisabled',message:'Linked mailbox request failed'});} finally {api.dispose();}
  });
});
