// Regression suite for the plain-text attachment preview: whatever the bytes or
// the transport do, the viewer renders a state inside its own panel and never
// throws to an enclosing error boundary.
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, waitFor } from '@solidjs/testing-library';
import { ErrorBoundary } from 'solid-js';
import { TextViewer } from './TextViewer.tsx';
import { decodeText, TEXT_LINE_CAP, TEXT_PREVIEW_CAP } from './textDecode.ts';

const part = { partId: null, blobId: 'b', size: 1, type: 'text/plain' };

function respond(body: Uint8Array | string, init: ResponseInit = {}): void {
  const bytes = typeof body === 'string' ? new TextEncoder().encode(body) : body;
  vi.stubGlobal('fetch', vi.fn(async () => new Response(bytes.slice().buffer as ArrayBuffer, init)));
}

/** Mount inside a boundary that records anything thrown to it. */
function mount(mime = 'text/plain', name = 'a.txt') {
  const caught: unknown[] = [];
  const view = render(() => (
    <ErrorBoundary fallback={(e) => { caught.push(e); return <p data-testid="boundary">boundary</p>; }}>
      <TextViewer part={part} blobUrl="blob:http://localhost/x" mime={mime} name={name} />
    </ErrorBoundary>
  ));
  const state = () => view.container.querySelector('[data-text-state]')?.getAttribute('data-text-state') ?? null;
  return { ...view, caught, state };
}

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe('TextViewer never throws to the boundary', () => {
  it('UTF-8', async () => {
    respond('héllo wörld');
    const v = mount();
    await waitFor(() => expect(v.state()).toBe('text'));
    expect(v.getByTestId('text-content').textContent).toBe('héllo wörld');
    expect(v.caught).toEqual([]);
  });

  it('UTF-16LE and UTF-16BE with BOM', async () => {
    respond(new Uint8Array([0xff, 0xfe, 0x68, 0x00, 0x69, 0x00]));
    let v = mount();
    await waitFor(() => expect(v.state()).toBe('text'));
    expect(v.getByTestId('text-content').textContent).toBe('hi');
    v.unmount();
    respond(new Uint8Array([0xfe, 0xff, 0x00, 0x68, 0x00, 0x69]));
    v = mount();
    await waitFor(() => expect(v.state()).toBe('text'));
    expect(v.getByTestId('text-content').textContent).toBe('hi');
    expect(v.caught).toEqual([]);
  });

  it('Latin-1 declared by charset', async () => {
    respond(new Uint8Array([0x63, 0x61, 0x66, 0xe9]));
    const v = mount('text/plain; charset=iso-8859-1');
    await waitFor(() => expect(v.state()).toBe('text'));
    expect(v.getByTestId('text-content').textContent).toBe('café');
  });

  it('an unknown charset label falls back to UTF-8', async () => {
    respond('plain');
    const v = mount('text/plain; charset=x-no-such-charset');
    await waitFor(() => expect(v.state()).toBe('text'));
    expect(v.getByTestId('text-content').textContent).toBe('plain');
    expect(v.caught).toEqual([]);
  });

  it('invalid UTF-8 becomes replacement characters', async () => {
    respond(new Uint8Array([0x61, 0xff, 0xfe, 0x62]));
    const v = mount();
    await waitFor(() => expect(v.state()).toBe('text'));
    expect(v.getByTestId('text-content').textContent).toContain('�');
  });

  it('empty file says so', async () => {
    respond(new Uint8Array());
    const v = mount();
    await waitFor(() => expect(v.state()).toBe('empty'));
    expect(v.caught).toEqual([]);
  });

  it('binary-looking bytes are not shown as text', async () => {
    respond(new Uint8Array(Array.from({ length: 4000 }, (_, i) => i % 256)));
    const v = mount();
    await waitFor(() => expect(v.state()).toBe('binary'));
    expect(v.container.querySelector('[data-testid="text-content"]')).toBeNull();
    expect(v.caught).toEqual([]);
  });

  it('a file past the cap shows the first part and says so', async () => {
    respond('abcdefgh\n'.repeat(Math.ceil((TEXT_PREVIEW_CAP + 5000) / 9)));
    const v = mount();
    await waitFor(() => expect(v.state()).toBe('text'));
    expect(v.getByTestId('text-content').textContent!.length).toBeLessThanOrEqual(TEXT_PREVIEW_CAP);
    expect(v.container.querySelector('.mw-text__note')).not.toBeNull();
  });

  it('CRLF is normalised', async () => {
    respond('a\r\nb\r\nc');
    const v = mount();
    await waitFor(() => expect(v.state()).toBe('text'));
    expect(v.getByTestId('text-content').textContent).toBe('a\nb\nc');
  });

  it('one extremely long line is cut', async () => {
    respond('y'.repeat(TEXT_LINE_CAP * 3));
    const v = mount();
    await waitFor(() => expect(v.state()).toBe('text'));
    expect(v.getByTestId('text-content').textContent!.length).toBe(TEXT_LINE_CAP);
  });

  it('markup in the file is text, not elements', async () => {
    respond('<img src=x onerror=alert(1)><script>1</script>');
    const v = mount();
    await waitFor(() => expect(v.state()).toBe('text'));
    expect(v.container.querySelector('img, script')).toBeNull();
  });
});

