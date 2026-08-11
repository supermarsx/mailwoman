// t22 L6 — the object-URL create/revoke BALANCE, counted on the real components.
//
// The instrument is a pair of counters on `URL.createObjectURL` /
// `URL.revokeObjectURL` (jsdom implements neither, so the stub is also what makes
// the components run at all). Counts, not bytes: jsdom does not model memory, so
// a megabyte figure here would be fiction. What can be proved honestly is that
// every URL the app mints is handed back.
//
// Two failure modes this file is written against:
//
//   * "measured while still mounted" — the balance is asserted AFTER `unmount()`,
//     because a component that revokes only at unmount passes a mounted check
//     trivially, and one that never revokes passes it too;
//   * "a leak test that only opens" — `created` is asserted to be non-zero before
//     the balance is believed, so a run where the fetch never happened (and the
//     balance is 0 = 0) cannot pass as a fix.
//
// Both components under test are the SHIPPING ones: `<ThumbnailStrip>` with the
// real `fetchObjectUrl`, and `<AttachmentsPane>` — the reader's attachment pane —
// exported from `Reader.tsx` for exactly this. Only the app context is a double.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, waitFor, fireEvent } from '@solidjs/testing-library';
import { AppContext } from '../state/context.ts';
import type { AppState } from '../state/store.ts';
import type { Email } from '../api/jmap-types.ts';
import { AttachmentsPane } from '../components/Reader.tsx';
import { ThumbnailStrip, type StripItem } from './ThumbnailStrip.tsx';
import { fetchObjectUrl } from './attachments.ts';
import { createObjectUrlOwner } from './objectUrl.ts';

let created = 0;
let revoked = 0;
/** URLs minted and not yet handed back — the live set the tab would be pinning. */
const live = new Set<string>();

beforeEach(() => {
  created = 0;
  revoked = 0;
  live.clear();
  Object.assign(URL, {
    createObjectURL: vi.fn(() => {
      created += 1;
      const url = `blob:mw/${created}`;
      live.add(url);
      return url;
    }),
    revokeObjectURL: vi.fn((url: string) => {
      revoked += 1;
      live.delete(url);
    }),
  });
  vi.stubGlobal(
    'fetch',
    vi.fn(async () => ({
      ok: true,
      blob: async () => new Blob([new Uint8Array([1, 2, 3])], { type: 'image/png' }),
      text: async () => 'x',
    })),
  );
});

afterEach(() => {
  vi.unstubAllGlobals();
});

function items(n: number, mime: string): StripItem[] {
  return Array.from({ length: n }, (_, i) => ({
    blobId: `b${i}`,
    name: `file${i}`,
    mime,
    size: 10,
  }));
}

/** The two accessors `<AttachmentsPane>` reads off the app state. */
function appDouble(): AppState {
  return {
    downloadUrl: () => '/jmap/download/{accountId}/{blobId}/{name}',
    accountId: () => 'acct1',
  } as unknown as AppState;
}

function emailWith(parts: StripItem[]): Email {
  return {
    id: 'm1',
    mailboxIds: { inbox: true },
    from: [{ name: null, email: 'a@example.org' }],
    to: [],
    subject: 's',
    receivedAt: '2026-01-01T00:00:00Z',
    preview: '',
    keywords: {},
    attachments: parts.map((p) => ({
      partId: null,
      blobId: p.blobId,
      size: p.size,
      type: p.mime,
      name: p.name,
    })),
  } as unknown as Email;
}

