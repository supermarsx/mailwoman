import { describe, it, expect, vi } from 'vitest';
import {
  draftCreateSpec,
  emailGetFull,
  fetchThreadingHeaders,
  invalidRecipients,
  isMailbox,
  listMailbox,
  mailboxGet,
  parseRecipients,
  parseThreadingHeaders,
  responseFor,
  sendEnvelope,
  uploadBlob,
} from './jmap.ts';
import { CAP_MAIL, CAP_SUBMISSION, type Invocation, type JmapResponse } from './jmap-types.ts';

describe('mailboxGet', () => {
  it('builds a Mailbox/get for all ids', () => {
    const r = mailboxGet('acct1');
    expect(r.using).toContain(CAP_MAIL);
    expect(r.methodCalls).toEqual([['Mailbox/get', { accountId: 'acct1', ids: null }, 'c0']]);
  });
});

describe('listMailbox', () => {
  it('builds Email/query + Email/get chained by result reference', () => {
    const r = listMailbox('acct1', 'mbox9', 25);
    expect(r.methodCalls).toHaveLength(2);

    const [query, get] = r.methodCalls as [Invocation, Invocation];
    expect(query[0]).toBe('Email/query');
    expect(query[1]).toMatchObject({
      accountId: 'acct1',
      filter: { inMailbox: 'mbox9' },
      limit: 25,
    });
    expect(query[1]['sort']).toEqual([{ property: 'receivedAt', isAscending: false }]);

    expect(get[0]).toBe('Email/get');
    expect(get[1]['#ids']).toEqual({ resultOf: 'q', name: 'Email/query', path: '/ids' });
    expect(get[1]['properties']).toContain('subject');
    expect(get[1]['properties']).toContain('preview');
  });
});

describe('emailGetFull', () => {
  it('requests body values with bounded size', () => {
    const r = emailGetFull('acct1', 'e5');
    const [get] = r.methodCalls as [Invocation];
    expect(get[1]).toMatchObject({
      accountId: 'acct1',
      ids: ['e5'],
      fetchHTMLBodyValues: true,
    });
    expect(get[1]['properties']).toContain('htmlBody');
    expect(get[1]['properties']).toContain('bodyValues');
    expect(typeof get[1]['maxBodyValueBytes']).toBe('number');
  });
});

describe('parseRecipients', () => {
  it('splits on comma/semicolon and trims', () => {
    expect(parseRecipients('a@x.org, b@y.org ; c@z.org')).toEqual([
      { name: null, email: 'a@x.org' },
      { name: null, email: 'b@y.org' },
      { name: null, email: 'c@z.org' },
    ]);
  });
  it('drops empties', () => {
    expect(parseRecipients(' , ')).toEqual([]);
  });
  it('separates the display name from the addr-spec', () => {
    // The form the contact autocomplete inserts (`suggestionDisplay`).
    expect(parseRecipients('Alice Example <alice@x.org>, bob@y.org, <carol@z.org>')).toEqual([
      { name: 'Alice Example', email: 'alice@x.org' },
      { name: null, email: 'bob@y.org' },
      { name: null, email: 'carol@z.org' },
    ]);
  });
  it('does not split inside a quoted display name, and unquotes it', () => {
    expect(parseRecipients('"Doe, Jane; PhD" <jane@z.org>; "Say \\"hi\\"" <q@z.org>')).toEqual([
      { name: 'Doe, Jane; PhD', email: 'jane@z.org' },
      { name: 'Say "hi"', email: 'q@z.org' },
    ]);
  });
  it('leaves a token it cannot read as one address, so validation refuses it whole', () => {
    expect(parseRecipients('Alice <alice@x.org> extra')).toEqual([
      { name: null, email: 'Alice <alice@x.org> extra' },
    ]);
    expect(invalidRecipients('Alice <alice@x.org> extra, bob@y.org, carol')).toEqual([
      'Alice <alice@x.org> extra',
      'carol',
    ]);
  });
});

