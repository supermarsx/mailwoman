import { afterEach, expect, it, vi } from 'vitest';
import { cleanup, render, screen, waitFor } from '@solidjs/testing-library';
import { MessageAttachmentViewer } from './MessageAttachmentViewer.tsx';
import { viewerKindFor } from '../contracts/viewer.ts';
import { apiFetch } from '../api/http.ts';

vi.mock('../api/http.ts', () => ({ apiFetch: vi.fn() }));
vi.mock('../api/transport.ts', () => ({ transportBase: () => '/webmail' }));
afterEach(() => { cleanup(); vi.resetAllMocks(); });
const props = { part: { partId: '2', blobId: 'b1', size: 3, type: 'message/rfc822' }, blobUrl: 'blob:message', mime: 'message/rfc822', name: 'message.eml' };

it('recognizes RFC822 and Outlook formats including generic download MIME', () => {
  for (const mime of ['message/rfc822', 'MESSAGE/GLOBAL; charset=UTF-8', 'application/ms-tnef', 'application/vnd.ms-tnef']) expect(viewerKindFor(mime)).toBe('message');
  expect(viewerKindFor('application/octet-stream', 'winmail.dat')).toBe('message');
  expect(viewerKindFor('', 'forwarded.EML')).toBe('message');
});
it('posts through the configured deployment prefix and uses an opaque frame', async () => {
  vi.mocked(apiFetch).mockResolvedValueOnce(new Response('abc')).mockResolvedValueOnce(new Response(JSON.stringify({ preview: { format: 'eml', subject: '<b>Subject</b>', html: '<p>Body</p>', attachments: [], warnings: [] } })));
  const { container } = render(() => <MessageAttachmentViewer {...props} />);
  await screen.findByRole('heading', { name: '<b>Subject</b>' });
  expect(apiFetch).toHaveBeenLastCalledWith('/webmail/api/attachments/preview', expect.objectContaining({ method: 'POST', body: JSON.stringify({ format: 'eml', bytesBase64: 'YWJj' }) }));
  expect(container.querySelector('iframe')).toHaveAttribute('sandbox', '');
  expect(container.querySelector('iframe')?.srcdoc).toContain("default-src 'none'");
  expect(container.querySelector('h2 b')).toBeNull();
});
it('rejects oversized attachments before fetching or uploading', async () => {
  render(() => <MessageAttachmentViewer {...props} part={{ ...props.part, size: 4 * 1024 * 1024 + 1 }} />);
  await screen.findByRole('alert');
  expect(apiFetch).not.toHaveBeenCalled();
});
it('does not parse a failed attachment download as a message', async () => {
  vi.mocked(apiFetch).mockResolvedValueOnce(new Response('denied', { status: 403 }));
  render(() => <MessageAttachmentViewer {...props} />);
  await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument());
  expect(apiFetch).toHaveBeenCalledTimes(1);
});
