import { describe, expect, it, vi } from 'vitest';
import { businessCardHtml, businessCardText, businessCardVcf, fieldsFromContact, fieldsFromIdentity } from './businessCard.ts';
import { agendaHtml, availabilityHtml, buildEventIcs, foldLine, freeSlots, localToInstant, snapshotHtml, zonedToUtc } from './calendarInsert.ts';
import { checkLink, linkHtml, linkText, normalizeHref, urlFromClipboard } from './links.ts';
import { rasteriseShape, shapeSvg, SHAPE_IDS, SHAPE_SCALE, type RasterEnv } from './shapes.ts';
import { emlFileName, flattenEntries, planFiles, summarizeDrag, type FsEntry } from './dropPolicy.ts';
import { prepareImage, presetSize, type ResizeEnv } from './imageResize.ts';
import { COMPOSE_FEATURES } from '../composeFeatures.ts';
import type { AttachmentPolicy } from '../../../api/attachmentPolicy.ts';
import { cardForAddress } from '../../../modules/contacts/share.ts';

const file = (name: string, type: string, size = 10): File => new File([new Uint8Array(size).fill(1)], name, { type });

describe('business card', () => {
  const fields = { name: 'Ada <Lovelace>', title: 'Engineer', org: 'Analytical', phone: '+44 20 7946 0000', email: 'ada@example.org', address: '1 Main St' };
  it('renders a mail-safe table with inline styles and escaped text', () => {
    const html = businessCardHtml(fields);
    expect(html.startsWith('<table')).toBe(true);
    expect(html).toContain('Ada &lt;Lovelace&gt;');
    expect(html).toContain('href="mailto:ada@example.org"');
    expect(html).toContain('href="tel:+442079460000"');
    expect(html).not.toMatch(/<style|<script|class=/);
  });
  it('builds a vCard 4.0 with the fields', () => {
    const vcf = businessCardVcf(fields, new Date('2026-01-01T00:00:00Z'));
    expect(vcf.name).toBe('Ada Lovelace.vcf');
    expect(vcf.type).toBe('text/vcard');
    expect(vcf.text).toContain('BEGIN:VCARD');
    expect(vcf.text).toContain('VERSION:4.0');
    expect(vcf.text).toContain('ada@example.org');
    expect(vcf.text).toContain('Engineer');
  });
  it('takes fields from an identity and a contact; plain text lists them', () => {
    expect(fieldsFromIdentity({ name: 'Me', email: 'me@x.org', organization: 'Org' })).toMatchObject({ name: 'Me', org: 'Org' });
    const card = { ...cardForAddress('c@x.org', 'Cee'), titles: ['CTO'], phones: [{ context: '', value: '123', pref: 1 }] };
    expect(fieldsFromContact(card)).toMatchObject({ name: 'Cee', title: 'CTO', phone: '123', email: 'c@x.org' });
    expect(businessCardText(fields).split('\n')).toHaveLength(5);
  });
  it('includes the photo only when a cid is given', () => {
    expect(businessCardHtml({ ...fields, photoSrc: 'cid:p@x' })).toContain('src="cid:p@x"');
    expect(businessCardHtml(fields)).not.toContain('<img');
  });
});

