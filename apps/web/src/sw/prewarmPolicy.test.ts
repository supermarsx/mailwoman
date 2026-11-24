// @vitest-environment node
import {expect,it} from 'vitest';
import {bootWorker,digestOf,TEST_SHELL} from './swHarness.ts';
const assets=['/assets/a-12345678.js','/assets/b-12345678.js','/assets/c-12345678.js','/assets/d-12345678.js','/assets/e-12345678.js','/assets/reader-12345678.ftl','/pdfjs/web/viewer.mjs'];
// t52w: the default install is the app list (offlineTiers.ts); all is an operator choice.
const app=[assets[0]!,assets[1]!];
const digest=Object.fromEntries([['/index.html',digestOf(TEST_SHELL)],...assets.map(path=>[path,digestOf('export{}')])]);
function configured(policy:unknown,saveData=false){
 const sw=bootWorker('https://mail.example.com/sw.js',{digests:digest,offlineAssets:assets,appAssets:app,saveData});let active=0,peak=0;
 sw.network.fetch=async request=>{
  const path=new URL(typeof request==='string'?request:request.url,'https://mail.example.com').pathname;
  if(path==='/api/public-config')return Response.json({webPerformance:policy,assetUrls:['https://bad.test/evil.js']});
  if(path==='/')return new Response(TEST_SHELL,{headers:{'content-type':'text/html'}});
  active++;peak=Math.max(peak,active);await Promise.resolve();active--;
  return new Response('export{}',{headers:{'content-type':'text/javascript'}});
 };
 return {sw,peak:()=>peak};
}
it('preserves all resources and applies the actual install concurrency limit',async()=>{
 const {sw,peak}=configured({precacheMode:'all',precacheConcurrency:2,respectSaveData:true});await sw.install();
 expect(peak()).toBe(2);for(const path of assets)expect(sw.caches.urls()).toContain('https://mail.example.com'+path);
 expect(sw.caches.urls()).not.toContain('https://mail.example.com/api/public-config');expect(sw.network.calls.some(url=>url.includes('bad.test'))).toBe(false);
});
it('app (the default) and the old core value install only the app list; other files still enter the integrity cache on use',async()=>{
 for(const precacheMode of ['app','core']){
  const {sw}=configured({precacheMode,precacheConcurrency:1});await sw.install();
  const fetched=sw.network.calls.filter(url=>/\/(?:assets|pdfjs)\//.test(url));
  expect(fetched,precacheMode).toEqual(app.map(path=>'https://mail.example.com'+path));
  expect((await sw.fetch(assets[5]!)).handled).toBe(true);expect(sw.caches.urls()).toContain('https://mail.example.com'+assets[5]);
 }
});
it('honors SaveData only when the administrator enables that policy (all -> app)',async()=>{
 for(const respectSaveData of [true,false]){
  const {sw}=configured({precacheMode:'all',respectSaveData},true);await sw.install();
  expect(sw.network.calls.includes('https://mail.example.com'+assets[6])).toBe(!respectSaveData);
  expect(sw.network.calls.includes('https://mail.example.com'+assets[0])).toBe(true);
 }
});
it('falls back to app/four on unavailable or malformed public hints without weakening digests',async()=>{
 const bounded=configured({precacheMode:'all',precacheConcurrency:100});await bounded.sw.install();expect(bounded.peak()).toBe(4);
 const {sw}=configured({precacheMode:'https://bad.test/'});await sw.install();expect(sw.network.calls.filter(url=>url.includes('/assets/')||url.includes('/pdfjs/'))).toEqual(app.map(path=>'https://mail.example.com'+path));
 const offline=configured(null).sw;offline.network.fetch=async request=>{
  const path=typeof request==='string'?request:request.url;
  if(path.endsWith('/api/public-config'))throw Error('configuration unavailable');
  return new Response(path==='/'?TEST_SHELL:'tampered',{headers:{'content-type':path==='/'?'text/html':'text/javascript'}});
 };await offline.install();expect(offline.network.calls).toContain('https://mail.example.com'+assets[0]);expect(offline.network.calls).not.toContain('https://mail.example.com'+assets[6]);
 expect(offline.caches.urls().some(path=>path.includes('/assets/'))).toBe(false);
});
