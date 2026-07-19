import { describe, it, expect, vi } from 'vitest';
import {
  emailGetFull,
  listMailbox,
  mailboxGet,
  parseRecipients,
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
