import { describe, expect, it } from 'vitest';
import {
  EMPTY_QUERY, offlineQueryOf, canonicalize, chipsOf, fieldTerm, parseQuery, quote, removeTerms, replaceTerm, resolveScope,
  serializeQuery, serverQueryText, setField, setOptionalField, setRange, setScope, toggleTerm, valueIn,
  type SearchQuery, includesTrash, setIncludeTrash, excludedSearchMailboxes,
} from './searchQuery.ts';

const rt = (input: string): string => serializeQuery(parseQuery(input));

describe('parseQuery', () => {
  it('reads field operators, bare words and phrases', () => {
    expect(parseQuery('from:x subject:y').terms).toEqual([
      { kind: 'field', op: 'from', value: 'x', negated: false, quoted: false },
      { kind: 'field', op: 'subject', value: 'y', negated: false, quoted: false },
    ]);
    expect(parseQuery('hello world "exact phrase"').terms).toEqual([
      { kind: 'text', value: 'hello world', negated: false, quoted: false },
      { kind: 'text', value: 'exact phrase', negated: false, quoted: true },
    ]);
  });

  it('reads quoted field values with escapes like the engine', () => {
    expect(parseQuery('subject:"a \\" OR from:other"').terms).toEqual([
      { kind: 'field', op: 'subject', value: 'a " OR from:other', negated: false, quoted: true },
    ]);
    expect(parseQuery('from:"back\\\\slash"').terms[0]).toMatchObject({ value: 'back\\slash' });
    expect(parseQuery('"C:\\dir"').terms[0]).toMatchObject({ kind: 'text', value: 'C:\\dir' });
  });

  it('reads negation by - and NOT, and drops AND', () => {
    expect(parseQuery('-from:spam NOT has:attachment AND is:unread').terms).toEqual([
      { kind: 'field', op: 'from', value: 'spam', negated: true, quoted: false },
      { kind: 'field', op: 'has', value: 'attachment', negated: true, quoted: false },
      { kind: 'field', op: 'is', value: 'unread', negated: false, quoted: false },
    ]);
    expect(parseQuery('NOT NOT x').terms[0]).toMatchObject({ kind: 'text', negated: false });
  });

  it('canonicalizes operator names and value spellings', () => {
    expect(rt('FROM:Ann HAS:files has:invite is:starred LARGER:1M')).toBe('from:Ann has:attachment has:calendar is:flagged larger:1m');
  });

  it('keeps groups and any top-level OR verbatim as one expression', () => {
    expect(parseQuery('from:a OR from:b').terms).toEqual([{ kind: 'expr', text: 'from:a OR from:b' }]);
    expect(parseQuery('subject:x -(from:a OR from:b) y').terms).toEqual([
      { kind: 'field', op: 'subject', value: 'x', negated: false, quoted: false },
      { kind: 'expr', text: '-(from:a OR from:b)' },
      { kind: 'text', value: 'y', negated: false, quoted: false },
    ]);
    expect(parseQuery('(a (b c)').terms).toEqual([{ kind: 'expr', text: '(a (b c)' }]);
  });

  it('treats unknown operators as words and tolerates junk', () => {
    expect(parseQuery('foo:bar').terms).toEqual([{ kind: 'text', value: 'foo:bar', negated: false, quoted: false }]);
    for (const junk of ['', '   ', '-', '"', '"unclosed', 'from:', ')', 'NOT', '((((', 'a\u0000b', '😀 from:😀']) {
      expect(() => parseQuery(junk)).not.toThrow();
    }
    expect(parseQuery('   ')).toEqual(EMPTY_QUERY);
  });
});

