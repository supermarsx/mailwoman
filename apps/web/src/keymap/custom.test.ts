import { expect, it } from 'vitest';
import { resolveCustomBindings, validBinding } from './custom.ts';
import type { SettingsMap } from '../api/settings.ts';
import { createKeymap } from './dispatcher.ts';
const entry = (value: unknown) => ({ value, source: 'user' as const, locked: false });
it('replaces inherited actions, disables individual actions and keeps unrelated shortcuts', () => {
  const result = resolveCustomBindings(new Map([['c','compose'],['e','archive'],['r','reply']]), {'shortcuts.compose':entry('Ctrl+Shift+N'),'shortcuts.archive':entry('disabled')});
  expect([...result.bindings]).toEqual([['r','reply'],['mod+shift+n','compose']]);
});
it('disables every colliding action instead of executing an unintended action', () => {
  const values: SettingsMap = {'shortcuts.compose':entry('r')};
  const result = resolveCustomBindings(new Map([['c','compose'],['r','reply']]),values);
  expect(result.bindings.size).toBe(0); expect(result.conflicts.sort()).toEqual(['compose','reply']);
});
it('honors global effective disable and refuses malformed or unsupported bindings', () => {
  expect(resolveCustomBindings(new Map([['c','compose']]), {'shortcuts.enabled':entry(false)}).bindings.size).toBe(0);
  for (const value of ['Hyper+x','UnknownKey','Ctrl+','\n','x'.repeat(65)]) expect(validBinding(value),value).toBe(false);
  for (const value of ['Ctrl+Shift+N','Alt+F4','ArrowDown','Space','inherit','disabled']) expect(validBinding(value),value).toBe(true);
});
it('lets the active retained tab handle a shortcut when a newer hidden tab declines', () => {
  const keymap = createKeymap({ bindings: () => new Map([['r','reply']]) });
  let replied = 0;
  keymap.register('reply', () => { replied++; });
  keymap.register('reply', () => false);
  expect(keymap.dispatch(new KeyboardEvent('keydown', { key:'r', cancelable:true }))).toBe(true);
  expect(replied).toBe(1);
  keymap.dispose();
});
