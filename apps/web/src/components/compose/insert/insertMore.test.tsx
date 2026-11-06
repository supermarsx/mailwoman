// Link editing, shapes through the rich schema, drop points and reader tokens,
// screenshot crop and the online picture dialog.

import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import { EditorState, TextSelection } from 'prosemirror-state';
import { loadCatalog } from '../../../i18n/index.ts';
import { docFromHtml, htmlFromDoc } from '../richtext.ts';
import { linkAt } from '../RichTextEditor.tsx';
import { parseShapeAttr, shapeAttrValue } from './shapes.ts';
import { ATTACHMENT_DRAG_TYPE, beginAttachmentDrag, readAttachmentDrag } from '../../attachmentDrag.ts';
import { ComposeDropZone, type DropHost } from './ComposeDropZone.tsx';
import { ScreenshotCropDialog, clampRect } from './ScreenshotCropDialog.tsx';
import { OnlinePictureDialog, onlinePictureUrl, pictureNameFromUrl } from './OnlinePictureDialog.tsx';

beforeAll(async () => {
  await loadCatalog('mail');
  await loadCatalog('compose-insert');
});
afterEach(() => cleanup());

describe('links in the rich schema', () => {
  it('keeps tel: hrefs and target/rel for a new-tab link, and drops unsafe ones', () => {
    const html = htmlFromDoc(docFromHtml('<p><a href="tel:+4930123">call</a> <a href="https://a.org/" target="_blank" rel="noopener noreferrer" title="T">site</a> <a href="javascript:alert(1)">bad</a></p>', new Set()));
    expect(html).toContain('<a href="tel:+4930123">call</a>');
    expect(html).toContain('<a href="https://a.org/" title="T" target="_blank" rel="noopener noreferrer">site</a>');
    expect(html).not.toContain('javascript:');
  });
  it('a link without target stays without rel', () => {
    expect(htmlFromDoc(docFromHtml('<p><a href="https://a.org/">x</a></p>', new Set()))).toBe('<p><a href="https://a.org/">x</a></p>');
  });
  it('finds the run of one link around a position', () => {
    const doc = docFromHtml('<p>see <a href="https://a.org/">our <strong>pricing</strong> page</a> now</p>', new Set());
    const state = EditorState.create({ doc });
    const text = doc.textContent;
    const at = 1 + text.indexOf('pricing') + 2;
    const found = linkAt(state.apply(state.tr.setSelection(TextSelection.create(doc, at))), at);
    expect(found).not.toBeNull();
    expect(doc.textBetween(found!.from, found!.to)).toBe('our pricing page');
    expect(found!.mark.attrs['href']).toBe('https://a.org/');
    expect(linkAt(state, 2)).toBeNull();
  });
});

describe('shape attribute through the rich schema', () => {
  const spec = { id: 'star' as const, width: 120, height: 80, fill: '#ff0000', stroke: '#000000', strokeWidth: 3 };
  it('round-trips a validated shape on an owned image', () => {
    const html = `<p><img src="cid:s1@mailwoman" alt="A star" data-mw-shape="${shapeAttrValue(spec).replace(/"/g, '&quot;')}"></p>`;
    const doc = docFromHtml(html, new Set(['s1@mailwoman']));
    let shape: unknown = null;
    doc.descendants((n) => { if (n.type.name === 'image') shape = n.attrs['shape']; });
    expect(parseShapeAttr(shape as string)).toEqual(spec);
    expect(htmlFromDoc(doc)).toContain('data-mw-shape=');
  });
  it('drops a malformed or hostile shape value', () => {
    for (const bad of ['{"id":"nope"}', 'not json', '{"id":"star","width":5000,"height":10,"fill":"red","stroke":"red","strokeWidth":1}']) {
      const doc = docFromHtml(`<p><img src="cid:s1@mailwoman" alt="x" data-mw-shape='${bad}'></p>`, new Set(['s1@mailwoman']));
      let shape: unknown = 'unset';
      doc.descendants((n) => { if (n.type.name === 'image') shape = n.attrs['shape']; });
      expect(shape).toBeNull();
    }
  });
});

describe('reader attachment drag token', () => {
  it('round-trips and rejects malformed data', () => {
    const store: Record<string, string> = {};
    const dt = { setData: (k: string, v: string) => { store[k] = v; }, getData: (k: string) => store[k] ?? '' } as unknown as DataTransfer;
    beginAttachmentDrag({ dataTransfer: dt } as DragEvent, { accountId: 'a', emailId: 'm1', blobId: 'b1', name: 'x.pdf', mime: 'application/pdf', size: 10 });
    expect(readAttachmentDrag(dt)).toEqual({ accountId: 'a', emailId: 'm1', blobId: 'b1', name: 'x.pdf', mime: 'application/pdf', size: 10 });
    expect(readAttachmentDrag({ getData: () => '{"emailId":1}' })).toBeNull();
    expect(readAttachmentDrag({ getData: () => 'nope' })).toBeNull();
  });
});