describe('calendar blocks', () => {
  it('zonedToUtc handles offsets', () => {
    expect(new Date(zonedToUtc(2026, 7, 1, 9, 0, 'Europe/Berlin')).toISOString()).toBe('2026-07-01T07:00:00.000Z');
    expect(new Date(zonedToUtc(2026, 1, 1, 9, 0, 'America/New_York')).toISOString()).toBe('2026-01-01T14:00:00.000Z');
  });
  it('computes free slots around busy time', () => {
    const w0 = Date.parse('2026-07-01T09:00:00Z'), w1 = Date.parse('2026-07-01T17:00:00Z');
    const slots = freeSlots([{ start: '2026-07-01T10:00:00Z', end: '2026-07-01T11:00:00Z' }, { start: '2026-07-01T10:30:00Z', end: '2026-07-01T12:00:00Z' }], w0, w1, 30);
    expect(slots.map(([a, b]) => [new Date(a).toISOString().slice(11, 16), new Date(b).toISOString().slice(11, 16)])).toEqual([['09:00', '10:00'], ['12:00', '17:00']]);
  });
  const labels = { day: 'Day', free: 'Free', none: 'No free time', note: 'Times shown in {tz}' };
  it('renders an availability table per day, in the zone, with the note, and never shows busy titles', () => {
    const html = availabilityHtml([{ start: '2026-07-01T07:00:00Z', end: '2026-07-01T09:00:00Z' }], { from: '2026-07-01', to: '2026-07-03', tz: 'Europe/Berlin', locale: 'en-GB', labels });
    expect(html).toContain('Times shown in Europe/Berlin');
    expect(html).toContain('11:00');
    expect((html.match(/<tr>/g) ?? []).length).toBe(1 + 3);
  });
  it('skips weekends by default', () => {
    const html = availabilityHtml([], { from: '2026-07-04', to: '2026-07-05', tz: 'UTC', labels });
    expect(html).not.toContain('<td');
  });
  it('agenda shows titles or only busy', () => {
    const ev = [{ start: '2026-07-01T08:00:00Z', end: '2026-07-01T09:00:00Z', title: 'Secret <plan>', location: 'HQ' }];
    const al = { busy: 'Busy', allDay: 'All day', empty: 'Nothing', note: 'in {tz}' };
    expect(agendaHtml(ev, { tz: 'UTC', visibility: 'titles', labels: al })).toContain('Secret &lt;plan&gt;');
    const busy = agendaHtml(ev, { tz: 'UTC', visibility: 'busy', labels: al });
    expect(busy).not.toContain('Secret');
    expect(busy).toContain('Busy');
    expect(agendaHtml([], { tz: 'UTC', visibility: 'busy', labels: al })).toContain('Nothing');
  });
  it('snapshot is a month grid with counts per visibility', () => {
    const ev = [{ start: '2026-07-15T08:00:00Z', end: '2026-07-15T09:00:00Z', title: 'Standup' }];
    const html = snapshotHtml(ev, { year: 2026, month: 7, tz: 'UTC', visibility: 'titles', labels: { note: '{tz}' } });
    expect(html).toContain('Standup');
    expect(html).toContain('<strong>31</strong>');
    expect(snapshotHtml(ev, { year: 2026, month: 7, tz: 'UTC', visibility: 'busy', labels: { note: '{tz}' } })).not.toContain('Standup');
  });
});

describe('event ics', () => {
  const base = { uid: 'u1@mailwoman', title: 'Kick-off, v1', start: Date.parse('2026-07-01T09:00:00Z'), end: Date.parse('2026-07-01T10:00:00Z'), tz: 'Europe/Berlin', attendees: [] as string[], organizer: { name: 'Me', email: 'me@x.org' } };
  it('is PUBLISH without attendees and carries no ORGANIZER', () => {
    const { method, text } = buildEventIcs(base, Date.parse('2026-06-01T00:00:00Z'));
    expect(method).toBe('PUBLISH');
    expect(text).toContain('METHOD:PUBLISH');
    expect(text).not.toContain('ORGANIZER');
    expect(text).toContain('SUMMARY:Kick-off\\, v1');
    expect(text).toContain('DTSTART:20260701T090000Z');
    expect(text.endsWith('\r\n')).toBe(true);
  });
  it('is REQUEST with ORGANIZER and ATTENDEE when invited', () => {
    const { method, text } = buildEventIcs({ ...base, attendees: ['a@x.org', 'b@x.org'], location: 'Room 1' });
    expect(method).toBe('REQUEST');
    expect(text).toContain('METHOD:REQUEST');
    expect(text).toContain('ORGANIZER;CN="Me":mailto:me@x.org');
    expect(text).toContain('mailto:a@x.org');
    expect(text.match(/ATTENDEE/g)).toHaveLength(2);
    expect(text).toContain('LOCATION:Room 1');
  });
  it('folds long lines and converts wall time', () => {
    for (const l of foldLine(`DESCRIPTION:${'x'.repeat(200)}`).split('\r\n')) expect(l.length).toBeLessThanOrEqual(75);
    expect(new Date(localToInstant('2026-07-01', '09:00', 'Europe/Berlin')).toISOString()).toBe('2026-07-01T07:00:00.000Z');
  });
});