describe('isMailbox', () => {
  // The same tables as `crates/mw-smtp/src/addr.rs` (`validate_mailbox`), which
  // is what refuses the draft server-side.
  it('accepts what the server accepts', () => {
    for (const ok of [
      'a@b',
      'bob@example.test',
      'first.last+tag@sub.example.test',
      'møt@example.com',
      'user@[192.0.2.1]',
      "o'brien@example.test",
    ]) {
      expect(isMailbox(ok), ok).toBe(true);
    }
  });
  it('refuses what the server refuses', () => {
    for (const bad of [
      '',
      'bob',
      '@example.test',
      'bob@',
      'a@b@c',
      'x@example.test>\r\nRCPT TO:<victim@example.test',
      'x@example.test\nDATA',
      'x@example.\0test',
      'x@example.test\u007f',
      'x@example.test\u0085',
      'x@example.test\u2028',
      'x @example.test',
      '\tx@example.test',
      'a@b>',
      '<a@b',
      '"a b"@example.test',
      'a\\@b@example.test',
      'a@b,c@d',
      'a@b;c@d',
      'a(comment)@b',
      `${'a'.repeat(310)}@example.test`,
    ]) {
      expect(isMailbox(bad), JSON.stringify(bad)).toBe(false);
    }
  });
});

describe('sendEnvelope', () => {
  it('creates a draft and submits it via creation-id back-reference in one request', () => {
    const r = sendEnvelope('acct1', {
      from: { name: 'Me', email: 'me@example.org' },
      to: 'you@example.org',
      subject: 'Hi',
      htmlBody: '<p>hello</p>',
      draftMailboxId: 'drafts1',
      sentMailboxId: 'sent1',
    });

    expect(r.using).toContain(CAP_SUBMISSION);
    const [emailSet, submissionSet] = r.methodCalls as [Invocation, Invocation];

    expect(emailSet[0]).toBe('Email/set');
    const create = emailSet[1]['create'] as Record<string, Record<string, unknown>>;
    expect(create['draft']).toBeDefined();
    expect(create['draft']!['mailboxIds']).toEqual({ drafts1: true });
    expect(create['draft']!['subject']).toBe('Hi');
    expect(create['draft']!['to']).toEqual([{ name: null, email: 'you@example.org' }]);

    expect(submissionSet[0]).toBe('EmailSubmission/set');
    const subCreate = submissionSet[1]['create'] as Record<string, Record<string, unknown>>;
    // Back-reference to the draft created in the SAME request.
    expect(subCreate['send']!['emailId']).toBe('#draft');
    expect(submissionSet[1]['onSuccessUpdateEmail']).toMatchObject({
      '#send': { mailboxIds: { sent1: true } },
    });
  });

  it('omits onSuccessUpdateEmail when there is no Sent mailbox', () => {
    const r = sendEnvelope('acct1', {
      from: { name: null, email: 'me@example.org' },
      to: 'you@example.org',
      subject: 'Hi',
      htmlBody: '<p>hello</p>',
      draftMailboxId: 'drafts1',
    });
    const [, submissionSet] = r.methodCalls as [Invocation, Invocation];
    expect(submissionSet[1]['onSuccessUpdateEmail']).toBeUndefined();
  });

  it('a recipient with a display name is addressed by name in To and by bare address in rcptTo', () => {
    const r = sendEnvelope('acct1', {
      from: { name: 'Me', email: 'me@example.org' },
      to: 'Alice Example <alice@example.org>, bob@example.org',
      subject: 'Hi',
      htmlBody: '<p>hello</p>',
      draftMailboxId: 'drafts1',
      sentMailboxId: 'sent1',
      holdSeconds: 10,
    });
    expect(r).toEqual({
      using: ['urn:ietf:params:jmap:core', CAP_MAIL, CAP_SUBMISSION],
      methodCalls: [
        [
          'Email/set',
          {
            accountId: 'acct1',
            create: {
              draft: {
                mailboxIds: { drafts1: true },
                keywords: { $draft: true, $seen: true },
                from: [{ name: 'Me', email: 'me@example.org' }],
                to: [
                  { name: 'Alice Example', email: 'alice@example.org' },
                  { name: null, email: 'bob@example.org' },
                ],
                subject: 'Hi',
                htmlBody: [{ partId: 'body', type: 'text/html' }],
                bodyValues: { body: { value: '<p>hello</p>' } },
              },
            },
          },
          'set',
        ],
        [
          'EmailSubmission/set',
          {
            accountId: 'acct1',
            create: {
              send: {
                emailId: '#draft',
                envelope: {
                  mailFrom: { email: 'me@example.org' },
                  rcptTo: [{ email: 'alice@example.org' }, { email: 'bob@example.org' }],
                },
                mailwomanHoldSeconds: 10,
              },
            },
            onSuccessUpdateEmail: { '#send': { mailboxIds: { sent1: true }, 'keywords/$draft': null } },
          },
          'submit',
        ],
      ],
    });
  });
});

