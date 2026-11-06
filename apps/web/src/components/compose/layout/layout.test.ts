import { expect, it } from 'vitest';
import { COLUMN_PRESETS, buildColumnsHtml, columnTables, equalWidths, mergeColumns, normaliseSpec, reflowColumns, wrapContentWidth } from './columns.ts';
import { THEMES, applyTheme, currentTheme, removeTheme, safeHex, themeWrapper } from './themes.ts';
import { resolveSentFolder, type SentFolderInput } from './sentFolder.ts';
import { DEFAULT_GATES, applyForced, buildHeaders, isDeniedHeader, rfc5322Date, validateCustomHeader } from './messageHeaders.ts';

const parse = (html: string): HTMLElement => { const d = document.createElement('div'); d.innerHTML = html; return d; };

it('builds mail-safe column tables for every preset', () => {
  for (const [name, widths] of Object.entries(COLUMN_PRESETS)) {
    const root = parse(buildColumnsHtml({ widths }));
    const t = root.querySelector('table')!;
    expect(t.getAttribute('role')).toBe('presentation');
    const tds = t.querySelectorAll(':scope > tbody > tr > td');
    expect(tds.length, name).toBe(widths.length);
    if (widths.length > 1) {
      tds.forEach((td, i) => { expect(td.getAttribute('valign')).toBe('top'); expect(td.getAttribute('width')).toBe(`${widths[i]}%`); expect(td.getAttribute('style')).toContain('max-width:'); });
      expect(t.getAttribute('style')).toContain('max-width:600px');
    }
    expect(buildColumnsHtml({ widths })).not.toMatch(/<script|display:flex|display:grid/);
  }
});

it('keeps widths summing to 100 and clamps count, gutter and width', () => {
  expect(equalWidths(3).reduce((a, b) => a + b, 0)).toBe(100);
  expect(equalWidths(9).length).toBe(4);
  expect(normaliseSpec({ widths: [10, 10, 10, 10, 10] }).widths.length).toBe(4);
  expect(normaliseSpec({ widths: [10, 10] }).widths.reduce((a, b) => a + b, 0)).toBe(100);
  expect(normaliseSpec({ gutter: 999, width: 5 })).toMatchObject({ gutter: 48, width: 240 });
});

it('merge folds cell content into one column; reflow keeps typed text', () => {
  const root = parse(buildColumnsHtml({ widths: [50, 50] }, ['one', 'two']));
  expect(columnTables(root).length).toBe(1);
  reflowColumns(columnTables(root)[0]!, { widths: [30, 70] });
  expect(root.textContent).toContain('one');
  const cells = root.querySelectorAll('td');
  expect(cells[0]!.getAttribute('width')).toBe('30%');
  mergeColumns(columnTables(root)[0]!);
  expect(root.querySelector('table')).toBeNull();
  expect(root.textContent).toBe('onetwo');
});

it('wraps content to a max width, centred or left', () => {
  const c = parse(wrapContentWidth('<p>x</p>', 640, 'center')).querySelector('table')!;
  expect(c.getAttribute('style')).toContain('max-width:640px');
  expect(c.getAttribute('align')).toBe('center');
  expect(parse(wrapContentWidth('<p>x</p>', 640, 'left')).querySelector('table')!.getAttribute('align')).toBeNull();
  expect(wrapContentWidth('<p>x</p>', 'full')).toBe('<p>x</p>');
});

it('themes apply and remove idempotently without losing content', () => {
  const body = parse('<p>hello</p>');
  const news = THEMES.find((t) => t.id === 'newsletter')!;
  applyTheme(body, news);
  applyTheme(body, news);
  expect(body.querySelectorAll('table[data-mw-theme]').length).toBe(1);
  expect(body.innerHTML).toContain('max-width:600px');
  expect(body.innerHTML).toContain('#eef1f5');
  applyTheme(body, THEMES.find((t) => t.id === 'modern')!);
  expect(currentTheme(body)!.getAttribute('data-mw-theme')).toBe('modern');
  expect(removeTheme(body)).toBe(true);
  expect(removeTheme(body)).toBe(false);
  expect(body.innerHTML).toBe('<p>hello</p>');
});

