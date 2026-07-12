import { describe, it, expect } from 'vitest';
import { CAP_CORE, CAP_MAIL, type Email, type Invocation } from '../api/jmap-types.ts';
import { viewerKindFor } from '../contracts/viewer.ts';
import {
  attachmentsQuery,
  buildDownloadUrl,
  categoryOf,
  filterAttachments,
  formatSize,
  parseAttachmentQuery,
  parseAttachments,
  parseSize,
  type AttachmentItem,
} from './attachments.ts';

describe('viewerKindFor — MIME dispatch (frozen §2.4)', () => {
  it('routes each family to its viewer', () => {
    expect(viewerKindFor('application/pdf')).toBe('pdf');
    expect(viewerKindFor('image/png')).toBe('image');
    expect(viewerKindFor('image/jpeg')).toBe('image');
    expect(viewerKindFor('audio/mpeg')).toBe('audio');
    expect(viewerKindFor('video/mp4')).toBe('video');
    expect(viewerKindFor('text/plain')).toBe('text');
    expect(viewerKindFor('application/octet-stream')).toBe('unsupported');
    expect(viewerKindFor('')).toBe('unsupported');
  });

  it('categoryOf groups by the same routing (unsupported → other)', () => {
    expect(categoryOf('image/gif')).toBe('image');
    expect(categoryOf('application/pdf')).toBe('pdf');
    expect(categoryOf('application/zip')).toBe('other');
  });
});

describe('attachmentsQuery', () => {
  it('builds Email/query{hasAttachment:true} + a chained Email/get', () => {
    const req = attachmentsQuery('acct1', 50);
    expect(req.using).toEqual([CAP_CORE, CAP_MAIL]);
    const [query, get] = req.methodCalls as [Invocation, Invocation];
    expect(query[0]).toBe('Email/query');
    expect(query[1]).toMatchObject({ accountId: 'acct1', filter: { hasAttachment: true }, limit: 50 });
    expect(get[0]).toBe('Email/get');
    expect(get[1]).toMatchObject({
      '#ids': { resultOf: 'aq', name: 'Email/query', path: '/ids' },
    });
    expect((get[1] as { properties: string[] }).properties).toContain('attachments');
  });
});

function email(over: Partial<Email> & { attachments?: unknown }): Email {
  return {
    id: 'e1',
    mailboxIds: {},
    from: [{ name: 'Alice', email: 'alice@example.org' }],
    to: null,
    subject: 'Report',
    receivedAt: '2026-01-02T00:00:00Z',
    preview: '',
    ...over,
  } as Email;
}

describe('parseAttachments', () => {
  it('flattens attachment lists and skips parts without a blobId', () => {
    const items = parseAttachments([
      email({
        id: 'e1',
        attachments: [
          { partId: '2', blobId: 'b1', size: 2048, type: 'application/pdf', name: 'q3.pdf' },
          { partId: '3', blobId: null, size: 1, type: 'image/png', name: 'inline.png' },
          { partId: '4', blobId: '', size: 1, type: 'image/png', name: 'empty.png' },
        ],
      }) as unknown as Email,
    ]);
    expect(items).toHaveLength(1);
    const it = items[0]!;
    expect(it).toMatchObject({ blobId: 'b1', name: 'q3.pdf', mime: 'application/pdf', size: 2048 });
    expect(it.from).toBe('Alice <alice@example.org>');
    expect(it.subject).toBe('Report');
  });

  it('defaults name/mime/subject when missing', () => {
    const items = parseAttachments([
      email({
        subject: null,
        from: null,
        attachments: [{ partId: '2', blobId: 'b9', size: 3, type: '', name: null }],
      }) as unknown as Email,
    ]);
    expect(items[0]).toMatchObject({
      name: '(unnamed)',
      mime: 'application/octet-stream',
      subject: '(no subject)',
      from: '',
    });
  });
});

