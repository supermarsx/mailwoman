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
import type { AttachedFile, WebDavEntry, Fetcher } from './service.ts';

function withLinks(fetcher:Fetcher):Fetcher{return (input,init)=>input==='/api/account/nextcloud'?Promise.resolve(okJson({accountId:'test-account',instances:[{id:'cloud',name:'Cloud'}],links:[{instanceId:'cloud',ready:true}]})):fetcher(input.replace('/api/nextcloud/files?instanceId=cloud&','/api/nextcloud/list?'),init);}

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
    render(() => <NextcloudAttach fetcher={withLinks(fetcher)} onAttached={(f) => (got = f)} />);

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
    render(() => <ShareLinkComposer path="/report.pdf" fetcher={withLinks(fetcher)} onCreated={() => {}} />);

    // Fields are hidden until their toggle is checked.
    expect(screen.queryByLabelText('Share password')).not.toBeInTheDocument();
    expect(screen.queryByLabelText('Expiry date')).not.toBeInTheDocument();

    fireEvent.click(screen.getByLabelText('Protect with a password'));
    fireEvent.click(screen.getByLabelText('Set an expiry date'));
    await waitFor(() => expect(screen.getByLabelText('Share password')).toBeInTheDocument());
    expect(screen.getByLabelText('Expiry date')).toBeInTheDocument();

    fireEvent.input(screen.getByLabelText('Share password'), { target: { value: 's3cret' } });
    fireEvent.input(screen.getByLabelText('Expiry date'), { target: { value: '2026-12-31' } });
    await waitFor(()=>expect(screen.getByRole('button',{name:'Create link'})).toBeEnabled());
    fireEvent.click(screen.getByRole('button', { name: 'Create link' }));

    await waitFor(() => expect(screen.getByTestId('nc-share-url')).toBeInTheDocument());
    expect(screen.getByTestId('nc-share-url')).toHaveTextContent('https://nc.example/s/AbC');

    const body = JSON.parse((fetcher.mock.calls.find((c) => c[0] === '/api/nextcloud/share-link')?.[1]?.body as string) ?? '{}') as {
      path: string;
      password?: string;
      expiresAt?: string;
    };
    expect(body).toEqual({ accountId:'test-account', instanceId:'cloud', path: '/report.pdf', password: 's3cret', expiresAt: '2026-12-31' });
  });

  it('omits password + expiry when their toggles are off', async () => {
    const fetcher = vi.fn(async (_input: string, _init?: RequestInit) =>
      okJson({ url: 'https://nc.example/s/XyZ', expiresAt: null, passwordProtected: false }),
    );
    render(() => <ShareLinkComposer path="/big.zip" fetcher={withLinks(fetcher)} onCreated={() => {}} />);

    await waitFor(()=>expect(screen.getByRole('button',{name:'Create link'})).toBeEnabled());
    fireEvent.click(screen.getByRole('button', { name: 'Create link' }));
    await waitFor(() => expect(screen.getByTestId('nc-share-url')).toBeInTheDocument());
    const body = JSON.parse((fetcher.mock.calls[0]?.[1]?.body as string) ?? '{}') as Record<string, unknown>;
    expect(body).toEqual({ accountId:'test-account', instanceId:'cloud', path: '/big.zip' });
  });

  it('validates that a chosen password is not empty', async () => {
    const fetcher = vi.fn(async () => okJson({}));
    render(() => <ShareLinkComposer path="/a" fetcher={withLinks(fetcher)} onCreated={() => {}} />);
    fireEvent.click(screen.getByLabelText('Protect with a password'));
    await waitFor(()=>expect(screen.getByRole('button',{name:'Create link'})).toBeEnabled());
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
        <ShareLinkComposer path="/a" fetcher={withLinks(fetcher)} onCreated={() => {}} />
        <ToastRegion />
      </>
    ));
    await waitFor(()=>expect(screen.getByRole('button',{name:'Create link'})).toBeEnabled());
    fireEvent.click(screen.getByRole('button', { name: 'Create link' }));
    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent('Nextcloud request refused (502).'));
    expect(toasts().map((i) => i.tone)).toEqual(['error']);
  });
});

