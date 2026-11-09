import { describe, it, expect, vi } from 'vitest';
import type { Client } from '../api/client.ts';
import { CAP_CORE, CAP_MAIL, type Email, type Invocation } from '../api/jmap-types.ts';
import { viewerKindFor } from '../contracts/viewer.ts';
import {
  attachmentsQuery,
  loadAttachmentPage,
  searchAttachmentPages,
  buildDownloadUrl,
  categoryOf,
  filterAttachments,
  formatSize,
  parseAttachmentQuery,
  parseAttachments,
  parseSize,
  type AttachmentItem,
  type AttachmentFilters,
} from './attachments.ts';

describe('account attachment search', () => {
  const file: AttachmentItem = {emailId:'old',blobId:'blob',name:'older-report.pdf',mime:'application/pdf',size:42,from:'a@example.org',subject:'Report',receivedAt:'2020-01-01T00:00:00Z'};
  it('finds older files beyond empty matching pages and deduplicates repeated entries', async () => {
    const load = vi.fn(async (position:number) => ({position,items:position===0?[]:[file,file],nextPosition:position===0?50:null}));
    const result = await searchAttachmentPages(load,{text:'report'},new AbortController().signal,vi.fn());
    expect(result.items).toEqual([file]);
    expect(result.pages).toBe(2);
    expect(load.mock.calls.map(call=>call[0])).toEqual([0,50]);
  });
  it('discards an in-flight page after cancellation', async () => {
    const controller = new AbortController(); const progress = vi.fn();
    const load = vi.fn(async()=>{controller.abort();return {position:0,items:[file],nextPosition:50};});
    await expect(searchAttachmentPages(load,{},controller.signal,progress)).rejects.toMatchObject({name:'AbortError'});
    expect(progress).not.toHaveBeenCalled(); expect(load).toHaveBeenCalledTimes(1);
  });
  it('rejects a nonadvancing page instead of repeatedly querying it', async () => {
    await expect(searchAttachmentPages(async()=>({position:0,items:[],nextPosition:0}),{},new AbortController().signal,vi.fn())).rejects.toThrow('Invalid attachment search page');
  });
  it('bounds retained results and reports the limit instead of claiming a complete search', async () => {
    const load = vi.fn(async()=>({position:0,items:Array.from({length:1100},(_,i)=>({...file,blobId:String(i)})),nextPosition:50}));
    const result = await searchAttachmentPages(load,{},new AbortController().signal,vi.fn());
    expect(result.items).toHaveLength(1000); expect(result.limited).toBe(true); expect(load).toHaveBeenCalledTimes(1);
  });
});

describe('viewerKindFor — MIME dispatch (frozen §2.4)', () => {
  it('routes each family to its viewer', () => {
    expect(viewerKindFor('application/pdf')).toBe('pdf');
    expect(viewerKindFor('image/png')).toBe('image');
    expect(viewerKindFor('image/jpeg')).toBe('image');
    expect(viewerKindFor('audio/mpeg')).toBe('audio');
    expect(viewerKindFor('video/mp4')).toBe('video');
    expect(viewerKindFor('text/plain')).toBe('text');
    expect(viewerKindFor('text/calendar; method=REQUEST')).toBe('calendar');
    expect(viewerKindFor('application/ics')).toBe('calendar');
    expect(viewerKindFor('application/octet-stream', 'invite.ics')).toBe('calendar');
    expect(viewerKindFor('image/png', 'x.ics')).toBe('image');
    expect(viewerKindFor('application/octet-stream')).toBe('unsupported');
    expect(viewerKindFor('')).toBe('unsupported');
  });

  it('categoryOf groups by the same routing (unsupported → other)', () => {
    expect(categoryOf('image/gif')).toBe('image');
    expect(categoryOf('application/pdf')).toBe('pdf');
    expect(categoryOf('application/zip')).toBe('archive');
  });
});

