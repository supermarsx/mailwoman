// The contextual ribbon tabs of the rich-text editor: Picture while an image is
// selected, Table layout while the caret is in a table, and the groups on them.

import { beforeAll, describe, expect, it, vi } from 'vitest';
import { createSignal } from 'solid-js';
import { fireEvent, render, screen, waitFor, within } from '@solidjs/testing-library';
import { EditorState, NodeSelection, TextSelection, type Command } from 'prosemirror-state';
import RichTextEditor, { type EditorContext } from './RichTextEditor.tsx';
import { PictureGroups } from './PictureGroups.tsx';
import { TableLayoutGroups } from './TableLayoutGroups.tsx';
import { docFromHtml, htmlFromDoc } from './richtext.ts';
import { insertTable } from './richCommands.ts';
import { selectedImage } from './imageCommands.ts';
import { STYLE_ENTRIES, currentStyle, shortcutLabel } from './StyleGallery.tsx';

beforeAll(() => {
  Object.defineProperty(Range.prototype, 'getClientRects', { configurable: true, value: () => [] });
  Object.defineProperty(Range.prototype, 'getBoundingClientRect', { configurable: true, value: () => ({ left: 0, right: 0, top: 0, bottom: 0, width: 0, height: 0 }) });
});

const OWNED = new Set(['c1@mailwoman']);

function imageState(): EditorState {
  const doc = docFromHtml('<p>a<img src="cid:c1@mailwoman" alt="chart">b</p>', OWNED);
  let at = -1;
  doc.descendants((node, pos) => { if (node.type.name === 'image') at = pos; });
  return EditorState.create({ doc, selection: NodeSelection.create(doc, at) });
}

/** Mount `view` against a state signal so a control's command is applied for real. */
function mountPicture(initial: EditorState, extra: { natural?: { width: number; height: number } | null; transform?: (op: unknown) => void; replace?: (file: File) => void; canStore?: boolean } = {}) {
  const [state, setState] = createSignal<EditorState>(initial);
  const execute = (command: Command): void => { command(state(), (tr) => setState(state().apply(tr))); };
  const transform = extra.transform ?? vi.fn();
  render(() => <PictureGroups state={state} execute={execute} naturalSize={() => extra.natural ?? { width: 800, height: 400 }} transform={transform as never} replace={extra.replace ?? vi.fn()} canStore={extra.canStore ?? true} />);
  return { state, transform };
}