describe('serializeQuery', () => {
  it('quotes values that need it and escapes quotes and backslashes', () => {
    expect(serializeQuery({ terms: [fieldTerm('subject', 'Q3 "draft" \\ notes')] })).toBe('subject:"Q3 \\"draft\\" \\\\ notes"');
    expect(serializeQuery({ terms: [fieldTerm('in', 'Sent Items')] })).toBe('in:"Sent Items"');
    expect(serializeQuery({ terms: [fieldTerm('from', 'a(b)')] })).toBe('from:"a(b)"');
    expect(quote('x')).toBe('"x"');
  });

  it('never lets a free-text value turn into an operator, a keyword or a negation', () => {
    const text = (value: string): SearchQuery => ({ terms: [{ kind: 'text', value, negated: false, quoted: false }] });
    expect(serializeQuery(text('from:x'))).toBe('"from:x"');
    expect(serializeQuery(text('OR'))).toBe('"OR"');
    expect(serializeQuery(text('-x'))).toBe('"-x"');
    expect(serializeQuery(text('plain words'))).toBe('plain words');
    expect(parseQuery(serializeQuery(text('a OR from:b'))).terms).toEqual([{ kind: 'text', value: 'a OR from:b', negated: false, quoted: true }]);
  });

  it('writes negation as a leading -', () => {
    expect(serializeQuery({ terms: [fieldTerm('from', 'x', true)] })).toBe('-from:x');
    expect(serializeQuery({ terms: [{ kind: 'text', value: 'two words', negated: true, quoted: false }] })).toBe('-"two words"');
  });

  it('round-trips: parse(serialize(parse(s))) equals parse(s), and serialize is idempotent', () => {
    const inputs = [
      'from:x subject:y', 'from:"Ann Lee" -to:bob', 'subject:"a \\" OR from:other"', 'hello world', '"exact phrase" more',
      '-has:attachment is:read', 'after:2026-01-01 before:2026-02-01 larger:1m smaller:5m', 'in:"Sent Items" in:INBOX',
      'from:a OR from:b', 'x -(a OR b) "y z"', 'from: "spaced"', 'from:""', 'proj* helo~2 subject:recieve~',
      'NOT from:x AND y', 'foo:bar :x', '\\ "\\\\"', 'from:a\tsubject:b\n c', 'tag:Work pinned:true is:pinned',
      'subject:é from:日本', '-"phrase"', 'body:"line \\\\n"',
    ];
    for (const input of inputs) {
      const once = parseQuery(input);
      const text = serializeQuery(once);
      expect(parseQuery(text), input).toEqual(once);
      expect(canonicalize(text), input).toBe(text);
    }
  });

  it('round-trips arbitrary values through every text operator', () => {
    const values = ['plain', 'two words', 'quote " inside', 'back\\slash', 'ends with \\', '"', '\\"', '(x)', '-lead', 'OR', 'a:b', 'from:x', '  pad  '];
    for (const op of ['from', 'to', 'subject', 'body', 'in'] as const) {
      for (const value of values) {
        const text = serializeQuery({ terms: [fieldTerm(op, value)] });
        expect(parseQuery(text).terms, `${op} ${value}`).toEqual([{ kind: 'field', op, value, negated: false, quoted: text !== `${op}:${value}` }]);
      }
    }
  });
});

describe('chips', () => {
  it('maps terms to chips and pairs range bounds', () => {
    const q = parseQuery('from:x before:2026-02-01 has:attachment has:calendar is:unread is:flagged after:2026-01-01 larger:1m in:INBOX words here (a OR b)');
    expect(chipsOf(q).map((c) => [c.kind, c.terms])).toEqual([
      ['from', [0]], ['date', [1, 6]], ['attachment', [2]], ['event', [3]], ['unread', [4]], ['flagged', [5]],
      ['size', [7]], ['scope', [8]], ['text', [9]], ['expr', [10]],
    ]);
  });

  it('toggles two-state chips', () => {
    expect(serializeQuery({ terms: [toggleTerm(fieldTerm('is', 'unread'))] })).toBe('is:read');
    expect(serializeQuery({ terms: [toggleTerm(fieldTerm('is', 'flagged'))] })).toBe('is:unflagged');
    expect(serializeQuery({ terms: [toggleTerm(fieldTerm('has', 'attachment'))] })).toBe('-has:attachment');
    expect(serializeQuery({ terms: [toggleTerm(fieldTerm('has', 'attachment', true))] })).toBe('has:attachment');
  });

  it('edits, removes and adds terms in place', () => {
    const q = parseQuery('from:x subject:y');
    expect(serializeQuery(replaceTerm(q, 1, fieldTerm('subject', 'new subject')))).toBe('from:x subject:"new subject"');
    expect(serializeQuery(removeTerms(q, [0]))).toBe('subject:y');
    expect(serializeQuery(setField(q, 'from', 'z'))).toBe('from:z subject:y');
    expect(serializeQuery(setField(q, 'to', 'me'))).toBe('from:x subject:y to:me');
    expect(serializeQuery(setOptionalField(q, 'from', null))).toBe('subject:y');
  });

  it('rewrites a date range chip, dropping an empty bound', () => {
    const q = parseQuery('from:x after:2026-01-01 before:2026-02-01');
    const [, date] = chipsOf(q);
    expect(valueIn(q, date!.terms, 'before')).toBe('2026-02-01');
    expect(serializeQuery(setRange(q, date!, 'after', 'before', { low: '2025-12-01', high: '' }))).toBe('from:x after:2025-12-01');
  });
});

