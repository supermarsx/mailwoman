// The Format tab's commands for the ProseMirror editor (FORMAT): font size steps,
// list types and numbering, indent, clear formatting, direction and the format
// painter. Each is a plain ProseMirror command, so the ribbon, the keymap and the
// tests all run the same code.

import { Selection, TextSelection, type Command, type EditorState, type Transaction } from 'prosemirror-state';
import { liftListItem, sinkListItem, wrapInList } from 'prosemirror-schema-list';
import { Fragment, type Mark, type Node as PMNode, type NodeType, type ResolvedPos } from 'prosemirror-model';
import type { EffectKind } from './effects.ts';
import { sortItems, type SortOptions } from './sortModel.ts';
import {
  DEFAULT_LIST_STYLE,
  INDENT_MAX,
  listStyleDef,
  nestedBulletStyle,
  nestedOrderedStyle,
  outlineStyle,
  pointsOf,
  pointText,
  stepFontSize,
  type ClearVariant,
  type ListKind,
  type PaintSummary,
} from './formatModel.ts';

const FORMAT_MARKS = ['strong', 'em', 'code', 'underline', 'strikethrough', 'textColor', 'highlight', 'fontFamily', 'fontSize'] as const;
const BLOCK_ATTRS = ['align', 'lineHeight', 'before', 'after', 'indent', 'shade', 'dir'] as const;
const TEXT_BLOCKS = ['paragraph', 'heading'];

/** Run `command` and hand back the transaction it would dispatch (null when it does not apply). */
function capture(state: EditorState, command: Command): Transaction | null {
  let found: Transaction | null = null;
  const ok = command(state, (tr) => { found = tr; });
  return ok ? found : null;
}

// ---------------------------------------------------------------------------
// Font size
// ---------------------------------------------------------------------------

/** The size, in pt, of the text at the selection's start, or null when the text has none of its own. */
export function currentFontSize(state: EditorState): number | null {
  const type = state.schema.marks['fontSize'];
  if (!type) return null;
  const { $from, from, to, empty } = state.selection;
  let marks: readonly Mark[] | null = empty ? (state.storedMarks ?? $from.marks()) : null;
  if (!marks) {
    state.doc.nodesBetween(from, to, (node) => {
      if (marks) return false;
      if (node.isText) marks = node.marks;
      return marks === null;
    });
  }
  const value = (marks ?? []).find((mark) => mark.type === type)?.attrs['value'];
  return typeof value === 'string' ? pointsOf(value) : null;
}

/** Grow (`1`) or shrink (`-1`) the selected text one step down the ramp; `fine` steps by 1 pt. */
export function stepFontSizeCommand(direction: 1 | -1, fine = false): Command {
  return (state, dispatch) => {
    const type = state.schema.marks['fontSize'];
    if (!type) return false;
    const { from, to, empty } = state.selection;
    const tr = state.tr;
    if (empty) {
      const next = stepFontSize(currentFontSize(state), direction, fine);
      tr.removeStoredMark(type).addStoredMark(type.create({ value: pointText(next) }));
    } else {
      state.doc.nodesBetween(from, to, (node, pos) => {
        if (!node.isText) return true;
        const start = Math.max(pos, from);
        const end = Math.min(pos + node.nodeSize, to);
        const own = node.marks.find((mark) => mark.type === type)?.attrs['value'];
        const next = stepFontSize(typeof own === 'string' ? pointsOf(own) : null, direction, fine);
        tr.removeMark(start, end, type).addMark(start, end, type.create({ value: pointText(next) }));
        return false;
      });
      if (!tr.docChanged) return false;
    }
    dispatch?.(tr);
    return true;
  };
}

// ---------------------------------------------------------------------------
// Lists
// ---------------------------------------------------------------------------

interface ListContext { depth: number; pos: number; node: PMNode }

/** The list the selection's start is in (innermost), with its depth and position. */
/** Where the selection starts, inside text: a select-all (or a node selection) starts at its first text position. */
export function anchorPos(state: Pick<EditorState, 'selection'>): ResolvedPos {
  const { $from } = state.selection;
  return $from.parent.inlineContent ? $from : Selection.near($from, 1).$from;
}