describe('attachmentsQuery', () => {
  it('uses the message count for the next page even when one message has multiple files', async () => {
    const client = {jmap:vi.fn(async()=>({methodResponses:[
      ['Email/query',{accountId:'a',position:50,ids:['e1'],total:52},'aq'],
      ['Email/get',{accountId:'a',list:[email({attachments:[{blobId:'b1',type:'image/png',size:1},{blobId:'b2',type:'image/png',size:1}]})]},'ag'],
    ]}))} as unknown as Client;
    const page = await loadAttachmentPage(client,'a',50);
    expect(page.items).toHaveLength(2);
    expect(page.nextPosition).toBe(51);
  });
  it('rejects a response belonging to a different account', async () => {
    const client = {jmap:vi.fn(async()=>({methodResponses:[
      ['Email/query',{accountId:'other',position:0,ids:[]},'aq'],
      ['Email/get',{accountId:'a',list:[]},'ag'],
    ]}))} as unknown as Client;
    await expect(loadAttachmentPage(client,'a')).rejects.toThrow('requested account');
  });
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
    expect((get[1] as { properties: string[] }).properties).toContain('mailboxIds');
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
    expect(it.mailboxIds).toEqual({});
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
  it('combines independent form fields with AND and matches free words across message/file context', () => {
    const items=sample.map(item=>({...item,mailboxIds:{inbox:true}}));
    const filters:AttachmentFilters={text:'REPORT Alice Q3',filename:'report',from:'ALICE',subject:'q3',category:'pdf',extensions:['.PDF'],minSize:2_000_000,maxSize:2_000_000,after:'2026-03-01',before:'2026-03-01',mailboxId:'inbox'};
    expect(filterAttachments(items,filters).map(item=>item.blobId)).toEqual(['b1']);
    expect(filterAttachments(items,{...filters,subject:'missing'})).toEqual([]);
    expect(filterAttachments(items,{...filters,mailboxId:'archive'})).toEqual([]);
    expect(filterAttachments(sample,{mailboxId:'inbox'})).toEqual([]);
  });
  it('preserves contradictory advanced/form constraints instead of overriding them', () => {
    expect(filterAttachments(sample,{and:[parseAttachmentQuery('from:alice'),{from:'bob'}]})).toEqual([]);
    expect(filterAttachments(sample,{and:[parseAttachmentQuery('type:video'),{category:'pdf'}]})).toEqual([]);
    expect(filterAttachments(sample,{and:[parseAttachmentQuery('filename:report alice'),{subject:'q3',extensions:['pdf']}]}).map(item=>item.blobId)).toEqual(['b1']);
  });
  it('matches separate free words across fields while retaining quoted phrases',()=>{
    const items=[{...sample[0]!,name:'Annual report.pdf',from:'Alice Smith <alice@example.org>'}];
    expect(filterAttachments(items,parseAttachmentQuery('Alice report'))).toHaveLength(1);
    expect(filterAttachments(items,parseAttachmentQuery('Alice "annual report"'))).toHaveLength(1);
    expect(filterAttachments(items,parseAttachmentQuery('"Alice Smith" report'))).toHaveLength(1);
    expect(filterAttachments(items,parseAttachmentQuery('"Alice report"'))).toHaveLength(0);
  });
  it('normalizes extension lists and permits any listed extension while preserving other filters', () => {
    expect(filterAttachments(sample,{extensions:[' .PDF ','.PNG'],maxSize:2_000_000}).map(item=>item.blobId)).toEqual(['b1','b2']);
    expect(filterAttachments([{...sample[0]!,name:'backup.TAR.GZ'}],{extensions:['.tar.gz']})).toHaveLength(1);
    expect(filterAttachments(sample,{extensions:['pdf'],from:'bob'})).toEqual([]);
  });
  it('includes the entire selected end date and compares offset timestamps as instants', () => {
    const items=['2026-01-15T00:00:00Z','2026-01-15T23:59:59.999Z','2026-01-16T00:00:00Z','2026-01-16T01:00:00+02:00'].map((receivedAt,index)=>({...sample[0]!,blobId:String(index),receivedAt}));
    expect(filterAttachments(items,{after:'2026-01-15',before:'2026-01-15'}).map(item=>item.blobId)).toEqual(['0','1','3']);
    expect(filterAttachments(items,{after:'2026-01-16T00:00:00Z'}).map(item=>item.blobId)).toEqual(['2']);
    for(const before of ['not-a-date','2026-02-31'])expect(filterAttachments(items,{before})).toEqual([]);
    expect(filterAttachments([{...items[0]!,receivedAt:'invalid'}],{before:'2026-01-15'})).toEqual([]);
  });
  it('bounds nested composite filters, including accidental cycles', () => {
    const cyclic:AttachmentFilters={};cyclic.and=[cyclic];
    expect(filterAttachments(sample,cyclic)).toEqual([]);
  });
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
  it('keeps filename, subject, extensions, folder, and free text as separate constraints', () => {
    expect(parseAttachmentQuery('filename:"annual report" subject:"Board meeting" ext:.pdf,.docx in:archive alice')).toEqual({filename:'annual report',subject:'Board meeting',extensions:['pdf','docx'],mailboxId:'archive',text:'alice'});
    expect(filterAttachments(sample,parseAttachmentQuery('name:report from:alice Q3')).map(item=>item.blobId)).toEqual(['b1']);
    for(const [value,category] of [['word','document'],['xlsx','spreadsheet'],['zip','archive']] as const)expect(parseAttachmentQuery(`type:${value}`).category).toBe(category);
  });
  it('parses filename/type/from/larger/smaller/before/after', () => {
    const f = parseAttachmentQuery('filename:report type:pdf from:alice larger:1mb');
    expect(f).toMatchObject({ filename: 'report', category: 'pdf', from: 'alice', minSize: 1_000_000 });
  });
  it('treats bare words as a free metadata search', () => {
    expect(parseAttachmentQuery('quarterly budget')).toEqual({ text: 'quarterly budget' });
  });
  it('maps type aliases and quotes', () => {
    expect(parseAttachmentQuery('type:img').category).toBe('image');
    expect(parseAttachmentQuery('name:"my file"').filename).toBe('my file');
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
    expect(parseSize('999999999999999999999999mb')).toBeUndefined();
    expect(parseSize('-1kb')).toBeUndefined();
  });
});