describe('sendEnvelope: Cc, Bcc and reply threading', () => {
  it('puts cc, bcc, inReplyTo and references on the draft and every recipient once in rcptTo', () => {
    const r = sendEnvelope('acct1', {
      from: { name: 'Me', email: 'me@example.org' },
      to: 'Alice <alice@example.org>',
      cc: 'bob@example.org, ALICE@example.org',
      bcc: 'Carol <carol@example.org>',
      inReplyTo: ['orig@example.org'],
      references: ['root@example.org', 'orig@example.org'],
      subject: 'Re: Hi',
      htmlBody: '<p>hello</p>',
      draftMailboxId: 'drafts1',
      holdSeconds: 10,
    });
    expect(r).toEqual({
      using: ['urn:ietf:params:jmap:core', CAP_MAIL, CAP_SUBMISSION],
      methodCalls: [
        [
          'Email/set',
          {
            accountId: 'acct1',
            create: {
              draft: {
                mailboxIds: { drafts1: true },
                keywords: { $draft: true, $seen: true },
                from: [{ name: 'Me', email: 'me@example.org' }],
                to: [{ name: 'Alice', email: 'alice@example.org' }],
                cc: [
                  { name: null, email: 'bob@example.org' },
                  { name: null, email: 'ALICE@example.org' },
                ],
                bcc: [{ name: 'Carol', email: 'carol@example.org' }],
                inReplyTo: ['orig@example.org'],
                references: ['root@example.org', 'orig@example.org'],
                subject: 'Re: Hi',
                htmlBody: [{ partId: 'body', type: 'text/html' }],
                bodyValues: { body: { value: '<p>hello</p>' } },
              },
            },
          },
          'set',
        ],
        [
          'EmailSubmission/set',
          {
            accountId: 'acct1',
            create: {
              send: {
                emailId: '#draft',
                envelope: {
                  mailFrom: { email: 'me@example.org' },
                  rcptTo: [
                    { email: 'alice@example.org' },
                    { email: 'bob@example.org' },
                    { email: 'carol@example.org' },
                  ],
                },
                mailwomanHoldSeconds: 10,
              },
            },
          },
          'submit',
        ],
      ],
    });
  });

  it('adds none of the four properties when they are empty', () => {
    const spec = draftCreateSpec({
      from: { name: null, email: 'me@example.org' },
      to: 'you@example.org',
      cc: ' ',
      bcc: '',
      inReplyTo: [],
      references: [],
      subject: 'Hi',
      htmlBody: '<p>x</p>',
      draftMailboxId: 'drafts1',
    });
    expect(spec).toEqual({
      mailboxIds: { drafts1: true },
      keywords: { $draft: true, $seen: true },
      from: [{ name: null, email: 'me@example.org' }],
      to: [{ name: null, email: 'you@example.org' }],
      subject: 'Hi',
      htmlBody: [{ partId: 'body', type: 'text/html' }],
      bodyValues: { body: { value: '<p>x</p>' } },
    });
  });
});

describe('threading headers of a raw message', () => {
  const RAW =
    'Received: by mx.example.org; Mon, 1 Jan 2026 00:00:00 +0000\r\n' +
    'Message-ID: <orig@example.org>\r\n' +
    'References: <root@example.org>\r\n <mid@example.org>\r\n' +
    'In-Reply-To: <mid@example.org>\r\n' +
    'Subject: hello\r\n' +
    '\r\n' +
    'Message-ID: <in-the-body@example.org>\r\nReferences: <body-ref@example.org>\r\n';

  it('reads Message-ID and the folded References, and nothing from the body', () => {
    expect(parseThreadingHeaders(RAW)).toEqual({
      messageId: 'orig@example.org',
      references: ['root@example.org', 'mid@example.org'],
    });
  });

  it('falls back to In-Reply-To when there is no References header', () => {
    expect(parseThreadingHeaders('message-id: <b@x>\nIn-Reply-To: <a@x> (their note)\n\nbody')).toEqual({
      messageId: 'b@x',
      references: ['a@x'],
    });
  });

  it('does not take several In-Reply-To ids for a reference chain', () => {
    expect(parseThreadingHeaders('Message-ID: <c@x>\nIn-Reply-To: <a@x> <b@x>\n\n')).toEqual({
      messageId: 'c@x',
      references: [],
    });
  });

  it('reports no id for a message without a Message-ID', () => {
    expect(parseThreadingHeaders('Subject: x\r\nX-Message-ID: <not-this@x>\r\n\r\n')).toEqual({
      messageId: null,
      references: [],
    });
  });

  it('stops reading the download once the header block has ended', async () => {
    const enc = new TextEncoder();
    const chunks = [RAW.slice(0, 40), RAW.slice(40), 'x'.repeat(1000), 'y'.repeat(1000)];
    let pulled = 0;
    let cancelled = false;
    const body = new ReadableStream<Uint8Array>({
      pull(controller) {
        const next = chunks[pulled];
        pulled += 1;
        if (next === undefined) controller.close();
        else controller.enqueue(enc.encode(next));
      },
      cancel() {
        cancelled = true;
      },
    }, { highWaterMark: 0 });
    const fetcher = vi.fn(async (_url: string, _init?: RequestInit) => new Response(body, { status: 200 }));
    const out = await fetchThreadingHeaders('/jmap/download/acct1/m1/message.eml', fetcher);
    expect(fetcher).toHaveBeenCalledWith('/jmap/download/acct1/m1/message.eml');
    expect(out).toEqual({ messageId: 'orig@example.org', references: ['root@example.org', 'mid@example.org'] });
    expect(cancelled).toBe(true);
    // The two body chunks after the header block were never asked for.
    expect(pulled).toBeLessThanOrEqual(3);
  });

  it('throws on a refused download', async () => {
    const fetcher = vi.fn(async (_url: string, _init?: RequestInit) => new Response('no', { status: 404 }));
    await expect(fetchThreadingHeaders('/x', fetcher)).rejects.toThrow('message download failed with 404');
  });
});

