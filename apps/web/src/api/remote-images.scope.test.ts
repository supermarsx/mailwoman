// The `emailId` scope parameter on the image-proxy URL (t22-e4, for t22-e7's P3).
//
// A separate file from `remote-images.test.ts` so that file stays the control
// that the un-scoped URL is byte-unchanged — those assertions must keep passing
// verbatim, since a call that omits `emailId` is what the tree does today.

import { describe, it, expect, afterEach } from 'vitest';
import { imageProxyUrl, rewriteGrantedImages } from './remote-images.ts';

const g = globalThis as { __MW_BASE__?: string };
afterEach(() => {
  delete g.__MW_BASE__;
});

describe('imageProxyUrl — message scope', () => {
  it('carries the message id so the request is scope-checkable', () => {
    expect(imageProxyUrl('https://cdn.example/logo.png', 'M1')).toBe(
      '/api/image-proxy?url=https%3A%2F%2Fcdn.example%2Flogo.png&emailId=M1',
    );
  });

  it('encodes an id that would otherwise forge a query parameter', () => {
    expect(imageProxyUrl('https://cdn.example/a.png', 'M&url=https://evil.example')).toBe(
      '/api/image-proxy?url=https%3A%2F%2Fcdn.example%2Fa.png' +
        '&emailId=M%26url%3Dhttps%3A%2F%2Fevil.example',
    );
  });

  it('is byte-unchanged when no id is given, so landing this alone is a no-op', () => {
    const unscoped = '/api/image-proxy?url=https%3A%2F%2Fcdn.example%2Flogo.png';
    expect(imageProxyUrl('https://cdn.example/logo.png')).toBe(unscoped);
    expect(imageProxyUrl('https://cdn.example/logo.png', '')).toBe(unscoped);
  });

  it('keeps the sub-path prefix ahead of both parameters', () => {
    g.__MW_BASE__ = '/mail';
    expect(imageProxyUrl('https://cdn.example/logo.png', 'M1')).toBe(
      '/mail/api/image-proxy?url=https%3A%2F%2Fcdn.example%2Flogo.png&emailId=M1',
    );
  });
});

describe('rewriteGrantedImages — forwards the scope', () => {
  const raw = '<p>hi</p><img src="https://cdn.example/logo.png"><img src="cid:inline">';
  const sanitized = '<p>hi</p><img><img src="cid:inline">';

  it('puts the message id on every rewritten image', () => {
    const out = rewriteGrantedImages(sanitized, raw, true, 'M42')!;
    expect(out).toContain('emailId=M42');
    expect(out).toContain('url=https%3A%2F%2Fcdn.example%2Flogo.png');
    // The cid: image is still untouched — scoping changed nothing about which
    // images are rewritten, only what the proxy URL carries.
    expect(out).toContain('src="cid:inline"');
  });

  it('rewrites without the id when none is passed — today\'s behaviour, unchanged', () => {
    const out = rewriteGrantedImages(sanitized, raw, true)!;
    expect(out).toContain('url=https%3A%2F%2Fcdn.example%2Flogo.png');
    expect(out).not.toContain('emailId');
  });

  it('an id does not weaken deny-by-default: ungranted stays byte-identical', () => {
    expect(rewriteGrantedImages(sanitized, raw, false, 'M42')).toBe(sanitized);
  });

  it('an id does not weaken the fail-closed alignment check', () => {
    const mismatched = '<p>hi</p><img>'; // one img vs raw's two
    expect(rewriteGrantedImages(mismatched, raw, true, 'M42')).toBe(mismatched);
  });
});