it('a theme keeps nested column tables in the content cell', () => {
  const body = parse(buildColumnsHtml({ widths: [50, 50] }, ['a', 'b']));
  applyTheme(body, THEMES[1]!);
  expect(body.querySelectorAll('table[data-mw-columns]').length).toBe(1);
  removeTheme(body);
  expect(body.querySelector('table')!.hasAttribute('data-mw-columns')).toBe(true);
});

it('page colour is sent and sanitised; only valid hex is accepted', () => {
  expect(safeHex('#ABC')).toBe('#abc');
  expect(safeHex('red;background:url(x)')).toBeUndefined();
  const html = themeWrapper(THEMES[0]!, '<p>x</p>', { pageColor: '#112233' });
  expect(html).toContain('bgcolor="#112233"');
  expect(themeWrapper(THEMES[0]!, '', { pageColor: 'javascript:x' })).not.toContain('javascript');
  // The reader-side dark background is view-only and never part of the sent markup.
  expect(html).not.toContain('data-mw-view');
});

const base: SentFolderInput = { policy: { mode: 'default', skipCopy: false }, recipients: ['a@client.example.com'], subject: 'Quote 12', writable: new Set(['sent', 'clients', 'quotes', 'work']), sent: 'sent' };
it('resolves the sent folder for every mode with fallbacks', () => {
  expect(resolveSentFolder(base)).toMatchObject({ folder: 'sent', source: 'default' });
  expect(resolveSentFolder({ ...base, policy: { mode: 'fixed', fixedId: 'work', skipCopy: false } })).toMatchObject({ folder: 'work', source: 'fixed' });
  expect(resolveSentFolder({ ...base, identityId: 'i1', policy: { mode: 'byIdentity', perIdentity: { i1: 'work' }, skipCopy: false } })).toMatchObject({ folder: 'work', source: 'identity' });
  const rules = [{ recipientDomain: 'example.com', folder: 'clients' }, { subjectContains: 'quote', folder: 'quotes' }];
  expect(resolveSentFolder({ ...base, policy: { mode: 'byRecipient', rules, skipCopy: false } })).toMatchObject({ folder: 'clients', source: 'recipient' });
  expect(resolveSentFolder({ ...base, recipients: ['x@other.org'], policy: { mode: 'bySubjectRule', rules, skipCopy: false } })).toMatchObject({ folder: 'quotes', source: 'subject' });
  // Rule order decides precedence.
  expect(resolveSentFolder({ ...base, policy: { mode: 'bySubjectRule', rules: [...rules].reverse(), skipCopy: false } })).toMatchObject({ folder: 'quotes' });
});
it('per-message choice wins, skipCopy policy wins over everything, missing folders fall back to Sent', () => {
  const policy = { mode: 'fixed' as const, fixedId: 'work', skipCopy: false };
  expect(resolveSentFolder({ ...base, policy, choice: { kind: 'folder', folder: 'quotes' } })).toMatchObject({ folder: 'quotes', source: 'message' });
  expect(resolveSentFolder({ ...base, policy, choice: { kind: 'none' } })).toEqual({ skip: true, reason: 'message' });
  expect(resolveSentFolder({ ...base, policy: { ...policy, skipCopy: true }, choice: { kind: 'folder', folder: 'quotes' } })).toEqual({ skip: true, reason: 'policy' });
  expect(resolveSentFolder({ ...base, policy: { mode: 'fixed', fixedId: 'gone', skipCopy: false } })).toEqual({ skip: false, folder: 'sent', source: 'fixed', fellBackFrom: 'gone' });
  expect(resolveSentFolder({ ...base, policy: { mode: 'byRecipient', rules: [{ recipientDomain: 'example.com', folder: 'work' }], skipCopy: false }, writable: new Set(['sent']) })).toMatchObject({ folder: 'sent', fellBackFrom: 'work' });
});