export function enclosingList(state: Pick<EditorState, 'selection'>, from = anchorPos(state)): ListContext | null {
  for (let depth = from.depth; depth > 0; depth -= 1) {
    const node = from.node(depth);
    if (node.type.name === 'bullet_list' || node.type.name === 'ordered_list') return { depth, pos: from.before(depth), node };
  }
  return null;
}

/** `listStyle` as stored: the plain default is `null`, so an untouched list stays a bare `<ul>` or `<ol>`. */
const storedStyle = (kind: ListKind, id: string): string | null => (id === DEFAULT_LIST_STYLE[kind] ? null : id);

/** The list type the selection is in: its kind and style id (the default when it has none). */
export function currentListType(state: EditorState): { kind: ListKind; style: string } | null {
  const list = enclosingList(state);
  if (!list) return null;
  const kind: ListKind = list.node.type.name === 'ordered_list' ? 'ordered' : 'bullet';
  return { kind, style: (list.node.attrs['listStyle'] as string | null) ?? DEFAULT_LIST_STYLE[kind] };
}

/** Keep each item's checkbox in step with its list: a checklist's items have one, others do not. */
function normalizeChecks(tr: Transaction, from: number, to: number): void {
  const edits: { pos: number; attrs: Record<string, unknown> }[] = [];
  tr.doc.nodesBetween(from, to, (node, pos) => {
    if (node.type.name !== 'bullet_list' && node.type.name !== 'ordered_list') return true;
    const check = node.attrs['listStyle'] === 'check';
    node.forEach((item, offset) => {
      const wanted = check ? (typeof item.attrs['checked'] === 'boolean' ? item.attrs['checked'] : false) : null;
      if (item.attrs['checked'] !== wanted) edits.push({ pos: pos + 1 + offset, attrs: { ...item.attrs, checked: wanted } });
    });
    return true;
  });
  for (const edit of edits) tr.setNodeMarkup(edit.pos, undefined, edit.attrs);
}

/** Lift the selected items out of every list around them, leaving paragraphs. */
function liftOutOfLists(state: EditorState): Transaction | null {
  const item = state.schema.nodes['list_item']!;
  const out = state.tr;
  let current = state;
  for (let guard = 0; guard < 12 && enclosingList(current); guard += 1) {
    const step = capture(current, liftListItem(item));
    if (!step) break;
    for (const s of step.steps) out.step(s);
    current = current.apply(step);
  }
  return out.docChanged ? out : null;
}

/** The outermost list around `from`. */
function rootList(doc: Pick<EditorState, 'selection'>, from: ResolvedPos): ListContext | null {
  let found: ListContext | null = null;
  for (let depth = from.depth; depth > 0; depth -= 1) {
    const node = from.node(depth);
    if (node.type.name === 'bullet_list' || node.type.name === 'ordered_list') found = { depth, pos: from.before(depth), node };
  }
  void doc;
  return found;
}

/** Give a numbered list tree the outline look: 1. at the top, a. under it, i. under that, around. */
function applyOutline(tr: Transaction): void {
  const root = rootList({ selection: tr.selection }, tr.selection.$from);
  if (!root) return;
  const edits: { pos: number; attrs: Record<string, unknown> }[] = [];
  const walk = (node: PMNode, pos: number, depth: number): void => {
    node.forEach((item, offset) => {
      item.forEach((child, childOffset) => {
        if (child.type.name === 'ordered_list' || child.type.name === 'bullet_list') {
          const childPos = pos + 1 + offset + 1 + childOffset;
          if (child.type.name === 'ordered_list') edits.push({ pos: childPos, attrs: { ...child.attrs, listStyle: storedStyle('ordered', outlineStyle(depth + 1)) } });
          walk(child, childPos, depth + 1);
        }
      });
    });
  };
  edits.push({ pos: root.pos, attrs: { ...root.node.attrs, listStyle: storedStyle('ordered', outlineStyle(0)) } });
  walk(root.node, root.pos, 0);
  for (const edit of edits) tr.setNodeMarkup(edit.pos, undefined, edit.attrs);
}

