import { describe, expect, it } from 'vitest';
import { EditorState, TextSelection, type Command } from 'prosemirror-state';
import { CellSelection } from 'prosemirror-tables';
import { docFromHtml, htmlFromDoc } from './richtext.ts';
import { cellLook, inTable, setCellAlign, setCellPadding, setCellPaddingPreset, setColumnWidth, setRowHeight } from './tableLayout.ts';
import { defaultCellStyles } from './richFormatting.ts';
import { insertTable } from './richCommands.ts';

function run(state: EditorState, command: Command): EditorState {
  let next = state;
  command(state, (tr) => { next = state.apply(tr); });
  return next;
}

const cellPositions = (state: EditorState): number[] => {
  const cells: number[] = [];
  state.doc.descendants((node, pos) => { if (node.type.name === 'table_cell') cells.push(pos); });
  return cells;
};

/** A table made by the Insert table command (cells without styles take the defaults), caret in cell `index`, row-major. */
function stateAt(index: number): EditorState {
  const doc = docFromHtml('<p>x</p>');
  const made = run(EditorState.create({ doc, selection: TextSelection.atStart(doc) }), insertTable(2, 2));
  return made.apply(made.tr.setSelection(TextSelection.near(made.doc.resolve(cellPositions(made)[index]! + 1))));
}

const stylesOfCells = (state: EditorState): (Record<string, string> | null)[] => {
  const out: (Record<string, string> | null)[] = [];
  state.doc.descendants((node) => { if (node.type.name === 'table_cell') out.push(node.attrs['emailStyles'] as Record<string, string> | null); });
  return out;
};

describe('table layout commands', () => {
  it('knows when the caret is in a table', () => {
    expect(inTable(stateAt(0))).toBe(true);
    const doc = docFromHtml('<p>x</p>');
    expect(inTable(EditorState.create({ doc, selection: TextSelection.atStart(doc) }))).toBe(false);
  });

  it('defaults a cell to a 6px 8px padding, a thin border and top alignment', () => {
    const styles = defaultCellStyles();
    expect(styles['padding-top']).toBe('6px');
    expect(styles['padding-left']).toBe('8px');
    expect(styles['vertical-align']).toBe('top');
    expect(styles['border-top-style']).toBe('solid');
  });

  it('pads only the cell around the caret, keeping its border', () => {
    const [first, second] = stylesOfCells(run(stateAt(0), setCellPadding(10, 14)));
    expect(first).toMatchObject({ 'padding-top': '10px', 'padding-left': '14px', 'border-top-style': 'solid' });
    expect(second).toBeNull();
  });

  it('pads every cell of a cell selection, with a preset', () => {
    const base = stateAt(0);
    const cells = cellPositions(base);
    const selected = base.apply(base.tr.setSelection(CellSelection.create(base.doc, cells[0]!, cells[3]!)));
    for (const styles of stylesOfCells(run(selected, setCellPaddingPreset('roomy')))) expect(styles).toMatchObject({ 'padding-top': '10px', 'padding-left': '14px' });
  });

  it('sets a row height on every cell of the row, and a column width on every cell of the column', () => {
    expect(stylesOfCells(run(stateAt(0), setRowHeight(48))).map((s) => s?.['height'])).toEqual(['48px', '48px', undefined, undefined]);
    expect(stylesOfCells(run(stateAt(3), setColumnWidth(120))).map((s) => s?.['width'])).toEqual([undefined, '120px', undefined, '120px']);
  });

  it('clears a size with null, refuses an out-of-range value and aligns cells', () => {
    const sized = run(stateAt(0), setRowHeight(48));
    expect(stylesOfCells(run(sized, setRowHeight(null))).map((s) => s?.['height'])).toEqual([undefined, undefined, undefined, undefined]);
    expect(setRowHeight(99999)(stateAt(0))).toBe(false);
    expect(stylesOfCells(run(stateAt(1), setCellAlign('center')))[1]).toMatchObject({ 'text-align': 'center' });
  });

  it('reports the look of the first selected cell', () => {
    expect(cellLook(run(stateAt(1), setCellPadding(2, 4)))).toMatchObject({ paddingVertical: 2, paddingHorizontal: 4, rows: 2, columns: 2, header: false });
    expect(cellLook(stateAt(0))).toMatchObject({ paddingVertical: 6, paddingHorizontal: 8 });
  });

  it('does nothing outside a table', () => {
    const doc = docFromHtml('<p>x</p>');
    const state = EditorState.create({ doc, selection: TextSelection.atStart(doc) });
    expect(setCellPadding(2, 2)(state)).toBe(false);
    expect(cellLook(state)).toBeNull();
  });
});

describe('table spacing in the sent HTML', () => {
  const sent = (state: EditorState): HTMLElement => {
    const holder = document.createElement('div');
    holder.innerHTML = htmlFromDoc(state.doc);
    return holder;
  };

  it('collapses borders and gives every cell of an inserted table the same padding', () => {
    const holder = sent(stateAt(0));
    expect(holder.querySelector('table')!.style.borderCollapse).toBe('collapse');
    const cells = Array.from(holder.querySelectorAll<HTMLElement>('td'));
    expect(cells).toHaveLength(4);
    for (const cell of cells) {
      expect(cell.style.paddingTop).toBe('6px');
      expect(cell.style.paddingBottom).toBe('6px');
      expect(cell.style.paddingLeft).toBe('8px');
      expect(cell.style.paddingRight).toBe('8px');
    }
  });

  it('has no stray margin from a paragraph at the top or bottom of a cell', () => {
    const holder = sent(stateAt(0));
    for (const p of Array.from(holder.querySelectorAll<HTMLElement>('td > p'))) {
      expect(p.style.marginTop).toBe('0pt');
      expect(p.style.marginBottom).toBe('0pt');
    }
  });

  it('keeps a margin the author set, and a second round trip changes nothing', () => {
    const html = '<table><tbody><tr><td><p style="margin-top:12pt">x</p><p>y</p></td></tr></tbody></table>';
    const once = htmlFromDoc(docFromHtml(html));
    const holder = document.createElement('div');
    holder.innerHTML = once;
    expect(holder.querySelector<HTMLElement>('td > p')!.style.marginTop).toBe('12pt');
    expect(holder.querySelectorAll<HTMLElement>('td > p')[1]!.style.marginBottom).toBe('0pt');
    expect(htmlFromDoc(docFromHtml(once))).toBe(once);
  });

  it('leaves a table that came in without styles as it came, so a quoted original keeps its look', () => {
    const holder = document.createElement('div');
    holder.innerHTML = htmlFromDoc(docFromHtml('<table><tbody><tr><td><p>a</p></td></tr></tbody></table>'));
    expect(holder.querySelector<HTMLElement>('td')!.style.paddingTop).toBe('');
  });
});
