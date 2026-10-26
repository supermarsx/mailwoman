import {describe,it,expect,vi} from 'vitest';
import {packagesApi} from './packages.ts';
import type {apiFetch} from './http.ts';
describe('package account authority',()=>{
  it('rejects metadata returned after an account switch',async()=>{
    const api=packagesApi('one',vi.fn<typeof apiFetch>(async()=>new Response(JSON.stringify({accountId:'two',state:{packages:[]}}))));
    await expect(api.load()).rejects.toMatchObject({code:'packageChanged'});
  });
  it('sends captured account and revision and only refreshes explicitly',async()=>{
    const fetcher=vi.fn<typeof apiFetch>(async()=>new Response('{}'));
    const api=packagesApi('one',fetcher);
    expect(fetcher).not.toHaveBeenCalled();
    await api.save({label:'Private',trackingNumber:'12345',carrier:'ups'},'rev');
    expect(fetcher.mock.calls[0]?.[0]).toBe('/api/account/packages');
    expect(JSON.parse(String(fetcher.mock.calls[0]?.[1]?.body))).toEqual({accountId:'one',revision:'rev',label:'Private',trackingNumber:'12345',carrier:'ups'});
    await api.action('local/id','new-rev','refresh');
    expect(fetcher.mock.calls[1]?.[0]).toBe('/api/account/packages/local%2Fid/refresh');
    expect(JSON.parse(String(fetcher.mock.calls[1]?.[1]?.body))).toEqual({accountId:'one',revision:'new-rev'});
  });
});