function dropHost(over: Partial<DropHost> = {}): DropHost {
  const root = document.createElement('div');
  document.body.append(root);
  return {
    root: () => root, attachAllowed: () => true, inlineAllowed: () => true, messagesAllowed: () => true, maxBytes: () => 1000, policy: () => null, accountId: () => 'a',
    attachFiles: vi.fn(async () => undefined), placeImages: vi.fn(async () => undefined), fetchAttachment: vi.fn(async () => new File(['x'], 'doc.pdf', { type: 'application/pdf' })),
    fetchMessage: vi.fn(async () => new File(['x'], 'm.eml')), onBody: () => true, report: vi.fn(), ...over,
  };
}
function drag(type: string, target: Element, files: File[], extra: { types?: string[]; data?: Record<string, string>; x?: number; y?: number } = {}): void {
  const e = new Event(type, { bubbles: true, cancelable: true }) as Event & Record<string, unknown>;
  e['dataTransfer'] = { types: [...(files.length ? ['Files'] : []), ...(extra.types ?? [])], items: files.map((f) => ({ kind: 'file', type: f.type })), files, getData: (k: string) => extra.data?.[k] ?? '', dropEffect: 'none' };
  e['shiftKey'] = false; e['clientX'] = extra.x ?? 0; e['clientY'] = extra.y ?? 0;
  target.dispatchEvent(e);
}
const png = (): File => new File([new Uint8Array(10)], 'p.png', { type: 'image/png' });

describe('drop zone, drop point and tokens', () => {
  it('passes the editor position computed from the drop coordinates to the inline placement', async () => {
    const host = dropHost({ dropPoint: vi.fn((x: number, y: number) => x + y) });
    render(() => <ComposeDropZone host={host} />);
    drag('dragenter', host.root()!, [png()]);
    drag('drop', host.root()!, [png()], { x: 30, y: 12 });
    await waitFor(() => expect(host.placeImages).toHaveBeenCalled());
    expect((host.placeImages as ReturnType<typeof vi.fn>).mock.calls[0]![1]).toBe(42);
  });
  it('the counts line reflects what remains after validation', async () => {
    const host = dropHost({ onBody: () => false, attachFiles: vi.fn(() => new Promise<void>(() => undefined)) });
    render(() => <ComposeDropZone host={host} />);
    const files = [new File(['x'], 'a.pdf', { type: 'application/pdf' }), new File([new Uint8Array(5000)], 'big.zip', { type: 'application/zip' }), new File([new Uint8Array(5000)], 'big2.zip', { type: 'application/zip' })];
    drag('dragenter', host.root()!, files);
    expect((await screen.findByTestId('compose-drop-counts')).textContent).toMatch(/3 files/);
    drag('drop', host.root()!, files);
    await waitFor(() => expect(host.attachFiles).toHaveBeenCalled());
  });
  it('a reader attachment is fetched, policy-checked and attached; another account is refused', async () => {
    const token = JSON.stringify({ accountId: 'a', emailId: 'm1', blobId: 'b1', name: 'doc.pdf', mime: 'application/pdf', size: 1 });
    const host = dropHost();
    render(() => <ComposeDropZone host={host} />);
    drag('dragenter', host.root()!, [], { types: [ATTACHMENT_DRAG_TYPE], data: { [ATTACHMENT_DRAG_TYPE]: token } });
    expect(await screen.findByTestId('compose-drop-overlay')).toBeInTheDocument();
    drag('drop', host.root()!, [], { types: [ATTACHMENT_DRAG_TYPE], data: { [ATTACHMENT_DRAG_TYPE]: token } });
    await waitFor(() => expect(host.attachFiles).toHaveBeenCalled());
    expect((host.attachFiles as ReturnType<typeof vi.fn>).mock.calls[0]![0][0].name).toBe('doc.pdf');
    cleanup();
    const other = dropHost({ accountId: () => 'b' });
    render(() => <ComposeDropZone host={other} />);
    drag('dragenter', other.root()!, [], { types: [ATTACHMENT_DRAG_TYPE], data: { [ATTACHMENT_DRAG_TYPE]: token } });
    drag('drop', other.root()!, [], { types: [ATTACHMENT_DRAG_TYPE], data: { [ATTACHMENT_DRAG_TYPE]: token } });
    expect((await screen.findByTestId('compose-drop-problem')).textContent).toMatch(/another account/);
    expect(other.attachFiles).not.toHaveBeenCalled();
    expect(other.fetchAttachment).not.toHaveBeenCalled();
  });
});

