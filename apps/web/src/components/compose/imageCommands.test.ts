import { describe, expect, it } from 'vitest';
import { EditorState, NodeSelection, type Command, type Transaction } from 'prosemirror-state';
import { docFromHtml, htmlFromDoc } from './richtext.ts';
import { cleanImageLook, imageStyle, lockedSide, readImageLook, scaledSize } from './imageAttrs.ts';
import { planTransform, removeImage, selectedImage, setImageAlt, setImageLook, transformImage, type ImageIO } from './imageCommands.ts';

const OWNED = new Set(['c1@mailwoman']);
const HTML = '<p>a<img src="cid:c1@mailwoman" alt="chart">b</p>';

function stateWithImageSelected(html = HTML): EditorState {
  const doc = docFromHtml(html, OWNED);
  let at = -1;
  doc.descendants((node, pos) => { if (node.type.name === 'image') at = pos; });
  return EditorState.create({ doc, selection: NodeSelection.create(doc, at) });
}

function run(state: EditorState, command: Command): EditorState {
  let next = state;
  command(state, (tr) => { next = state.apply(tr); });
  return next;
}

const attrsOf = (state: EditorState): Record<string, unknown> => selectedImage(state)!.node.attrs;

describe('image look', () => {
  it('clamps every value to what the schema keeps', () => {
    expect(cleanImageLook({ width: 0, height: 99999, align: 'diagonal', border: 3, radius: 5, title: 'a\nb' })).toEqual({ width: null, height: null, align: null, title: 'a b', border: 0, radius: 0 });
    expect(cleanImageLook({ width: '120px', align: 'center', border: 2, radius: 8 })).toMatchObject({ width: 120, align: 'center', border: 2, radius: 8 });
  });

  it('writes size, alignment, border and radius as inline style', () => {
    const style = Object.fromEntries(imageStyle({ width: 200, height: null, align: 'center', title: '', border: 1, radius: 8 }));
    expect(style).toMatchObject({ width: '200px', height: 'auto', display: 'block', 'margin-left': 'auto', 'margin-right': 'auto', border: '1px solid #808080', 'border-radius': '8px' });
    expect(Object.fromEntries(imageStyle({ width: null, height: null, align: 'left', title: '', border: 0, radius: 0 }))).toMatchObject({ 'margin-left': '0', 'margin-right': 'auto' });
  });

  it('reads a look back from markup', () => {
    const img = document.createElement('img');
    img.setAttribute('width', '300');
    img.setAttribute('title', 'Chart');
    img.style.display = 'block';
    img.style.marginLeft = 'auto';
    img.style.marginRight = '0';
    img.style.borderRadius = '50%';
    expect(readImageLook(img)).toMatchObject({ width: 300, align: 'right', title: 'Chart', radius: 9999 });
  });

  it('scales to a preset and keeps the ratio when locked', () => {
    expect(scaledSize({ width: 800, height: 400 }, 0.25)).toEqual({ width: 200, height: 100 });
    expect(lockedSide('width', 300, 2)).toBe(150);
    expect(lockedSide('height', 100, 2)).toBe(200);
  });
});

describe('picture commands', () => {
  it('finds the selected image, and nothing for a text selection', () => {
    expect(selectedImage(stateWithImageSelected())).not.toBeNull();
    const doc = docFromHtml('<p>x</p>');
    expect(selectedImage(EditorState.create({ doc }))).toBeNull();
    expect(setImageLook({ width: 10 })(EditorState.create({ doc }))).toBe(false);
  });

  it('sets size, alignment, title, border and radius, and sends them', () => {
    const next = run(stateWithImageSelected(), setImageLook({ width: 120, height: 60, align: 'center', title: 'Q3', border: 2, radius: 8 }));
    expect(attrsOf(next)).toMatchObject({ width: 120, height: 60, align: 'center', title: 'Q3', border: 2, radius: 8 });
    const html = htmlFromDoc(next.doc);
    expect(html).toContain('width="120"');
    expect(html).toContain('title="Q3"');
    expect(html).toContain('border-radius: 8px');
    expect(html).toContain('cid:c1@mailwoman');
  });

  it('refuses an out-of-range value', () => {
    expect(attrsOf(run(stateWithImageSelected(), setImageLook({ width: 999999 })))['width']).toBeNull();
  });

  it('sets the alt text and removes the image', () => {
    expect(attrsOf(run(stateWithImageSelected(), setImageAlt('A chart')))['alt']).toBe('A chart');
    const removed = run(stateWithImageSelected(), removeImage);
    expect(htmlFromDoc(removed.doc)).toBe('<p>ab</p>');
  });

  it('round-trips a look through the sent HTML', () => {
    const next = run(stateWithImageSelected(), setImageLook({ width: 120, align: 'right', border: 1 }));
    const again = docFromHtml(htmlFromDoc(next.doc), OWNED);
    let look: Record<string, unknown> = {};
    again.descendants((node) => { if (node.type.name === 'image') look = node.attrs; });
    expect(look).toMatchObject({ width: 120, align: 'right', border: 1 });
  });
});

