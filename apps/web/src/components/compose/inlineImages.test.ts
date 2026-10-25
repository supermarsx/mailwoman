import { describe, it, expect } from 'vitest';
import { inlineAttachments, inlineRefusal, newCid, referencedCids } from './inlineImages.ts';

const img = (cid: string): { cid: string; blobId: string; type: string; name: string; size: number } => ({
  cid,
  blobId: `b-${cid}`,
  type: 'image/png',
  name: `${cid}.png`,
  size: 10,
});

describe('inline images (t29-e12)', () => {
  it('makes Content-IDs the engine accepts (atext, ".", "@", at most 255 bytes)', () => {
    const cid = newCid();
    expect(cid).toMatch(/^[A-Za-z0-9!#$%&'*+/=?^_`{|}~.@-]{1,255}$/);
    expect(cid.endsWith('@mailwoman')).toBe(true);
    expect(newCid()).not.toBe(cid);
  });

  it('accepts raster images within the size limit only', () => {
    expect(inlineRefusal({ type: 'image/png', size: 10 }, 100)).toBeNull();
    expect(inlineRefusal({ type: 'image/webp', size: 100 }, 100)).toBeNull();
    expect(inlineRefusal({ type: 'image/svg+xml', size: 10 }, 100)).toBe('type');
    expect(inlineRefusal({ type: 'application/pdf', size: 10 }, 100)).toBe('type');
    expect(inlineRefusal({ type: 'image/jpeg', size: 101 }, 100)).toBe('size');
  });

  it('lists the cid references of a body once each, in order', () => {
    expect(
      referencedCids('<p><img src="cid:b@mailwoman" alt=""><img src="cid:a@mailwoman"><img src="cid:b@mailwoman"></p>'),
    ).toEqual(['b@mailwoman', 'a@mailwoman']);
    expect(referencedCids('<p><img src="https://x/y.png"></p>')).toEqual([]);
  });

  it('sends only the images the body still shows, as inline attachments', () => {
    expect(inlineAttachments([img('a'), img('b'), img('c')], ['c', 'a', 'unknown'])).toEqual([
      { blobId: 'b-c', cid: 'c', disposition: 'inline', type: 'image/png', name: 'c.png' },
      { blobId: 'b-a', cid: 'a', disposition: 'inline', type: 'image/png', name: 'a.png' },
    ]);
    expect(inlineAttachments([img('a')], [])).toEqual([]);
  });
});
