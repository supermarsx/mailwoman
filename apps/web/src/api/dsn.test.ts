import { afterEach, expect, it, vi } from 'vitest';
import { apiFetch } from './http.ts';
import { fetchMailHeaders } from './mailHeaders.ts';
import { fetchProxyDeliveryReport, isDeliveryReportHeader, MAX_DELIVERY_BYTES } from './dsn.ts';
vi.mock('./http.ts',()=>({apiFetch:vi.fn()}));
vi.mock('./transport.ts',()=>({transportBase:()=>'',createConfiguredClient:vi.fn()}));
vi.mock('./mailHeaders.ts',async original=>({...await original<typeof import('./mailHeaders.ts')>(),fetchMailHeaders:vi.fn()}));
afterEach(()=>vi.resetAllMocks());
const header='Content-Type: multipart/report; boundary="a";\r\n report-type="delivery-status"';
it('only root delivery report headers qualify, never MDNs or duplicate content types',()=>{
  expect(isDeliveryReportHeader(header)).toBe(true);
  expect(isDeliveryReportHeader(header.replace('delivery-status','global-delivery-status'))).toBe(true);
  expect(isDeliveryReportHeader(header.replace('delivery-status','disposition-notification'))).toBe(false);
  expect(isDeliveryReportHeader('Content-Type: text/plain\r\n\r\n'+header)).toBe(false);
  expect(isDeliveryReportHeader(header+'\r\nContent-Type: text/plain')).toBe(false);
});
it('ordinary mail fetches only bounded headers, not a complete message or parser request',async()=>{
  vi.mocked(fetchMailHeaders).mockResolvedValue('Content-Type: text/plain');
  expect(await fetchProxyDeliveryReport('/jmap/download/owned',new AbortController().signal)).toBeNull();
  expect(apiFetch).not.toHaveBeenCalled();
});
it('streams candidate bytes then sends only raw bytes to the local authenticated parser',async()=>{
  vi.mocked(fetchMailHeaders).mockResolvedValue(header);
  vi.mocked(apiFetch).mockResolvedValueOnce(new Response('report bytes')).mockResolvedValueOnce(new Response(JSON.stringify({report:null}),{headers:{'content-type':'application/json'}}));
  const signal=new AbortController().signal;
  expect(await fetchProxyDeliveryReport('/jmap/download/owned',signal)).toBeNull();
  expect(apiFetch).toHaveBeenNthCalledWith(1,'/jmap/download/owned',{signal});
  const [url,options]=vi.mocked(apiFetch).mock.calls[1]!;
  expect(url).toBe('/api/mail/delivery-status');expect(options?.signal).toBe(signal);
  expect(JSON.parse(options?.body as string)).toEqual({bytesBase64:btoa('report bytes')});
});
it('cancels an oversized stream even without a content length and never posts it',async()=>{
  vi.mocked(fetchMailHeaders).mockResolvedValue(header);const cancel=vi.fn();
  vi.mocked(apiFetch).mockResolvedValue(new Response(new ReadableStream({start(controller){controller.enqueue(new Uint8Array(MAX_DELIVERY_BYTES));controller.enqueue(new Uint8Array(1));},cancel})));
  await expect(fetchProxyDeliveryReport('/jmap/download/owned',new AbortController().signal)).rejects.toThrow(/exceeds limit/);
  expect(cancel).toHaveBeenCalledOnce();expect(apiFetch).toHaveBeenCalledOnce();
});