/**
 * Make the selected paragraphs a list of `style` (a bullet or numbered type). In a
 * list of that type already, the same style turns the list off and another style
 * changes it; in a list of the other kind the list changes kind.
 */
export function applyListStyle(kind: ListKind, styleId: string): Command {
  return (state, dispatch) => {
    const types = state.schema.nodes;
    const target = kind === 'bullet' ? types['bullet_list'] : types['ordered_list'];
    const def = listStyleDef(styleId);
    if (!target || !def || def.kind !== kind) return false;
    const listStyle = styleId === 'outline' ? null : storedStyle(kind, styleId);
    const list = enclosingList(state);
    const same = list !== null && list.node.type === target && ((list.node.attrs['listStyle'] as string | null) ?? DEFAULT_LIST_STYLE[kind]) === styleId;
    let tr: Transaction | null;
    if (same) {
      tr = liftOutOfLists(state);
      if (!tr) return false;
      dispatch?.(tr.scrollIntoView());
      return true;
    }
    if (list) {
      tr = state.tr;
      tr.setNodeMarkup(list.pos, target, { ...list.node.attrs, listStyle });
    } else {
      tr = capture(state, wrapInList(target, { listStyle }));
    }
    if (!tr) return false;
    const { from, to } = tr.selection;
    normalizeChecks(tr, Math.min(from, to), Math.max(from, to));
    if (styleId === 'outline') applyOutline(tr);
    dispatch?.(tr.scrollIntoView());
    return true;
  };
}

/** Number the selected item's list from `start`, from the list before it ('continue') or from 1 ('restart'). */
export function setListStart(start: number | 'restart' | 'continue'): Command {
  return (state, dispatch) => {
    const list = enclosingList(state);
    if (!list || list.node.type.name !== 'ordered_list') return false;
    const $from = anchorPos(state);
    const index = $from.index(list.depth);
    const tr = state.tr;
    let order: number;
    if (start === 'continue') {
      const parent = $from.node(list.depth - 1);
      const before = parent.maybeChild($from.index(list.depth - 1) - 1);
      if (!before || before.type !== list.node.type) return false;
      order = (Number(before.attrs['order']) || 1) + before.childCount;
    } else {
      order = start === 'restart' ? 1 : Math.max(1, Math.min(9999, Math.floor(start)));
    }
    if (!Number.isFinite(order)) return false;
    if (index > 0 && start !== 'continue') {
      // Numbering restarts at this item: the list splits before it.
      const at = $from.posAtIndex(index, list.depth);
      tr.split(at, 1);
      tr.setNodeMarkup(at + 1, undefined, { ...list.node.attrs, order });
    } else if (list.node.attrs['order'] === order) {
      return false;
    } else {
      tr.setNodeMarkup(list.pos, undefined, { ...list.node.attrs, order });
    }
    dispatch?.(tr);
    return true;
  };
}

/** True when the selection's item can number from the previous list ('Continue numbering'). */
export function canContinueNumbering(state: EditorState): boolean {
  return setListStart('continue')(state);
}

/** Toggle the checkbox of the item the selection is in. */
export function toggleChecked(at?: number): Command {
  return (state, dispatch) => {
    const $pos = at === undefined ? anchorPos(state) : state.doc.resolve(at);
    for (let depth = $pos.depth; depth > 0; depth -= 1) {
      const node = $pos.node(depth);
      if (node.type.name === 'list_item' && typeof node.attrs['checked'] === 'boolean') {
        dispatch?.(state.tr.setNodeMarkup($pos.before(depth), undefined, { ...node.attrs, checked: !node.attrs['checked'] }));
        return true;
      }
    }
    return false;
  };
}

