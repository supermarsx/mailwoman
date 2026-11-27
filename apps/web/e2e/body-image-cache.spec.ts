import {test,expect} from '@playwright/test';
import {createServer,type Server} from 'node:http';

test.describe('body images in recreated sandboxed frames',()=>{
 let server:Server,origin:string;
 const requests=new Map<string,number>(),bodies=new Map<string,number>();
 test.beforeAll(async()=>{
  const png=Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==','base64');
  server=createServer((req,res)=>{
   const path=req.url??'/';
   if(path==='/'){res.setHeader('Content-Type','text/html');res.end('<!doctype html><title>Message frame cache</title><main></main>');return;}
   requests.set(path,(requests.get(path)??0)+1);
   res.setHeader('Cache-Control',path.includes('disabled')?'private, no-store':path.includes('expired')?'private, max-age=0, must-revalidate':path.includes('three-years')?'private, max-age=94608000, must-revalidate':'private, max-age=600, must-revalidate');
   res.setHeader('Vary','Cookie, Authorization, X-Mailwoman-Account');
   res.setHeader('ETag','"body-image"');res.setHeader('Cross-Origin-Resource-Policy','cross-origin');
   if(req.headers['if-none-match']==='"body-image"'){res.writeHead(304);res.end();return;}
   bodies.set(path,(bodies.get(path)??0)+1);res.setHeader('Content-Type','image/png');res.end(png);
  });
  await new Promise<void>(resolve=>server.listen(0,'127.0.0.1',resolve));const address=server.address();if(!address||typeof address==='string')throw new Error('Missing image fixture address');origin=`http://127.0.0.1:${address.port}`;
 });
 test.afterAll(async()=>{server.closeAllConnections();await new Promise<void>(resolve=>server.close(()=>resolve()));});
 test('reopening uses fresh copies, disabling downloads again, and expiry revalidates without image bytes',async({page,browserName})=>{
  await page.goto(origin);
  const reopen=async(path:string)=>{
   await page.evaluate(src=>{document.querySelector('iframe')?.remove();const frame=document.createElement('iframe');frame.setAttribute('sandbox','');frame.setAttribute('credentialless','');frame.referrerPolicy='no-referrer';frame.srcdoc=`<img src="${src}" alt="Message image">`;document.querySelector('main')!.append(frame);},origin+path);
   await expect.poll(()=>page.frameLocator('iframe').locator('img').evaluate(node=>{const image=node as HTMLImageElement;return image.complete&&image.naturalWidth>0;})).toBe(true);
  };
  for(const kind of ['enabled','three-years','disabled','expired']){
   const path=`/${browserName}-${kind}.png`;await reopen(path);await reopen(path);
   expect(requests.get(path)).toBe(kind==='disabled'||kind==='expired'?2:1);expect(bodies.get(path)).toBe(kind==='disabled'?2:1);
  }
 });
});
