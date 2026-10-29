import { describe, expect, it } from 'vitest';
import { bindingsFor, createKeymap } from './dispatcher.ts';
import { chordId, chordOf } from './chords.ts';

describe('reader zoom bindings (t59)', () => {
  it('binds Ctrl/Cmd + = / + / - / 0 in every preset but Off', () => {
    for (const preset of ['default', 'gmail', 'outlook', 'vim'] as const) {
      const map = bindingsFor(preset);
      expect(map.get('mod+=')).toBe('zoomIn');
      expect(map.get('mod++')).toBe('zoomIn');
      expect(map.get('mod+-')).toBe('zoomOut');
      expect(map.get('mod+0')).toBe('zoomReset');
    }
    expect(bindingsFor('off').size).toBe(0);
  });
  it('matches the keys as pressed, Cmd included, and Shift+= as plus', () => {
    expect(chordId(chordOf(new KeyboardEvent('keydown', { key: '=', metaKey: true })))).toBe('mod+=');
    expect(chordId(chordOf(new KeyboardEvent('keydown', { key: '+', ctrlKey: true, shiftKey: true })))).toBe('mod++');
  });
  it('leaves the key to the browser when the reader declines', () => {
    const keymap = createKeymap({ target: new EventTarget() as unknown as Document, bindings: () => bindingsFor('default') });
    keymap.register('zoomIn', () => false);
    const e = new KeyboardEvent('keydown', { key: '=', ctrlKey: true, cancelable: true });
    expect(keymap.dispatch(e)).toBe(false);
    expect(e.defaultPrevented).toBe(false);
    keymap.dispose();
  });
});
