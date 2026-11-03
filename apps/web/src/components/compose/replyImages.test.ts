import {beforeAll,afterEach,expect,it,vi} from 'vitest';
import {readFileSync} from 'node:fs';
import {initSync} from '../../wasm/mw-sanitize/mw_sanitize.js';
import {embedReplyImages,type ReplyImageOptions} from './replyImages.ts';
import {quoteForReply} from './quote.ts';
import {referencedCids} from './inlineImages.ts';
import type {signContent} from '../../api/remote-content.ts';
import type {apiFetch} from '../../api/http.ts';
beforeAll(()=>initSync({module:readFileSync('src/wasm/mw-sanitize/mw_sanitize_bg.wasm')}));
afterEach(()=>vi.useRealTimers());
const remote='https://images.example.org/logo.png';
const image=(id='0',url=remote)=>`<span hidden data-mw-rc="${id}" data-mw-rc-url="${url}"></span>`;
function png(width=40,height=30):Response{const bytes=new Uint8Array(24);bytes.set([137,80,78,71,13,10,26,10]);const view=new DataView(bytes.buffer);view.setUint32(16,width);view.setUint32(20,height);return new Response(bytes,{headers:{'content-type':'image/png'}});}
function options(over:Partial<ReplyImageOptions>={}){
 const sign=vi.fn<typeof signContent>().mockResolvedValue({images:[{url:remote,src:'/api/rc/i/valid.signature'}],parts:[],files:[],refused:[]});
 const fetcher=vi.fn<typeof apiFetch>().mockImplementation(async()=>png());let count=0;
 const upload=vi.fn<ReplyImageOptions['upload']>().mockImplementation(async file=>({cid:`reply-${++count}@mailwoman`,blobId:`blob${count}`,type:file.type,name:file.name,size:file.size}));
 return {emailId:'message',accountId:'account',cidParts:{},allowRemote:true,allowInline:true,blockTrackers:true,current:()=>true,sign,fetcher,upload,...over};
}
it('deduplicates img/table/CSS sources and quotes their uploaded CIDs with authored layout intact',async()=>{
 const o=options(),result=await embedReplyImages(`<style>.logo{background-image:url( "mw-rc:0" )}</style><table background="mw-rc:0" cellpadding="7"><tr><td><img src="mw-rc:0" width="90" alt="Logo">Reply text</td></tr></table>${image()}`,o);
 expect(o.sign).toHaveBeenCalledExactlyOnceWith('message',{urls:[remote],parts:[]},'account');expect(o.fetcher).toHaveBeenCalledExactlyOnceWith(expect.stringContaining('/api/rc/i/valid.signature'),expect.objectContaining({redirect:'error',headers:{'X-Mailwoman-Account':'account'}}));expect(o.upload).toHaveBeenCalledOnce();
 expect(result.inline).toHaveLength(1);expect(result.omitted).toBe(0);expect(result.html.match(/cid:reply-1@mailwoman/g)).toHaveLength(3);expect(result.html).toContain('width="90"');expect(result.html).toContain('alt="Logo"');expect(result.html).not.toMatch(/api\/rc|mw-rc:|images\.example/);
 const quote=await quoteForReply({html:result.html,ownedCids:new Set(result.inline.map(value=>value.cid))},'Alice wrote:');expect(quote.html).toContain('cellpadding="7"');expect(quote.html).toContain('background-image');expect(referencedCids(quote.html)).toEqual(['reply-1@mailwoman']);expect(quote.text).toContain('Reply text');
});
it('keeps owned image-only quotes and still drops hostile or unowned image-only originals',async()=>{
 const result=await embedReplyImages(`<img src="mw-rc:0">${image()}`,options());
 expect((await quoteForReply({html:result.html,ownedCids:new Set(result.inline.map(value=>value.cid))},'Alice wrote:')).html).toContain('src="cid:reply-1@mailwoman"');
 expect(await quoteForReply({html:'<img src="https://evil.example/image"><img src="cid:unowned@mailwoman">'},'Alice wrote:')).toEqual({html:'',text:''});
});
it('does not fetch blocked remote resources, known trackers, foreign CIDs or existing private URLs',async()=>{
 const o=options({allowRemote:false,allowInline:false}),result=await embedReplyImages(`<p>Keep this text</p><img src="mw-rc:0"><img src="cid:foreign"><table background="https://evil.example/a"><tr><td style="background:url(/api/rc/i/old.secret)">Cell</td></tr></table>${image()}`,o);
 expect(o.sign).not.toHaveBeenCalled();expect(o.fetcher).not.toHaveBeenCalled();expect(result.html).toContain('Keep this text');expect(result.html).not.toMatch(/mw-rc|api\/rc|evil\.example|cid:foreign/);expect(result.inline).toEqual([]);
 const track=options();await embedReplyImages('<img src="mw-rc:0"><span hidden data-mw-rc="0" data-mw-rc-url="https://track.example/a" data-mw-tracker></span>',track);expect(track.sign).not.toHaveBeenCalled();
});
it('accepts authorized inline PNG parts while refusing foreign signed routes and partial failures',async()=>{
 const o=options({cidParts:{logo:'part'},sign:vi.fn<typeof signContent>().mockResolvedValue({images:[{url:remote,src:'https://evil.example/image'}],parts:[{blobId:'part',src:'/api/rc/p/part.signature'}],files:[],refused:[]})});
 const result=await embedReplyImages(`<p>Keep text</p><img src="mw-rc:0"><img src="cid:logo">${image()}`,o);expect(o.fetcher).toHaveBeenCalledOnce();expect(vi.mocked(o.fetcher).mock.calls[0]?.[0]).toContain('/api/rc/p/');expect(result.inline).toHaveLength(1);expect(result.omitted).toBe(1);expect(result.html).toContain('Keep text');expect(result.html).not.toContain('/api/rc/');
});
it('drops server tracker substitutes, tiny decoded trackers and non-PNG responses without losing text',async()=>{
 for(const response of [new Response('blocked',{headers:{'content-type':'image/svg+xml'}}),png(1,1),new Response(new Uint8Array(24),{headers:{'content-type':'image/png','x-mw-remote-content':'tracker'}})]){
  const o=options({fetcher:vi.fn<typeof apiFetch>().mockResolvedValue(response)}),result=await embedReplyImages(`<p>Text remains</p><img src="mw-rc:0">${image()}`,o);expect(o.upload).not.toHaveBeenCalled();expect(result.inline).toEqual([]);expect(result.omitted).toBe(1);expect(result.html).toContain('Text remains');expect(result.html).not.toContain('/api/rc/');
 }
});
it('cancels rejected response bodies instead of leaving uncounted downloads running',async()=>{
 const cancel=vi.fn(),response=new Response(new ReadableStream<Uint8Array>({cancel}),{headers:{'content-type':'image/png','x-mw-remote-content':'tracker'}});
 const o=options({fetcher:vi.fn<typeof apiFetch>().mockResolvedValue(response)});
 const result=await embedReplyImages(`<p>Keep text</p><img src="mw-rc:0">${image()}`,o);
 expect(cancel).toHaveBeenCalledOnce();expect(result.omitted).toBe(1);expect(o.upload).not.toHaveBeenCalled();expect(result.html).toContain('Keep text');
});
it('never uploads or publishes into a changed account after a late image response',async()=>{
 let current=true,release!:(response:Response)=>void;const o=options({current:()=>current,fetcher:vi.fn<typeof apiFetch>().mockImplementation(()=>new Promise(resolve=>{release=resolve;}))});const work=embedReplyImages(`<img src="mw-rc:0">${image()}`,o);await vi.waitFor(()=>expect(o.fetcher).toHaveBeenCalledOnce());current=false;release(png());await expect(work).rejects.toMatchObject({name:'AbortError'});expect(o.upload).not.toHaveBeenCalled();
});
it('bounds the complete preparation deadline even if an image never responds',async()=>{
 vi.useFakeTimers();const o=options({fetcher:vi.fn<typeof apiFetch>().mockImplementation(()=>new Promise(()=>{}))});const work=embedReplyImages(`<p>Keep reply</p><img src="mw-rc:0">${image()}`,o);await vi.advanceTimersByTimeAsync(12_001);const result=await work;expect(result.html).toContain('Keep reply');expect(result.inline).toEqual([]);expect(result.omitted).toBe(1);expect(o.upload).not.toHaveBeenCalled();
});
it('uses only two image workers for a large signed batch',async()=>{
 const urls=Array.from({length:8},(_,index)=>`https://images.example.org/${index}.png`);let active=0,peak=0;
 const o=options({sign:vi.fn<typeof signContent>().mockResolvedValue({images:urls.map((url,index)=>({url,src:`/api/rc/i/image${index}.signature`})),parts:[],files:[],refused:[]}),fetcher:vi.fn<typeof apiFetch>().mockImplementation(async()=>{peak=Math.max(peak,++active);await new Promise(resolve=>setTimeout(resolve,5));active--;return png();})});
 const result=await embedReplyImages(urls.map((url,index)=>`<img src="mw-rc:${index}">${image(String(index),url)}`).join(''),o);expect(result.inline).toHaveLength(8);expect(peak).toBe(2);
});