describe('share-link composer — large local file (t39)', () => {
  it('uploads the file in chunks to the selected instance, then shares the stored copy', async () => {
    const order: string[] = [];
    const fetcher = vi.fn(async (input: string, init?: RequestInit) => {
      order.push(`${init?.method ?? 'GET'} ${input.split('?')[0]}`);
      if (input === '/api/nextcloud/upload') return okJson({ transfer: 'c'.repeat(32), path: '/Mail attachments/huge.zip', chunkBytes: 8 });
      if (input.includes('/finish')) return okJson({ path: '/Mail attachments/huge.zip', name: 'huge.zip', size: 20 });
      if (input === '/api/nextcloud/share-link') return okJson({ url: 'https://nc.example/s/Big', expiresAt: null, passwordProtected: false });
      return new Response(null, { status: 204 });
    });
    const created: string[] = [];
    let dismissed = false;
    const file = new File([new Uint8Array(20)], 'huge.zip');
    render(() => <ShareLinkComposer file={file} fetcher={withLinks(fetcher)} onCreated={(l) => created.push(l.url)} onDismiss={() => (dismissed = true)} />);
    await waitFor(() => expect(screen.getByText(/huge\.zip.*larger than the attachment limit/)).toBeInTheDocument());
    await waitFor(() => expect(screen.getByRole('button', { name: 'Upload and insert link' })).toBeEnabled());
    fireEvent.click(screen.getByRole('button', { name: 'Upload and insert link' }));
    await waitFor(() => expect(created).toEqual(['https://nc.example/s/Big']));
    expect(order.filter((line) => line.startsWith('PUT')).length).toBe(3);
    expect(order.at(-1)).toBe('POST /api/nextcloud/share-link');
    const share = JSON.parse((fetcher.mock.calls.find((c) => c[0] === '/api/nextcloud/share-link')?.[1]?.body as string) ?? '{}') as Record<string, unknown>;
    expect(share).toEqual({ accountId: 'test-account', instanceId: 'cloud', path: '/Mail attachments/huge.zip' });
    // One link per panel: the action is done.
    expect(screen.getByRole('button', { name: 'Upload and insert link' })).toBeDisabled();
    fireEvent.click(screen.getByRole('button', { name: 'Not now' }));
    expect(dismissed).toBe(true);
  });

  it('shares the already stored copy when the link is retried', async () => {
    let shares = 0;
    const fetcher = vi.fn(async (input: string, init?: RequestInit) => {
      if (input === '/api/nextcloud/upload') return okJson({ transfer: 'd'.repeat(32), path: '/Mail attachments/a.bin', chunkBytes: 8 });
      if (input.includes('/finish')) return okJson({ path: '/Mail attachments/a.bin', name: 'a.bin', size: 4 });
      if (input === '/api/nextcloud/share-link') {
        shares++;
        return shares === 1 ? new Response('', { status: 502 }) : okJson({ url: 'https://nc.example/s/R', expiresAt: null, passwordProtected: false });
      }
      void init;
      return new Response(null, { status: 204 });
    });
    render(() => (
      <>
        <ShareLinkComposer file={new File([new Uint8Array(4)], 'a.bin')} fetcher={withLinks(fetcher)} onCreated={() => {}} />
        <ToastRegion />
      </>
    ));
    await waitFor(() => expect(screen.getByRole('button', { name: 'Upload and insert link' })).toBeEnabled());
    fireEvent.click(screen.getByRole('button', { name: 'Upload and insert link' }));
    await waitFor(() => expect(shares).toBe(1));
    await waitFor(() => expect(screen.getByRole('button', { name: 'Upload and insert link' })).toBeEnabled());
    fireEvent.click(screen.getByRole('button', { name: 'Upload and insert link' }));
    await waitFor(() => expect(screen.getByTestId('nc-share-url')).toHaveTextContent('https://nc.example/s/R'));
    expect(fetcher.mock.calls.filter((c) => c[0] === '/api/nextcloud/upload')).toHaveLength(1);
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
        <SaveToNextcloud attachment={{ emailId:'message-1', blobId: 'b1', name: 'a.pdf' }} fetcher={withLinks(fetcher)} />
        <ToastRegion />
      </>
    ));
    await waitFor(()=>expect(screen.getByRole('button',{name:/Save here/})).toBeEnabled());
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
        <NextcloudAttach fetcher={withLinks(fetcher)} onAttached={() => {}} />
        <ToastRegion />
      </>
    ));
    await waitFor(() => expect(screen.getByText('report.pdf')).toBeInTheDocument());
    fireEvent.click(screen.getByRole('button', { name: 'Attach' }));
    await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument());
    expect(toasts()).toEqual([]);
    fireEvent.click(screen.getByText('report.pdf'));
    fireEvent.click(screen.getByRole('button', { name: /Attach 1 file/ }));
    await waitFor(() => expect(toasts().map((i) => [i.tone, i.message])).toEqual([['error', 'Nextcloud request refused (500).']]));
  });
});


describe('picker loading isolation',()=>{
  it('keeps the containing workspace mounted while listing and after refusal',async()=>{
    let reject!:(error:Error)=>void;
    const service=new NextcloudService(withLinks(()=>new Promise<Response>((_resolve,fail)=>{reject=fail;})));
    render(()=><Suspense fallback={<p>Workspace suspended</p>}><span>Retained workspace</span><FilePicker service={service} mode="files"/></Suspense>);
    expect(screen.getByText('Retained workspace')).toBeVisible();
    expect(screen.queryByText('Workspace suspended')).not.toBeInTheDocument();
    await waitFor(()=>expect(reject).toBeDefined());
    reject(new Error('Access refused'));
    await screen.findByRole('alert');
    expect(screen.getByText('Retained workspace')).toBeVisible();
  });
});


describe('Nextcloud attachment warning acknowledgment',()=>{
  it('never automatically retries a warning and requires a deliberate acknowledgment',async()=>{
    const writes:Record<string,unknown>[]=[];
    const fetcher:Fetcher=async(path,init)=>{
      if(path.startsWith('/api/nextcloud/list'))return okJson({entries:[]});
      const body=JSON.parse(init?.body as string) as Record<string,unknown>;writes.push(body);
      return body['acknowledgeWarning']===true?okJson({entry:{name:'note.txt',path:'/note.txt',size:1,isDir:false}}):new Response(JSON.stringify({code:'attachmentWarningRequired'}),{status:403});
    };
    render(()=><SaveToNextcloud accountId="test-account" attachment={{emailId:'message',blobId:'blob',name:'note.txt'}} fetcher={withLinks(fetcher)}/>);
    const save=screen.getByRole('button',{name:/Save here/});await waitFor(()=>expect(save).toBeEnabled());fireEvent.click(save);
    await screen.findByRole('alert');expect(writes).toHaveLength(1);expect(save).toBeDisabled();
    fireEvent.click(screen.getByRole('checkbox',{name:/I understand the attachment warning/}));fireEvent.click(save);
    await waitFor(()=>expect(writes).toHaveLength(2));expect(writes[0]?.['acknowledgeWarning']).toBeUndefined();expect(writes[1]?.['acknowledgeWarning']).toBe(true);
  });
});
