// The Format tab's commands for the native HTML editor (FORMAT). They work on the
// editable DOM directly (no execCommand), so the same code runs in the browser and
// in jsdom, and the markup they leave is plain: inline styles and the legacy
// `type`/`start`/`dir` attributes mail clients draw. Editor-only `data-mw-*`
// attributes (marker lists, checkboxes) are turned into marker text on send by
// `materializeLists`.

import type { NativeHtmlCommands } from './nativeHtmlCommands.ts';
import {
  DEFAULT_LIST_STYLE,
  INDENT_MAX,
  INDENT_STEP,
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
import { CHECKED_ATTRIBUTE, LIST_ATTRIBUTE } from './listMarkers.ts';
import { effectDef, effectStyle, effectTag, type EffectKind } from './effects.ts';
import { sortItems, type SortOptions } from './sortModel.ts';

const LEAF = 'p,div,li,blockquote,pre,h1,h2,h3,h4,h5,h6,td,th';
const LISTS = 'ul,ol';
const INLINE_FORMAT = new Set(['B', 'STRONG', 'I', 'EM', 'U', 'S', 'STRIKE', 'DEL', 'INS', 'FONT', 'SUB', 'SUP', 'MARK', 'CODE', 'TT', 'BIG', 'SMALL', 'CITE', 'SPAN']);
const TEXT_PROPERTIES = ['font-family', 'font-size', 'font-weight', 'font-style', 'font-variant', 'color', 'background', 'background-color', 'text-decoration', 'text-decoration-line', 'text-decoration-style', 'text-transform', 'letter-spacing', 'text-shadow', 'vertical-align', 'line-height'];

const isList = (node: Node | null): node is HTMLElement => node?.nodeType === 1 && (node as Element).matches(LISTS);

export interface NativePaint {
  inline: Record<string, string>;
  block: Record<string, string>;
}

export function createNativeFormat(commands: NativeHtmlCommands) {
  const host = (): HTMLElement | undefined => commands.host();
  const doc = (): Document | undefined => host()?.ownerDocument;

  /** Leaf blocks the selection touches; bare inline content directly in the editor is wrapped in a paragraph first. */
  function leafBlocks(range: Range): HTMLElement[] {
    const root = host();
    if (!root) return [];
    const d = root.ownerDocument;
    // Wrap runs of inline content that sit directly in the editor.
    const touched = Array.from(root.childNodes).filter((node) => range.intersectsNode(node) || (range.collapsed && node.contains(range.startContainer)));
    const runs: Node[][] = [];
    for (const node of touched) {
      const inline = node.nodeType === 3 || (node.nodeType === 1 && !(node as Element).matches(`${LEAF},${LISTS},table,hr`));
      if (!inline) continue;
      const last = runs[runs.length - 1];
      if (last && last[last.length - 1]!.nextSibling === node) last.push(node);
      else runs.push([node]);
    }
    for (const run of runs) {
      if (run.length === 1 && run[0]!.nodeType === 3 && !(run[0]!.textContent ?? '').trim()) continue;
      const p = d.createElement('p');
      run[0]!.parentNode!.insertBefore(p, run[0]!);
      for (const node of run) p.appendChild(node);
      if (range.startContainer === root) { const next = d.createRange(); next.selectNodeContents(p); next.collapse(true); commands.setRange(next); }
    }
    const live = commands.restore() ?? range;
    const all = Array.from(root.querySelectorAll<HTMLElement>(LEAF)).filter((block) => live.intersectsNode(block) && !Array.from(block.querySelectorAll(LEAF)).some((child) => live.intersectsNode(child)));
    if (live.collapsed || all.length === 0) {
      const active = (commands.current()?.closest(LEAF) ?? null) as HTMLElement | null;
      if (active && active !== root && root.contains(active)) return [active];
    }
    return all;
  }

  const selectedBlocks = (): HTMLElement[] => {
    const range = commands.restore();
    return range ? leafBlocks(range) : [];
  };

  /** Select from the start of `first` to the end of `last` (the DOM moved under the old selection). */
  function select(first: Node | undefined, last: Node | undefined = first): void {
    const d = doc();
    if (!d || !first || !last || !first.isConnected || !last.isConnected) return;
    const next = d.createRange();
    next.setStart(first, 0);
    next.setEnd(last, last.nodeType === 3 ? (last.textContent ?? '').length : last.childNodes.length);
    commands.setRange(next);
  }

  // ---- font size ---------------------------------------------------------
  function sizeOf(node: Node): number | null {
    const el = node.nodeType === 1 ? (node as Element) : node.parentElement;
    const view = el?.ownerDocument.defaultView;
    if (!el || !view) return null;
    return pointsOf(view.getComputedStyle(el).fontSize);
  }

  function stepFont(direction: 1 | -1, fine = false): boolean {
    const range = commands.restore();
    const root = host();
    if (!range || !root) return false;
    if (range.collapsed) {
      const next = stepFontSize(sizeOf(range.startContainer), direction, fine);
      commands.inlineStyle('font-size', pointText(next));
      return true;
    }
    const nodes = commands.selectedText();
    if (nodes.length === 0) return false;
    const plan = nodes.map(({ node, from, to }) => ({ node, from, to, size: stepFontSize(sizeOf(node), direction, fine) }));
    const spans: HTMLElement[] = [];
    for (const { node, from, to, size } of plan.reverse()) {
      const piece = root.ownerDocument.createRange();
      piece.setStart(node, from);
      piece.setEnd(node, to);
      const span = root.ownerDocument.createElement('span');
      span.style.setProperty('font-size', pointText(size));
      piece.surroundContents(span);
      spans.unshift(span);
    }
    const next = root.ownerDocument.createRange();
    next.setStart(spans[0]!, 0);
    next.setEnd(spans[spans.length - 1]!, spans[spans.length - 1]!.childNodes.length);
    commands.setRange(next);
    commands.commit();
    return true;
  }

  // ---- lists -------------------------------------------------------------
  function styleList(list: HTMLElement, styleId: string): void {
    const def = listStyleDef(styleId);
    list.removeAttribute(LIST_ATTRIBUTE);
    list.removeAttribute('type');
    list.style.removeProperty('list-style-type');
    if (list.getAttribute('style') === '') list.removeAttribute('style');
    if (!def) return;
    const tag = list.tagName === 'OL' ? 'ordered' : 'bullet';
    if (def.marker) list.setAttribute(LIST_ATTRIBUTE, def.id);
    else if (styleId !== 'outline' && styleId !== DEFAULT_LIST_STYLE[tag]) {
      list.style.setProperty('list-style-type', def.css!);
      if (tag === 'ordered' && def.type) list.setAttribute('type', def.type);
    }
    for (const item of Array.from(list.children)) {
      if (item.tagName !== 'LI') continue;
      if (def.marker === 'check') { if (!item.hasAttribute(CHECKED_ATTRIBUTE)) item.setAttribute(CHECKED_ATTRIBUTE, 'false'); }
      else item.removeAttribute(CHECKED_ATTRIBUTE);
    }
  }

  /** The list style id a list carries. */
  function styleOf(list: HTMLElement): string {
    const marker = list.getAttribute(LIST_ATTRIBUTE);
    if (marker) return marker;
    const css = list.style.listStyleType;
    if (css) return css;
    const kind: ListKind = list.tagName === 'OL' ? 'ordered' : 'bullet';
    return DEFAULT_LIST_STYLE[kind];
  }

  function retag(list: HTMLElement, tag: 'ul' | 'ol'): HTMLElement {
    if (list.tagName.toLowerCase() === tag) return list;
    const next = list.ownerDocument.createElement(tag);
    while (list.firstChild) next.appendChild(list.firstChild);
    list.replaceWith(next);
    return next;
  }

  function listType(): { kind: ListKind; style: string } | null {
    const li = commands.current()?.closest('li');
    const list = li?.parentElement;
    if (!li || !list || !isList(list) || !host()?.contains(list)) return null;
    return { kind: list.tagName === 'OL' ? 'ordered' : 'bullet', style: styleOf(list) };
  }

  function outdentItem(li: HTMLElement): HTMLElement {
    const list = li.parentElement as HTMLElement;
    const d = li.ownerDocument;
    const outer = list.parentElement;
    const following = Array.from(li.parentElement!.children).slice(Array.from(list.children).indexOf(li) + 1);
    if (outer?.tagName === 'LI') {
      if (following.length) {
        const nested = list.cloneNode(false) as HTMLElement;
        for (const sibling of following) nested.appendChild(sibling);
        li.appendChild(nested);
      }
      outer.parentElement!.insertBefore(li, outer.nextSibling);
    } else {
      if (following.length) {
        const rest = list.cloneNode(false) as HTMLElement;
        for (const sibling of following) rest.appendChild(sibling);
        list.after(rest);
      }
      const p = d.createElement('p');
      const nestedLists: HTMLElement[] = [];
      for (const child of Array.from(li.childNodes)) {
        if (isList(child)) nestedLists.push(child);
        else if (child.nodeType === 1 && (child as Element).matches('p,div')) { while (child.firstChild) p.appendChild(child.firstChild); }
        else p.appendChild(child);
      }
      if (!p.firstChild) p.appendChild(d.createElement('br'));
      list.after(p);
      let cursor: Element = p;
      for (const nested of nestedLists) { cursor.after(nested); cursor = nested; }
      li.remove();
      if (!list.querySelector('li')) list.remove();
      return p;
    }
    if (!list.querySelector('li')) list.remove();
    return li;
  }

  function sinkItem(li: HTMLElement): boolean {
    const previous = li.previousElementSibling as HTMLElement | null;
    const list = li.parentElement as HTMLElement | null;
    if (!previous || previous.tagName !== 'LI' || !list) return false;
    let nested = previous.lastElementChild as HTMLElement | null;
    if (!isList(nested)) {
      nested = list.cloneNode(false) as HTMLElement;
      const parentStyle = styleOf(list);
      const def = listStyleDef(parentStyle);
      nested.removeAttribute('type');
      nested.removeAttribute('start');
      nested.style.removeProperty('list-style-type');
      if (def?.marker) nested.setAttribute(LIST_ATTRIBUTE, def.id);
      else if (list.tagName === 'OL') styleList(nested, nestedOrderedStyle(parentStyle));
      else if (list.style.listStyleType) styleList(nested, nestedBulletStyle(parentStyle));
      previous.appendChild(nested);
    }
    nested.appendChild(li);
    if (nested.getAttribute(LIST_ATTRIBUTE) === 'check' && !li.hasAttribute(CHECKED_ATTRIBUTE)) li.setAttribute(CHECKED_ATTRIBUTE, 'false');
    return true;
  }

  /** Give a numbered tree the outline look: 1. at the top, a. under it, i. under that. */
  function outline(root: HTMLElement, depth = 0): void {
    if (root.tagName === 'OL') styleList(root, outlineStyle(depth) === 'decimal' ? 'decimal' : outlineStyle(depth));
    for (const item of Array.from(root.children)) for (const child of Array.from(item.children)) if (isList(child)) outline(child, depth + 1);
  }

  function applyList(kind: ListKind, styleId: string): boolean {
    const range = commands.restore();
    const root = host();
    const def = listStyleDef(styleId);
    if (!range || !root || !def || def.kind !== kind) return false;
    const tag = kind === 'bullet' ? 'ul' : 'ol';
    const blocks = leafBlocks(range);
    if (blocks.length === 0) return false;
    const items = blocks.filter((block) => block.tagName === 'LI');
    const allSame = items.length === blocks.length && items.every((li) => {
      const list = li.parentElement as HTMLElement;
      return list.tagName.toLowerCase() === tag && styleOf(list) === styleId;
    });
    let final: HTMLElement[] = [];
    if (allSame && styleId !== 'outline') {
      for (const li of [...items].reverse()) { let now: HTMLElement = li; while (now.isConnected && now.parentElement && isList(now.parentElement)) now = outdentItem(now); final.unshift(now); }
    } else {
      const lists = new Set(items.map((li) => li.parentElement as HTMLElement));
      for (const list of lists) {
        const next = retag(list, tag);
        styleList(next, styleId === 'outline' ? 'decimal' : styleId);
        if (styleId === 'outline') outline(next);
        final.push(...(Array.from(next.children) as HTMLElement[]));
      }
      // Paragraphs become items of one new list per run of neighbours.
      const plain = blocks.filter((block) => block.tagName !== 'LI' && !block.matches('td,th'));
      let current: HTMLElement | null = null;
      for (const block of plain) {
        if (!current || current.nextElementSibling !== block) {
          current = root.ownerDocument.createElement(tag);
          block.before(current);
          styleList(current, styleId === 'outline' ? 'decimal' : styleId);
        }
        const li = root.ownerDocument.createElement('li');
        while (block.firstChild) li.appendChild(block.firstChild);
        if (!li.firstChild) li.appendChild(root.ownerDocument.createElement('br'));
        if (current.getAttribute(LIST_ATTRIBUTE) === 'check') li.setAttribute(CHECKED_ATTRIBUTE, 'false');
        current.appendChild(li);
        block.remove();
        final.push(li);
      }
    }
    final = final.filter((node) => node.isConnected);
    select(final[0], final[final.length - 1]);
    commands.commit();
    return true;
  }

  function listStart(value: number | 'restart' | 'continue'): boolean {
    commands.restore();
    const li = commands.current()?.closest('li');
    const list = li?.parentElement as HTMLElement | null | undefined;
    if (!li || !list || list.tagName !== 'OL') return false;
    let order: number;
    if (value === 'continue') {
      const before = list.previousElementSibling as HTMLElement | null;
      if (!before || before.tagName !== 'OL') return false;
      order = (Number(before.getAttribute('start')) || 1) + Array.from(before.children).filter((c) => c.tagName === 'LI').length;
    } else order = value === 'restart' ? 1 : Math.max(1, Math.min(9999, Math.floor(value)));
    const index = Array.from(list.children).indexOf(li);
    let target = list;
    if (index > 0 && value !== 'continue') {
      target = list.cloneNode(false) as HTMLElement;
      for (const sibling of Array.from(list.children).slice(index)) target.appendChild(sibling);
      list.after(target);
    }
    if (order === 1) target.removeAttribute('start');
    else target.setAttribute('start', String(order));
    select(li);
    commands.commit();
    return true;
  }

  function canContinue(): boolean {
    const list = commands.current()?.closest('li')?.parentElement;
    const before = list?.previousElementSibling;
    return !!list && list.tagName === 'OL' && before?.tagName === 'OL';
  }

  function toggleChecked(item?: HTMLElement): boolean {
    const li = item ?? commands.current()?.closest('li');
    if (!li || !li.hasAttribute(CHECKED_ATTRIBUTE)) return false;
    li.setAttribute(CHECKED_ATTRIBUTE, li.getAttribute(CHECKED_ATTRIBUTE) === 'true' ? 'false' : 'true');
    commands.commit();
    return true;
  }

  // ---- indent ------------------------------------------------------------
  function indent(direction: 1 | -1): boolean {
    const blocks = selectedBlocks();
    if (blocks.length === 0) return false;
    let changed = false;
    const after: HTMLElement[] = [];
    const items = blocks.filter((block) => block.tagName === 'LI');
    for (const li of direction > 0 ? items : [...items].reverse()) {
      if (direction > 0) changed = sinkItem(li) || changed;
      else if (li.parentElement && isList(li.parentElement)) { after.unshift(outdentItem(li)); changed = true; }
    }
    if (direction > 0) after.push(...items);
    for (const block of blocks.filter((b) => b.tagName !== 'LI')) {
      const level = Math.round((parseFloat(block.style.marginLeft) || 0) / INDENT_STEP);
      const next = Math.max(0, Math.min(INDENT_MAX, level + direction));
      if (next === level && !(direction < 0 && block.style.marginLeft)) continue;
      if (next === 0) block.style.removeProperty('margin-left');
      else block.style.setProperty('margin-left', `${next * INDENT_STEP}px`);
      if (block.getAttribute('style') === '') block.removeAttribute('style');
      changed = true;
      after.push(block);
    }
    if (changed) { select(after[0], after[after.length - 1]); commands.commit(); }
    return changed;
  }

  function canOutdent(): boolean {
    const block = commands.current()?.closest(LEAF) as HTMLElement | null | undefined;
    return !!block && (block.tagName === 'LI' || !!block.style.marginLeft);
  }

  // ---- paragraph attributes ---------------------------------------------
  function setBlock(property: string, value: string): boolean {
    const blocks = selectedBlocks();
    if (blocks.length === 0) return false;
    for (const block of blocks) {
      if (value) block.style.setProperty(property, value);
      else block.style.removeProperty(property);
      if (block.getAttribute('style') === '') block.removeAttribute('style');
    }
    commands.commit();
    return true;
  }
  function setDirection(value: 'ltr' | 'rtl'): boolean {
    const blocks = selectedBlocks();
    if (blocks.length === 0) return false;
    for (const block of blocks) block.setAttribute('dir', value);
    commands.commit();
    return true;
  }
  const currentBlock = (): HTMLElement | null => (commands.current()?.closest(LEAF) as HTMLElement | null) ?? null;

  // ---- clear formatting --------------------------------------------------
  function cleanFragment(fragment: ParentNode, keepLinks: boolean): void {
    for (const el of Array.from(fragment.querySelectorAll<HTMLElement>('*')).reverse()) {
      if (el.tagName === 'A' && keepLinks) { for (const p of TEXT_PROPERTIES) el.style.removeProperty(p); continue; }
      for (const p of TEXT_PROPERTIES) el.style.removeProperty(p);
      for (const attr of ['face', 'color', 'size']) el.removeAttribute(attr);
      if (el.getAttribute('style') === '') el.removeAttribute('style');
      if ((INLINE_FORMAT.has(el.tagName) && (el.tagName !== 'SPAN' || el.attributes.length === 0 || (el.attributes.length === 1 && el.hasAttribute('data-mw-native-caret')))) || (el.tagName === 'A' && !keepLinks)) el.replaceWith(...Array.from(el.childNodes));
    }
  }

  /** Move a collapsed range up out of inline formatting elements, splitting them around it. */
  function hoist(range: Range, root: HTMLElement, keepLinks: boolean): void {
    let container: Node = range.startContainer;
    let offset = range.startOffset;
    if (container.nodeType === 3) {
      const rest = (container as Text).splitText(offset);
      container = container.parentNode!;
      offset = Array.from(container.childNodes).indexOf(rest);
    }
    while (container !== root && container.nodeType === 1 && ((INLINE_FORMAT.has((container as Element).tagName)) || ((container as Element).tagName === 'A' && !keepLinks))) {
      const after = (container as Element).cloneNode(false) as Element;
      for (const child of Array.from(container.childNodes).slice(offset)) after.appendChild(child);
      (container as Element).after(after);
      const parent: Node = container.parentNode!;
      offset = Array.from(parent.childNodes).indexOf(after);
      container = parent;
    }
    range.setStart(container, offset);
    range.collapse(true);
  }

  /** Remove inline elements a split left empty. */
  function pruneEmpty(root: HTMLElement): void {
    for (const el of Array.from(root.querySelectorAll('b,strong,i,em,u,s,strike,del,font,span,sub,sup,mark,code,a'))) {
      if (!el.firstChild && !(el as HTMLElement).hasAttribute('data-mw-native-caret')) el.remove();
    }
  }

  function clearText(links: boolean): boolean {
    const range = commands.restore();
    const root = host();
    if (!range || !root || range.collapsed) return false;
    const fragment = range.extractContents();
    cleanFragment(fragment, !links);
    const first = fragment.firstChild;
    const last = fragment.lastChild;
    hoist(range, root, !links);
    range.insertNode(fragment);
    if (first && last) {
      const next = root.ownerDocument.createRange();
      next.setStartBefore(first);
      next.setEndAfter(last);
      commands.setRange(next);
    }
    root.normalize();
    pruneEmpty(root);
    return true;
  }

  function clearParagraph(): boolean {
    const blocks = selectedBlocks();
    let changed = false;
    for (const block of [...blocks].reverse()) {
      let target = block;
      if (target.tagName === 'LI') {
        while (target.isConnected && target.parentElement && isList(target.parentElement)) outdentItem(target);
        changed = true;
        continue;
      }
      if (target.matches('td,th')) continue;
      if (target.getAttribute('style') || target.hasAttribute('align') || target.hasAttribute('dir') || !/^(P|DIV)$/.test(target.tagName)) {
        target.removeAttribute('style');
        target.removeAttribute('align');
        target.removeAttribute('dir');
        if (!/^(P|DIV)$/.test(target.tagName)) {
          const p = target.ownerDocument.createElement('p');
          while (target.firstChild) p.appendChild(target.firstChild);
          target.replaceWith(p);
          target = p;
        }
        changed = true;
      }
    }
    return changed;
  }

  function clear(variant: ClearVariant): boolean {
    let done = false;
    if (variant !== 'paragraph') done = clearText(variant === 'everything') || done;
    if (variant !== 'text') done = clearParagraph() || done;
    if (!done) {
      // At a caret: drop the pending style span.
      const caret = commands.current()?.closest('[data-mw-native-caret]');
      if (caret && variant !== 'paragraph') { caret.replaceWith(...Array.from(caret.childNodes)); done = true; }
    }
    if (done) commands.commit();
    return done;
  }

  // ---- format painter ----------------------------------------------------
  function capture(): { value: NativePaint; summary: PaintSummary } | null {
    commands.restore();
    const el = commands.current();
    const view = el?.ownerDocument.defaultView;
    if (!el || !view) return null;
    const cs = view.getComputedStyle(el);
    const inline: Record<string, string> = {};
    for (const p of ['font-family', 'font-size', 'font-weight', 'font-style', 'color', 'background-color']) inline[p] = cs.getPropertyValue(p);
    const decoration = cs.getPropertyValue('text-decoration-line') || cs.getPropertyValue('text-decoration');
    if (decoration && decoration !== 'none') inline['text-decoration'] = decoration;
    if (inline['background-color'] === 'rgba(0, 0, 0, 0)' || inline['background-color'] === 'transparent') delete inline['background-color'];
    const blockEl = el.closest(LEAF) as HTMLElement | null;
    const block: Record<string, string> = {};
    if (blockEl) for (const p of ['text-align', 'line-height', 'margin-top', 'margin-bottom', 'margin-left']) { const v = blockEl.style.getPropertyValue(p); if (v) block[p] = v; }
    const hex = (value: string | undefined): string | null => {
      const m = value ? /^rgba?\((\d+),\s*(\d+),\s*(\d+)/.exec(value) : null;
      return m ? `#${m.slice(1).map((c) => Number(c).toString(16).padStart(2, '0')).join('')}` : null;
    };
    return {
      value: { inline, block },
      summary: {
        family: inline['font-family'] ?? null,
        size: pointsOf(inline['font-size']),
        bold: Number(inline['font-weight']) >= 600 || inline['font-weight'] === 'bold',
        italic: inline['font-style'] === 'italic',
        underline: (inline['text-decoration'] ?? '').includes('underline'),
        strike: (inline['text-decoration'] ?? '').includes('line-through'),
        color: hex(inline['color']),
        highlight: hex(inline['background-color']),
      },
    };
  }

  /** One span with several declarations around each selected run (or at the caret). */
  function styleSelection(props: Record<string, string>): boolean {
    const range = commands.restore();
    const root = host();
    if (!range || !root) return false;
    const d = root.ownerDocument;
    if (range.collapsed) {
      const span = d.createElement('span');
      for (const [p, v] of Object.entries(props)) span.style.setProperty(p, v);
      span.setAttribute('data-mw-native-caret', 'true');
      span.textContent = '​';
      range.insertNode(span);
      const next = d.createRange();
      next.selectNodeContents(span);
      next.collapse(false);
      commands.setRange(next);
      return true;
    }
    const spans: HTMLElement[] = [];
    for (const { node, from, to } of commands.selectedText().reverse()) {
      const piece = d.createRange();
      piece.setStart(node, from);
      piece.setEnd(node, to);
      const span = d.createElement('span');
      for (const [p, v] of Object.entries(props)) span.style.setProperty(p, v);
      piece.surroundContents(span);
      spans.unshift(span);
    }
    if (!spans.length) return false;
    const next = d.createRange();
    next.setStart(spans[0]!, 0);
    next.setEnd(spans[spans.length - 1]!, spans[spans.length - 1]!.childNodes.length);
    commands.setRange(next);
    return true;
  }

  function applyPaint(paint: NativePaint): boolean {
    const range = commands.restore();
    if (!range) return false;
    if (!range.collapsed) clearText(false);
    const done = styleSelection(paint.inline);
    for (const block of selectedBlocks()) {
      for (const p of ['text-align', 'line-height', 'margin-top', 'margin-bottom', 'margin-left']) {
        if (paint.block[p]) block.style.setProperty(p, paint.block[p]!);
        else block.style.removeProperty(p);
      }
      if (block.getAttribute('style') === '') block.removeAttribute('style');
    }
    commands.commit();
    return done;
  }


  // ---- font effects ------------------------------------------------------
  const EFFECT_OFF: Record<string, [string, string][]> = {
    doubleStrike: [['text-decoration', 'none']],
    smallCaps: [['font-variant', 'normal']],
    allCaps: [['text-transform', 'none']],
    underlineStyle: [['text-decoration', 'none']],
    ligatures: [['font-variant', 'normal']],
    scale: [['font-stretch', '100%'], ['letter-spacing', 'normal']],
    spacing: [['letter-spacing', 'normal']],
    outline: [['text-shadow', 'none']],
    shadow: [['text-shadow', 'none']],
  };

  /** Whether the caret's text carries effect `kind` (its declarations on the element or an ancestor). */
  function effectOn(kind: EffectKind): boolean {
    const el = commands.current();
    const root = host();
    if (!el || !root) return false;
    const tag = effectTag(kind);
    if (tag) return !!el.closest(tag) && root.contains(el.closest(tag));
    const wanted = effectStyle(kind, effectDef(kind)?.defaultValue ?? '');
    if (wanted.length === 0) return false;
    // The innermost element that sets the first property decides, as in the cascade.
    for (let node: Element | null = el; node && node !== root; node = node.parentElement) {
      const value = (node as HTMLElement).style?.getPropertyValue(wanted[0]![0]);
      if (value) return wanted.every(([property, expected]) => (node as HTMLElement).style.getPropertyValue(property) === expected);
    }
    return false;
  }

  function toggleEffect(kind: EffectKind, value = ''): boolean {
    const range = commands.restore();
    const root = host();
    if (!range || !root) return false;
    const tag = effectTag(kind);
    const on = effectOn(kind);
    const d = root.ownerDocument;
    if (tag) {
      if (on) {
        const element = commands.current()?.closest(tag);
        if (element) element.replaceWith(...Array.from(element.childNodes));
      } else {
        const opposite = tag === 'sup' ? 'sub' : 'sup';
        const collapsed = range.collapsed;
        const runs = collapsed ? [] : commands.selectedText();
        for (const element of Array.from(root.querySelectorAll(opposite))) if (range.intersectsNode(element)) element.replaceWith(...Array.from(element.childNodes));
        if (collapsed) {
          const wrapper = d.createElement(tag);
          wrapper.setAttribute('data-mw-native-caret', 'true');
          wrapper.textContent = '​';
          range.insertNode(wrapper);
          const next = d.createRange();
          next.selectNodeContents(wrapper);
          next.collapse(false);
          commands.setRange(next);
        } else {
          const wrapped: HTMLElement[] = [];
          for (const { node, from, to } of runs.reverse()) {
            const piece = d.createRange();
            piece.setStart(node, from);
            piece.setEnd(node, to);
            const wrapper = d.createElement(tag);
            piece.surroundContents(wrapper);
            wrapped.unshift(wrapper);
          }
          if (wrapped.length) select(wrapped[0], wrapped[wrapped.length - 1]);
        }
      }
      commands.commit();
      return true;
    }
    const declarations = on ? EFFECT_OFF[kind] ?? [] : effectStyle(kind, value);
    if (declarations.length === 0) return false;
    const done = styleSelection(Object.fromEntries(declarations));
    if (done) commands.commit();
    return done;
  }

  // ---- sort --------------------------------------------------------------
  const ownText = (item: Element): string => Array.from(item.childNodes).filter((node) => !isList(node)).map((node) => node.textContent ?? '').join('');

  function sortBlocks(options: SortOptions, onResult?: (outcome: { kind: 'rows' | 'items' | 'blocks'; count: number; removed: number }) => void, random?: () => number): boolean {
    const range = commands.restore();
    if (!range) return false;
    let nodes: HTMLElement[] = [];
    let textOf: (node: HTMLElement) => string = (node) => node.textContent ?? '';
    let kind: 'rows' | 'items' | 'blocks' = 'blocks';
    let keep: HTMLElement[] = [];
    const cell = commands.current()?.closest('td,th') as HTMLTableCellElement | null | undefined;
    const li = commands.current()?.closest('li');
    if (cell && host()?.contains(cell)) {
      const table = cell.closest('table')!;
      const rows = Array.from(table.rows) as HTMLElement[];
      const column = options.column ?? cell.cellIndex;
      keep = options.header && rows.length > 1 ? rows.slice(0, 1) : [];
      nodes = rows.slice(keep.length);
      textOf = (row) => (row as HTMLTableRowElement).cells[column]?.textContent ?? '';
      kind = 'rows';
    } else if (li?.parentElement && isList(li.parentElement)) {
      nodes = Array.from(li.parentElement.children).filter((child): child is HTMLElement => child.tagName === 'LI');
      textOf = ownText;
      kind = 'items';
    } else {
      const blocks = leafBlocks(range);
      if (blocks.length < 2 || blocks.some((block) => block.parentElement !== blocks[0]!.parentElement)) return false;
      nodes = blocks;
    }
    if (nodes.length < 2) return false;
    const result = sortItems(nodes, textOf, options, random);
    if (!result.changed) { onResult?.({ kind, count: nodes.length, removed: 0 }); return false; }
    const parent = nodes[0]!.parentElement!;
    const after = nodes[nodes.length - 1]!.nextSibling;
    for (const node of nodes) node.remove();
    for (const node of result.items) parent.insertBefore(node, after);
    void keep;
    select(result.items[0], result.items[result.items.length - 1]);
    commands.commit();
    onResult?.({ kind, count: result.items.length, removed: result.removed });
    return true;
  }

  return {
    toggleEffect, effectOn, sortBlocks,
    stepFont, listType, applyList, listStart, canContinue, toggleChecked, indent, canOutdent,
    setBlock, setDirection, currentBlock, clear, capture, applyPaint, sinkItem, outdentItem, doc,
  };
}

export type NativeFormat = ReturnType<typeof createNativeFormat>;
