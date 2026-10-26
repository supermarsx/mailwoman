import {afterEach,expect,it,vi} from 'vitest';
import {downloadMessages,forwardMessages,BULK_BYTE_LIMIT} from './mailBulk.ts';
import {mkEmail} from '../components/appHarness.tsx';
afterEach(()=>vi.unstubAllGlobals());
it('forwards originals as uniquely named MIME attachments without recipients or injected HTML',()=>{
  const initial=forwardMessages([mkEmail('a',{blobId:'raw-a',subject:'<script>/',size:42}),mkEmail('b',{blobId:'raw-b',subject:'<script>/',size:56})],'Two messages');
  expect(initial.to).toBe('');expect(initial.bodyHtml).toBe('');expect(initial.attachments.map(a=>a.blobId)).toEqual(['raw-a','raw-b']);
  expect(new Set(initial.attachments.map(a=>a.name)).size).toBe(2);expect(initial.attachments.every(a=>a.contentType==='message/rfc822'&&!a.name.includes('/'))).toBe(true);
  expect(initial.sources).toEqual([{emailId:'a',keyword:'$forwarded'},{emailId:'b',keyword:'$forwarded'}]);
  expect(()=>forwardMessages([mkEmail('bad')],'')).toThrow();expect(()=>forwardMessages([mkEmail('big',{blobId:'x',size:BULK_BYTE_LIMIT+1})],'')).toThrow();
});
it('makes one ordered mbox from individually authorized exports and deduplicates selection',async()=>{
  const fetcher=vi.fn().mockResolvedValueOnce(new Response('From first\r\nOne\r\n')).mockResolvedValueOnce(new Response('From second\r\nTwo\r\n'));vi.stubGlobal('fetch',fetcher);
  const blob=await downloadMessages(['a','b','a'],()=>true,new AbortController().signal);
  const text=await new Promise<string>((resolve,reject)=>{const reader=new FileReader();reader.onload=()=>resolve(String(reader.result));reader.onerror=reject;reader.readAsText(blob);});
  expect(text).toBe('From first\r\nOne\r\nFrom second\r\nTwo\r\n');expect(fetcher).toHaveBeenCalledTimes(2);expect(fetcher.mock.calls[0]?.[0]).toContain('/api/export/a?format=mbox');
});
it('stops before exporting another account or returning a partially authorized archive',async()=>{
  let current=true;const fetcher=vi.fn().mockImplementation(async()=>{current=false;return new Response('private');});vi.stubGlobal('fetch',fetcher);
  await expect(downloadMessages(['a','b'],()=>current,new AbortController().signal)).rejects.toThrow('account');expect(fetcher).toHaveBeenCalledOnce();
  fetcher.mockResolvedValue(new Response('denied',{status:403}));await expect(downloadMessages(['a'],()=>true,new AbortController().signal)).rejects.toThrow('export');
});
it('cancels an oversized streamed archive rather than retaining it',async()=>{
  const cancel=vi.fn();vi.stubGlobal('fetch',vi.fn().mockResolvedValue(new Response(new ReadableStream({start(controller){controller.enqueue(new Uint8Array(BULK_BYTE_LIMIT+1));},cancel}))));
  await expect(downloadMessages(['a'],()=>true,new AbortController().signal)).rejects.toThrow('size');expect(cancel).toHaveBeenCalledOnce();
});