describe('shapes', () => {
  it('every shape yields an svg of the right size', () => {
    for (const id of SHAPE_IDS) {
      const svg = shapeSvg({ id, width: 120, height: 80, fill: '#ff0000', stroke: '#000', strokeWidth: 3 });
      expect(svg).toContain('width="120"');
      expect(svg).toContain('<svg');
      expect(svg).not.toMatch(/script|onload/i);
    }
  });
  it('rejects hostile colours', () => {
    const svg = shapeSvg({ id: 'rectangle', width: 50, height: 50, fill: '"/><script>', stroke: 'red', strokeWidth: 2 });
    expect(svg).not.toContain('script');
  });
  it('rasterises at 2x through the canvas', async () => {
    const drawImage = vi.fn();
    const toBlob = vi.fn((cb: (b: Blob | null) => void) => cb(new Blob(['png'], { type: 'image/png' })));
    const createCanvas = vi.fn(() => ({ getContext: () => ({ drawImage }), toBlob }));
    const env: RasterEnv = { loadImage: vi.fn(async () => ({}) as CanvasImageSource), createCanvas };
    const out = await rasteriseShape({ id: 'star', width: 100, height: 60, fill: 'gold', stroke: '#333', strokeWidth: 2 }, env);
    expect(createCanvas).toHaveBeenCalledWith(100 * SHAPE_SCALE, 60 * SHAPE_SCALE);
    expect(drawImage).toHaveBeenCalledWith(expect.anything(), 0, 0, 200, 120);
    expect(out.blob.type).toBe('image/png');
    expect(out.svg).toContain('<polygon');
  });
});

describe('links', () => {
  it('normalises addresses', () => {
    expect(normalizeHref('example.org/a')).toBe('https://example.org/a');
    expect(normalizeHref('a@b.org')).toBe('mailto:a@b.org');
    expect(normalizeHref('+49 30 1234567')).toBe('tel:+49301234567');
    expect(normalizeHref('#top')).toBe('#top');
  });
  it('warns for http, shorteners, IDN, lookalikes, credentials and script', () => {
    expect(checkLink('http://example.org')).toContain('http');
    expect(checkLink('https://bit.ly/x')).toContain('shortener');
    expect(checkLink('https://xn--pypal-4ve.com/')).toContain('idn');
    expect(checkLink('https://paypa1.com/')).toContain('lookalike');
    expect(checkLink('https://paypal.evil.com/')).toContain('lookalike');
    expect(checkLink('https://user:pw@example.org/')).toContain('credentials');
    expect(checkLink('javascript:alert(1)')).toEqual(['script']);
    expect(checkLink('https://paypal.com/')).toEqual([]);
  });
  it('renders html with rel/target policy and different captions for one URL', () => {
    const a = linkHtml({ caption: 'Docs', href: 'https://example.org/', newTab: true, title: 'T' });
    expect(a).toContain('target="_blank"');
    expect(a).toContain('rel="noopener noreferrer"');
    expect(a).toContain('title="T"');
    const b = linkHtml({ caption: 'Pricing <b>', href: 'https://example.org/' });
    expect(b).not.toContain('target');
    expect(b).toContain('Pricing &lt;b&gt;');
    expect(linkHtml({ caption: '', href: '#top' })).toBe('<a href="#top">#top</a>');
  });
  it('renders plain text as caption (url)', () => {
    expect(linkText({ caption: 'Docs', href: 'https://example.org/' })).toBe('Docs (https://example.org/)');
    expect(linkText({ caption: '', href: 'https://example.org/' })).toBe('https://example.org/');
    expect(linkText({ caption: 'Ada', href: 'mailto:ada@x.org' })).toBe('Ada (ada@x.org)');
  });
  it('detects a url on the clipboard', () => {
    expect(urlFromClipboard(' https://a.org/x ')).toBe('https://a.org/x');
    expect(urlFromClipboard('hello world')).toBeNull();
  });
});

