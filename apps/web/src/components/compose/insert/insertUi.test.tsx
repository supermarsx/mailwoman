// The Insert tab's dialogs, groups and the drop zone, rendered against stub hosts.

import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { createSignal } from 'solid-js';
import { cleanup, fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import axe from 'axe-core';
import { loadCatalog } from '../../../i18n/index.ts';
import { clearSettings, loadSettings, type Fetcher } from '../../../api/settings.ts';
import { cardForAddress } from '../../../modules/contacts/share.ts';
import type { AttachmentItem } from '../../../viewers/attachments.ts';
import { LinkDialog } from './LinkDialog.tsx';
import { BusinessCardDialog } from './BusinessCardDialog.tsx';
import { EventDialog } from './EventDialog.tsx';
import { CalendarDialog } from './CalendarDialog.tsx';
import { MessagePicturesDialog, filterMessageItems } from './MessagePicturesDialog.tsx';
import { PictureFileDialog } from './PictureFileDialog.tsx';
import { ShapeDialog } from './ShapeDialog.tsx';
import { ComposeDropZone, type DropHost } from './ComposeDropZone.tsx';
import { InsertGroups, type InsertHost } from './InsertGroups.tsx';
import { MESSAGE_DRAG_TYPE } from './dropPolicy.ts';
import type { CalendarController } from '../../../modules/calendar/controller.ts';

beforeAll(async () => {
  await loadCatalog('mail');
  await loadCatalog('compose-insert');
});
afterEach(() => { cleanup(); clearSettings(); });

const noop = (): void => undefined;
/** Click a dialog's apply button once it is enabled (dialogs load their data first). */
const apply = async (id: string): Promise<void> => {
  await waitFor(() => expect(screen.getByTestId(id)).not.toBeDisabled());
  fireEvent.click(screen.getByTestId(id));
};

describe('link dialog', () => {
  it('inserts html and the plain-text form, with the caption and tooltip', () => {
    const onInsert = vi.fn();
    render(() => <LinkDialog initial={{ caption: 'Docs', href: '', title: '', newTab: false, existing: false }} richMode returnTo={null} onClose={noop} onInsert={onInsert} />);
    fireEvent.input(screen.getByTestId('insert-link-href'), { target: { value: 'example.org/a' } });
    fireEvent.input(screen.getByTestId('insert-link-title'), { target: { value: 'Tip' } });
    fireEvent.click(screen.getByTestId('insert-link-newtab'));
    fireEvent.click(screen.getByTestId('insert-link-dialog-apply'));
    expect(onInsert).toHaveBeenCalledWith('<a href="https://example.org/a" title="Tip" target="_blank" rel="noopener noreferrer">Docs</a>', 'Docs (https://example.org/a)');
  });
  it('warns for lookalikes and refuses script addresses', () => {
    const onInsert = vi.fn();
    render(() => <LinkDialog initial={{ caption: 'x', href: 'https://paypa1.com/', title: '', newTab: false, existing: false }} richMode={false} returnTo={null} onClose={noop} onInsert={onInsert} />);
    expect(screen.getByTestId('insert-link-warn-lookalike')).toBeInTheDocument();
    expect(screen.queryByTestId('insert-link-title')).toBeNull();
    fireEvent.input(screen.getByTestId('insert-link-href'), { target: { value: 'javascript:alert(1)' } });
    expect(screen.getByTestId('insert-link-dialog-apply')).toBeDisabled();
  });
  it('edits an existing link in place and can remove it', () => {
    const onApply = vi.fn(), onRemove = vi.fn(), onClose = vi.fn();
    render(() => <LinkDialog initial={{ caption: 'Old', href: 'https://a.org/', title: '', newTab: false, existing: true, onApply, onRemove }} richMode returnTo={null} onClose={onClose} onInsert={noop} />);
    fireEvent.input(screen.getByTestId('insert-link-caption'), { target: { value: 'New' } });
    fireEvent.click(screen.getByTestId('insert-link-dialog-apply'));
    expect(onApply).toHaveBeenCalledWith({ caption: 'New', href: 'https://a.org/', title: '', newTab: false });
    fireEvent.click(screen.getByTestId('insert-link-remove'));
    expect(onRemove).toHaveBeenCalled();
  });
  it('closes on Escape', () => {
    const onClose = vi.fn();
    render(() => <LinkDialog initial={{ caption: '', href: '', title: '', newTab: false, existing: false }} richMode returnTo={null} onClose={onClose} onInsert={noop} />);
    fireEvent.keyDown(screen.getByRole('dialog'), { key: 'Escape' });
    expect(onClose).toHaveBeenCalled();
  });
  it('has no accessibility violations', async () => {
    render(() => <LinkDialog initial={{ caption: '', href: '', title: '', newTab: false, existing: false }} richMode returnTo={null} onClose={noop} onInsert={noop} />);
    const result = await axe.run(screen.getByRole('dialog'), { rules: { 'color-contrast': { enabled: false }, region: { enabled: false } } });
    expect(result.violations.map((v) => `${v.id}: ${v.nodes.map((n) => n.target.join(' ')).join(', ')}`)).toEqual([]);
  });
});

describe('business card dialog', () => {
  const identity = { name: 'Ada Lovelace', email: 'ada@example.org', organization: 'Analytical' };
  it('"My card" inserts the card and the vCard by default; card-only and vCard-only drop the other', () => {
    const onInsert = vi.fn();
    render(() => <BusinessCardDialog source={{ kind: 'self' }} identity={identity} contacts={[]} returnTo={null} onClose={noop} onInsert={onInsert} />);
    expect((screen.getByTestId('insert-card-name') as HTMLInputElement).value).toBe('Ada Lovelace');
    fireEvent.input(screen.getByTestId('insert-card-title'), { target: { value: 'Engineer' } });
    fireEvent.click(screen.getByTestId('insert-card-dialog-apply'));
    const [html, , vcf] = onInsert.mock.calls[0]!;
    expect(html).toContain('Ada Lovelace');
    expect(html).toContain('Engineer');
    expect(vcf).toMatchObject({ name: 'Ada Lovelace.vcf', type: 'text/vcard' });
    expect(vcf.text).toContain('VERSION:4.0');
    fireEvent.change(screen.getByTestId('insert-card-mode'), { target: { value: 'card' } });
    fireEvent.click(screen.getByTestId('insert-card-dialog-apply'));
    expect(onInsert.mock.calls[1]![2]).toBeNull();
    fireEvent.change(screen.getByTestId('insert-card-mode'), { target: { value: 'vcard' } });
    fireEvent.click(screen.getByTestId('insert-card-dialog-apply'));
    expect(onInsert.mock.calls[2]![0]).toBeNull();
    expect(onInsert.mock.calls[2]![2]).not.toBeNull();
  });
  it('"Contact…" lists and filters contacts and fills the fields', () => {
    const onInsert = vi.fn();
    const contacts = [cardForAddress('bob@x.org', 'Bob Baker'), cardForAddress('carol@x.org', 'Carol Cole')];
    render(() => <BusinessCardDialog source={{ kind: 'contact' }} identity={identity} contacts={contacts} returnTo={null} onClose={noop} onInsert={onInsert} />);
    fireEvent.input(screen.getByTestId('insert-card-search'), { target: { value: 'carol' } });
    const picks = screen.getAllByTestId('insert-card-pick');
    expect(picks).toHaveLength(1);
    fireEvent.click(picks[0]!);
    expect((screen.getByTestId('insert-card-email') as HTMLInputElement).value).toBe('carol@x.org');
  });
});

/** A calendar controller stub: one master event, one default calendar. */
function controller(extra: Partial<CalendarController> = {}): CalendarController {
  const soon = new Date(Date.now() + 2 * 86_400_000);
  const local = `${soon.getFullYear()}-${String(soon.getMonth() + 1).padStart(2, '0')}-${String(soon.getDate()).padStart(2, '0')}T10:00:00`;
  return {
    load: vi.fn(async () => undefined),
    calendars: () => [{ id: 'c1', name: 'Work', color: '#123456', order: 0, isVisible: true, isSubscribed: false, role: 'default', shareWith: [], caldavUrl: null, syncToken: null, isReadOnlyOverlay: false }],
    masters: () => [{ id: 'e1', calendarId: 'c1', uid: 'u', title: 'Planning', description: '', locations: [{ name: 'Room 2' }], start: local, timeZone: null, duration: 'PT1H', showWithoutTime: false, recurrenceRules: [], recurrenceOverrides: {}, excludedRecurrenceDates: [], status: 'confirmed', priority: 0, freeBusyStatus: 'busy', participants: {}, alerts: {}, sequence: 0, etag: null }],
    queryFreeBusy: vi.fn(async () => [{ start: new Date(Date.now() + 3600_000).toISOString(), end: new Date(Date.now() + 7200_000).toISOString(), status: 'busy' as const }]),
    createEvent: vi.fn(async () => 'new1'),
    ...extra,
  } as unknown as CalendarController;
}

describe('event dialog', () => {
  const me = { name: 'Me', email: 'me@x.org' };
  it('a new event with attendees inserts a summary and a METHOD:REQUEST ics with ORGANIZER', async () => {
    const onInsert = vi.fn();
    render(() => <EventDialog existing={false} controller={controller()} identity={me} returnTo={null} onClose={noop} onInsert={onInsert} />);
    fireEvent.input(screen.getByTestId('insert-event-title'), { target: { value: 'Kick-off' } });
    fireEvent.input(screen.getByTestId('insert-event-attendees'), { target: { value: 'a@x.org, b@x.org' } });
    expect(screen.getByTestId('insert-event-method').textContent).toMatch(/invitation/);
    await apply('insert-event-dialog-apply');
    const [html, , ics] = onInsert.mock.calls[0]!;
    expect(html).toContain('Kick-off');
    expect(ics.method).toBe('REQUEST');
    expect(ics.text).toContain('ORGANIZER;CN="Me":mailto:me@x.org');
    expect(ics.text).toContain('ATTENDEE;CN="a@x.org"');
  });
  it('without attendees it publishes, and can also create the event in the calendar', async () => {
    const onInsert = vi.fn();
    const c = controller();
    render(() => <EventDialog existing={false} controller={c} identity={me} returnTo={null} onClose={noop} onInsert={onInsert} />);
    fireEvent.input(screen.getByTestId('insert-event-title'), { target: { value: 'Lunch' } });
    fireEvent.click(screen.getByTestId('insert-event-also-create'));
    await apply('insert-event-dialog-apply');
    await waitFor(() => expect(onInsert).toHaveBeenCalled());
    expect(onInsert.mock.calls[0]![2].method).toBe('PUBLISH');
    expect(c.createEvent).toHaveBeenCalledWith(expect.objectContaining({ calendarId: 'c1', title: 'Lunch' }));
  });
  it('picks an existing event of the user\'s calendars', async () => {
    render(() => <EventDialog existing controller={controller()} identity={me} returnTo={null} onClose={noop} onInsert={noop} />);
    fireEvent.click(await screen.findByTestId('insert-event-pick'));
    expect((screen.getByTestId('insert-event-title') as HTMLInputElement).value).toBe('Planning');
    expect((screen.getByTestId('insert-event-location') as HTMLInputElement).value).toBe('Room 2');
  });
});

describe('calendar dialog', () => {
  it('availability lists free time from the free/busy answer, with the time zone note', async () => {
    const onInsert = vi.fn();
    const c = controller();
    render(() => <CalendarDialog mode="availability" controller={c} returnTo={null} onClose={noop} onInsert={onInsert} />);
    fireEvent.input(screen.getByTestId('insert-cal-tz'), { target: { value: 'Europe/Berlin' } });
    await apply('insert-calendar-dialog-apply');
    await waitFor(() => expect(onInsert).toHaveBeenCalled());
    expect(c.queryFreeBusy).toHaveBeenCalled();
    expect(onInsert.mock.calls[0]![0]).toContain('Times shown in Europe/Berlin');
  });
  it('agenda respects the busy-only visibility', async () => {
    const onInsert = vi.fn();
    render(() => <CalendarDialog mode="agenda" controller={controller()} returnTo={null} onClose={noop} onInsert={onInsert} />);
    fireEvent.input(screen.getByTestId('insert-cal-to'), { target: { value: new Date(Date.now() + 10 * 86_400_000).toISOString().slice(0, 10) } });
    await apply('insert-calendar-dialog-apply');
    await waitFor(() => expect(onInsert).toHaveBeenCalled());
    expect(onInsert.mock.calls[0]![0]).not.toContain('Planning');
    onInsert.mockClear();
    fireEvent.change(screen.getByTestId('insert-cal-visibility'), { target: { value: 'titles' } });
    await apply('insert-calendar-dialog-apply');
    await waitFor(() => expect(onInsert).toHaveBeenCalled());
    expect(onInsert.mock.calls[0]![0]).toContain('Planning');
  });
});

describe('shape dialog', () => {
  it('rasterises and inserts a PNG file with alt text', async () => {
    const onInsert = vi.fn();
    const proto = HTMLCanvasElement.prototype;
    vi.spyOn(proto, 'getContext').mockReturnValue({ drawImage: vi.fn() } as never);
    vi.spyOn(proto, 'toBlob').mockImplementation((cb) => cb(new Blob(['png'], { type: 'image/png' })));
    const OriginalImage = globalThis.Image;
    globalThis.Image = class { onload: (() => void) | null = null; set src(_v: string) { queueMicrotask(() => this.onload?.()); } } as never;
    render(() => <ShapeDialog returnTo={null} onClose={noop} onInsert={onInsert} />);
    fireEvent.click(screen.getByTestId('insert-shape-star'));
    fireEvent.input(screen.getByTestId('insert-shape-alt'), { target: { value: 'A star' } });
    fireEvent.click(screen.getByTestId('insert-shape-dialog-apply'));
    await waitFor(() => expect(onInsert).toHaveBeenCalled());
    globalThis.Image = OriginalImage;
    const [file, alt] = onInsert.mock.calls[0]!;
    expect(file.type).toBe('image/png');
    expect(file.name).toBe('star.png');
    expect(alt).toBe('A star');
    vi.restoreAllMocks();
  });
});

const item = (over: Partial<AttachmentItem>): AttachmentItem => ({ emailId: 'm1', blobId: 'b1', name: 'a.png', mime: 'image/png', size: 5, from: 'Ann <ann@x.org>', subject: 'Holiday', receivedAt: '2026-01-01T00:00:00Z', ...over });

describe('message picture picker', () => {
  beforeEach(() => { Object.assign(URL, { createObjectURL: vi.fn(() => 'blob:t'), revokeObjectURL: vi.fn() }); });
  it('filters by sender, subject and name; images only for inline', () => {
    const items = [item({}), item({ blobId: 'b2', name: 'doc.pdf', mime: 'application/pdf', from: 'Bob', subject: 'Report' })];
    expect(filterMessageItems(items, 'ann', false)).toHaveLength(1);
    expect(filterMessageItems(items, '', true)).toHaveLength(1);
    expect(filterMessageItems(items, 'report', false)).toHaveLength(1);
  });
  it('lists thumbnails, searches, and returns the chosen files fetched through the host', async () => {
    const pages = { items: [item({}), item({ blobId: 'b2', name: 'b.jpg', mime: 'image/jpeg', from: 'Bob <bob@x.org>', subject: 'Trip' }), item({ blobId: 'b3', name: 'doc.pdf', mime: 'application/pdf' })], position: 0, nextPosition: null };
    const fetchPicture = vi.fn(async (i: AttachmentItem) => new File(['x'], i.name, { type: i.mime }));
    const onPick = vi.fn();
    render(() => <MessagePicturesDialog as="inline" loadPage={async () => pages} fetchPicture={fetchPicture} returnTo={null} onClose={noop} onPick={onPick} />);
    const cells = await screen.findAllByTestId('insert-messages-item');
    expect(cells).toHaveLength(2);
    fireEvent.input(screen.getByTestId('insert-messages-search'), { target: { value: 'bob' } });
    await waitFor(() => expect(screen.getAllByTestId('insert-messages-item')).toHaveLength(1));
    fireEvent.click(screen.getByTestId('insert-messages-item'));
    fireEvent.click(screen.getByTestId('insert-messages-dialog-apply'));
    await waitFor(() => expect(onPick).toHaveBeenCalled());
    expect(onPick.mock.calls[0]![0].map((f: File) => f.name)).toEqual(['b.jpg']);
  });
});

describe('picture file dialog', () => {
  it('passes the chosen files, alt text and placement; the admin default preset is preselected', async () => {
    const onInsert = vi.fn();
    const file = new File(['x'], 'a.gif', { type: 'image/gif' });
    render(() => <PictureFileDialog files={[file]} defaultPreset="medium" maxBytes={1000} returnTo={null} onClose={noop} onInsert={onInsert} />);
    expect((screen.getByTestId('insert-picture-preset') as HTMLSelectElement).value).toBe('medium');
    fireEvent.input(screen.getByTestId('insert-picture-alt'), { target: { value: 'An animation' } });
    fireEvent.click(screen.getByTestId('insert-picture-dialog-apply'));
    await waitFor(() => expect(onInsert).toHaveBeenCalledWith([file], 'An animation', false));
  });
  it('refuses a picture over the inline limit but allows it as an attachment', async () => {
    const onInsert = vi.fn();
    const big = new File([new Uint8Array(2000)], 'big.gif', { type: 'image/gif' });
    render(() => <PictureFileDialog files={[big]} defaultPreset="original" maxBytes={1000} returnTo={null} onClose={noop} onInsert={onInsert} />);
    fireEvent.click(screen.getByTestId('insert-picture-dialog-apply'));
    await screen.findByText(/larger than the inline picture limit/);
    expect(onInsert).not.toHaveBeenCalled();
    fireEvent.change(screen.getByTestId('insert-picture-placement'), { target: { value: 'attach' } });
    await apply('insert-picture-dialog-apply');
    await waitFor(() => expect(onInsert).toHaveBeenCalledWith([big], '', true));
  });
});

function dropHost(over: Partial<DropHost> = {}): DropHost {
  const root = document.createElement('div');
  document.body.append(root);
  return {
    root: () => root, attachAllowed: () => true, inlineAllowed: () => true, messagesAllowed: () => true, maxBytes: () => 1000, policy: () => null, accountId: () => 'a',
    attachFiles: vi.fn(async () => undefined), placeImages: vi.fn(async () => undefined), fetchAttachment: vi.fn(async () => new File(['x'], 'a.pdf', { type: 'application/pdf' })), fetchMessage: vi.fn(async () => new File(['eml'], 'm.eml', { type: 'message/rfc822' })),
    onBody: () => false, report: vi.fn(), ...over,
  };
}

/** A drag event with a data transfer carrying `files`. */
function dragEvent(type: string, target: Element, files: File[], opts: { shiftKey?: boolean; types?: string[] } = {}): Event {
  const event = new Event(type, { bubbles: true, cancelable: true }) as Event & { dataTransfer: unknown; shiftKey: boolean };
  event.dataTransfer = {
    types: [...(files.length ? ['Files'] : []), ...(opts.types ?? [])],
    items: files.map((f) => ({ kind: 'file', type: f.type })),
    files,
    getData: () => '',
    dropEffect: 'none',
  };
  event.shiftKey = opts.shiftKey === true;
  target.dispatchEvent(event);
  return event;
}

describe('drop zone', () => {
  const png = (name = 'p.png', size = 10): File => new File([new Uint8Array(size)], name, { type: 'image/png' });
  it('shows the overlay with counts and the limit while dragging, hides it on leave', async () => {
    const host = dropHost();
    render(() => <ComposeDropZone host={host} />);
    dragEvent('dragenter', host.root()!, [png(), new File(['x'], 'a.pdf', { type: 'application/pdf' })]);
    expect(await screen.findByTestId('compose-drop-overlay')).toBeInTheDocument();
    expect(screen.getByTestId('compose-drop-counts').textContent).toMatch(/2 files/);
    expect(screen.getByTestId('compose-drop-overlay').textContent).toMatch(/Shift/);
    dragEvent('dragleave', host.root()!, []);
    await waitFor(() => expect(screen.queryByTestId('compose-drop-overlay')).toBeNull());
  });
  it('attaches files, places images inline with Shift, and names refused files', async () => {
    const host = dropHost();
    render(() => <ComposeDropZone host={host} />);
    const files = [new File(['x'], 'a.pdf', { type: 'application/pdf' }), png('p.png'), new File([new Uint8Array(5000)], 'big.zip', { type: 'application/zip' })];
    dragEvent('dragenter', host.root()!, files);
    dragEvent('drop', host.root()!, files, { shiftKey: true });
    await waitFor(() => expect(host.attachFiles).toHaveBeenCalled());
    expect((host.attachFiles as ReturnType<typeof vi.fn>).mock.calls[0]![0].map((f: File) => f.name)).toEqual(['a.pdf']);
    expect((host.placeImages as ReturnType<typeof vi.fn>).mock.calls[0]![0].map((f: File) => f.name)).toEqual(['p.png']);
    const problem = await screen.findByTestId('compose-drop-problem');
    expect(problem.textContent).toMatch(/big\.zip/);
  });
  it('attaches images as files when not Shift and not on the body', async () => {
    const host = dropHost();
    render(() => <ComposeDropZone host={host} />);
    dragEvent('dragenter', host.root()!, [png()]);
    dragEvent('drop', host.root()!, [png()]);
    await waitFor(() => expect(host.attachFiles).toHaveBeenCalled());
    expect(host.placeImages).not.toHaveBeenCalled();
  });
  it('is inert when the features are off', () => {
    const host = dropHost({ attachAllowed: () => false, inlineAllowed: () => false, messagesAllowed: () => false });
    render(() => <ComposeDropZone host={host} />);
    const e = dragEvent('dragenter', host.root()!, [png()]);
    expect(e.defaultPrevented).toBe(false);
    expect(screen.queryByTestId('compose-drop-overlay')).toBeNull();
  });
  it('ignores a drag that carries no files or messages', () => {
    const host = dropHost();
    render(() => <ComposeDropZone host={host} />);
    dragEvent('dragenter', host.root()!, [], { types: ['text/plain'] });
    expect(screen.queryByTestId('compose-drop-overlay')).toBeNull();
    void MESSAGE_DRAG_TYPE;
  });
});

function insertHost(over: Partial<InsertHost> = {}): InsertHost {
  const [link, setLink] = createSignal<null | { caption: string; href: string; title: string; newTab: boolean; existing: boolean }>(null);
  return {
    disabled: () => false, insertBlock: vi.fn(), insertInline: vi.fn(), attachFiles: vi.fn(async () => undefined), placeImages: vi.fn(async () => undefined),
    identity: () => ({ name: 'Me', email: 'me@x.org' }), selectedText: () => '', contacts: () => [], loadContacts: async () => undefined, calendar: () => controller(),
    loadMessagePictures: async () => ({ items: [], position: 0, nextPosition: null }), fetchMessagePicture: async () => new File([], 'x'), richMode: () => true,
    maxBytes: () => 1e6, maxInlineMB: () => 10, resizeDefault: () => 'large', linkRequest: link, setLinkRequest: setLink, screenshotSupported: true,
    captureScreenshot: async () => null, selectedShape: () => null, placeShape: vi.fn(async () => undefined), report: vi.fn(), ...over,
  };
}

async function features(values: Record<string, boolean>): Promise<void> {
  const settings = Object.fromEntries(Object.entries(values).map(([k, v]) => [`compose.feature.${k}`, { value: v, source: 'user', locked: false }]));
  await loadSettings({ refresh: true, fetcher: (async () => new Response(JSON.stringify({ settings }), { status: 200, headers: { 'content-type': 'application/json' } })) as Fetcher });
}

describe('insert groups and feature flags', () => {
  it('renders every group when all features are on', () => {
    render(() => <InsertGroups host={insertHost()} />);
    for (const id of ['insert-group-illustrations', 'insert-group-links', 'insert-group-cards', 'insert-group-symbols']) expect(screen.getByTestId(id)).toBeInTheDocument();
    for (const id of ['insert-picture-file', 'insert-shapes', 'insert-screenshot', 'insert-link', 'insert-business-card', 'insert-calendar', 'insert-event', 'insert-symbols', 'insert-hr', 'insert-date', 'insert-attach-messages', 'insert-picture-messages']) expect(screen.getByTestId(id)).toBeInTheDocument();
  });
  it('a disabled feature is not rendered', async () => {
    await features({ 'insert.shapes': false, 'insert.businessCard': false, 'insert.link': false, 'insert.horizontalLine': false, 'insert.pictureFromMessages': false });
    render(() => <InsertGroups host={insertHost()} />);
    await waitFor(() => expect(screen.queryByTestId('insert-shapes')).toBeNull());
    for (const id of ['insert-business-card', 'insert-link', 'insert-hr', 'insert-picture-messages']) expect(screen.queryByTestId(id)).toBeNull();
    expect(screen.getByTestId('insert-calendar')).toBeInTheDocument();
  });
  it('a group whose features are all off disappears', async () => {
    await features({ 'insert.symbols': false, 'insert.horizontalLine': false, 'insert.dateStamp': false });
    render(() => <InsertGroups host={insertHost()} />);
    await waitFor(() => expect(screen.queryByTestId('insert-group-symbols')).toBeNull());
  });
  it('the horizontal rule and the date stamp insert through the host; the symbol inserts inline', () => {
    const host = insertHost();
    render(() => <InsertGroups host={host} />);
    fireEvent.click(screen.getByTestId('insert-hr'));
    expect(host.insertBlock).toHaveBeenCalledWith('<hr>', expect.stringContaining('---'));
    fireEvent.click(screen.getByTestId('insert-date'));
    expect(host.insertInline).toHaveBeenCalled();
  });
  it('the Link button asks the host for the link dialog with the selection as the caption', () => {
    const host = insertHost({ selectedText: () => 'selected words' });
    render(() => <InsertGroups host={host} />);
    fireEvent.click(screen.getByTestId('insert-link'));
    expect(screen.getByTestId('insert-link-caption')).toHaveValue('selected words');
  });
  it('picture from file opens the options dialog and then places the images', async () => {
    const host = insertHost();
    render(() => <InsertGroups host={host} />);
    const input = screen.getByLabelText('Picture from file…') as HTMLInputElement;
    const file = new File(['x'], 'a.gif', { type: 'image/gif' });
    fireEvent.change(input, { target: { files: [file] } });
    fireEvent.click(await screen.findByTestId('insert-picture-dialog-apply'));
    await waitFor(() => expect(host.placeImages).toHaveBeenCalledWith([file], undefined));
  });
});
