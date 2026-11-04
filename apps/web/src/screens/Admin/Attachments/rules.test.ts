import { describe, expect, it } from 'vitest';
import { MAX_RULES, addBulk, conflictsFor, covers, exportText, listOf, normaliseRule, splitBytes, splitBulk, toBytes } from './rules.ts';

describe('attachment rule normalisation', () => {
  it('normalises extensions: dot, star-dot, case and spaces', () => {
    for (const raw of ['pdf', '.pdf', '*.PDF', ' .Pdf ']) expect(normaliseRule(raw, false)).toEqual({ ok: true, rule: 'pdf' });
    expect(normaliseRule('.tar.gz', false)).toEqual({ ok: true, rule: 'tar.gz' });
  });
  it('rejects extensions the server would refuse', () => {
    for (const raw of ['', '..', 'a b', 'p/df', 'x'.repeat(33), 'pdf.', 'image/png']) expect(normaliseRule(raw, false).ok).toBe(false);
  });
  it('accepts MIME types and top-level wildcards, lowercased', () => {
    expect(normaliseRule('Image/PNG', true)).toEqual({ ok: true, rule: 'image/png' });
    expect(normaliseRule('image/*', true)).toEqual({ ok: true, rule: 'image/*' });
    expect(normaliseRule('application/vnd.ms-excel', true).ok).toBe(true);
  });
  it('rejects bad MIME patterns', () => {
    for (const raw of ['image', '*/png', 'image/', '/png', 'a/b/c', 'image/p ng', '*/*']) expect(normaliseRule(raw, true).ok).toBe(false);
  });
});

describe('bulk paste', () => {
  it('splits on whitespace, commas and semicolons', () => {
    expect(splitBulk('pdf, docx;\n xlsx  .png')).toEqual(['pdf', 'docx', 'xlsx', '.png']);
  });
  it('adds valid new rules and reports duplicates and invalid entries', () => {
    const r = addBulk(['pdf'], 'PDF .exe bad/one .scr', false);
    expect(r.rules).toEqual(['pdf', 'exe', 'scr']);
    expect(r.duplicates).toEqual(['pdf']);
    expect(r.invalid.map((i) => i.input)).toEqual(['bad/one']);
  });
  it('stops at the server limit of 100 rules', () => {
    const many = Array.from({ length: 105 }, (_, i) => `e${i}`).join(' ');
    const r = addBulk([], many, false);
    expect(r.rules).toHaveLength(MAX_RULES);
    expect(r.overflow).toHaveLength(5);
  });
  it('exports one rule per line', () => {
    expect(exportText(['a', 'b'])).toBe('a\nb\n');
  });
});

describe('conflicts between lists', () => {
  const none = { allowed: [], blocked: [], warn: [] };
  it('flags a rule in the allowed and blocked lists, blocked winning', () => {
    const c = conflictsFor('allowed', ['exe'], { ...none, blocked: ['exe'] }, false);
    expect(c).toEqual([{ rule: 'exe', other: 'exe', list: 'allowed', kind: 'blockedOverAllowed' }]);
  });
  it('flags a warning that a block hides, from either side', () => {
    expect(conflictsFor('warn', ['zip'], { ...none, blocked: ['zip'] }, false)[0]?.kind).toBe('blockedOverWarn');
    expect(conflictsFor('blocked', ['zip'], { ...none, warn: ['zip'] }, false)[0]?.kind).toBe('blockedOverWarn');
  });
  it('notes allowed plus warn', () => {
    expect(conflictsFor('warn', ['doc'], { ...none, allowed: ['doc'] }, false)[0]?.kind).toBe('allowedAndWarn');
  });
  it('sees a wildcard cover a specific type in the other list', () => {
    expect(covers('image/*', 'image/png', true)).toBe(true);
    expect(covers('image/png', 'image/*', true)).toBe(false);
    expect(conflictsFor('allowed', ['image/png'], { ...none, blocked: ['image/*'] }, true)).toHaveLength(1);
    expect(conflictsFor('allowed', ['text/plain'], { ...none, blocked: ['image/*'] }, true)).toHaveLength(0);
  });
  it('is quiet when the lists do not meet', () => {
    expect(conflictsFor('allowed', ['pdf'], { allowed: ['pdf'], blocked: ['exe'], warn: ['zip'] }, false)).toEqual([]);
  });
});

describe('key and size helpers', () => {
  it('maps registry keys to a list and a kind', () => {
    expect(listOf('attachments.blockedMimeTypes')).toEqual({ list: 'blocked', mime: true });
    expect(listOf('attachments.warnExtensions')).toEqual({ list: 'warn', mime: false });
    expect(listOf('attachments.maxBytes')).toBeUndefined();
  });
  it('shows bytes in the largest even unit and converts back', () => {
    expect(splitBytes(25 * 1024 ** 2)).toEqual({ amount: 25, unit: 'MiB' });
    expect(splitBytes(1536)).toEqual({ amount: 1536, unit: 'B' });
    expect(splitBytes(0)).toEqual({ amount: 0, unit: 'MiB' });
    expect(toBytes(1.5, 'KiB')).toBe(1536);
    expect(toBytes(2, 'GiB')).toBe(2 * 1024 ** 3);
  });
});