const sample: AttachmentItem[] = [
  { emailId: 'e1', blobId: 'b1', name: 'Q3-report.pdf', mime: 'application/pdf', size: 2_000_000, from: 'Alice <alice@example.org>', subject: 'Q3', receivedAt: '2026-03-01T00:00:00Z' },
  { emailId: 'e2', blobId: 'b2', name: 'logo.png', mime: 'image/png', size: 40_000, from: 'Bob <bob@corp.com>', subject: 'Logo', receivedAt: '2026-01-15T00:00:00Z' },
  { emailId: 'e3', blobId: 'b3', name: 'demo.mp4', mime: 'video/mp4', size: 8_000_000, from: 'Alice <alice@example.org>', subject: 'Demo', receivedAt: '2026-02-10T00:00:00Z' },
];

describe('filterAttachments', () => {
  it('filters by category', () => {
    expect(filterAttachments(sample, { category: 'image' }).map((i) => i.blobId)).toEqual(['b2']);
    expect(filterAttachments(sample, { category: 'all' })).toHaveLength(3);
  });
  it('filters by filename substring (case-insensitive)', () => {
    expect(filterAttachments(sample, { text: 'REPORT' }).map((i) => i.blobId)).toEqual(['b1']);
  });
  it('filters by sender substring', () => {
    expect(filterAttachments(sample, { from: 'alice' }).map((i) => i.blobId)).toEqual(['b1', 'b3']);
  });
  it('filters by size range', () => {
    expect(filterAttachments(sample, { minSize: 1_000_000 }).map((i) => i.blobId)).toEqual(['b1', 'b3']);
    expect(filterAttachments(sample, { maxSize: 100_000 }).map((i) => i.blobId)).toEqual(['b2']);
  });
  it('filters by date window', () => {
    expect(filterAttachments(sample, { after: '2026-02-01T00:00:00Z' }).map((i) => i.blobId)).toEqual(['b1', 'b3']);
    expect(filterAttachments(sample, { before: '2026-01-31T00:00:00Z' }).map((i) => i.blobId)).toEqual(['b2']);
  });
});

describe('parseAttachmentQuery — shared operators', () => {
  it('parses filename/type/from/larger/smaller/before/after', () => {
    const f = parseAttachmentQuery('filename:report type:pdf from:alice larger:1mb');
    expect(f).toMatchObject({ text: 'report', category: 'pdf', from: 'alice', minSize: 1_000_000 });
  });
  it('treats bare words as a filename search', () => {
    expect(parseAttachmentQuery('quarterly budget')).toEqual({ text: 'quarterly budget' });
  });
  it('maps type aliases and quotes', () => {
    expect(parseAttachmentQuery('type:img').category).toBe('image');
    expect(parseAttachmentQuery('name:"my file"').text).toBe('my file');
  });
  it('drops unknown operators to the free-text bucket', () => {
    expect(parseAttachmentQuery('bogus:x report').text).toBe('bogus:x report');
  });
  it('round-trips through filterAttachments', () => {
    const f = parseAttachmentQuery('type:video from:alice');
    expect(filterAttachments(sample, f).map((i) => i.blobId)).toEqual(['b3']);
  });
});

describe('parseSize', () => {
  it('parses byte units', () => {
    expect(parseSize('2048')).toBe(2048);
    expect(parseSize('500kb')).toBe(500_000);
    expect(parseSize('1.5mb')).toBe(1_500_000);
    expect(parseSize('2 gb')).toBe(2_000_000_000);
  });
  it('returns undefined for garbage', () => {
    expect(parseSize('big')).toBeUndefined();
  });
});

describe('buildDownloadUrl', () => {
  it('substitutes the RFC 8620 template vars, url-encoded', () => {
    const url = buildDownloadUrl('/jmap/download/{accountId}/{blobId}/{name}?type={type}', {
      accountId: 'a 1',
      blobId: 'b/2',
      name: 'my file.pdf',
      mime: 'application/pdf',
    });
    expect(url).toBe('/jmap/download/a%201/b%2F2/my%20file.pdf?type=application%2Fpdf');
  });
});

describe('formatSize', () => {
  it('renders human units', () => {
    expect(formatSize(512)).toBe('512 B');
    expect(formatSize(2048)).toBe('2.0 KB');
    expect(formatSize(1_500_000)).toBe('1.4 MB');
  });
});