describe('expanded attachment metadata categories',()=>{
  it.each([
    ['application/msword','','document'],
    ['application/vnd.openxmlformats-officedocument.wordprocessingml.document','','document'],
    ['application/vnd.ms-powerpoint','','document'],
    ['application/vnd.oasis.opendocument.text','','document'],
    ['application/vnd.openxmlformats-officedocument.spreadsheetml.sheet','','spreadsheet'],
    ['text/csv; charset=utf-8','','spreadsheet'],
    ['application/x-7z-compressed','','archive'],
    ['application/gzip','','archive'],
    ['application/octet-stream','Report.DOCX','document'],
    ['application/octet-stream','Budget.XLSX','spreadsheet'],
    ['application/octet-stream','Backup.tar.gz','archive'],
    ['image/png','renamed.docx','image'],
  ] as const)('classifies %s / %s as %s',(mime,name,category)=>expect(categoryOf(mime,name)).toBe(category));
  it('copies folder metadata so a later source mutation cannot change a flattened item',()=>{
    const source=email({mailboxIds:{inbox:true},attachments:[{blobId:'b',type:'application/pdf',name:'a.pdf',size:1}]});
    const [item]=parseAttachments([source]);source.mailboxIds['inbox']=false;
    expect(item?.mailboxIds).toEqual({inbox:true});
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
    expect(formatSize(512)).toBe('512 byte');
    expect(formatSize(2048)).toBe('2 kB');
    expect(formatSize(1_500_000)).toBe('1.4 MB');
  });
});