describe('Picture tab groups', () => {
  it('has Size, Arrange, Text, Style and Edit groups', () => {
    mountPicture(imageState());
    expect(Array.from(document.querySelectorAll('[data-ribbon-group]')).map((g) => g.getAttribute('data-testid'))).toEqual(['rt-group-picture-size', 'rt-group-picture-arrange', 'rt-group-picture-text', 'rt-group-picture-style', 'rt-group-picture-edit']);
  });

  it('resizes with the aspect ratio locked, and presets scale from the natural size', () => {
    const { state } = mountPicture(imageState());
    fireEvent.change(screen.getByTestId('rt-picture-width'), { target: { value: '400' } });
    expect(selectedImage(state())!.node.attrs).toMatchObject({ width: 400, height: 200 });
    fireEvent.click(screen.getByTestId('rt-picture-scale-25'));
    expect(selectedImage(state())!.node.attrs).toMatchObject({ width: 200, height: 100 });
    fireEvent.click(screen.getByTestId('rt-picture-reset-size'));
    expect(selectedImage(state())!.node.attrs).toMatchObject({ width: null, height: null });
  });

  it('sets one side only when the lock is off', () => {
    const { state } = mountPicture(imageState());
    fireEvent.click(screen.getByTestId('rt-picture-lock'));
    expect(screen.getByTestId('rt-picture-lock')).toHaveAttribute('aria-pressed', 'false');
    fireEvent.change(screen.getByTestId('rt-picture-width'), { target: { value: '300' } });
    expect(selectedImage(state())!.node.attrs).toMatchObject({ width: 300, height: null });
  });

  it('aligns, with the current choice marked pressed, and back in line', () => {
    const { state } = mountPicture(imageState());
    expect(screen.getByTestId('rt-picture-align-inline')).toHaveAttribute('aria-pressed', 'true');
    fireEvent.click(screen.getByTestId('rt-picture-align-center'));
    expect(selectedImage(state())!.node.attrs['align']).toBe('center');
    expect(screen.getByTestId('rt-picture-align-center')).toHaveAttribute('aria-pressed', 'true');
    fireEvent.click(screen.getByTestId('rt-picture-align-inline'));
    expect(selectedImage(state())!.node.attrs['align']).toBeNull();
  });

  it('edits alt text and title, border and radius', () => {
    const { state } = mountPicture(imageState());
    fireEvent.change(screen.getByTestId('rt-picture-alt'), { target: { value: 'Quarterly chart' } });
    fireEvent.change(screen.getByTestId('rt-picture-title'), { target: { value: 'Q3' } });
    fireEvent.change(screen.getByTestId('rt-picture-border'), { target: { value: '2' } });
    fireEvent.change(screen.getByTestId('rt-picture-radius'), { target: { value: '8' } });
    expect(selectedImage(state())!.node.attrs).toMatchObject({ alt: 'Quarterly chart', title: 'Q3', border: 2, radius: 8 });
    const html = htmlFromDoc(state().doc);
    expect(html).toContain('alt="Quarterly chart"');
    expect(html).toContain('border: 2px solid');
  });

  it('asks for rotate, flip and crop, removes the picture, and offers Replace only when the host can store', () => {
    const transform = vi.fn();
    const { state } = mountPicture(imageState(), { transform });
    fireEvent.click(screen.getByTestId('rt-picture-rotate-cw'));
    fireEvent.click(screen.getByTestId('rt-picture-rotate-ccw'));
    fireEvent.click(screen.getByTestId('rt-picture-flip-h'));
    fireEvent.click(screen.getByTestId('rt-picture-flip-v'));
    expect(transform.mock.calls.map((c) => c[0])).toEqual([{ kind: 'rotate', dir: 'cw' }, { kind: 'rotate', dir: 'ccw' }, { kind: 'flip', axis: 'h' }, { kind: 'flip', axis: 'v' }]);
    fireEvent.click(screen.getByTestId('rt-picture-crop'));
    const form = screen.getByTestId('rt-picture-crop-form');
    fireEvent.input(within(form).getByLabelText('Left (%)'), { target: { value: '10' } });
    fireEvent.input(within(form).getByLabelText('Top (%)'), { target: { value: '20' } });
    fireEvent.click(within(form).getByTestId('rt-picture-crop-apply'));
    expect(transform).toHaveBeenLastCalledWith({ kind: 'crop', left: 0.1, top: 0.2, right: 0, bottom: 0 });
    expect(screen.getByTestId('rt-picture-replace')).toBeInTheDocument();
    fireEvent.click(screen.getByTestId('rt-picture-remove'));
    expect(htmlFromDoc(state().doc)).toBe('<p>ab</p>');
  });

  it('leaves Replace out when the host cannot store images, and disables everything with no picture selected', () => {
    const doc = docFromHtml('<p>x</p>');
    mountPicture(EditorState.create({ doc }), { canStore: false });
    expect(screen.queryByTestId('rt-picture-replace')).toBeNull();
    expect(screen.getByTestId('rt-picture-rotate-cw')).toBeDisabled();
    expect(screen.getByTestId('rt-picture-width')).toBeDisabled();
  });
});

describe('Table layout groups', () => {
  function mountTable() {
    const doc = docFromHtml('<p>x</p>');
    let initial = EditorState.create({ doc, selection: TextSelection.atStart(doc) });
    insertTable(2, 2)(initial, (tr) => { initial = initial.apply(tr); });
    const [state, setState] = createSignal<EditorState>(initial);
    const execute = (command: Command): void => { command(state(), (tr) => setState(state().apply(tr))); };
    render(() => <TableLayoutGroups state={state} execute={execute} shading={() => <span />} clearShading={() => <span />} />);
    return state;
  }
  const cells = (state: EditorState): number => { let n = 0; state.doc.descendants((node) => { if (node.type.name === 'table_cell') n += 1; }); return n; };

  it('has Rows, Columns and Cells groups, then shading and the table', () => {
    mountTable();
    expect(Array.from(document.querySelectorAll('[data-ribbon-group]')).map((g) => g.getAttribute('data-testid'))).toEqual(['rt-group-table-rows', 'rt-group-table-columns', 'rt-group-table-cells', 'rt-group-table-shading', 'rt-group-table']);
  });

  it('inserts and deletes rows and columns', () => {
    const state = mountTable();
    expect(cells(state())).toBe(4);
    fireEvent.click(screen.getByRole('button', { name: 'Row below' }));
    expect(cells(state())).toBe(6);
    fireEvent.click(screen.getByRole('button', { name: 'Column after' }));
    expect(cells(state())).toBe(9);
    fireEvent.click(screen.getByRole('button', { name: 'Delete row' }));
    fireEvent.click(screen.getByRole('button', { name: 'Delete column' }));
    expect(cells(state())).toBe(4);
  });

  it('sets row height, column width, padding and alignment', () => {
    const state = mountTable();
    fireEvent.change(screen.getByTestId('rt-table-row-height'), { target: { value: '48' } });
    fireEvent.change(screen.getByTestId('rt-table-column-width'), { target: { value: '120' } });
    fireEvent.change(screen.getByTestId('rt-table-padding'), { target: { value: 'roomy' } });
    fireEvent.click(screen.getByTestId('rt-table-align-center'));
    const first = (() => { let found: Record<string, string> | null = null; state().doc.descendants((n) => { if (!found && n.type.name === 'table_cell') found = n.attrs['emailStyles'] as Record<string, string>; }); return found!; })();
    expect(first).toMatchObject({ height: '48px', width: '120px', 'padding-top': '10px', 'padding-left': '14px', 'text-align': 'center' });
    expect(screen.getByTestId('rt-table-align-center')).toHaveAttribute('aria-pressed', 'true');
  });
});

