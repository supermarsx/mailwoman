import { describe, it, expect } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { Attachments } from './Attachments.tsx';
import type { AttachmentItem } from '../viewers/attachments.ts';
import { Suspense } from 'solid-js';

const items: AttachmentItem[] = [
  { emailId: 'e1', blobId: 'b1', name: 'Q3-report.pdf', mime: 'application/pdf', size: 2_000_000, from: 'Alice <alice@example.org>', subject: 'Q3', receivedAt: '2026-03-01T00:00:00Z' },
  { emailId: 'e2', blobId: 'b2', name: 'logo.png', mime: 'image/png', size: 40_000, from: 'Bob <bob@corp.com>', subject: 'Logo', receivedAt: '2026-01-15T00:00:00Z' },
  { emailId: 'e3', blobId: 'b3', name: 'demo.mp4', mime: 'video/mp4', size: 8_000_000, from: 'Alice <alice@example.org>', subject: 'Demo', receivedAt: '2026-02-10T00:00:00Z' },
];

describe('Attachments module', () => {
  it('keeps the surrounding app visible while the attachment query is pending', async () => {
    let finish!: (value: AttachmentItem[]) => void;
    const pending = new Promise<AttachmentItem[]>(resolve => { finish = resolve; });
    render(() => <Suspense fallback={<p>Whole app loading</p>}><aside>Mailbox navigation</aside><Attachments load={() => pending} /></Suspense>);
    expect(screen.getByText('Mailbox navigation')).toBeVisible();
    expect(screen.queryByText('Whole app loading')).not.toBeInTheDocument();
    finish(items);
    await screen.findByText('Q3-report.pdf');
    expect(screen.getByText('Mailbox navigation')).toBeVisible();
  });
  it('renders a card per attachment', async () => {
    render(() => <Attachments items={items} />);
    await waitFor(() => expect(screen.getByText('Q3-report.pdf')).toBeInTheDocument());
    expect(screen.getByText('logo.png')).toBeInTheDocument();
    expect(screen.getByText('demo.mp4')).toBeInTheDocument();
  });

  it('filters via the shared search operators', async () => {
    render(() => <Attachments items={items} />);
    await waitFor(() => expect(screen.getByText('Q3-report.pdf')).toBeInTheDocument());

    fireEvent.input(screen.getByLabelText('Search attachments'), {
      target: { value: 'type:video from:alice' },
    });

    await waitFor(() => {
      expect(screen.getByText('demo.mp4')).toBeInTheDocument();
      expect(screen.queryByText('Q3-report.pdf')).not.toBeInTheDocument();
      expect(screen.queryByText('logo.png')).not.toBeInTheDocument();
    });
  });

  it('filters via the type dropdown', async () => {
    render(() => <Attachments items={items} />);
    await waitFor(() => expect(screen.getByText('logo.png')).toBeInTheDocument());

    fireEvent.change(screen.getByLabelText('Filter by type'), { target: { value: 'image' } });

    await waitFor(() => {
      expect(screen.getByText('logo.png')).toBeInTheDocument();
      expect(screen.queryByText('Q3-report.pdf')).not.toBeInTheDocument();
    });
  });

  it('shows an empty state when nothing matches', async () => {
    render(() => <Attachments items={items} />);
    await waitFor(() => expect(screen.getByText('logo.png')).toBeInTheDocument());
    fireEvent.input(screen.getByLabelText('Search attachments'), {
      target: { value: 'filename:nonexistent-zzz' },
    });
    await waitFor(() => expect(screen.getByText('No attachments match.')).toBeInTheDocument());
  });
});
