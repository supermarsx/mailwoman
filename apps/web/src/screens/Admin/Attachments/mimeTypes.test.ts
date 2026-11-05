import { describe, expect, it } from 'vitest';
import { DEFAULT_MIME, MIME_ENTRIES, detectMime, filterMimeGroups, isMimeShape, mimeForFilename } from './mimeTypes.ts';

describe('mimeTypes', () => {
  it('has unique, well-formed entries', () => {
    const values = MIME_ENTRIES.map((x) => x.value);
    expect(new Set(values).size).toBe(values.length);
    expect(values.every(isMimeShape)).toBe(true);
    expect(values).toContain('message/rfc822');
  });
  it('validates type/subtype shape', () => {
    expect(isMimeShape('application/x-custom+json')).toBe(true);
    expect(isMimeShape(' text/plain ')).toBe(true);
    for (const bad of ['', 'text', 'text/', '/plain', 'a/b/c', 'te xt/plain', 'text/plain; charset=utf-8']) expect(isMimeShape(bad)).toBe(false);
  });
  it('filters by type, label and extension, dropping empty groups', () => {
    expect(filterMimeGroups('').length).toBeGreaterThan(5);
    expect(filterMimeGroups('pdf').map((g) => g.id)).toEqual(['documents']);
    expect(filterMimeGroups('.EXE')[0]!.entries[0]!.value).toBe('application/x-msdownload');
    expect(filterMimeGroups('zzzz')).toEqual([]);
  });
  it('guesses from a file name and detects for a chosen file', () => {
    expect(mimeForFilename('a.JPG')).toBe('image/jpeg');
    expect(mimeForFilename('noext')).toBeNull();
    expect(detectMime({ name: 'x.pdf', type: '' })).toBe('application/pdf');
    expect(detectMime({ name: 'x.pdf', type: 'application/x-weird' })).toBe('application/x-weird');
    expect(detectMime({ name: 'x.unknown', type: '' })).toBe(DEFAULT_MIME);
  });
});
