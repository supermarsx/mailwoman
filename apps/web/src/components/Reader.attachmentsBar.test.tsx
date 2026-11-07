import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, render, waitFor } from '@solidjs/testing-library';
import { AppContext } from '../state/context.ts';
import type { AppState } from '../state/store.ts';
import type { Email } from '../api/jmap-types.ts';
import { AttachmentsPane } from './Reader.tsx';

const policy = { previewAllowed: true, downloadAllowed: true, maxBytes: 0, allowedExtensions: [], blockedExtensions: [], warnExtensions: [], allowedMimeTypes: [], blockedMimeTypes: [], warnMimeTypes: [] };

function setup(email: Record<string, unknown>, policyResponse: unknown = policy) {
  const showToast = vi.fn();
  const openMessage = vi.fn(() => Promise.resolve());
  vi.stubGlobal('fetch', vi.fn(() => Promise.resolve(policyResponse === null
    ? new Response('no', { status: 500 })
    : new Response(JSON.stringify(policyResponse), { status: 200, headers: { 'content-type': 'application/json' } }))));
  const app = {
    accountId: () => 'a1', downloadUrl: () => 'https://m/d/{accountId}/{blobId}/{name}?type={type}', showToast, openMessage,
    nextcloud: { enabled: () => false, ensureEnabled: () => Promise.resolve(), service: {} },
  } as unknown as AppState;
  const view = render(() => <AppContext.Provider value={app}><AttachmentsPane email={{ id: email['id'] as string, ...email } as unknown as Email} /></AppContext.Provider>);
  return { showToast, openMessage, view };
}

afterEach(() => { cleanup(); vi.unstubAllGlobals(); });

describe('AttachmentsPane bar and reasons', () => {
  it('renders the bar and raises no toast for a normal attachment', async () => {
    const { showToast, view } = setup({ id: 'ok1', hasAttachment: true, attachments: [{ blobId: 'b.1', type: 'application/pdf', name: 'a.pdf', size: 10 }] });
    expect(view.queryByTestId('reader-attachments-bar')).not.toBeNull();
    await new Promise((r) => setTimeout(r, 20));
    expect(showToast).not.toHaveBeenCalled();
  });
  it('shows a compact bar with Retry, and toasts once, when hasAttachment has no list', async () => {
    const { showToast, openMessage, view } = setup({ id: 'miss1', hasAttachment: true });
    expect(view.queryByTestId('reader-attachments-bar')).not.toBeNull();
    expect(view.queryByTestId('reader-attachments-retry')).not.toBeNull();
    await waitFor(() => expect(showToast).toHaveBeenCalledTimes(1));
    const [, , , extra] = showToast.mock.calls[0]!;
    extra.action.run();
    expect(openMessage).toHaveBeenCalledWith('miss1');
  });
  it('is silent for a message with no attachments', async () => {
    const { showToast, view } = setup({ id: 'none1', hasAttachment: false, attachments: [] });
    expect(view.queryByTestId('reader-attachments-bar')).toBeNull();
    await new Promise((r) => setTimeout(r, 20));
    expect(showToast).not.toHaveBeenCalled();
  });
  it('keeps the bar and toasts when the policy cannot be loaded', async () => {
    const { showToast, view } = setup({ id: 'pol1', hasAttachment: true, attachments: [{ blobId: 'b.1', type: 'application/pdf', name: 'a.pdf', size: 10 }] }, null);
    expect(view.queryByTestId('reader-attachments-bar')).not.toBeNull();
    await waitFor(() => expect(showToast).toHaveBeenCalledTimes(1));
    expect(showToast.mock.calls[0]![3].action).toBeDefined();
  });
  it('keeps the bar and toasts when the policy blocks the attachments', async () => {
    const { showToast, view } = setup({ id: 'blk1', hasAttachment: true, attachments: [{ blobId: 'b.1', type: 'application/pdf', name: 'a.pdf', size: 10 }] }, { ...policy, downloadAllowed: false });
    expect(view.queryByTestId('reader-attachments-bar')).not.toBeNull();
    await waitFor(() => expect(showToast).toHaveBeenCalledTimes(1));
  });
});
