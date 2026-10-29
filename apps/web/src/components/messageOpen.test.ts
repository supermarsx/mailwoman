import { describe, expect, it } from 'vitest';
import { classifyRowClick, dblClickOpensTab, messageOpenTarget } from './messageOpen.ts';

// t74: where a message opened from the Mail list goes, and which clicks open it.

const plain = { shiftKey: false, ctrlKey: false, metaKey: false, altKey: false };

describe('messageOpenTarget', () => {
  it('reading pane on: an activation goes to the pane, an explicit ask to a tab', () => {
    for (const readingPane of ['right', 'bottom'] as const) {
      expect(messageOpenTarget('activate', { readingPane, narrow: false })).toBe('pane');
      expect(messageOpenTarget('tab', { readingPane, narrow: false })).toBe('tab');
    }
  });
  it('reading pane off: every open is a tab', () => {
    expect(messageOpenTarget('activate', { readingPane: 'off', narrow: false })).toBe('tab');
    expect(messageOpenTarget('tab', { readingPane: 'off', narrow: false })).toBe('tab');
  });
  it('phone width has no pane beside the list: every open is a tab, whatever the setting', () => {
    for (const readingPane of ['right', 'bottom', 'off'] as const) {
      expect(messageOpenTarget('activate', { readingPane, narrow: true })).toBe('tab');
    }
  });
});

describe('classifyRowClick', () => {
  const state = { modifierSelection: true, selecting: false };
  it('a plain click, a tap or Enter (detail 0) activates', () => {
    expect(classifyRowClick({ ...plain, detail: 1 }, state)).toBe('activate');
    expect(classifyRowClick({ ...plain, detail: 0 }, state)).toBe('activate');
  });
  it('the second and later clicks of a multi-click are left to dblclick', () => {
    expect(classifyRowClick({ ...plain, detail: 2 }, state)).toBe('ignore');
    expect(classifyRowClick({ ...plain, detail: 3 }, state)).toBe('ignore');
  });
  it('Ctrl/Cmd+click toggles and Shift+click selects a range; neither opens', () => {
    expect(classifyRowClick({ ...plain, ctrlKey: true, detail: 1 }, state)).toBe('toggle');
    expect(classifyRowClick({ ...plain, metaKey: true, detail: 2 }, state)).toBe('toggle');
    expect(classifyRowClick({ ...plain, shiftKey: true, detail: 1 }, state)).toBe('range');
  });
  it('selection mode toggles on every click', () => {
    expect(classifyRowClick({ ...plain, detail: 1 }, { modifierSelection: true, selecting: true })).toBe('toggle');
    expect(classifyRowClick({ ...plain, detail: 2 }, { modifierSelection: false, selecting: true })).toBe('toggle');
  });
  it('with modifier selection turned off, a modified click activates', () => {
    expect(classifyRowClick({ ...plain, ctrlKey: true, detail: 1 }, { modifierSelection: false, selecting: false })).toBe('activate');
  });
});

describe('dblClickOpensTab', () => {
  it('only an unmodified double-click outside selection mode asks for a tab', () => {
    expect(dblClickOpensTab(plain, { selecting: false })).toBe(true);
    expect(dblClickOpensTab({ ...plain, ctrlKey: true }, { selecting: false })).toBe(false);
    expect(dblClickOpensTab({ ...plain, metaKey: true }, { selecting: false })).toBe(false);
    expect(dblClickOpensTab({ ...plain, shiftKey: true }, { selecting: false })).toBe(false);
    expect(dblClickOpensTab({ ...plain, altKey: true }, { selecting: false })).toBe(false);
    expect(dblClickOpensTab(plain, { selecting: true })).toBe(false);
  });
});