describe('TextViewer failure states stay inside the panel', () => {
  it('a network failure shows Retry and Download, and Retry re-fetches', async () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => undefined);
    const fetchMock = vi.fn()
      .mockRejectedValueOnce(new TypeError('Failed to fetch'))
      .mockResolvedValue(new Response('now fine'));
    vi.stubGlobal('fetch', fetchMock);
    const v = mount();
    await waitFor(() => expect(v.state()).toBe('error'));
    expect(v.caught).toEqual([]);
    expect(JSON.stringify(warn.mock.calls)).toContain('text/plain');
    expect(JSON.stringify(warn.mock.calls)).not.toContain('a.txt');
    fireEvent.click(v.getByTestId('text-retry'));
    await waitFor(() => expect(v.state()).toBe('text'));
    expect(v.getByTestId('text-content').textContent).toBe('now fine');
  });

  it('an HTTP error status is a contained error, not a throw', async () => {
    vi.spyOn(console, 'warn').mockImplementation(() => undefined);
    respond('{"error":"nope"}', { status: 500 });
    const v = mount();
    await waitFor(() => expect(v.state()).toBe('error'));
    expect(v.caught).toEqual([]);
  });

  it('a body that cannot be read is a contained error', async () => {
    vi.spyOn(console, 'warn').mockImplementation(() => undefined);
    vi.stubGlobal('fetch', vi.fn(async () => ({
      ok: true, status: 200, headers: new Headers(),
      arrayBuffer: async () => { throw new TypeError('body used already'); },
    })));
    const v = mount();
    await waitFor(() => expect(v.state()).toBe('error'));
    expect(v.caught).toEqual([]);
  });
});

describe('TextViewer tools', () => {
  it('line numbers need wrap off, and the gutter counts lines', async () => {
    respond('a\nb\nc');
    const v = mount();
    await waitFor(() => expect(v.state()).toBe('text'));
    expect((v.getByTestId('text-numbers') as HTMLInputElement).disabled).toBe(true);
    fireEvent.click(v.getByTestId('text-wrap'));
    fireEvent.click(v.getByTestId('text-numbers'));
    await waitFor(() => expect(v.container.querySelector('.mw-text__gutter')?.textContent).toBe('1\n2\n3'));
  });

  it('search counts matches case-insensitively', async () => {
    respond('Foo bar foo\nFOO');
    const v = mount();
    await waitFor(() => expect(v.state()).toBe('text'));
    fireEvent.input(v.getByTestId('text-search'), { target: { value: 'foo' } });
    await waitFor(() => expect(v.getByTestId('text-search-count').textContent).toContain('3'));
    fireEvent.input(v.getByTestId('text-search'), { target: { value: 'zzz' } });
    await waitFor(() => expect(v.getByTestId('text-search-count').textContent).not.toContain('3'));
  });

  it('copy writes the text to the clipboard', async () => {
    respond('copy me');
    const writeText = vi.fn(async () => undefined);
    vi.stubGlobal('navigator', { ...navigator, clipboard: { writeText } });
    const v = mount();
    await waitFor(() => expect(v.state()).toBe('text'));
    fireEvent.click(v.getByTestId('text-copy'));
    await waitFor(() => expect(writeText).toHaveBeenCalledWith('copy me'));
  });
});

describe('decodeText', () => {
  it('is total over odd inputs', () => {
    for (const mime of ['', 'text/plain; charset=', 'text/plain; charset="utf-8"', 'text/plain; charset=bogus']) {
      expect(() => decodeText(new Uint8Array([0xc3]), mime)).not.toThrow();
      expect(() => decodeText(new Uint8Array([0xff, 0xfe, 0x41]), mime)).not.toThrow();
    }
  });
});
