import { describe, it, expect } from 'vitest';
import { rulesToSieve, runsAtFor, lintSieve, tokenizeSieve, dryRun } from './sieve.ts';
import type { MailRule } from '../../api/crypto-types.ts';

function rule(over: Partial<MailRule> = {}): MailRule {
  return {
    id: '1',
    name: 'r',
    matchAll: true,
    conditions: [{ type: 'from', op: 'contains', value: 'news@example.com' }],
    actions: [{ type: 'move', value: 'INBOX/News' }],
    enabled: true,
    runsAt: 'engine',
    ...over,
  };
}

describe('rulesToSieve (mirrors mw-sieve codegen for the MailRule subset)', () => {
  it('renders a single-condition rule with a require header', () => {
    const out = rulesToSieve([rule()]);
    expect(out).toContain('require ["fileinto"];');
    expect(out).toContain('# rule: r');
    expect(out).toContain('if address :contains "from" "news@example.com" {');
    expect(out).toContain('    fileinto "INBOX/News";');
  });

  it('uses allof/anyof for multiple conditions and collects extensions', () => {
    const out = rulesToSieve([
      rule({
        matchAll: false,
        conditions: [
          { type: 'to', op: 'is', value: 'me@x' },
          { type: 'subject', op: 'contains', value: 'sale' },
        ],
        actions: [
          { type: 'tag', value: 'promo' },
          { type: 'stop', value: null },
        ],
      }),
    ]);
    expect(out).toContain('require ["imap4flags"];');
    expect(out).toContain('if anyof (address :is "to" "me@x", header :contains "subject" "sale") {');
    expect(out).toContain('    addflag "promo";');
    expect(out).toContain('    stop;');
  });

  it('emits unguarded actions for a conditionless rule', () => {
    const out = rulesToSieve([rule({ conditions: [], actions: [{ type: 'archive', value: null }] })]);
    expect(out).toContain('# rule: r\nfileinto "Archive";');
  });

  it('skips disabled rules and escapes strings', () => {
    expect(rulesToSieve([rule({ enabled: false })])).toBe('');
    const out = rulesToSieve([rule({ conditions: [{ type: 'subject', op: 'is', value: 'a"b\\c' }], actions: [{ type: 'stop', value: null }] })]);
    expect(out).toContain('"a\\"b\\\\c"');
  });

  it('generated Sieve lints clean', () => {
    expect(lintSieve(rulesToSieve([rule()]))).toEqual([]);
  });
});

describe('runsAtFor', () => {
  it('is server-sieve for a fully Sieve-expressible rule', () => {
    expect(runsAtFor({ conditions: [{ type: 'from', op: 'is', value: 'x' }], actions: [{ type: 'move', value: 'A' }] })).toBe('server-sieve');
  });
  it('is engine for thread/suppressNotify (no Sieve surface)', () => {
    expect(runsAtFor({ conditions: [{ type: 'thread', op: 'is', value: 't' }], actions: [{ type: 'stop', value: null }] })).toBe('engine');
    expect(runsAtFor({ conditions: [], actions: [{ type: 'suppressNotify', value: null }] })).toBe('engine');
  });
});

describe('lintSieve', () => {
  it('flags an unbalanced brace', () => {
    expect(lintSieve('if x {\n keep;')).toContainEqual(expect.stringContaining('unclosed `{`'));
  });
  it('flags an unterminated string', () => {
    expect(lintSieve('fileinto "Archive;')).toContainEqual(expect.stringContaining('unterminated string'));
  });
  it('flags a command missing its require', () => {
    expect(lintSieve('fileinto "Spam";')).toContainEqual(expect.stringContaining('not in `require`'));
  });
  it('accepts a covered command', () => {
    expect(lintSieve('require ["fileinto"];\nfileinto "Spam";')).toEqual([]);
  });
});

describe('tokenizeSieve', () => {
  it('classifies keywords, tags, strings, and comments', () => {
    const kinds = tokenizeSieve('# c\nif address :is "from" "x" { stop; }').map((t) => t.kind);
    expect(kinds).toContain('comment');
    expect(kinds).toContain('keyword');
    expect(kinds).toContain('tag');
    expect(kinds).toContain('string');
    expect(kinds).toContain('punct');
  });
  it('reassembles the source byte-for-byte', () => {
    const src = 'require ["fileinto"];\n# rule: r\nif address :is "from" "x@y" {\n    fileinto "A";\n}\n';
    expect(tokenizeSieve(src).map((t) => t.text).join('')).toBe(src);
  });
});

describe('dryRun (mirrors eval.rs semantics for the MailRule subset)', () => {
  const rules: MailRule[] = [
    rule({ id: '1', name: 'from-news', conditions: [{ type: 'from', op: 'contains', value: 'news@' }], actions: [{ type: 'move', value: 'News' }, { type: 'stop', value: null }] }),
    rule({ id: '2', name: 'catch-all', conditions: [], actions: [{ type: 'tag', value: 'seen' }] }),
  ];

  it('matches contains case-insensitively and reports actions', () => {
    const res = dryRun(rules, { from: 'NEWS@example.com', to: '', subject: '' });
    expect(res[0]!.matched).toBe(true);
    expect(res[0]!.actions).toEqual(['move to News', 'stop processing']);
  });

  it('short-circuits rules after a matched stop', () => {
    const res = dryRun(rules, { from: 'news@x', to: '', subject: '' });
    expect(res[1]!.shortCircuited).toBe(true);
    expect(res[1]!.matched).toBe(false);
  });

  it('evaluates later rules when the first does not match', () => {
    const res = dryRun(rules, { from: 'someone@else', to: '', subject: '' });
    expect(res[0]!.matched).toBe(false);
    expect(res[1]!.matched).toBe(true); // unconditional catch-all fires
    expect(res[1]!.actions).toEqual(['tag seen']);
  });
});
