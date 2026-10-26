import {Suspense} from 'solid-js';
import {FilePicker} from './FilePicker.tsx';
import {NextcloudService} from './service.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
beforeEach(()=>{suspendAccountPreferences();activateAccountPreferences('test-account');});
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { NextcloudAttach } from './NextcloudAttach.tsx';
import { ShareLinkComposer } from './ShareLinkComposer.tsx';
import { SaveToNextcloud } from './SaveToNextcloud.tsx';
import { ToastRegion, toasts } from '../../components/toast/index.ts';
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
    // a11y: rows are real buttons (keyboard-operable, focusable), not click-only <li>.
    const row = screen.getByText('report.pdf').closest('button');
    expect(row).not.toBeNull();
    expect(row).toHaveAttribute('aria-pressed', 'false');
    fireEvent.click(screen.getByText('report.pdf'));
    await waitFor(() => expect(screen.getByTestId('nc-selected')).toBeInTheDocument());
    // a11y: selection is exposed non-visually via aria-pressed (WCAG 1.4.1), not colour alone.
    expect(row).toHaveAttribute('aria-pressed', 'true');

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
    expect(body).toEqual({ accountId:'test-account', path: '/report.pdf', password: 's3cret', expiresAt: '2026-12-31' });
  });

  it('omits password + expiry when their toggles are off', async () => {
    const fetcher = vi.fn(async (_input: string, _init?: RequestInit) =>
      okJson({ url: 'https://nc.example/s/XyZ', expiresAt: null, passwordProtected: false }),
    );
    render(() => <ShareLinkComposer path="/big.zip" fetcher={fetcher} onCreated={() => {}} />);

    fireEvent.click(screen.getByRole('button', { name: 'Create link' }));
    await waitFor(() => expect(screen.getByTestId('nc-share-url')).toBeInTheDocument());
    const body = JSON.parse((fetcher.mock.calls[0]?.[1]?.body as string) ?? '{}') as Record<string, unknown>;
    expect(body).toEqual({ accountId:'test-account', path: '/big.zip' });
  });

  it('validates that a chosen password is not empty', async () => {
    const fetcher = vi.fn(async () => okJson({}));
    render(() => <ShareLinkComposer path="/a" fetcher={fetcher} onCreated={() => {}} />);
    fireEvent.click(screen.getByLabelText('Protect with a password'));
    fireEvent.click(screen.getByRole('button', { name: 'Create link' }));
    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent(/password/));
    expect(fetcher).not.toHaveBeenCalled();
    // A missing field stays on the form, not in a toast.
    expect(toasts()).toEqual([]);
  });

  it('reports a refused share link as an error toast', async () => {
    const fetcher = vi.fn(async () => new Response('', { status: 502 }));
    render(() => (
      <>
        <ShareLinkComposer path="/a" fetcher={fetcher} onCreated={() => {}} />
        <ToastRegion />
      </>
    ));
    fireEvent.click(screen.getByRole('button', { name: 'Create link' }));
    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent('nextcloud request failed: 502'));
    expect(toasts().map((i) => i.tone)).toEqual(['error']);
  });
});

describe('results as toasts (t32 f39)', () => {
  it('a save to Nextcloud says where it went in a success toast', async () => {
    const saved: WebDavEntry = { name: 'a.pdf', path: '/Docs/a.pdf', isDir: false, size: 1, modified: null, contentType: null };
    const fetcher = vi.fn(async (input: string) => {
      if (input.startsWith('/api/nextcloud/list')) return okJson({ entries: rootEntries });
      if (input === '/api/nextcloud/save') return okJson({ entry: saved });
      return okJson({});
    });
    render(() => (
      <>
        <SaveToNextcloud attachment={{ emailId:'message-1', blobId: 'b1', name: 'a.pdf' }} fetcher={fetcher} />
        <ToastRegion />
      </>
    ));
    fireEvent.click(await screen.findByRole('button', { name: /Save here/ }));
    await waitFor(() => expect(screen.getByTestId('nc-saved')).toHaveAttribute('role', 'status'));
    expect(screen.getByTestId('nc-saved')).toHaveTextContent('/Docs/a.pdf');
  });

  it('a failed attach is an error toast; "select a file" stays on the panel', async () => {
    const fetcher = vi.fn(async (input: string) => {
      if (input.startsWith('/api/nextcloud/list')) return okJson({ entries: rootEntries });
      return new Response('', { status: 500 });
    });
    render(() => (
      <>
        <NextcloudAttach fetcher={fetcher} onAttached={() => {}} />
        <ToastRegion />
      </>
    ));
    await waitFor(() => expect(screen.getByText('report.pdf')).toBeInTheDocument());
    fireEvent.click(screen.getByRole('button', { name: 'Attach' }));
    await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument());
    expect(toasts()).toEqual([]);
    fireEvent.click(screen.getByText('report.pdf'));
    fireEvent.click(screen.getByRole('button', { name: /Attach 1 file/ }));
    await waitFor(() => expect(toasts().map((i) => [i.tone, i.message])).toEqual([['error', 'nextcloud request failed: 500']]));
  });
});


describe('picker loading isolation',()=>{
  it('keeps the containing workspace mounted while listing and after refusal',async()=>{
    let reject!:(error:Error)=>void;
    const service=new NextcloudService(()=>new Promise<Response>((_resolve,fail)=>{reject=fail;}));
    render(()=><Suspense fallback={<p>Workspace suspended</p>}><span>Retained workspace</span><FilePicker service={service} mode="files"/></Suspense>);
    expect(screen.getByText('Retained workspace')).toBeVisible();
    expect(screen.queryByText('Workspace suspended')).not.toBeInTheDocument();
    reject(new Error('Access refused'));
    await screen.findByRole('alert');
    expect(screen.getByText('Retained workspace')).toBeVisible();
  });
});