/** Sink the selected item one level, giving a new nested list the look its level calls for. */
export const sinkItem: Command = (state, dispatch) => {
  const item = state.schema.nodes['list_item'];
  if (!item) return false;
  const tr = capture(state, sinkListItem(item));
  if (!tr) return false;
  const inner = enclosingList(tr, tr.selection.$from);
  const outer = inner ? enclosingList(tr, tr.doc.resolve(inner.pos)) : null;
  if (inner && outer && inner.node.type === outer.node.type && inner.node.attrs['listStyle'] === null) {
    const ordered = inner.node.type.name === 'ordered_list';
    const outerStyle = outer.node.attrs['listStyle'] as string | null;
    const def = listStyleDef(outerStyle);
    let next: string | null = null;
    if (def?.marker) next = outerStyle;
    else if (ordered) next = nestedOrderedStyle(outerStyle);
    else if (outerStyle !== null) next = nestedBulletStyle(outerStyle);
    if (next !== null) {
      tr.setNodeMarkup(inner.pos, undefined, { ...inner.node.attrs, listStyle: next === 'decimal' ? null : next });
      const { from, to } = tr.selection;
      normalizeChecks(tr, Math.min(from, to), Math.max(from, to));
    }
  }
  dispatch?.(tr.scrollIntoView());
  return true;
};

/** Lift the selected item one level (out of the list from the top level). */
export const liftItem: Command = (state, dispatch) => {
  const item = state.schema.nodes['list_item'];
  return item ? liftListItem(item)(state, dispatch) : false;
};

// ---------------------------------------------------------------------------
// Block attributes, indent, direction, spacing
// ---------------------------------------------------------------------------

/** Set paragraph attributes on the selected text blocks, dropping the imported inline style they replace. */
export function setBlockAttrs(attrs: Record<string, unknown>, prune: readonly string[] = []): Command {
  return (state, dispatch) => {
    const tr = state.tr;
    let changed = false;
    state.doc.nodesBetween(state.selection.from, state.selection.to, (node, pos) => {
      if (!TEXT_BLOCKS.includes(node.type.name)) return true;
      let email = node.attrs['emailStyles'] as Record<string, string> | null;
      if (email && prune.some((property) => property in email!)) {
        email = Object.fromEntries(Object.entries(email).filter(([property]) => !prune.includes(property)));
      }
      const next: Record<string, unknown> = { ...node.attrs, ...attrs, emailStyles: email && Object.keys(email).length === 0 ? null : email };
      if (Object.keys(next).some((key) => next[key] !== node.attrs[key])) {
        tr.setNodeMarkup(pos, undefined, next);
        changed = true;
      }
      return false;
    });
    if (changed) dispatch?.(tr);
    return changed;
  };
}

/** The first text block's paragraph attributes at the selection. */
export function blockAttrs(state: EditorState): Record<string, unknown> {
  const parent = anchorPos(state).parent;
  return TEXT_BLOCKS.includes(parent.type.name) ? parent.attrs : {};
}

export function setLineSpacing(value: string | null): Command {
  return setBlockAttrs({ lineHeight: value }, ['line-height']);
}
export function setSpacing(side: 'before' | 'after', points: number | null): Command {
  return setBlockAttrs({ [side]: points }, [side === 'before' ? 'margin-top' : 'margin-bottom']);
}
export function setAlignment(value: string | null): Command {
  return setBlockAttrs({ align: value }, ['text-align']);
}
export function setDirection(value: 'ltr' | 'rtl'): Command {
  return setBlockAttrs({ dir: value }, ['direction']);
}