describe('object-URL ownership — the balance (t22 L6)', () => {
  it('ThumbnailStrip: every preview URL is revoked when the strip unmounts', async () => {
    const strip = items(12, 'image/png');
    const { unmount } = render(() => (
      <ThumbnailStrip items={strip} resolveThumb={(it) => fetchObjectUrl(`/d/${it.blobId}`)} />
    ));

    // One object URL per image thumbnail — the strip renders all of them, with
    // no windowing. This is the master-failing state: 12 created, 0 revoked.
    await waitFor(() => expect(created).toBe(12));
    expect(revoked).toBe(0);

    unmount();

    await waitFor(() => expect(revoked).toBe(created));
    expect(live.size).toBe(0);
    // Negative control: the balance above is only evidence if URLs were minted.
    expect(created).toBeGreaterThan(0);
  });

  it('ThumbnailStrip: a download that lands after unmount is still revoked', async () => {
    let settle: (() => void) | undefined;
    const gate = new Promise<void>((r) => {
      settle = r;
    });
    const { unmount } = render(() => (
      <ThumbnailStrip
        items={items(3, 'image/png')}
        resolveThumb={async (it) => {
          await gate;
          return await fetchObjectUrl(`/d/${it.blobId}`);
        }}
      />
    ));
    unmount();
    settle?.();

    await waitFor(() => expect(created).toBe(3));
    await waitFor(() => expect(revoked).toBe(3));
    expect(live.size).toBe(0);
  });

  it('AttachmentsPane: opening 12 attachments in turn keeps at most one live', async () => {
    // Non-image parts, so the strip mints nothing and `live` counts only the
    // opened-attachment path.
    const parts = items(12, 'application/zip');
    const { container, unmount } = render(() => (
      <AppContext.Provider value={appDouble()}>
        <AttachmentsPane email={emailWith(parts)} />
      </AppContext.Provider>
    ));

    const thumbs = container.querySelectorAll<HTMLButtonElement>('.mw-thumb');
    expect(thumbs.length).toBe(12);
    expect(created).toBe(0);

    for (let i = 0; i < thumbs.length; i++) {
      fireEvent.click(thumbs[i]!);
      await waitFor(() => expect(created).toBe(i + 1));
      // The superseded attachment is released as it is superseded, not banked
      // until unmount: one open attachment, one live Blob.
      await waitFor(() => expect(live.size).toBe(1));
    }

    // Closing the viewer releases the last one while still mounted.
    fireEvent.click(container.querySelector<HTMLButtonElement>('.attachment-modal button')!);
    await waitFor(() => expect(live.size).toBe(0));

    unmount();
    expect(created).toBe(12);
    expect(revoked).toBe(created);
    expect(live.size).toBe(0);
  });

  it('AttachmentsPane: closing the message (unmount) with an attachment open balances', async () => {
    const parts = items(4, 'application/zip');
    const { container, unmount } = render(() => (
      <AppContext.Provider value={appDouble()}>
        <AttachmentsPane email={emailWith(parts)} />
      </AppContext.Provider>
    ));
    fireEvent.click(container.querySelector<HTMLButtonElement>('.mw-thumb')!);
    await waitFor(() => expect(created).toBe(1));
    expect(revoked).toBe(0);

    unmount();

    expect(created).toBeGreaterThan(0);
    expect(revoked).toBe(created);
    expect(live.size).toBe(0);
  });

  it('image attachments: strip previews AND the opened blob both balance', async () => {
    const parts = items(6, 'image/png');
    const { container, unmount } = render(() => (
      <AppContext.Provider value={appDouble()}>
        <AttachmentsPane email={emailWith(parts)} />
      </AppContext.Provider>
    ));
    await waitFor(() => expect(created).toBe(6)); // six thumbnails
    fireEvent.click(container.querySelector<HTMLButtonElement>('.mw-thumb')!);
    await waitFor(() => expect(created).toBe(7)); // + the opened attachment

    unmount();

    expect(revoked).toBe(7);
    expect(live.size).toBe(0);
  });
});

describe('createObjectUrlOwner', () => {
  it('revokes on scope disposal and ignores the empty placeholder', () => {
    let own: ReturnType<typeof createObjectUrlOwner> | undefined;
    const { unmount } = render(() => {
      own = createObjectUrlOwner();
      own.adopt(URL.createObjectURL(new Blob(['a'])));
      own.adopt(''); // "no account yet" — never enters the registry
      return <span />;
    });
    expect(own?.size()).toBe(1);
    unmount();
    expect(own?.size()).toBe(0);
    expect(revoked).toBe(1);
  });

  it('release() is a no-op for a URL it does not own', () => {
    let own: ReturnType<typeof createObjectUrlOwner> | undefined;
    const { unmount } = render(() => {
      own = createObjectUrlOwner();
      return <span />;
    });
    own?.release('blob:not-ours');
    expect(revoked).toBe(0);
    unmount();
  });
});