it('serialises RFC date formats and importance headers', () => {
  expect(rfc5322Date(new Date(Date.UTC(2026, 10, 3, 17, 0, 0)))).toBe('Tue, 03 Nov 2026 17:00:00 +0000');
  const h = Object.fromEntries(buildHeaders({ importance: 'high', sensitivity: 'private', replyBy: new Date(Date.UTC(2026, 10, 3, 17)), expires: new Date(Date.UTC(2026, 11, 1)) }).headers);
  expect(h).toMatchObject({ Importance: 'high', 'X-Priority': '1', 'X-MSMail-Priority': 'High', Sensitivity: 'private', 'Reply-By': 'Tue, 03 Nov 2026 17:00:00 +0000' });
  expect(h['Expires']).toBe(h['Expiry-Date']);
  expect(buildHeaders({ importance: 'low' }, DEFAULT_GATES, false).headers).toEqual([['Importance', 'low']]);
  expect(buildHeaders({ importance: 'normal', sensitivity: 'normal' }).headers).toEqual([]);
});

it('validates receipts, supersedes and gates', () => {
  expect(buildHeaders({ readReceiptTo: 'me@example.com' }).headers).toEqual([['Disposition-Notification-To', 'me@example.com']]);
  expect(buildHeaders({ readReceiptTo: 'x@y.com\r\nBcc: z@z.com' }).errors[0]!.field).toBe('readReceiptTo');
  expect(buildHeaders({ returnReceiptTo: 'me@example.com' }).errors[0]).toEqual({ field: 'returnReceiptTo', error: 'denied' });
  expect(buildHeaders({ returnReceiptTo: 'me@example.com' }, { ...DEFAULT_GATES, allowLegacyReturnReceipt: true }).headers[0]![0]).toBe('Return-Receipt-To');
  expect(buildHeaders({ supersedes: '<a@b>' }).headers).toEqual([['Supersedes', '<a@b>']]);
  expect(buildHeaders({ supersedes: 'a@b' }).errors[0]!.error).toBe('invalid');
  expect(buildHeaders({ autoSubmitted: 'auto-replied' }).headers).toEqual([]);
  expect(buildHeaders({ autoSubmitted: 'auto-replied' }, { ...DEFAULT_GATES, allowAutoSubmitted: true }).headers).toEqual([['Auto-Submitted', 'auto-replied']]);
  expect(buildHeaders({ inReplyTo: '<a@b>' }).errors[0]!.field).toBe('threading');
  expect(buildHeaders({ inReplyTo: '<a@b>', references: ['<x@y>', 'bad'] }, { ...DEFAULT_GATES, allowThreadingEdit: true }).headers).toEqual([['In-Reply-To', '<a@b>'], ['References', '<x@y>']]);
});

it('custom headers: deny-list, token names, CRLF injection, duplicates and the 20 cap', () => {
  for (const n of ['From', 'to', 'Cc', 'BCC', 'Subject', 'Date', 'Message-ID', 'Content-Type', 'content-transfer-encoding', 'MIME-Version', 'Received', 'Return-Path', 'Sender', 'X-Mailwoman-Id']) expect(isDeniedHeader(n)).toBe(true);
  expect(validateCustomHeader({ name: 'X-Ticket', value: '42' })).toBeNull();
  expect(validateCustomHeader({ name: 'X Bad', value: '1' })).toBe('name');
  expect(validateCustomHeader({ name: 'X:Bad', value: '1' })).toBe('name');
  expect(validateCustomHeader({ name: 'X-A', value: 'ok\r\nBcc: evil@x.com' })).toBe('crlf');
  expect(validateCustomHeader({ name: 'X-A\nBcc', value: 'v' })).toBe('crlf');
  expect(validateCustomHeader({ name: 'X-A', value: 'x'.repeat(2000) })).toBe('value');
  const gates = { ...DEFAULT_GATES, allowCustom: true };
  expect(buildHeaders({ custom: [{ name: 'X-A', value: '1' }] }).errors[0]!.error).toBe('denied');
  const many = Array.from({ length: 25 }, (_, i) => ({ name: `X-H${i}`, value: 'v' }));
  const r = buildHeaders({ custom: many }, gates);
  expect(r.headers.length).toBe(20);
  expect(r.errors.length).toBe(5);
  expect(buildHeaders({ custom: [{ name: 'X-A', value: '1' }, { name: 'x-a', value: '2' }] }, gates).errors[0]!.error).toBe('duplicate');
});

