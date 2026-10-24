import { describe, it, expect, vi, afterEach } from 'vitest';
import { bindingsFor, createKeymap, activeBindings, type Keymap } from './dispatcher.ts';
import { chordId, chordOf, parseChord } from './chords.ts';
import { DEFAULT_PREFS, PRESET_BINDINGS, savePrefs } from '../screens/Settings/prefs.ts';

// The one key dispatcher (t28-e16). Each test builds its own dispatcher on a
// fresh element, so the app's shared one is untouched.

let keymap: Keymap | undefined;
afterEach(() => {
  keymap?.dispose();
  keymap = undefined;
  document.body.innerHTML = '';
  localStorage.clear();
});

function press(target: Element, key: string, init: KeyboardEventInit = {}): KeyboardEvent {
  const e = new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true, ...init });
  target.dispatchEvent(e);
  return e;
}

describe('chords', () => {
  it('parses the preset syntax', () => {
    expect(parseChord('e')).toEqual({ key: 'e', mod: false, alt: false, shift: false });
    expect(parseChord('Ctrl+N')).toEqual({ key: 'n', mod: true, alt: false, shift: false });
    expect(parseChord('↓')).toEqual({ key: 'ArrowDown', mod: false, alt: false, shift: false });
    expect(parseChord('Backspace')?.key).toBe('Backspace');
    expect(parseChord('Shift+U')).toEqual({ key: 'u', mod: false, alt: false, shift: true });
    expect(parseChord('Hyper+x')).toBeNull();
    expect(parseChord('')).toBeNull();
  });

  it('reads a key press the same way, with Cmd as Ctrl and Shift only where it is not in the character', () => {
    const chord = (init: KeyboardEventInit): string => chordId(chordOf(new KeyboardEvent('keydown', init)));
    expect(chord({ key: 'n', metaKey: true })).toBe(chordId(parseChord('Ctrl+N')!));
    expect(chord({ key: 'n', ctrlKey: true })).toBe(chordId(parseChord('Ctrl+N')!));
    expect(chord({ key: 'U', shiftKey: true })).toBe(chordId(parseChord('Shift+U')!));
    expect(chord({ key: '?', shiftKey: true })).toBe(chordId(parseChord('?')!));
    expect(chord({ key: 'E' , shiftKey: true })).not.toBe(chordId(parseChord('e')!));
  });

  it('every binding in every preset parses', () => {
    for (const bindings of Object.values(PRESET_BINDINGS)) {
      for (const { keys } of bindings) expect(parseChord(keys), keys).not.toBeNull();
    }
  });
});

describe('bindings', () => {
  it('maps a preset to action ids, and "off" to nothing', () => {
    expect(bindingsFor('gmail').get('e')).toBe('archive');
    expect(bindingsFor('vim').get('x')).toBe('archive');
    expect(bindingsFor('outlook').get('mod+n')).toBe('compose');
    expect(bindingsFor('outlook').get('ArrowDown')).toBe('next');
    expect(bindingsFor('off').size).toBe(0);
  });

  it('follows the preset saved in Settings at each press', () => {
    savePrefs({ ...DEFAULT_PREFS, keyboardPreset: 'vim' });
    expect(activeBindings().get('x')).toBe('archive');
    savePrefs({ ...DEFAULT_PREFS, keyboardPreset: 'off' });
    expect(activeBindings().get('e')).toBeUndefined();
  });
});

describe('dispatcher', () => {
  function setup(preset: Parameters<typeof bindingsFor>[0] = 'gmail'): { root: HTMLElement; archive: ReturnType<typeof vi.fn> } {
    const root = document.createElement('div');
    root.innerHTML = `
      <button id="row">row</button>
      <input id="field" type="text" />
      <input id="box" type="checkbox" />
      <div contenteditable="true" id="editor"><p id="para">x</p></div>
      <div role="dialog" aria-modal="true"><button id="in-modal">ok</button></div>`;
    document.body.appendChild(root);
    keymap = createKeymap({ target: root.ownerDocument, bindings: () => bindingsFor(preset) });
    const archive = vi.fn(() => true);
    keymap.register('archive', archive);
    return { root, archive };
  }

  it('runs the action bound to the key and consumes the press', () => {
    const { archive } = setup();
    const e = press(document.getElementById('row')!, 'e');
    expect(archive).toHaveBeenCalledTimes(1);
    expect(e.defaultPrevented).toBe(true);
  });

  it('with the preset off, the same key does nothing', () => {
    const { archive } = setup('off');
    const e = press(document.getElementById('row')!, 'e');
    expect(archive).not.toHaveBeenCalled();
    expect(e.defaultPrevented).toBe(false);
  });

  it('leaves typing, modals, IME composition and keys an element already handled', () => {
    const { archive } = setup();
    press(document.getElementById('field')!, 'e');
    press(document.getElementById('para')!, 'e');
    press(document.getElementById('in-modal')!, 'e');
    press(document.getElementById('row')!, 'e', { isComposing: true } as KeyboardEventInit);
    const row = document.getElementById('row')!;
    row.addEventListener('keydown', (ev) => ev.preventDefault(), { once: true });
    press(row, 'e');
    expect(archive).not.toHaveBeenCalled();
    // A checkbox takes no text, so the shortcut still applies there.
    press(document.getElementById('box')!, 'e');
    expect(archive).toHaveBeenCalledTimes(1);
  });

  it('a handler that declines leaves the key alone', () => {
    setup();
    keymap!.register('compose', () => false);
    const e = press(document.getElementById('row')!, 'c');
    expect(e.defaultPrevented).toBe(false);
  });

  it('the newest registration answers; removing it hands back to the previous one', () => {
    const { archive } = setup();
    const newer = vi.fn(() => true);
    const remove = keymap!.register('archive', newer);
    press(document.getElementById('row')!, 'e');
    expect(newer).toHaveBeenCalledTimes(1);
    expect(archive).not.toHaveBeenCalled();
    remove();
    press(document.getElementById('row')!, 'e');
    expect(archive).toHaveBeenCalledTimes(1);
  });

  it('dispose detaches the listener', () => {
    const { archive } = setup();
    keymap!.dispose();
    press(document.getElementById('row')!, 'e');
    expect(archive).not.toHaveBeenCalled();
  });
});
