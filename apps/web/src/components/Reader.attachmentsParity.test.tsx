// The Security panel lists every attachment-class part of the message
// (`AttachmentRisk[]`); the reader's bar must never show fewer. Fixtures mirror
// crates/mw-mime/tests/attachment_parity.rs, as the engine maps them.
import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render } from '@solidjs/testing-library';
import { AppContext } from '../state/context.ts';
import type { AppState } from '../state/store.ts';
import type { Email } from '../api/jmap-types.ts';
import { AttachmentsPane } from './Reader.tsx';
import { isInlineResourcePart } from './attachmentBarReason.ts';

const policy = { previewAllowed: true, downloadAllowed: true, maxBytes: 0, allowedExtensions: [], blockedExtensions: ['exe'], warnExtensions: [], allowedMimeTypes: [], blockedMimeTypes: [], warnMimeTypes: [] };
let n = 0;
const part = (name: string | null, type: string, extra: Record<string, unknown> = {}) => ({ blobId: `b.${++n}`, type, name, size: 100, ...extra });

const FIXTURES: Record<string, ReturnType<typeof part>[]> = {
  'pdf': [part('a.pdf', 'application/pdf', { disposition: 'attachment' })],
  'no filename': [part(null, 'application/octet-stream', { disposition: 'attachment' })],
  'tnef': [part('winmail.dat', 'application/ms-tnef', { disposition: 'attachment' })],
  'calendar': [part('invite.ics', 'text/calendar', { disposition: 'attachment' })],
  'smime signature': [part('smime.p7s', 'application/pkcs7-signature', { disposition: 'attachment' })],
  'pgp signature': [part('signature.asc', 'application/pgp-signature')],
  'forwarded message': [part('fwd.eml', 'message/rfc822', { disposition: 'attachment' })],
  'blocked exe': [part('bad.exe', 'application/x-msdownload', { disposition: 'attachment' })],
  'cid exe, no disposition': [part('x.exe', 'application/x-msdownload', { cid: 'x@y', disposition: 'inline' })],
  'inline image only': [part('l.png', 'image/png', { cid: 'l@y', disposition: 'inline' })],
  'file and inline images': [part('a.pdf', 'application/pdf', { disposition: 'attachment' }), part('l.png', 'image/png', { cid: 'l@y', disposition: 'inline' }), part('m.png', 'image/png', { cid: 'm@y', disposition: 'inline' })],
};

function mount(attachments: unknown[] | undefined, hasAttachment = true) {
  vi.stubGlobal('fetch', vi.fn(() => Promise.resolve(new Response(JSON.stringify(policy), { status: 200, headers: { 'content-type': 'application/json' } }))));
  const app = { accountId: () => 'a1', downloadUrl: () => 'https://m/d/{accountId}/{blobId}/{name}?type={type}', showToast: vi.fn(), openMessage: vi.fn(), nextcloud: { enabled: () => false, ensureEnabled: () => Promise.resolve(), service: {} } } as unknown as AppState;
  const email = { id: `m${++n}`, hasAttachment, ...(attachments ? { attachments } : {}) } as unknown as Email;
  return render(() => <AppContext.Provider value={app}><AttachmentsPane email={email} /></AppContext.Provider>);
}
const expand = (view: ReturnType<typeof mount>) => fireEvent.click(view.getByTestId('reader-attachments-toggle'));

afterEach(() => { cleanup(); vi.unstubAllGlobals(); });

describe('attachments bar lists every part the Security panel lists', () => {
  for (const [label, parts] of Object.entries(FIXTURES)) {
    it(`${label}: one thumbnail per listed part`, async () => {
      const view = mount(parts, !label.startsWith('inline') && !label.startsWith('cid'));
      expect(view.queryByTestId('reader-attachments-bar')).not.toBeNull();
      expand(view);
      await vi.waitFor(() => expect(view.container.querySelectorAll('.mw-thumb').length).toBe(parts.length));
    });
  }

  it('groups cid resources apart from files, and counts files in the headline', async () => {
    const view = mount(FIXTURES['file and inline images']);
    expect(view.getByTestId('reader-attachments-bar').textContent).toContain('1 attachment');
    expand(view);
    await vi.waitFor(() => expect(view.queryByTestId('reader-attachments-inline')).not.toBeNull());
    expect(view.getByTestId('reader-attachments-inline').textContent).toContain('2 inline images');
  });

  it('an inline-only message still gets a bar, headed by its inline images', () => {
    const view = mount(FIXTURES['inline image only'], false);
    expect(view.getByTestId('reader-attachments-bar').textContent).toContain('1 inline image');
  });

  it('a policy-blocked part stays in the bar', async () => {
    const view = mount(FIXTURES['blocked exe']);
    expand(view);
    await vi.waitFor(() => expect(view.container.querySelectorAll('.mw-thumb').length).toBe(1));
  });

  it('hasAttachment with an empty list shows the compact bar with Retry, not nothing', () => {
    const view = mount([], true);
    expect(view.queryByTestId('reader-attachments-bar')).not.toBeNull();
    expect(view.queryByTestId('reader-attachments-retry')).not.toBeNull();
  });

  it('no attachments at all: no bar', () => {
    expect(mount([], false).queryByTestId('reader-attachments-bar')).toBeNull();
  });

  it('classifies like mw-mime is_inline_resource', () => {
    expect(isInlineResourcePart({ cid: 'a', disposition: 'attachment' })).toBe(false);
    expect(isInlineResourcePart({ cid: 'a', disposition: 'inline', name: 'l.png' })).toBe(true);
    expect(isInlineResourcePart({ disposition: 'inline', name: 'a.pdf' })).toBe(false);
    expect(isInlineResourcePart({ disposition: 'inline', name: null })).toBe(true);
    expect(isInlineResourcePart({ disposition: 'attachment', name: null })).toBe(false);
  });
});