/** Increase (`1`) or decrease (`-1`) the indent: a list item changes level, a paragraph moves 40 px. */
export function changeIndent(direction: 1 | -1): Command {
  return (state, dispatch) => {
    if (enclosingList(state)) return direction > 0 ? sinkItem(state, dispatch) : liftItem(state, dispatch);
    const tr = state.tr;
    let changed = false;
    state.doc.nodesBetween(state.selection.from, state.selection.to, (node, pos) => {
      if (!TEXT_BLOCKS.includes(node.type.name)) return true;
      const level = Number(node.attrs['indent']) || 0;
      const next = Math.max(0, Math.min(INDENT_MAX, level + direction));
      const email = node.attrs['emailStyles'] as Record<string, string> | null;
      const carried = email !== null && 'margin-left' in email;
      if (next === level && !(carried && direction < 0)) return false;
      const pruned = email ? Object.fromEntries(Object.entries(email).filter(([property]) => property !== 'margin-left')) : null;
      tr.setNodeMarkup(pos, undefined, { ...node.attrs, indent: next, emailStyles: pruned && Object.keys(pruned).length > 0 ? pruned : null });
      changed = true;
      return false;
    });
    if (changed) dispatch?.(tr);
    return changed;
  };
}

/** True when 'Decrease indent' would do something here. */
export function canOutdent(state: EditorState): boolean {
  return changeIndent(-1)(state);
}

// ---------------------------------------------------------------------------
// Clear formatting
// ---------------------------------------------------------------------------

/** Reset the selection: text formatting, paragraph formatting (lists, headings, spacing), or both; links only for 'everything'. */
export function clearFormatting(variant: ClearVariant): Command {
  return (state, dispatch) => {
    const text = variant !== 'paragraph';
    const paragraph = variant !== 'text';
    const links = variant === 'everything';
    let tr: Transaction = state.tr;
    if (paragraph) {
      const lifted = liftOutOfLists(state);
      if (lifted) tr = lifted;
    }
    const from = tr.mapping.map(state.selection.from);
    const to = tr.mapping.map(state.selection.to);
    if (text) {
      const names: string[] = [...FORMAT_MARKS, ...(links ? ['link'] : [])];
      if (state.selection.empty) {
        const keep = (state.storedMarks ?? state.selection.$from.marks()).filter((mark) => !names.includes(mark.type.name));
        tr.setStoredMarks(keep);
      } else {
        for (const name of names) {
          const type = state.schema.marks[name];
          if (type) tr.removeMark(from, to, type);
        }
      }
    }
    if (paragraph) {
      const plain = state.schema.nodes['paragraph']!;
      const reset = new Map<number, PMNode>();
      tr.doc.nodesBetween(from, to, (node, pos) => {
        if (['paragraph', 'heading', 'code_block'].includes(node.type.name)) {
          const blank = plain.create(null, node.content, node.marks);
          if (!blank.eq(node) || node.type !== plain) reset.set(pos, blank);
          return false;
        }
        return true;
      });
      for (const [pos, blank] of [...reset].sort((a, b) => b[0] - a[0])) tr.replaceWith(pos, pos + tr.doc.nodeAt(pos)!.nodeSize, blank);
    }
    if (!tr.docChanged && !tr.storedMarksSet) return false;
    if (tr.docChanged && !tr.selectionSet) {
      try { tr.setSelection(TextSelection.create(tr.doc, Math.min(tr.mapping.map(state.selection.from), tr.doc.content.size), Math.min(tr.mapping.map(state.selection.to), tr.doc.content.size))); } catch { /* keep the mapped selection */ }
    }
    dispatch?.(tr);
    return true;
  };
}

// ---------------------------------------------------------------------------
// Format painter
// ---------------------------------------------------------------------------

export interface PaintedFormat {
  marks: readonly Mark[];
  block: Record<string, unknown>;
  summary: PaintSummary;
}

