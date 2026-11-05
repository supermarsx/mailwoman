import { describe, expect, it } from 'vitest';
import { headerOpen, headerSummary, isHeaderCollapseMode, readHeaderMemory, writeHeaderMemory, type HeaderOpenInput } from './headerCollapse.ts';

const labels = { to: 'To', cc: 'Cc', bcc: 'Bcc', subject: 'Subject', noSubject: '(no subject)' };
const base: HeaderOpenInput = { mode: 'expanded', remembered: null, focusWithin: false, attention: false, empty: false, peek: false };

describe('headerSummary', () => {
  it('names the first recipient, counts the rest and shows the subject', () => {
    expect(headerSummary({ to: 'Ana <ana@x.test>, b@x.test, c@x.test', cc: 'd@x.test', bcc: '', subject: 'Re: Invoice' }, labels))
      .toBe('To: Ana +2 · Cc: 1 · Subject: Re: Invoice');
  });
  it('falls back to the local part and to the no-subject label', () => {
    expect(headerSummary({ to: 'bob@x.test', cc: '', bcc: 'a@x.test, b@x.test', subject: '  ' }, labels)).toBe('To: bob · Bcc: 2 · Subject: (no subject)');
  });
});

describe('headerOpen', () => {
  it('follows the configured mode', () => {
    expect(headerOpen(base)).toBe(true);
    expect(headerOpen({ ...base, mode: 'collapsed' })).toBe(false);
    expect(headerOpen({ ...base, mode: 'auto' })).toBe(false);
  });
  it('lets the remembered choice win in expanded and collapsed, not in auto', () => {
    expect(headerOpen({ ...base, remembered: false })).toBe(false);
    expect(headerOpen({ ...base, mode: 'collapsed', remembered: true })).toBe(true);
    expect(headerOpen({ ...base, mode: 'auto', remembered: true })).toBe(false);
  });
  it('opens for focus, attention, an empty header and a peek', () => {
    expect(headerOpen({ ...base, mode: 'collapsed', focusWithin: true })).toBe(true);
    expect(headerOpen({ ...base, mode: 'collapsed', attention: true })).toBe(true);
    expect(headerOpen({ ...base, mode: 'collapsed', empty: true })).toBe(true);
    expect(headerOpen({ ...base, mode: 'auto', peek: true })).toBe(true);
  });
  it('validates modes', () => {
    expect(isHeaderCollapseMode('auto')).toBe(true);
    expect(isHeaderCollapseMode('x')).toBe(false);
  });
});

describe('header memory', () => {
  it('round-trips and survives broken storage', () => {
    const data = new Map<string, string>();
    const storage = { getItem: (k: string) => data.get(k) ?? null, setItem: (k: string, v: string) => void data.set(k, v) };
    expect(readHeaderMemory(storage)).toBeNull();
    writeHeaderMemory(false, storage);
    expect(readHeaderMemory(storage)).toBe(false);
    writeHeaderMemory(true, storage);
    expect(readHeaderMemory(storage)).toBe(true);
    const broken = { getItem: () => { throw new Error('blocked'); }, setItem: () => { throw new Error('blocked'); } };
    expect(readHeaderMemory(broken)).toBeNull();
    expect(() => writeHeaderMemory(true, broken)).not.toThrow();
  });
});
