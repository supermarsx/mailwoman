import { beforeAll, describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { initSync, sanitizeEmailHtmlReader } from '../wasm/mw-sanitize/mw_sanitize.js';
import { readerReferences, rewriteReaderContent, signedSource, EMPTY_SIGNED, type RemotePolicy } from './remote-content.ts';
beforeAll(() => initSync({ module: readFileSync(resolve('src/wasm/mw-sanitize/mw_sanitize_bg.wasm')) }));
const policy: RemotePolicy = { mode: 'proxy', trackers: true, cleanLinks: true, allowDirect: false, locked: { mode: false, trackers: false, cleanLinks: false } };
const sanitize = (s: string, trackers = true) => sanitizeEmailHtmlReader(s, trackers, true);
const doc = (s: string) => new DOMParser().parseFromString(s, 'text/html');
describe('signed reader content', () => {
  it('rewrites real sanitizer image, srcset and CSS placeholders only to signed server paths', () => {
    const html = sanitize('<img src="https://cdn.test/a.png" srcset="https://cdn.test/b.png 2x"><p style="background-image:url(https://cdn.test/c.png)">text</p>');
    const refs = readerReferences(html); expect(refs.urls).toEqual(expect.arrayContaining(['https://cdn.test/a.png', 'https://cdn.test/b.png', 'https://cdn.test/c.png']));
    const output = rewriteReaderContent(html, { ...EMPTY_SIGNED, images: refs.urls.map((url, i) => ({ url, src: `/api/rc/i/token${i}.signature` })) }, policy, {});
    const root = doc(output); expect(root.querySelector('img')?.getAttribute('src')).toContain('/api/rc/i/token');
    expect(root.querySelector('img')?.getAttribute('srcset')).toContain('/api/rc/i/token');
    expect(root.querySelector('p')?.getAttribute('style')).toContain('/api/rc/i/token');
    for (const e of root.querySelectorAll('[src],[srcset],[style]')) expect(e.outerHTML).not.toContain('https://cdn.test');
  });
  it('marks the first viewport images high priority and the rest low', () => {
    const urls = Array.from({ length: 8 }, (_, i) => `https://cdn.test/${i}.png`);
    const html = sanitize(urls.map(u => `<img src="${u}">`).join(''));
    const output = rewriteReaderContent(html, { ...EMPTY_SIGNED, images: urls.map((url, i) => ({ url, src: `/api/rc/i/token${i}.sig` })) }, policy, {});
    const priorities = [...doc(output).querySelectorAll('img')].map(i => i.getAttribute('fetchpriority'));
    expect(priorities).toEqual(['high', 'high', 'high', 'high', 'high', 'high', 'low', 'low']);
  });
  it('keeps refused references inert and does not enable an image merely because another URL was signed', () => {
    const output = rewriteReaderContent(sanitize('<img src="https://cdn.test/a.png">'), { ...EMPTY_SIGNED, images: [{ url: 'https://cdn.test/b.png', src: '/api/rc/i/token.sig' }] }, policy, {});
    expect(doc(output).querySelector('img')?.src).toMatch(/^data:/);
  });
  it('a block policy overrides even an existing valid signed image', () => {
    const output = rewriteReaderContent(sanitize('<img src="https://cdn.test/a.png">'), { ...EMPTY_SIGNED, images: [{ url: 'https://cdn.test/a.png', src: '/api/rc/i/token.sig' }] }, { ...policy, mode: 'block' }, {});
    expect(doc(output).querySelector('img')?.src).toMatch(/^data:/);
  });
  it('refuses foreign URLs, wrong token kinds, path traversal and empty tokens', () => {
    for (const src of ['https://attacker.test/api/rc/i/a', '/api/rc/f/a', '/api/rc/i/../x', '/api/rc/i/', '/api/rc/i/a?url=foo', 'javascript:alert(1)']) expect(signedSource(src, 'i')).toBeNull();
    expect(signedSource('https://user:secret@mail.test/api/rc/i/token', 'i', 'https://mail.test')).toBeNull();
    expect(signedSource('/api/rc/i/a.signature', 'i')).toContain('/api/rc/i/a.signature');
  });
  it('maps a CID only through the current message part map and signed blob', () => {
    const html = sanitize('<img src="cid:logo%40test"><img src="cid:unknown">');
    expect(readerReferences(html).cids).toEqual(['logo@test', 'unknown']);
    const output = doc(rewriteReaderContent(html, { ...EMPTY_SIGNED, parts: [{ blobId: 'msg.2', src: '/api/rc/p/token.sig' }] }, policy, { 'logo@test': 'msg.2' }));
    expect(output.querySelectorAll('img')[0]?.src).toContain('/api/rc/p/token.sig');
    expect(output.querySelectorAll('img')[1]?.hasAttribute('src')).toBe(false);
  });
  it('allows only HTTPS in explicitly permitted direct mode', () => {
    const html = sanitize('<img src="https://cdn.test/a.png"><img src="http://cdn.test/b.png">');
    const output = doc(rewriteReaderContent(html, EMPTY_SIGNED, { ...policy, mode: 'direct', allowDirect: true }, {}));
    expect(output.querySelectorAll('img')[0]?.src).toBe('https://cdn.test/a.png'); expect(output.querySelectorAll('img')[1]?.src).toMatch(/^data:/);
    const refused = doc(rewriteReaderContent(html, EMPTY_SIGNED, { ...policy, mode: 'direct', allowDirect: false }, {}));
    expect(refused.querySelector('img')?.src).toMatch(/^data:/);
  });
  it('retains tracker refusal even when a direct mode or signed grant is available', () => {
    const html = sanitize('<img src="https://x.list-manage.com/track/open.php?u=1">', false);
    expect(readerReferences(html).trackers).toBeGreaterThan(0);
    expect(doc(rewriteReaderContent(html, EMPTY_SIGNED, { ...policy, mode: 'direct', allowDirect: true }, {})).querySelector('img')?.src).toMatch(/^data:/);
  });
  it('shows the original tracking URL on hover and discovers downloadable file links', () => {
    const html = sanitize('<a href="https://docs.test/report.pdf?utm_source=mail">report</a>');
    expect(readerReferences(html).files).toEqual(['https://docs.test/report.pdf']);
    const link = doc(rewriteReaderContent(html, EMPTY_SIGNED, policy, {})).querySelector('a');
    expect(link?.href).toBe('https://docs.test/report.pdf'); expect(link?.title).toContain('utm_source=mail');
  });
});

it('rewrites CID candidates in srcset and CSS only through signed message membership', () => {
  const html = sanitize('<img srcset="cid:logo%40test 1x, cid:unknown 2x"><p style="background-image:url(cid:logo%40test)">text</p>');
  expect(readerReferences(html).cids).toContain('logo@test');
  const out = doc(rewriteReaderContent(html, { ...EMPTY_SIGNED, parts: [{ blobId:'msg.2', src:'/api/rc/p/token.sig' }] }, policy, { 'logo@test':'msg.2' }));
  expect(out.querySelector('img')?.getAttribute('srcset')).toContain('/api/rc/p/token.sig');
  expect(out.querySelector('img')?.getAttribute('srcset')).not.toContain('cid:');
  expect(out.querySelector('p')?.getAttribute('style')).toContain('/api/rc/p/token.sig');
});
it('offers only HTTP message links for explicit arbitrary file downloads', () => {
  expect(readerReferences(sanitize('<a href="https://docs.test/download?id=1">file</a><a href="mailto:a@test">mail</a>')).links).toEqual(['https://docs.test/download?id=1']);
});

it('finds stylesheet-only CID backgrounds once and maps CSS remote/CID images through fresh signed sources',()=>{
  const html=sanitize('<style>.remote{background-image:url(https://cdn.test/style-only.png)}.inline{background-image:url( "cid:style%40test" )}.again{background:url(cid:style%40test)}</style><p class="remote inline">Text</p>');
  const refs=readerReferences(html);
  expect(refs.urls).toEqual(['https://cdn.test/style-only.png']);
  expect(refs.cids).toEqual(['style@test']);
  const output=rewriteReaderContent(html,{...EMPTY_SIGNED,images:[{url:refs.urls[0]!,src:'/api/rc/i/style.signature'}],parts:[{blobId:'original-style',src:'/api/rc/p/style-inline.signature'}]},policy,{'style@test':'original-style'});
  const css=doc(output).querySelector('style')!.textContent!;
  expect(css).toContain('/api/rc/i/style.signature');expect(css.match(/\/api\/rc\/p\/style-inline.signature/g)).toHaveLength(2);
  expect(css).not.toMatch(/mw-rc:|cid:|https:\/\/cdn\.test/);
});

it('keeps unsigned, refused, tracker and foreign stylesheet sources as none without a direct fallback',()=>{
  const html=sanitize('<style>.remote{background:url(https://cdn.test/missing.png)}.inline{background:url(cid:foreign)}</style><p>Retained text</p>');
  for(const signed of [EMPTY_SIGNED,{...EMPTY_SIGNED,images:[{url:'https://cdn.test/missing.png',src:'https://evil.test/api/rc/i/fake.signature'}],parts:[{blobId:'other',src:'/api/rc/p/other.signature'}]}]){
    const output=rewriteReaderContent(html,signed,policy,{}),css=doc(output).querySelector('style')!.textContent!;
    expect(css).toContain('none');expect(css).not.toMatch(/url\(|mw-rc:|cid:|https:\/\//);expect(output).toContain('Retained text');
  }
  const tracker=sanitize('<style>.pixel{background:url(https://x.list-manage.com/track/open.php?u=1)}</style>',false);
  const refs=readerReferences(tracker);
  const css=doc(rewriteReaderContent(tracker,{...EMPTY_SIGNED,images:refs.urls.map(url=>({url,src:'/api/rc/i/tracker.signature'}))},policy,{})).querySelector('style')!.textContent!;
  expect(css).toContain('none');expect(css).not.toContain('/api/rc/i/');
});

it('does not discover CSS comment/string CIDs and bounds stylesheet-only CID requests',()=>{
  const onlyText='<style>/*url(cid:comment)*/.label::before{content:"url(cid:string)"}</style>';
  expect(readerReferences(onlyText).cids).toEqual([]);
  const styles=Array.from({length:300},(_,index)=>`.c${index}{background:url(cid:image${index})}`).join('');
  const refs=readerReferences(`<style>${styles}</style>`);
  expect(refs.cids).toHaveLength(256);expect(refs.cids.at(-1)).toBe('image255');
});