it('forced headers override user values and cannot target denied names', () => {
  const out = applyForced([['Sensitivity', 'personal']], ['Sensitivity=company-confidential', 'From=evil@x.com', 'X-Policy=a\r\nBcc: z', 'X-Tag=ok', 'broken']);
  expect(out).toEqual([['Sensitivity', 'company-confidential'], ['X-Tag', 'ok']]);
});

it('merges columns tables even after an editor dropped the marker', async () => {
  const { mergeColumnsInHtml, isColumnsLike } = await import('./columns.ts');
  const html = buildColumnsHtml({ widths: [50, 50] }, ['one', 'two']).replace(' data-mw-columns="2"', '');
  const t = parse(html).querySelector('table')!;
  expect(isColumnsLike(t)).toBe(true);
  const out = mergeColumnsInHtml(`<p>before</p>${html}<table><tr><td>a</td><td>b</td></tr></table>`);
  expect(out.merged).toBe(1);
  expect(out.html).toContain('one');
  expect(out.html).toContain('<table><tbody><tr><td>a</td>');
});

it('parses the sent-folder settings and maps roles to writable mailbox ids', async () => {
  const { parseSentFolderPolicy, writableFolders } = await import('./sentFolder.ts');
  const p = parseSentFolderPolicy({ mode: 'bySubjectRule', fixedId: 'work', perIdentity: ['id1=work', 'broken', '=x'], rules: ['domain:example.com=clients', 'subject:Invoice = billing', 'nonsense=x', 'domain:x.org='], skipCopy: true });
  expect(p).toEqual({ mode: 'bySubjectRule', fixedId: 'work', perIdentity: { id1: 'work' }, rules: [{ recipientDomain: 'example.com', folder: 'clients' }, { subjectContains: 'Invoice', folder: 'billing' }], skipCopy: true });
  expect(parseSentFolderPolicy({ mode: 'evil' }).mode).toBe('default');
  const w = writableFolders([{ id: 'm1', role: 'sent' }, { id: 'm2', role: null, myRights: { mayAddItems: false } }, { id: 'm3', role: 'archive' }]);
  expect(w.get('sent')).toBe('m1');
  expect(w.has('m2')).toBe(false);
  expect(w.get('archive')).toBe('m3');
  // The resolved role becomes a mailbox id through the same map.
  expect(resolveSentFolder({ ...base, writable: new Set(w.keys()), policy: { mode: 'fixed', fixedId: 'archive', skipCopy: false } })).toMatchObject({ folder: 'archive' });
});

it('options survive a draft round trip and drop what is malformed', async () => {
  const { serializeHeaderOptions, parseHeaderOptions } = await import('./messageHeaders.ts');
  const when = new Date(Date.UTC(2026, 10, 3, 17));
  const o = { sensitivity: 'private' as const, replyBy: when, expires: when, supersedes: '<a@b>', returnReceiptTo: 'me@example.com', autoSubmitted: 'auto-replied' as const, custom: [{ name: 'X-A', value: '1' }] };
  expect(parseHeaderOptions(serializeHeaderOptions(o))).toEqual(o);
  expect(parseHeaderOptions('not json')).toEqual({});
  expect(parseHeaderOptions(JSON.stringify({ sensitivity: 'evil', supersedes: 'x', replyBy: 'nope', custom: [{ name: 'From', value: 'x' }, { name: 'X-B', value: 'a\r\nBcc: z' }, 5] }))).toEqual({ custom: [] });
});
