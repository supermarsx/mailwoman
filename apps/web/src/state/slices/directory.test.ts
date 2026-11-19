import {expect,it,vi} from 'vitest';
import {createDirectorySlice} from './directory.ts';
it.each([false,true])('discovers directory configuration (%s) without a sentinel search',async configured=>{
 const fetcher=vi.fn(async(_input:string)=>new Response(JSON.stringify({configured}),{headers:{'content-type':'application/json'}}));
 const directory=createDirectorySlice(fetcher);await directory.ensureEnabled();await directory.ensureEnabled();
 expect(directory.enabled()).toBe(configured);expect(fetcher).toHaveBeenCalledTimes(1);
 expect(fetcher.mock.calls[0]?.[0]).toBe('/api/directory/status');
});
