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