describe('contextual tabs in the editor', () => {
  it('reports a table context once a table is inserted', async () => {
    const contexts: EditorContext[] = [];
    render(() => <RichTextEditor initialHtml="<p>Keep</p>" ariaLabel="Body" onChange={() => undefined} onContext={(c) => contexts.push(c)} />);
    expect(contexts.at(-1)).toEqual({ image: false, table: false });
    fireEvent.click(screen.getByRole('tab', { name: 'Insert' }));
    fireEvent.click(screen.getByRole('button', { name: 'Insert table' }));
    await waitFor(() => expect(contexts.at(-1)).toEqual({ image: false, table: true }));
  });

  it('keeps the Picture tab out of the ribbon until an image is selected', () => {
    // jsdom cannot select a node by pointer, so the selected state is covered through the groups above;
    // here: with an image in the document but only a text caret, there is no Picture tab.
    render(() => <RichTextEditor initialHtml={'<p>a<img src="cid:c1@mailwoman" alt="chart">b</p>'} ariaLabel="Body" onChange={() => undefined} inline={{ owned: () => OWNED, preview: () => null, upload: async () => [] }} />);
    expect(document.querySelector('.ProseMirror img')).not.toBeNull();
    expect(screen.queryByRole('tab', { name: 'Picture' })).toBeNull();
  });
});

describe('paragraph style gallery', () => {
  it('lists Normal, Heading 1 to 4, Quote and Code first, with shortcuts', () => {
    expect(STYLE_ENTRIES.filter((e) => e.group === 'main').map((e) => e.id)).toEqual(['normal', 'heading1', 'heading2', 'heading3', 'heading4', 'quote', 'code']);
    expect(shortcutLabel('Mod-Alt-1', false)).toBe('Ctrl+Alt+1');
    expect(shortcutLabel('Mod-Alt-1', true)).toBe('⌘⌥1');
  });

  it('opens a listbox of styles drawn in their own look, applies one and marks the current', async () => {
    let html = '';
    render(() => <RichTextEditor initialHtml="<p>Title me</p>" ariaLabel="Body" onChange={(value) => { html = value; }} />);
    const button = screen.getByTestId('rt-style-gallery');
    expect(button).toHaveTextContent('Normal');
    fireEvent.click(button);
    const list = screen.getByRole('listbox', { name: 'Paragraph style' });
    expect(within(list).getAllByRole('option').length).toBe(STYLE_ENTRIES.length);
    expect(within(list).getByTestId('rt-style-heading1')).toHaveTextContent('Ctrl+Alt+1');
    expect(within(list).getByTestId('rt-style-normal')).toHaveAttribute('aria-selected', 'true');
    fireEvent.click(within(list).getByTestId('rt-style-heading2'));
    await waitFor(() => expect(html).toContain('<h2'));
    expect(screen.getByTestId('rt-style-gallery')).toHaveTextContent('Heading 2');
  });

  it('applies Quote and Code, and Code toggles back', async () => {
    let html = '';
    render(() => <RichTextEditor initialHtml="<p>text</p>" ariaLabel="Body" onChange={(value) => { html = value; }} />);
    fireEvent.click(screen.getByTestId('rt-style-gallery'));
    fireEvent.click(screen.getByTestId('rt-style-code'));
    await waitFor(() => expect(html).toContain('<pre'));
    fireEvent.click(screen.getByTestId('rt-style-gallery'));
    fireEvent.click(screen.getByTestId('rt-style-code'));
    await waitFor(() => expect(html).not.toContain('<pre'));
    fireEvent.click(screen.getByTestId('rt-style-gallery'));
    fireEvent.click(screen.getByTestId('rt-style-quote'));
    await waitFor(() => expect(html).toContain('<blockquote'));
  });

  it('reads the current style from the selection', () => {
    expect(currentStyle(null)).toBe('normal');
    const doc = docFromHtml('<h3>x</h3>');
    expect(currentStyle(EditorState.create({ doc, selection: TextSelection.atStart(doc) }))).toBe('heading3');
    const quote = docFromHtml('<blockquote><p>q</p></blockquote>');
    expect(currentStyle(EditorState.create({ doc: quote, selection: TextSelection.atStart(quote) }))).toBe('quote');
  });
});
