// The Table layout ribbon's commands: cell padding, row height, column width and
// alignment, on the selected cells (a cell selection, or the cell around the
// caret). Values go through `emailStyles` on each cell, the same validated
// attribute an imported table uses, so what the editor shows is what is sent.
//
// A table cell has no margin in HTML. "Spacing" here is the cell's padding; the
// gap between cells comes from the collapsed border. Row and column sizes are
// written on every cell of the row or column so mail clients agree.

import type { Command, EditorState } from 'prosemirror-state';
import type { Node as PMNode } from 'prosemirror-model';
import { TableMap, isInTable, selectedRect } from 'prosemirror-tables';
import { defaultCellStyles } from './richFormatting.ts';
import { safeEmailStyle } from './safeEmailStyles.ts';

export const PADDING_PRESETS = {
  compact: { vertical: 2, horizontal: 4 },
  normal: { vertical: 6, horizontal: 8 },
  roomy: { vertical: 10, horizontal: 14 },
} as const;
export type PaddingPreset = keyof typeof PADDING_PRESETS;
export const ROW_HEIGHTS = [24, 32, 48, 64, 96] as const;
export const COLUMN_WIDTHS = [60, 90, 120, 160, 240] as const;
export type CellAlign = 'left' | 'center' | 'right';
export type CellVertical = 'top' | 'middle' | 'bottom';

export const inTable = (state: EditorState): boolean => isInTable(state);

interface CellRef { node: PMNode; pos: number }

function rectOf(state: EditorState): { cells: CellRef[]; rows: CellRef[]; columns: CellRef[] } | null {
  if (!isInTable(state)) return null;
  const rect = selectedRect(state);
  const { map, table, tableStart } = rect;
  const at = (offsets: number[]): CellRef[] => [...new Set(offsets)].map((offset) => ({ node: table.nodeAt(offset)!, pos: tableStart + offset })).filter((cell) => cell.node !== null);
  return {
    cells: at(map.cellsInRect(rect)),
    rows: at(map.cellsInRect({ left: 0, right: map.width, top: rect.top, bottom: rect.bottom })),
    columns: at(map.cellsInRect({ left: rect.left, right: rect.right, top: 0, bottom: map.height })),
  };
}

function stylesOf(node: PMNode): Record<string, string> {
  const own = node.attrs['emailStyles'] as Record<string, string> | null;
  return own === null ? defaultCellStyles() : { ...own };
}

function patchCells(cells: CellRef[], patch: Record<string, string | null>): Command {
  return (state, dispatch) => {
    if (cells.length === 0) return false;
    for (const [property, value] of Object.entries(patch)) if (value !== null && !safeEmailStyle(property, value)) return false;
    if (dispatch) {
      const tr = state.tr;
      for (const { node, pos } of cells) {
        const styles = stylesOf(node);
        for (const [property, value] of Object.entries(patch)) {
          if (value === null) delete styles[property];
          else styles[property] = value;
        }
        tr.setNodeMarkup(pos, undefined, { ...node.attrs, emailStyles: styles });
      }
      dispatch(tr);
    }
    return true;
  };
}

const cellsCommand = (pick: (r: NonNullable<ReturnType<typeof rectOf>>) => CellRef[], patch: Record<string, string | null>): Command =>
  (state, dispatch) => {
    const found = rectOf(state);
    return found ? patchCells(pick(found), patch)(state, dispatch) : false;
  };

/** Padding of the selected cells: a vertical and a horizontal size in px. */
export const setCellPadding = (vertical: number, horizontal: number): Command =>
  cellsCommand((r) => r.cells, { 'padding-top': `${vertical}px`, 'padding-bottom': `${vertical}px`, 'padding-left': `${horizontal}px`, 'padding-right': `${horizontal}px` });

export const setCellPaddingPreset = (preset: PaddingPreset): Command => setCellPadding(PADDING_PRESETS[preset].vertical, PADDING_PRESETS[preset].horizontal);

/** Height of every row the selection touches (written on each cell of the row); null clears it. */
export const setRowHeight = (px: number | null): Command => cellsCommand((r) => r.rows, { height: px === null ? null : `${px}px` });

/** Width of every column the selection touches (written on each cell of the column); null clears it. */
export const setColumnWidth = (px: number | null): Command => cellsCommand((r) => r.columns, { width: px === null ? null : `${px}px` });

export const setCellAlign = (align: CellAlign): Command => cellsCommand((r) => r.cells, { 'text-align': align });
export const setCellVerticalAlign = (align: CellVertical): Command => cellsCommand((r) => r.cells, { 'vertical-align': align });

/** What the controls show: the first selected cell's effective values. */
export interface CellLook { paddingVertical: number | null; paddingHorizontal: number | null; rowHeight: number | null; columnWidth: number | null; align: CellAlign | null; vertical: CellVertical | null; header: boolean; rows: number; columns: number }

const px = (value: string | undefined): number | null => {
  const n = value === undefined ? NaN : Number.parseInt(value, 10);
  return Number.isNaN(n) ? null : n;
};

export function cellLook(state: EditorState): CellLook | null {
  const found = rectOf(state);
  const first = found?.cells[0];
  if (!found || !first) return null;
  const styles = stylesOf(first.node);
  const rect = selectedRect(state);
  const align = styles['text-align'];
  const vertical = styles['vertical-align'];
  return {
    paddingVertical: px(styles['padding-top']),
    paddingHorizontal: px(styles['padding-left']),
    rowHeight: px(styles['height']),
    columnWidth: px(styles['width']),
    align: align === 'left' || align === 'center' || align === 'right' ? align : null,
    vertical: vertical === 'top' || vertical === 'middle' || vertical === 'bottom' ? vertical : null,
    header: first.node.type.name === 'table_header',
    rows: rect.map.height,
    columns: TableMap.get(rect.table).width,
  };
}
