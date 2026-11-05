import { describe, expect, it } from 'vitest';
import { chipsOf, commitRecipient, removeRecipient, splitPasted } from './recipientChips.ts';

describe('commitRecipient', () => {
  it('turns a valid address followed by a space into a finished recipient', () => {
    expect(commitRecipient('a@b.example ', 12, 'space', 'comma')).toEqual({ value: 'a@b.example, ', caret: 13 });
  });
  it('follows the configured separator', () => {
    expect(commitRecipient('x@y.example, a@b.example ', 25, 'space', 'semicolon')?.value).toBe('x@y.example, a@b.example; ');
  });
  it('replaces a typed comma or semicolon with the configured separator', () => {
    expect(commitRecipient('a@b.example;', 12, 'semicolon', 'comma')?.value).toBe('a@b.example, ');
    expect(commitRecipient('a@b.example,', 12, 'comma', 'comma')?.value).toBe('a@b.example, ');
  });
  it('leaves invalid text alone', () => {
    expect(commitRecipient('nope ', 5, 'space', 'comma')).toBeNull();
    expect(commitRecipient('John ', 5, 'space', 'comma')).toBeNull();
  });
  it('commits on blur and enter, and supports Name <addr>', () => {
    expect(commitRecipient('a@b.example', 11, 'blur', 'comma')?.value).toBe('a@b.example, ');
    expect(commitRecipient('Ann <a@b.example>', 17, 'enter', 'comma')?.value).toBe('Ann <a@b.example>, ');
  });
  it('does nothing for an already finished list', () => {
    expect(commitRecipient('a@b.example, ', 13, 'blur', 'comma')).toBeNull();
  });
  it('commits in the middle of the field and keeps what follows', () => {
    expect(commitRecipient('a@b.example c@d.example, ', 12, 'space', 'comma')?.value).toBe('a@b.example, c@d.example, ');
  });
});

describe('splitPasted', () => {
  it('splits several addresses, newline, whitespace or separator separated', () => {
    expect(splitPasted('a@b.example\nc@d.example', 'comma')).toBe('a@b.example, c@d.example, ');
    expect(splitPasted('a@b.example c@d.example', 'semicolon')).toBe('a@b.example; c@d.example; ');
    expect(splitPasted('Ann <a@b.example>, c@d.example', 'comma')).toBe('Ann <a@b.example>, c@d.example, ');
  });
  it('lets a single entry paste normally', () => {
    expect(splitPasted('a@b.example', 'comma')).toBeNull();
  });
});

describe('chipsOf and removeRecipient', () => {
  it('marks valid and invalid entries', () => {
    const chips = chipsOf('Ann <a@b.example>, nope, ');
    expect(chips.map(c => [c.email, c.name, c.valid])).toEqual([['a@b.example', 'Ann', true], ['nope', null, false]]);
  });
  it('removes one entry and keeps the rest separated', () => {
    expect(removeRecipient('a@b.example, nope, c@d.example, ', 1, 'comma')).toBe('a@b.example, c@d.example, ');
    expect(removeRecipient('a@b.example, ', 0, 'comma')).toBe('');
  });
});
