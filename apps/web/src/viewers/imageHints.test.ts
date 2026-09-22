// t22 L6 — `loading`/`decoding`/intrinsic dimensions on message-body images,
// asserted on REAL sanitized output.
//
// The input to every assertion here is whatever `mw-sanitize` actually emits,
// produced by the committed wasm build (the same one the crypto worker loads,
// instantiated the way `crypto/sanitize.test.ts` does it). A hand-written
// "sanitized" fixture is precisely the shape that lets a producer/consumer
// mismatch survive a green test, so none is used: the fixtures below are the
// DIRTY input, and the sanitizer decides what the transform receives.
//
// Two contracts are pinned:
//   * what the sanitizer keeps — a sender's `width`/`height` survives (ammonia's
//     default `img` allow-list) while `loading`/`decoding` do not. That is why
//     the hints are applied client-side rather than by widening the allow-list;
//   * what the transform adds — `loading="lazy"` + `decoding="async"` on every
//     `<img>` that reaches the body frame, without disturbing anything else.

import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { beforeAll, describe, expect, it } from 'vitest';
import { initSync, sanitizeEmailHtml } from '../wasm/mw-sanitize/mw_sanitize.js';
import { bodyFrameDoc, withImageLoadingHints } from './sandbox.ts';

beforeAll(() => {
  const wasmPath = resolve(process.cwd(), 'src/wasm/mw-sanitize/mw_sanitize_bg.wasm');
  initSync({ module: readFileSync(wasmPath) });
});

/** Parse a fragment and read back one attribute per `<img>`, in document order. */
function attrs(html: string, name: string): (string | null)[] {
  const doc = new DOMParser().parseFromString(html, 'text/html');
  return Array.from(doc.querySelectorAll('img')).map((img) => img.getAttribute(name));
}

describe('what mw-sanitize actually emits for <img> (the producer)', () => {
  it('keeps the sender intrinsic width/height and drops loading/decoding', () => {
    const clean = sanitizeEmailHtml(
      '<p>hi</p><img src="cid:logo" width="600" height="200" alt="logo" loading="eager" decoding="sync">',
    );
    expect(clean).toContain('cid:logo');
    expect(attrs(clean, 'width')).toEqual(['600']);
    expect(attrs(clean, 'height')).toEqual(['200']);
    // Not in the allow-list: the sanitizer strips them, so the sender cannot
    // force `loading="eager"` and the hints must come from our own code.
    expect(attrs(clean, 'loading')).toEqual([null]);
    expect(attrs(clean, 'decoding')).toEqual([null]);
  });
});

describe('withImageLoadingHints over real sanitized output (the consumer)', () => {
  it('adds lazy/async to every surviving <img> and keeps the intrinsic size', () => {
    const clean = sanitizeEmailHtml(
      '<img src="cid:a" width="600" height="200"><p>x</p><img src="cid:b">' +
        '<img src="https://tracker.example/p.gif">',
    );
    const out = withImageLoadingHints(clean);

    expect(attrs(out, 'loading').every((v) => v === 'lazy')).toBe(true);
    expect(attrs(out, 'decoding').every((v) => v === 'async')).toBe(true);
    expect(attrs(out, 'width')[0]).toBe('600');
    expect(attrs(out, 'height')[0]).toBe('200');
    // The sanitizer's own decisions are untouched: the remote src is still gone.
    // The host remains only inside the deliberate hidden `data-mw-blocked-host`
    // breadcrumb (t16 S9), which never carries a loadable URL — so assert the
    // property that matters (nothing can LOAD from it) rather than the mere
    // absence of the string. The flat form passed only while the committed
    // mw-sanitize guest predated 26.16 and emitted no marker (t24-e13).
    expect(out).not.toMatch(/src\s*=\s*["'][^"']*tracker\.example/i);
    expect(out.replace(/\sdata-mw-blocked-host="[^"]*"/g, '')).not.toContain('tracker.example');
    expect(out).toContain('cid:a');
    expect(out).toContain('cid:b');
    // Negative control: the assertion above is only evidence if imgs survived.
    expect(attrs(out, 'loading').length).toBeGreaterThan(1);
  });

  it('does not touch a body with no images (byte-identical)', () => {
    const clean = sanitizeEmailHtml('<p>hello <b>there</b></p><a href="https://e.example">x</a>');
    expect(withImageLoadingHints(clean)).toBe(clean);
  });

  it('an <img> smuggled into an alt value cannot break out of its attribute', () => {
    // A text-level insertion after `<img` would land inside this quoted value —
    // HTML serialization does not escape `<` in attribute values — and break the
    // quoting. The DOMParser path cannot: the count of images stays 1.
    const clean = sanitizeEmailHtml('<img src="cid:a" alt="a &lt;img src=x onerror=y&gt; b">');
    const out = withImageLoadingHints(clean);
    // One image before, one image after: the alt text is never parsed as markup,
    // and the added attributes did not land inside the quoted value. Note the two
    // serializers differ — ammonia escapes `<` in an attribute value, the DOM
    // serializer does not — and the round-trip is stable either way, which is the
    // property that matters and the one a regex would not have.
    expect(attrs(clean, 'src').length).toBe(1);
    expect(attrs(out, 'src').length).toBe(1);
    expect(attrs(out, 'loading')).toEqual(['lazy']);
    expect(attrs(out, 'alt')[0]).toBe('a <img src=x onerror=y> b');
    expect(attrs(clean, 'alt')[0]).toBe(attrs(out, 'alt')[0]);
  });

  it('respects hints already present rather than duplicating them', () => {
    const out = withImageLoadingHints('<img src="cid:a" loading="eager">');
    expect(attrs(out, 'loading')).toEqual(['eager']);
    expect(attrs(out, 'decoding')).toEqual(['async']);
  });
});

describe('the body frame carries the hints and the CSS that makes them useful', () => {
  it('bodyFrameDoc applies the hints to sanitized HTML in every HTML mode', () => {
    const clean = sanitizeEmailHtml('<img src="cid:a" width="600" height="200">');
    for (const mode of ['full-sanitized', 'sanitized-no-media'] as const) {
      const frame = bodyFrameDoc(mode, { html: clean });
      expect(frame).toContain('loading="lazy"');
      expect(frame).toContain('decoding="async"');
      expect(frame).toContain('width="600"');
      // `height:auto` is what lets the intrinsic width/height act as an
      // aspect-ratio reservation instead of pinning the box.
      expect(frame).toContain('img,video{max-width:100%;height:auto}');
    }
  });

  it('plain-text mode still renders escaped text, with no img markup at all', () => {
    const frame = bodyFrameDoc('plain-text', { text: '<img src="cid:a">' });
    expect(frame).toContain('&lt;img');
    expect(frame).not.toContain('loading="lazy"');
  });
});
