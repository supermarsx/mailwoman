import { afterEach, expect, it, vi } from 'vitest';
import { apiFetch } from './http.ts';
import { fetchMailHeaders } from './mailHeaders.ts';
vi.mock('./http.ts',()=>({apiFetch:vi.fn()}));
afterEach(()=>vi.resetAllMocks());
it('cancels a server response at the header boundary rather than downloading its body',async()=>{
  let canceled=false;
  const body=new ReadableStream({start(controller){controller.enqueue(new TextEncoder().encode('X-Spam-Status: No, score=-1 required=5\r\n\r\nbody'));},cancel(){canceled=true;}});
  vi.mocked(apiFetch).mockResolvedValue(new Response(body));
  const headers=await fetchMailHeaders('/mail/raw',new AbortController().signal);
  expect(headers).toBe('X-Spam-Status: No, score=-1 required=5');expect(canceled).toBe(true);
  expect(apiFetch).toHaveBeenCalledWith('/mail/raw',expect.objectContaining({headers:{range:'bytes=0-262143'}}));
});
it('refuses headers beyond the bounded limit and cancels the stream',async()=>{
  let canceled=false;
  const body=new ReadableStream({start(controller){controller.enqueue(new Uint8Array(300000).fill(65));},cancel(){canceled=true;}});
  vi.mocked(apiFetch).mockResolvedValue(new Response(body));
  await expect(fetchMailHeaders('/mail/raw',new AbortController().signal)).rejects.toThrow('exceed limit');expect(canceled).toBe(true);
});
