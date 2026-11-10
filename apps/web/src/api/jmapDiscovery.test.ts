import {afterEach,expect,it,vi} from 'vitest';
import {createClient} from './client.ts';
import {jmapApiPath} from './jmapDiscovery.ts';
const base='https://mail.example.org/mail';
const session={apiUrl:`${base}/jmap/custom-api`,capabilities:{},accounts:{},primaryAccounts:{},username:'user',state:'s'};
const json=(body:unknown,status=200)=>new Response(JSON.stringify(body),{status,headers:{'content-type':'application/json'}});
afterEach(()=>vi.unstubAllGlobals());
it('discovers the native session and consumes its API address with bearer auth',async()=>{
 const fetcher=vi.fn(async(input:RequestInfo|URL)=>String(input).endsWith('/.well-known/jmap')?json(session):json({methodResponses:[],sessionState:'s'}));vi.stubGlobal('fetch',fetcher);
 const client=createClient(base,{token:async()=> 'native-token'});await client.session();await client.jmap({using:[],methodCalls:[]});
 expect(fetcher.mock.calls.map(call=>String(call[0]))).toEqual([`${base}/.well-known/jmap`,`${base}/jmap/custom-api`]);
 const init=(fetcher.mock.calls[1] as unknown as [string,RequestInit])[1];expect(new Headers(init.headers).get('authorization')).toBe('Bearer native-token');expect(init.credentials).toBe('omit');
});
it('falls back to the legacy Session route only when discovery is missing',async()=>{
 const fetcher=vi.fn(async(input:RequestInfo|URL)=>String(input).endsWith('/.well-known/jmap')?json({},404):json(session));vi.stubGlobal('fetch',fetcher);
 await createClient(base,{token:async()=> 'token'}).session();expect(fetcher.mock.calls.map(call=>String(call[0]))).toEqual([`${base}/.well-known/jmap`,`${base}/jmap/session`]);
});
it('does not retry authentication refusals or send credentials to an advertised foreign host',async()=>{
 const fetcher=vi.fn(async()=>json({},401));vi.stubGlobal('fetch',fetcher);await expect(createClient(base,{token:async()=> 'token'}).session()).rejects.toMatchObject({status:401});expect(fetcher).toHaveBeenCalledTimes(1);
 fetcher.mockImplementation(async()=>json({...session,apiUrl:'https://upstream.invalid/jmap/api'}));await expect(createClient(base,{token:async()=> 'token'}).session()).rejects.toMatchObject({status:502});expect(fetcher).toHaveBeenCalledTimes(2);
});
it('preserves the browser Session route and cookie authentication',async()=>{
 const fetcher=vi.fn(async(_input:RequestInfo|URL)=>json(session));vi.stubGlobal('fetch',fetcher);await createClient('/mail').session();expect(fetcher.mock.calls[0]?.[0]).toBe('/mail/jmap/session');
});
it('resolves deployment prefixes while refusing upstreams, credentials, and paths outside JMAP',()=>{
 expect(jmapApiPath('/mail/jmap/api',base)).toBe('/jmap/api');
 for(const url of ['https://foreign.invalid/mail/jmap/api','https://u:p@mail.example.org/mail/jmap/api','/mail/api/password','/jmap/api','/mail/jmap/api#fragment','/mail/jmap/api?to=elsewhere'])expect(()=>jmapApiPath(url,base)).toThrow();
});
