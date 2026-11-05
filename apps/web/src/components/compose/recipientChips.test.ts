import { describe, expect, it } from 'vitest';
import { entriesOf, isCompleteRecipient, recipientOf, serializeRecipients, splitPastedEntries } from './recipientChips.ts';

describe('recipient helpers', () => {
  it('recognises a complete recipient', () => {
    expect(isCompleteRecipient('a@b.example')).toBe(true);
    expect(isCompleteRecipient('Ann <a@b.example>')).toBe(true);
    expect(isCompleteRecipient('nope')).toBe(false);
    expect(isCompleteRecipient('a@b.example c@d.example')).toBe(false);
  });
  it('parses a recipient', () => {
    expect(recipientOf(' Ann <a@b.example> ')).toMatchObject({ text: 'Ann <a@b.example>', email: 'a@b.example', name: 'Ann', valid: true });
    expect(recipientOf('nope').valid).toBe(false);
  });
  it('serialises with the configured separator and keeps the draft', () => {
    const list = [{ text: 'a@b.example' }, { text: 'Ann <c@d.example>' }];
    expect(serializeRecipients(list, '', 'comma')).toBe('a@b.example, Ann <c@d.example>, ');
    expect(serializeRecipients(list, 'dr', 'semicolon')).toBe('a@b.example; Ann <c@d.example>; dr');
    expect(serializeRecipients([], 'x', 'comma')).toBe('x');
  });
  it('splits entries and pasted text', () => {
    expect(entriesOf('a@b.example; c@d.example, ')).toEqual(['a@b.example', 'c@d.example']);
    expect(splitPastedEntries('a@b.example\nc@d.example')).toEqual(['a@b.example', 'c@d.example']);
    expect(splitPastedEntries('a@b.example c@d.example')).toEqual(['a@b.example', 'c@d.example']);
    expect(splitPastedEntries('Ann <a@b.example>, c@d.example')).toEqual(['Ann <a@b.example>', 'c@d.example']);
    expect(splitPastedEntries('a@b.example')).toEqual(['a@b.example']);
  });
});
