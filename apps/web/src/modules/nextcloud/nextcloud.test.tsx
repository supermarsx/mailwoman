import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { NextcloudAttach } from './NextcloudAttach.tsx';
import { ShareLinkComposer } from './ShareLinkComposer.tsx';
import type { AttachedFile, WebDavEntry } from './service.ts';

function okJson(body: unknown): Response {
  return new Response(JSON.stringify(body), { status: 200, headers: { 'content-type': 'application/json' } });
}

const rootEntries: WebDavEntry[] = [
  { name: 'Docs', path: '/Docs', isDir: true, size: 0, modified: null, contentType: null },
  { name: 'report.pdf', path: '/report.pdf', isDir: false, size: 2048, modified: null, contentType: 'application/pdf' },
];

describe('attach-from-Nextcloud (WebDAV picker)', () => {
  it('lists files, selects one, and hands the materialised attachment to the composer', async () => {
    const attached: AttachedFile[] = [{ name: 'report.pdf', blobId: 'blob-1', size: 2048, contentType: 'application/pdf' }];
    const fetcher = vi.fn(async (input: string, init?: RequestInit) => {
      if (input.startsWith('/api/nextcloud/list')) return okJson({ entries: rootEntries });
      if (input === '/api/nextcloud/attach' && init?.method === 'POST') return okJson({ attachments: attached });
      return okJson({});
    });
    let got: AttachedFile[] | null = null;
    render(() => <NextcloudAttach fetcher={fetcher} onAttached={(f) => (got = f)} />);

    await waitFor(() => expect(screen.getByText('report.pdf')).toBeInTheDocument());
    fireEvent.click(screen.getByText('report.pdf'));
    await waitFor(() => expect(screen.getByTestId('nc-selected')).toBeInTheDocument());

    fireEvent.click(screen.getByRole('button', { name: /Attach 1 file/ }));
    await waitFor(() => expect(got).not.toBeNull());
    expect(got!).toHaveLength(1);
    const body = JSON.parse((fetcher.mock.calls.find((c) => c[0] === '/api/nextcloud/attach')?.[1]?.body as string) ?? '{}') as {
      paths: string[];
    };
    expect(body.paths).toEqual(['/report.pdf']);
  });
});

describe('share-link composer — password + expiry controls', () => {
  it('reveals the password and expiry fields on toggle and sends them', async () => {
    const fetcher = vi.fn(async (input: string, _init?: RequestInit) => {
      if (input === '/api/nextcloud/share-link') {
        return okJson({ url: 'https://nc.example/s/AbC', expiresAt: '2026-12-31', passwordProtected: true });
      }
      return okJson({});
    });
    render(() => <ShareLinkComposer path="/report.pdf" fetcher={fetcher} onCreated={() => {}} />);

    // Fields are hidden until their toggle is checked.
    expect(screen.queryByLabelText('Share password')).not.toBeInTheDocument();
    expect(screen.queryByLabelText('Expiry date')).not.toBeInTheDocument();

    fireEvent.click(screen.getByLabelText('Protect with a password'));
    fireEvent.click(screen.getByLabelText('Set an expiry date'));
    await waitFor(() => expect(screen.getByLabelText('Share password')).toBeInTheDocument());
    expect(screen.getByLabelText('Expiry date')).toBeInTheDocument();

    fireEvent.input(screen.getByLabelText('Share password'), { target: { value: 's3cret' } });
    fireEvent.input(screen.getByLabelText('Expiry date'), { target: { value: '2026-12-31' } });
    fireEvent.click(screen.getByRole('button', { name: 'Create link' }));

    await waitFor(() => expect(screen.getByTestId('nc-share-url')).toBeInTheDocument());
    expect(screen.getByTestId('nc-share-url')).toHaveTextContent('https://nc.example/s/AbC');

    const body = JSON.parse((fetcher.mock.calls.find((c) => c[0] === '/api/nextcloud/share-link')?.[1]?.body as string) ?? '{}') as {
      path: string;
      password?: string;
      expiresAt?: string;
    };
    expect(body).toEqual({ path: '/report.pdf', password: 's3cret', expiresAt: '2026-12-31' });
  });

  it('omits password + expiry when their toggles are off', async () => {
    const fetcher = vi.fn(async (_input: string, _init?: RequestInit) =>
      okJson({ url: 'https://nc.example/s/XyZ', expiresAt: null, passwordProtected: false }),
    );
    render(() => <ShareLinkComposer path="/big.zip" fetcher={fetcher} onCreated={() => {}} />);

    fireEvent.click(screen.getByRole('button', { name: 'Create link' }));
    await waitFor(() => expect(screen.getByTestId('nc-share-url')).toBeInTheDocument());
    const body = JSON.parse((fetcher.mock.calls[0]?.[1]?.body as string) ?? '{}') as Record<string, unknown>;
    expect(body).toEqual({ path: '/big.zip' });
  });

  it('validates that a chosen password is not empty', async () => {
    const fetcher = vi.fn(async () => okJson({}));
    render(() => <ShareLinkComposer path="/a" fetcher={fetcher} onCreated={() => {}} />);
    fireEvent.click(screen.getByLabelText('Protect with a password'));
    fireEvent.click(screen.getByRole('button', { name: 'Create link' }));
    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent(/password/));
    expect(fetcher).not.toHaveBeenCalled();
  });
});