describe('folder scope', () => {
  const boxes = [
    { id: 'm1', name: 'Inbox', role: 'inbox' },
    { id: 'm2', name: 'Sent Items', role: 'sent' },
    { id: 'm3', name: 'Work' },
    { id: 'm4', name: 'Work' },
  ];
  it('names a mailbox by its unique name, else by id', () => {
    expect(serializeQuery(setScope(EMPTY_QUERY, boxes[1]!, boxes))).toBe('in:"Sent Items"');
    expect(serializeQuery(setScope(EMPTY_QUERY, boxes[2]!, boxes))).toBe('in:m3');
    expect(serializeQuery(setScope(parseQuery('in:Inbox x'), null, boxes))).toBe('x');
  });
  it('resolves names, ids and roles for the server', () => {
    expect(resolveScope('inbox', boxes)?.id).toBe('m1');
    expect(resolveScope('Work', boxes)).toBeNull();
    expect(serverQueryText('in:"sent items" from:a', boxes)).toBe('in:m2 from:a');
    expect(serverQueryText('from:a', boxes)).toBe('from:a');
    expect(serverQueryText('in:nowhere', boxes)).toBe('in:nowhere');
  });
});

describe('Trash search option',()=>{
 const boxes=[{id:'inbox',name:'Inbox',role:'inbox'},{id:'bin',name:'Trash',role:'trash'}];
 it('excludes Trash by default but respects explicit folder scope and the opt-in',()=>{
  expect(excludedSearchMailboxes(parseQuery('hello'),boxes)).toEqual(['bin']);
  expect(excludedSearchMailboxes(parseQuery('in:Trash hello'),boxes)).toEqual([]);
  const query=setIncludeTrash(parseQuery('hello'),true);
  expect(includesTrash(parseQuery(serializeQuery(query)))).toBe(true);
  expect(excludedSearchMailboxes(query,boxes)).toEqual([]);
  expect(serverQueryText(serializeQuery(query),boxes)).toBe('hello');
  expect(offlineQueryOf(serializeQuery(query),boxes)).toEqual({text:'hello'});
  expect(serializeQuery(setIncludeTrash(query,false))).toBe('hello');
 });
 it('keeps an OR expression grouped when adding metadata and strips only the option',()=>{
  const query=setIncludeTrash(parseQuery('from:a OR from:b'),true);
  const text=serializeQuery(query);
  expect(text).toBe('(from:a OR from:b) include:trash');
  expect(includesTrash(parseQuery(text))).toBe(true);
  expect(serverQueryText(text,boxes)).toBe('(from:a OR from:b)');
  expect(serializeQuery(setIncludeTrash(setIncludeTrash(parseQuery(text),false),true))).toBe(text);
  expect(serverQueryText('subject:"include:trash"',boxes)).toBe('subject:"include:trash"');
 });
 it('resolves an explicit offline Trash scope and supports an option-only query',()=>{
  expect(offlineQueryOf('in:Trash hello',boxes)).toEqual({inMailbox:'bin',text:'hello'});
  expect(serverQueryText('include:trash',boxes)).toBe('');
  expect(offlineQueryOf('include:trash',boxes)).toEqual({});
 });
});

it('round-trips exact sender-domain operators and preserves them for offline matching',()=>{
 const input='from-domain:"example.org"';
 expect(parseQuery(input).terms).toEqual([{kind:'field',op:'from-domain',value:'example.org',negated:false,quoted:true}]);
 expect(serializeQuery(parseQuery(input))).toBe(input);
 expect(offlineQueryOf(input)).toEqual({fromDomain:'example.org'});
});
