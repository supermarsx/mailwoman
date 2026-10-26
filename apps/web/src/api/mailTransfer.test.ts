import {describe,it,expect,vi,beforeEach} from 'vitest';
import {mailTransferApi,MailTransferError} from './mailTransfer.ts';
import {apiFetch} from './http.ts';
vi.mock('./http.ts',()=>({apiFetch:vi.fn()}));
describe('mail transfer transport',()=>{
  beforeEach(()=>vi.mocked(apiFetch).mockReset());
  it('uses the documented explicit mailbox selector and sends the file once',async()=>{
    vi.mocked(apiFetch).mockResolvedValue(new Response(JSON.stringify({imported:1,failed:0,total:1,mailboxId:'a/b'})));
    const file=new File(['message'],'archive.mbox');const signal=new AbortController().signal;
    await mailTransferApi.import('mbox',file,'a/b',signal);
    expect(apiFetch).toHaveBeenCalledOnce();expect(apiFetch).toHaveBeenCalledWith('/api/import/mbox?mailboxId=a%2Fb',expect.objectContaining({method:'POST',body:file,signal,headers:{'content-type':'application/mbox'}}));
  });
  it('refuses an absent destination without uploading',async()=>{
    await expect(mailTransferApi.import('eml',new File(['x'],'x.eml'),'',new AbortController().signal)).rejects.toThrow('destination');
    expect(apiFetch).not.toHaveBeenCalled();
  });
  it('retains a policy refusal instead of parsing it as a successful import',async()=>{
    vi.mocked(apiFetch).mockResolvedValue(new Response('{}',{status:403}));
    await expect(mailTransferApi.formats(new AbortController().signal)).rejects.toEqual(new MailTransferError(403));
  });
});