describe('transforms', () => {
  it('plans a clockwise rotation as a swapped canvas', () => {
    expect(planTransform({ kind: 'rotate', dir: 'cw' }, 400, 200)).toMatchObject({ width: 200, height: 400, matrix: [0, 1, -1, 0, 200, 0] });
    expect(planTransform({ kind: 'rotate', dir: 'ccw' }, 400, 200)).toMatchObject({ width: 200, height: 400, matrix: [0, -1, 1, 0, 0, 400] });
  });

  it('plans a flip on the same canvas', () => {
    expect(planTransform({ kind: 'flip', axis: 'h' }, 400, 200)).toMatchObject({ width: 400, height: 200, matrix: [-1, 0, 0, 1, 400, 0] });
    expect(planTransform({ kind: 'flip', axis: 'v' }, 400, 200)).toMatchObject({ matrix: [1, 0, 0, -1, 0, 200] });
  });

  it('plans a crop as a window, never leaving less than a tenth', () => {
    expect(planTransform({ kind: 'crop', left: 0.1, top: 0.25, right: 0.1, bottom: 0.25 }, 400, 200)).toMatchObject({ width: 320, height: 100, source: { x: 40, y: 50, width: 320, height: 100 } });
    const extreme = planTransform({ kind: 'crop', left: 0.9, top: 0, right: 0.9, bottom: 0 }, 400, 200);
    expect(extreme.width).toBeGreaterThanOrEqual(40);
  });

  function io(over: Partial<ImageIO> = {}): ImageIO {
    return {
      sourceUrl: () => 'blob:preview',
      render: async () => new Blob(['x'], { type: 'image/png' }),
      store: async () => ({ cid: 'edited@mailwoman' }),
      ...over,
    };
  }
  const viewOf = (state: EditorState) => {
    const view = { state, dispatch(tr: Transaction) { view.state = view.state.apply(tr); } };
    return view;
  };

  it('replaces the picture with the edited copy and resets its size', async () => {
    const view = viewOf(run(stateWithImageSelected(), setImageLook({ width: 100, height: 50 })));
    expect(await transformImage(view, { kind: 'rotate', dir: 'cw' }, io())).toBe('done');
    expect(attrsOf(view.state)).toMatchObject({ cid: 'edited@mailwoman', width: null, height: null });
  });

  it('keeps the size on a flip', async () => {
    const view = viewOf(run(stateWithImageSelected(), setImageLook({ width: 100 })));
    await transformImage(view, { kind: 'flip', axis: 'h' }, io());
    expect(attrsOf(view.state)['width']).toBe(100);
  });

  it('reports no image, no local copy and a failed render or store without touching the document', async () => {
    const doc = docFromHtml('<p>x</p>');
    expect(await transformImage(viewOf(EditorState.create({ doc })), { kind: 'flip', axis: 'h' }, io())).toBe('no-image');
    expect(await transformImage(viewOf(stateWithImageSelected()), { kind: 'flip', axis: 'h' }, io({ sourceUrl: () => null }))).toBe('no-source');
    const view = viewOf(stateWithImageSelected());
    expect(await transformImage(view, { kind: 'flip', axis: 'h' }, io({ render: async () => { throw new Error('x'); } }))).toBe('failed');
    expect(await transformImage(view, { kind: 'flip', axis: 'h' }, io({ store: async () => null }))).toBe('failed');
    expect(attrsOf(view.state)['cid']).toBe('c1@mailwoman');
  });
});