/** What the painter takes from the selection: its text marks (not the link) and its paragraph's look. */
export function capturePaint(state: EditorState): PaintedFormat {
  const { $from, from, to, empty } = state.selection;
  let source: readonly Mark[] | null = state.storedMarks ?? (empty ? $from.marks() : null);
  if (!source) {
    state.doc.nodesBetween(from, to, (node) => {
      if (source) return false;
      if (node.isText) source = node.marks;
      return source === null;
    });
  }
  const marks = (source ?? []).filter((mark) => (FORMAT_MARKS as readonly string[]).includes(mark.type.name));
  const attrs = blockAttrs(state);
  const block: Record<string, unknown> = {};
  for (const name of BLOCK_ATTRS) if (attrs[name] !== undefined && attrs[name] !== null && attrs[name] !== 0) block[name] = attrs[name];
  const value = (name: string): string | null => {
    const found = marks.find((mark) => mark.type.name === name)?.attrs['value'];
    return typeof found === 'string' ? found : null;
  };
  const has = (name: string): boolean => marks.some((mark) => mark.type.name === name);
  return {
    marks,
    block,
    summary: { family: value('fontFamily'), size: pointsOf(value('fontSize')), bold: has('strong'), italic: has('em'), underline: has('underline'), strike: has('strikethrough'), color: value('textColor'), highlight: value('highlight') },
  };
}

/** Put a painted look on the selection (on the caret, it becomes what is typed next). */
export function applyPaint(paint: PaintedFormat): Command {
  return (state, dispatch) => {
    const { from, to, empty } = state.selection;
    const tr = state.tr;
    if (!empty) {
      for (const name of FORMAT_MARKS) {
        const type = state.schema.marks[name];
        if (type) tr.removeMark(from, to, type);
      }
      for (const mark of paint.marks) tr.addMark(from, to, mark);
    }
    state.doc.nodesBetween(from, to, (node, pos) => {
      if (!TEXT_BLOCKS.includes(node.type.name)) return true;
      const next = { ...node.attrs };
      for (const name of BLOCK_ATTRS) next[name] = paint.block[name] ?? (name === 'indent' ? 0 : null);
      if (Object.keys(next).some((key) => next[key] !== node.attrs[key])) tr.setNodeMarkup(pos, undefined, next);
      return false;
    });
    if (empty) {
      // After the steps: a step clears pending marks.
      const keep = (state.storedMarks ?? state.selection.$from.marks()).filter((mark) => !(FORMAT_MARKS as readonly string[]).includes(mark.type.name));
      tr.setStoredMarks([...keep, ...paint.marks]);
    }
    dispatch?.(tr);
    return true;
  };
}

// ---------------------------------------------------------------------------
// Font effects
// ---------------------------------------------------------------------------

/** Whether every text run in the selection (or the pending marks at a caret) carries effect `kind`. */
export function effectActive(state: EditorState, kind: EffectKind): boolean {
  const type = state.schema.marks['effect'];
  if (!type) return false;
  const has = (marks: readonly Mark[]): boolean => marks.some((mark) => mark.type === type && mark.attrs['kind'] === kind);
  const { from, to, empty, $from } = state.selection;
  if (empty) return has(state.storedMarks ?? $from.marks());
  let any = false;
  let all = true;
  state.doc.nodesBetween(from, to, (node) => {
    if (!node.isText) return true;
    any = true;
    if (!has(node.marks)) all = false;
    return false;
  });
  return any && all;
}

/** Switch effect `kind` on (with `value`) or, when it is already on, off. Super and subscript replace each other. */
export function toggleEffect(kind: EffectKind, value = ''): Command {
  return (state, dispatch) => {
    const type = state.schema.marks['effect'];
    if (!type) return false;
    const turnOff = effectActive(state, kind) && (value === '' || currentEffectValue(state, kind) === value);
    const opposite = kind === 'superscript' ? 'subscript' : kind === 'subscript' ? 'superscript' : null;
    const isKind = (mark: Mark, name: string): boolean => mark.type === type && mark.attrs['kind'] === name;
    const { from, to, empty } = state.selection;
    const tr = state.tr;
    if (empty) {
      const base = (state.storedMarks ?? state.selection.$from.marks()).filter((mark) => !isKind(mark, kind) && !(opposite && isKind(mark, opposite)));
      tr.setStoredMarks(turnOff ? base : [...base, type.create({ kind, value })]);
    } else {
      state.doc.nodesBetween(from, to, (node, pos) => {
        if (!node.isText) return true;
        const start = Math.max(pos, from);
        const end = Math.min(pos + node.nodeSize, to);
        for (const mark of node.marks) if (isKind(mark, kind) || (opposite && isKind(mark, opposite))) tr.removeMark(start, end, mark);
        return false;
      });
      if (!turnOff) tr.addMark(from, to, type.create({ kind, value }));
    }
    dispatch?.(tr);
    return true;
  };
}