describe('drop planning', () => {
  const opts = { maxBytes: 1000, inlineImages: false, attachAllowed: true };
  it('attaches files, refuses oversized and empty ones', () => {
    const plan = planFiles([file('a.pdf', 'application/pdf', 500), file('big.zip', 'application/zip', 5000), file('e.txt', 'text/plain', 0)], opts);
    expect(plan.attach.map((f) => f.name)).toEqual(['a.pdf']);
    expect(plan.refused).toEqual([{ name: 'big.zip', reason: 'size' }, { name: 'e.txt', reason: 'empty' }]);
  });
  it('places images inline when asked, otherwise attaches them', () => {
    const img = file('p.png', 'image/png', 100);
    expect(planFiles([img], { ...opts, inlineImages: true }).inline).toHaveLength(1);
    expect(planFiles([img], opts).attach).toHaveLength(1);
  });
  it('applies the admin policy: blocked refused, warned flagged', () => {
    const policy: AttachmentPolicy = { previewAllowed: true, downloadAllowed: true, maxBytes: 0, allowedExtensions: [], blockedExtensions: ['exe'], warnExtensions: ['zip'], allowedMimeTypes: [], blockedMimeTypes: [], warnMimeTypes: [] };
    const plan = planFiles([file('x.exe', 'application/x-msdownload', 10), file('y.zip', 'application/zip', 10)], { ...opts, policy });
    expect(plan.refused).toEqual([{ name: 'x.exe', reason: 'blocked' }]);
    expect(plan.warned).toEqual(['y.zip']);
    expect(plan.attach.map((f) => f.name)).toEqual(['y.zip']);
  });
  it('warns for executables without a policy and refuses all when attaching is off', () => {
    expect(planFiles([file('setup.exe', '', 10)], opts).warned).toEqual(['setup.exe']);
    expect(planFiles([file('a.txt', 'text/plain', 10)], { ...opts, attachAllowed: false }).refused[0]?.reason).toBe('blocked');
  });
  it('summarises a drag and flattens folders', async () => {
    const s = summarizeDrag({ types: ['Files', 'application/x-mailwoman-message-selection'], items: [{ kind: 'file', type: 'image/png' }, { kind: 'file', type: 'text/plain' }] as unknown as DataTransferItemList });
    expect(s).toMatchObject({ files: 2, images: 1, messages: 1 });
    const leaf = (name: string): FsEntry => ({ isFile: true, isDirectory: false, name, file: (ok) => ok(file(name, 'text/plain')) });
    let served = false;
    const dir: FsEntry = { isFile: false, isDirectory: true, name: 'docs', createReader: () => ({ readEntries: (ok) => { ok(served ? [] : [leaf('a.txt'), leaf('b.txt')]); served = true; } }) };
    const flat = await flattenEntries([dir]);
    expect(flat.files.map((f) => f.name)).toEqual(['docs - a.txt', 'docs - b.txt']);
    expect(emlFileName('Re: a/b?')).toBe('Re a b.eml');
  });
});

describe('image presets', () => {
  it('downscales the longest side and never upscales', () => {
    expect(presetSize(4000, 2000, 'large')).toEqual({ width: 1600, height: 800, scaled: true });
    expect(presetSize(800, 600, 'large').scaled).toBe(false);
    expect(presetSize(4000, 2000, 'original').scaled).toBe(false);
  });
  it('re-encodes through the canvas, keeps GIFs and original PNGs', async () => {
    const draw = vi.fn(async (_s: CanvasImageSource, _w: number, _h: number, type: string) => new Blob(['x'], { type }));
    const env: ResizeEnv = { decode: async () => ({ width: 4000, height: 3000, source: {} as CanvasImageSource }), draw };
    const out = await prepareImage(file('a.jpg', 'image/jpeg', 50), { preset: 'medium', stripExif: true }, env);
    expect(draw).toHaveBeenCalledWith(expect.anything(), 1024, 768, 'image/jpeg');
    expect(out.name).toBe('a.jpg');
    expect(await prepareImage(file('a.gif', 'image/gif'), { preset: 'small', stripExif: true }, env)).toMatchObject({ name: 'a.gif' });
    draw.mockClear();
    await prepareImage(file('a.png', 'image/png'), { preset: 'original', stripExif: true }, env);
    expect(draw).not.toHaveBeenCalled();
    await prepareImage(file('a.jpg', 'image/jpeg'), { preset: 'original', stripExif: true }, env);
    expect(draw).toHaveBeenCalledTimes(1);
  });
});

describe('feature flags', () => {
  it('declares the insert features, on by default', () => {
    const ids = COMPOSE_FEATURES.filter((f) => f.group === 'insert').map((f) => f.id);
    for (const id of ['insert.attachFile', 'insert.attachFromMessages', 'insert.pictureFile', 'insert.pictureFromMessages', 'insert.shapes', 'insert.screenshot', 'insert.businessCard', 'insert.calendar', 'insert.event', 'insert.symbols', 'insert.horizontalLine', 'insert.dateStamp', 'insert.link', 'insert.dragDropAttach', 'insert.dragDropInline', 'insert.dropMessages']) expect(ids).toContain(id);
    expect(COMPOSE_FEATURES.every((f) => f.defaultOn)).toBe(true);
  });
});
