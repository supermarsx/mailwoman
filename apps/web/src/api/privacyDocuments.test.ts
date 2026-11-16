import {afterEach,expect,it,vi} from 'vitest';
import {loadPolicyDocument,loadProcessingDocument,PrivacyError} from './privacy.ts';
import {apiFetch} from './http.ts';
vi.mock('./http.ts',()=>({apiFetch:vi.fn()}));
afterEach(()=>vi.mocked(apiFetch).mockReset());
it('loads the owned policy through the authenticated relative transport and keeps versioned structured sections',async()=>{
 const doc={version:2,date:'2026-11-16',lang:'pt-PT',sections:[{id:'rights',title:'Direitos',body:'Texto',missing:false}]};vi.mocked(apiFetch).mockResolvedValue(Response.json(doc));const signal=new AbortController().signal;
 expect(await loadPolicyDocument('/privacy','pt-PT',signal)).toEqual(doc);expect(apiFetch).toHaveBeenCalledWith('/privacy?format=json&lang=pt-PT',expect.objectContaining({signal}));
});
it('never fetches an external operator policy with mailbox credentials',async()=>{expect(await loadPolicyDocument('https://external.example/privacy','en',new AbortController().signal)).toBeNull();expect(apiFetch).not.toHaveBeenCalled();});
it('loads private processing records without caching and rejects authentication or malformed document responses',async()=>{
 vi.mocked(apiFetch).mockResolvedValueOnce(Response.json({activities:[]})).mockResolvedValueOnce(new Response('{}',{status:401})).mockResolvedValueOnce(Response.json({}));const signal=new AbortController().signal;
 await loadProcessingDocument('en',signal);expect(apiFetch).toHaveBeenCalledWith('/api/privacy/processing?lang=en',expect.objectContaining({cache:'no-store',signal}));await expect(loadProcessingDocument('en',signal)).rejects.toBeInstanceOf(PrivacyError);await expect(loadPolicyDocument('/privacy','en',signal)).rejects.toBeInstanceOf(PrivacyError);
});