/** The value effect `kind` has at the selection's start ('' when off or valueless). */
export function currentEffectValue(state: EditorState, kind: EffectKind): string {
  const type = state.schema.marks['effect'];
  const { $from, from, empty } = state.selection;
  const marks = empty ? (state.storedMarks ?? $from.marks()) : (state.doc.nodeAt(from)?.marks ?? []);
  const found = marks.find((mark) => mark.type === type && mark.attrs['kind'] === kind);
  return typeof found?.attrs['value'] === 'string' ? found.attrs['value'] : '';
}

// ---------------------------------------------------------------------------
// Sort
// ---------------------------------------------------------------------------

export interface SortOutcome { kind: 'rows' | 'items' | 'blocks'; count: number; removed: number }

/** What a sort would act on: table rows, list items or sibling blocks, with the item text for each. */
function sortTarget(state: EditorState, options: SortOptions): { from: number; to: number; nodes: PMNode[]; textOf: (node: PMNode) => string; kind: SortOutcome['kind']; keep: PMNode[] } | null {
  const { $from, $to } = state.selection;
  const anchor = anchorPos(state);
  // A table: the rows the selection touches (all rows when it is inside one).
  for (let depth = anchor.depth; depth > 0; depth -= 1) {
    if (anchor.node(depth).type.spec.tableRole !== 'table') continue;
    const table = anchor.node(depth);
    const start = anchor.start(depth);
    const caretRow = anchor.index(depth);
    const cellDepth = depth + 2;
    const column = options.column ?? (anchor.depth >= cellDepth ? anchor.index(depth + 1) : 0);
    const rows: PMNode[] = [];
    table.forEach((row) => rows.push(row));
    const headerRows = options.header && rows.length > 1 ? rows.slice(0, 1) : [];
    const body = rows.slice(headerRows.length);
    void caretRow;
    const textOf = (row: PMNode): string => row.maybeChild(column)?.textContent ?? '';
    const offset = headerRows.reduce((sum, row) => sum + row.nodeSize, 0);
    return { from: start + offset, to: start + table.content.size, nodes: body, textOf, kind: 'rows', keep: headerRows };
  }
  const list = enclosingList(state, anchor);
  if (list) {
    const items: PMNode[] = [];
    list.node.forEach((item) => items.push(item));
    return { from: list.pos + 1, to: list.pos + list.node.nodeSize - 1, nodes: items, textOf: (item) => item.firstChild?.textContent ?? item.textContent, kind: 'items', keep: [] };
  }
  const range = $from.blockRange($to);
  if (!range) return null;
  const nodes: PMNode[] = [];
  for (let i = range.startIndex; i < range.endIndex; i += 1) nodes.push(range.parent.child(i));
  if (nodes.length < 2 || !nodes.every((node) => node.isTextblock)) return null;
  return { from: range.start, to: range.end, nodes, textOf: (node) => node.textContent, kind: 'blocks', keep: [] };
}

/** Sort, reverse, shuffle or de-duplicate the selected blocks, list items or table rows, as one undoable step. */
export function sortSelection(options: SortOptions, onResult?: (outcome: SortOutcome) => void, random?: () => number): Command {
  return (state, dispatch) => {
    const target = sortTarget(state, options);
    if (!target || target.nodes.length < 2) return false;
    const result = sortItems(target.nodes, target.textOf, options, random);
    if (!result.changed) { onResult?.({ kind: target.kind, count: result.items.length, removed: 0 }); return false; }
    if (dispatch) {
      const tr = state.tr;
      tr.replaceWith(target.from, target.to, Fragment.from(result.items));
      dispatch(tr);
      onResult?.({ kind: target.kind, count: result.items.length, removed: result.removed });
    }
    return true;
  };
}

export type { NodeType };
