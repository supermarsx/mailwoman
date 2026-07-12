import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, waitFor } from '@solidjs/testing-library';
import { AttachmentViewer } from './AttachmentViewer.tsx';
import { ImageViewer } from './ImageViewer.tsx';
import { TextViewer } from './TextViewer.tsx';
import type { EmailBodyPart } from '../api/jmap-types.ts';

const part: EmailBodyPart = { partId: '2', blobId: 'b1', size: 3, type: 'image/png' };

/** Stub fetch(blobUrl) → a Blob so the sandboxed viewers can build their srcdoc. */
function stubFetch(bytes: Uint8Array, type: string): void {
  vi.stubGlobal(
    'fetch',
    vi.fn(async () => ({
      ok: true,
      blob: async () => new Blob([bytes.buffer as ArrayBuffer], { type }),
      text: async () => new TextDecoder().decode(bytes),
    })),
  );
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('AttachmentViewer — MIME dispatch container', () => {
  beforeEach(() => stubFetch(new Uint8Array([1, 2, 3]), 'application/octet-stream'));

  it.each([
    ['image/png', 'image'],
    ['text/plain', 'text'],
    ['audio/mpeg', 'audio'],
    ['video/mp4', 'video'],
    ['application/zip', 'unsupported'],
  ])('routes %s to data-viewer-kind=%s', (mime, kind) => {
    const { container } = render(() => (
      <AttachmentViewer part={{ ...part, type: mime }} blobUrl="blob:x" mime={mime} name="f" />
    ));
    expect(container.querySelector('[data-viewer-kind]')?.getAttribute('data-viewer-kind')).toBe(kind);
  });
});

describe('sandboxed frame attributes (plan §2.4)', () => {
  it('ImageViewer renders an <iframe sandbox> with no allow-scripts/allow-same-origin', async () => {
    stubFetch(new Uint8Array([137, 80, 78, 71]), 'image/png');
    const { container } = render(() => (
      <ImageViewer part={part} blobUrl="blob:img" mime="image/png" name="logo.png" />
    ));
    const frame = await waitFor(() => {
      const el = container.querySelector('iframe');
      if (el === null) throw new Error('no iframe yet');
      return el as HTMLIFrameElement;
    });
    // sandbox attribute is PRESENT but empty → opaque origin, script-free
    expect(frame.hasAttribute('sandbox')).toBe(true);
    expect(frame.getAttribute('sandbox')).toBe('');
    const srcdoc = frame.getAttribute('srcdoc') ?? '';
    expect(srcdoc).toContain("default-src 'none'");
    expect(srcdoc).toContain('img-src data:');
  });

  it('TextViewer renders a sandboxed frame with escaped content', async () => {
    stubFetch(new TextEncoder().encode('<script>alert(1)</script>'), 'text/plain');
    const { container } = render(() => (
      <TextViewer part={{ ...part, type: 'text/plain' }} blobUrl="blob:t" mime="text/plain" name="a.txt" />
    ));
    const frame = await waitFor(() => {
      const el = container.querySelector('iframe');
      if (el === null) throw new Error('no iframe yet');
      return el as HTMLIFrameElement;
    });
    expect(frame.getAttribute('sandbox')).toBe('');
    const srcdoc = frame.getAttribute('srcdoc') ?? '';
    expect(srcdoc).toContain('&lt;script&gt;');
    expect(srcdoc).not.toContain('<script>alert');
  });
});
