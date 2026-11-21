import {afterEach,expect,it,vi} from 'vitest';
import {createClient} from './client.ts';
import {ApiError,ActionCompletionError} from './errors.ts';
import {apiFetch} from './http.ts';
vi.mock('./http.ts',()=>({apiFetch:vi.fn()}));
afterEach(()=>vi.clearAllMocks());
it('preserves terminal action outcomes instead of converting them into replayable offline network errors',async()=>{
 const client=createClient(location.origin);
 for(const error of [new ApiError(409,'Unknown',{code:'outcomeUnknown'}),new ActionCompletionError(new Response('{"saved":true}'))]){
  vi.mocked(apiFetch).mockRejectedValueOnce(error);
  await expect(client.me()).rejects.toBe(error);
 }
 expect(apiFetch).toHaveBeenCalledTimes(2);
});