describe('uploadBlob', () => {
  const okUpload = (over: Record<string, unknown> = {}): Response =>
    new Response(
      JSON.stringify({ accountId: 'acct1', blobId: 'Uabc123', type: 'text/plain', size: 5, ...over }),
      { status: 200, headers: { 'content-type': 'application/json' } },
    );

  it('POSTs the file to the account-substituted uploadUrl with the file content-type', async () => {
    const fetcher = vi.fn(async (_url: string, _init?: RequestInit) => okUpload());
    const file = new File(['hello'], 'note.txt', { type: 'text/plain' });
    const out = await uploadBlob('/jmap/upload/{accountId}', 'acct1', file, fetcher);

    expect(fetcher).toHaveBeenCalledTimes(1);
    const [url, init] = fetcher.mock.calls[0]!;
    expect(url).toBe('/jmap/upload/acct1');
    expect(init!.method).toBe('POST');
    expect((init!.headers as Record<string, string>)['content-type']).toBe('text/plain');
    expect(init!.body).toBe(file);
    expect(out).toEqual({ accountId: 'acct1', blobId: 'Uabc123', type: 'text/plain', size: 5 });
  });

  it('defaults the content-type to application/octet-stream when the file reports none', async () => {
    const fetcher = vi.fn(async (_url: string, _init?: RequestInit) =>
      okUpload({ type: 'application/octet-stream' }),
    );
    const file = new File([new Uint8Array([1, 2, 3])], 'blob.bin', { type: '' });
    await uploadBlob('/jmap/upload/{accountId}', 'acct1', file, fetcher);
    const init = fetcher.mock.calls[0]![1]!;
    expect((init.headers as Record<string, string>)['content-type']).toBe('application/octet-stream');
  });

  it('url-encodes the account id in the upload URL', async () => {
    const fetcher = vi.fn(async (_url: string, _init?: RequestInit) => okUpload());
    await uploadBlob('/jmap/upload/{accountId}', 'a b', new File(['x'], 'x.txt'), fetcher);
    expect(fetcher.mock.calls[0]![0]).toBe('/jmap/upload/a%20b');
  });

  it('throws with the status on a non-2xx response', async () => {
    const fetcher = vi.fn(async (_url: string, _init?: RequestInit) => new Response('too big', { status: 413 }));
    const file = new File(['x'], 'x.txt', { type: 'text/plain' });
    await expect(uploadBlob('/jmap/upload/{accountId}', 'acct1', file, fetcher)).rejects.toThrow(/413/);
  });
});

describe('responseFor', () => {
  const res: JmapResponse = {
    methodResponses: [
      ['Mailbox/get', { accountId: 'a', list: [] }, 'c0'],
      ['error', { type: 'unknownMethod', description: 'nope' }, 'bad'],
    ],
    sessionState: 's1',
  };

  it('returns the args for a matching call id', () => {
    expect(responseFor(res, 'c0')).toMatchObject({ accountId: 'a' });
  });
  it('throws on a method error response', () => {
    expect(() => responseFor(res, 'bad')).toThrow(/unknownMethod/);
  });
  it('throws when the call id is absent', () => {
    expect(() => responseFor(res, 'missing')).toThrow(/no method response/);
  });
});
