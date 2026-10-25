import { describe, it, expect } from 'vitest';
import { toastTarget } from './context.ts';
describe('contextual toast targets', () => {
 it('uses a readable fallback and removes controls without inserting markup', () => {
  expect(toastTarget(' \n ', 'Untitled')).toBe('Untitled');
  expect(toastTarget('Hello\u202e\nworld <b>', 'Untitled')).toBe('Hello world <b>');
 });
 it('bounds long titles without splitting a Unicode code point', () => {
  const target = toastTarget('😀'.repeat(90), 'Untitled');
  expect(Array.from(target)).toHaveLength(80); expect(target.endsWith('…')).toBe(true);
 });
});
