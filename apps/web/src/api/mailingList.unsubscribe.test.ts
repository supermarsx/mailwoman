import {afterEach,expect,it,vi} from 'vitest';
import {apiFetch} from './http.ts';
import {unsubscribeOneClick} from './mailingList.ts';
vi.mock('./http.ts',()=>({apiFetch:vi.fn()}));
afterEach(()=>vi.resetAllMocks());
it('requires an explicit successful JSON response rather than accepting a login page',async()=>{
 vi.mocked(apiFetch).mockResolvedValueOnce(new Response('<html>Sign in</html>',{status:200})).mockResolvedValueOnce(Response.json({ok:false,reason:'refused'})).mockResolvedValueOnce(Response.json({ok:true}));
 await expect(unsubscribeOneClick('https://list.example.test/u')).resolves.toEqual({ok:false,reason:'rejected'});
 await expect(unsubscribeOneClick('https://list.example.test/u')).resolves.toEqual({ok:false,reason:'refused'});
 await expect(unsubscribeOneClick('https://list.example.test/u')).resolves.toEqual({ok:true});
});
it.each([['LinkedContextUnsupportedError','refused'],['AbortError','cancelled'],['TypeError','network']])('distinguishes %s from a network failure',async(name,reason)=>{
 const error=new Error('failed');error.name=name;vi.mocked(apiFetch).mockRejectedValue(error);
 await expect(unsubscribeOneClick('https://list.example.test/u')).resolves.toEqual({ok:false,reason});
});