describe('screenshot crop', () => {
  it('clamps rectangles into the picture', () => {
    expect(clampRect({ x: -5, y: 3, w: 9999, h: 0 }, 100, 50)).toEqual({ x: 0, y: 3, w: 100, h: 1 });
    expect(clampRect({ x: 99.6, y: 49, w: 10, h: 10 }, 100, 50)).toEqual({ x: 99, y: 49, w: 1, h: 1 });
  });
  it('the number fields change the selection and Use crops through the canvas', async () => {
    Object.assign(URL, { createObjectURL: vi.fn(() => 'blob:shot'), revokeObjectURL: vi.fn() });
    const close = vi.fn();
    const draw = vi.fn();
    vi.stubGlobal('createImageBitmap', vi.fn(async () => ({ width: 200, height: 100, close })));
    vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue({ drawImage: draw } as never);
    vi.spyOn(HTMLCanvasElement.prototype, 'toBlob').mockImplementation((cb) => cb(new Blob(['png'], { type: 'image/png' })));
    const onDone = vi.fn();
    render(() => <ScreenshotCropDialog file={new File(['x'], 's.png', { type: 'image/png' })} returnTo={null} onClose={() => undefined} onDone={onDone} />);
    const img = document.querySelector('img') as HTMLImageElement;
    Object.defineProperty(img, 'naturalWidth', { value: 200 });
    Object.defineProperty(img, 'naturalHeight', { value: 100 });
    fireEvent.load(img);
    fireEvent.input(screen.getByTestId('insert-crop-w'), { target: { value: '50' } });
    fireEvent.input(screen.getByTestId('insert-crop-h'), { target: { value: '20' } });
    fireEvent.click(screen.getByTestId('insert-crop-dialog-apply'));
    await waitFor(() => expect(onDone).toHaveBeenCalled());
    expect(draw).toHaveBeenCalledWith(expect.anything(), 0, 0, 50, 20, 0, 0, 50, 20);
    expect(onDone.mock.calls[0]![0].type).toBe('image/png');
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
  });
});

describe('online picture', () => {
  it('accepts only plain http(s) addresses and names the file', () => {
    expect(onlinePictureUrl('https://x.org/a/b.png')).not.toBeNull();
    expect(onlinePictureUrl('ftp://x.org/a.png')).toBeNull();
    expect(onlinePictureUrl('https://u:p@x.org/a.png')).toBeNull();
    expect(pictureNameFromUrl(new URL('https://x.org/a/b.png'), 'image/png')).toBe('b.png');
    expect(pictureNameFromUrl(new URL('https://x.org/'), 'image/jpeg')).toBe('picture.jpg');
  });
  it('shows the remote-content warning and fetches only through the image proxy', async () => {
    const fetcher = vi.fn(async () => ({ status: 200, ok: true, blob: async () => new Blob(['png'], { type: 'image/png' }) }));
    const onPicked = vi.fn();
    render(() => <OnlinePictureDialog returnTo={null} maxBytes={1000} fetcher={fetcher as never} onClose={() => undefined} onPicked={onPicked} />);
    expect(screen.getByTestId('insert-online-warning').textContent).toMatch(/image proxy/);
    expect(screen.getByTestId('insert-online-dialog-apply')).toBeDisabled();
    fireEvent.input(screen.getByTestId('insert-online-url'), { target: { value: 'https://cdn.example.org/cat.png' } });
    fireEvent.click(screen.getByTestId('insert-online-dialog-apply'));
    await waitFor(() => expect(onPicked).toHaveBeenCalled());
    expect(String((fetcher.mock.calls[0] as unknown[])[0])).toContain('/api/image-proxy?url=');
    expect(onPicked.mock.calls[0]![0].name).toBe('cat.png');
  });
  it('reports a proxy policy refusal and a non-picture answer', async () => {
    const refuse = vi.fn(async () => ({ status: 403, ok: false, blob: async () => new Blob([]) }));
    render(() => <OnlinePictureDialog returnTo={null} maxBytes={1000} fetcher={refuse as never} onClose={() => undefined} onPicked={() => undefined} />);
    fireEvent.input(screen.getByTestId('insert-online-url'), { target: { value: 'https://cdn.example.org/cat.png' } });
    fireEvent.click(screen.getByTestId('insert-online-dialog-apply'));
    expect((await screen.findByTestId('insert-online-error')).textContent).toMatch(/policy/);
    cleanup();
    const html = vi.fn(async () => ({ status: 200, ok: true, blob: async () => new Blob(['<html>'], { type: 'text/html' }) }));
    render(() => <OnlinePictureDialog returnTo={null} maxBytes={1000} fetcher={html as never} onClose={() => undefined} onPicked={() => undefined} />);
    fireEvent.input(screen.getByTestId('insert-online-url'), { target: { value: 'https://cdn.example.org/x' } });
    fireEvent.click(screen.getByTestId('insert-online-dialog-apply'));
    expect((await screen.findByTestId('insert-online-error')).textContent).toMatch(/not a PNG/);
  });
});
