import {beforeEach,describe,expect,it,vi} from 'vitest';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import {NextcloudService,type Fetcher} from './service.ts';
beforeEach(()=>{suspendAccountPreferences();activateAccountPreferences('alice');});
describe('Nextcloud operation account boundary',()=>{
  it('sends the authenticated account fence and authoritative message reference',async()=>{
    const fetcher=vi.fn<Fetcher>(async()=>new Response(JSON.stringify({entry:{path:'/note.txt'}})));
    const service=new NextcloudService(fetcher);
    await service.saveTo('blob','/','message');
    expect(JSON.parse(fetcher.mock.calls[0]?.[1]?.body as string)).toEqual({blobId:'blob',path:'/',emailId:'message',accountId:'alice'});
  });
  it('rejects a completed old-account attachment instead of adding it to a new draft',async()=>{
    let complete!:(response:Response)=>void;
    const fetcher=vi.fn(()=>new Promise<Response>(resolve=>{complete=resolve;}));
    const service=new NextcloudService(fetcher);const pending=service.attach(['/note.txt']);
    suspendAccountPreferences();activateAccountPreferences('bob');
    complete(new Response(JSON.stringify({attachments:[{blobId:'alice-blob'}]})));
    await expect(pending).rejects.toThrow('active account changed');expect(fetcher).toHaveBeenCalledTimes(1);
  });
  it('refuses an old-account draft before making a request',async()=>{
    const fetcher=vi.fn();
    await expect(new NextcloudService(fetcher).forAccount('bob').attach(['/note.txt'])).rejects.toThrow('different account');
    expect(fetcher).not.toHaveBeenCalled();
  });
  it('does not contact Nextcloud while authentication is suspended',async()=>{
    const fetcher=vi.fn();suspendAccountPreferences();
    await expect(new NextcloudService(fetcher).attach(['/note.txt'])).rejects.toThrow('Sign in');
    expect(fetcher).not.toHaveBeenCalled();
  });
});

describe('large file upload to Nextcloud',()=>{
  it('sends the file in chunks to the transfer, then assembles it',async()=>{
    const calls:{url:string;method:string;size:number}[]=[];
    const fetcher=vi.fn<Fetcher>(async(url,init)=>{
      const body=init?.body;const size=body instanceof Blob?body.size:typeof body==='string'?body.length:0;
      calls.push({url,method:init?.method??'GET',size});
      if(url.endsWith('/api/nextcloud/upload'))return new Response(JSON.stringify({transfer:'a'.repeat(32),path:'/Mail attachments/big.bin',chunkBytes:4}));
      if(url.includes('/finish'))return new Response(JSON.stringify({path:'/Mail attachments/big.bin',name:'big.bin',size:10}));
      return new Response(null,{status:204});
    });
    const service=new NextcloudService(fetcher);service.selectInstance('cloud');
    const progress:number[]=[];
    const file=new File([new Uint8Array(10)],'big.bin');
    const stored=await service.uploadFile(file,sent=>progress.push(sent));
    expect(stored.path).toBe('/Mail attachments/big.bin');
    expect(progress).toEqual([4,8,10]);
    const puts=calls.filter(call=>call.method==='PUT');
    expect(puts.map(call=>call.size)).toEqual([4,4,2]);
    expect(puts[0]!.url).toContain(`/api/nextcloud/upload/${'a'.repeat(32)}/1?accountId=alice&path=%2FMail+attachments%2Fbig.bin&instanceId=cloud`);
    expect(puts[2]!.url).toContain(`/${'a'.repeat(32)}/3?`);
    const finish=fetcher.mock.calls.find(call=>call[0].includes('/finish'))!;
    expect(JSON.parse(finish[1]!.body as string)).toEqual({path:'/Mail attachments/big.bin',size:10,accountId:'alice',instanceId:'cloud'});
  });
  it('drops the partial chunks when a chunk is refused',async()=>{
    const fetcher=vi.fn<Fetcher>(async(url,init)=>{
      if(url.endsWith('/api/nextcloud/upload'))return new Response(JSON.stringify({transfer:'b'.repeat(32),path:'/Mail attachments/x.bin',chunkBytes:4}));
      if(init?.method==='PUT')return new Response(JSON.stringify({error:'full'}),{status:507});
      return new Response(null,{status:204});
    });
    await expect(new NextcloudService(fetcher).uploadFile(new File([new Uint8Array(6)],'x.bin'))).rejects.toThrow('507');
    await vi.waitFor(()=>expect(fetcher.mock.calls.some(call=>call[0].includes('/abort'))).toBe(true));
    expect(fetcher.mock.calls.some(call=>call[0].includes('/finish'))).toBe(false);
  });
});
