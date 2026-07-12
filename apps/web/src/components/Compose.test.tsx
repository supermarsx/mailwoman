import { describe, it, expect, beforeEach } from 'vitest';
import { screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { Compose } from './Compose.tsx';
import { renderWithApp } from './appHarness.tsx';
import type { Identity } from '../api/jmap-types.ts';

const IDENTITIES: Identity[] = [
  { id: 'id1', name: 'Personal', email: 'me@example.org', replyTo: null, signatureHtml: null, signatureText: 'Sent from Mailwoman', sentMailboxId: 'sent' },
  { id: 'id2', name: 'Work', email: 'work@corp.example', replyTo: null, signatureHtml: null, signatureText: null, sentMailboxId: 'sent' },
];

describe('Compose', () => {
  beforeEach(() => localStorage.clear());

  it('keeps the core To/Subject/Body labels + Send button (e2e contract)', () => {
    renderWithApp(() => <Compose onClose={() => undefined} />);
    expect(screen.getByLabelText('To')).toBeInTheDocument();
    expect(screen.getByLabelText('Subject')).toBeInTheDocument();
    expect(screen.getByLabelText('Body')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Send' })).toBeInTheDocument();
  });

  it('offers the sending identities once loaded', async () => {
    const { app } = renderWithApp(() => <Compose onClose={() => undefined} />, { identities: IDENTITIES });
    await app.loadIdentities();
    const select = await screen.findByLabelText('From');
    expect(select).toBeInTheDocument();
    expect(screen.getByRole('option', { name: /Work/ })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: /Personal/ })).toBeInTheDocument();
  });

  it('sends via the chosen identity', async () => {
    const { app } = renderWithApp(() => <Compose onClose={() => undefined} />, { identities: IDENTITIES });
    await app.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });
    await app.loadIdentities();

    fireEvent.input(screen.getByLabelText('To'), { target: { value: 'you@example.org' } });
    fireEvent.change(await screen.findByLabelText('From'), { target: { value: 'id2' } });
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));

    // The compose flow completes (draft + submission built with the identity)
    // and surfaces the undo-send toast rather than an error.
    await waitFor(() => expect(app.pendingUndo()?.actionLabel).toBe('Cancel'));
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('switches the Send button to Schedule when a send-later time is set', () => {
    renderWithApp(() => <Compose onClose={() => undefined} />);
    const later = screen.getByLabelText('Send later');
    fireEvent.input(later, { target: { value: '2099-01-01T09:00' } });
    expect(screen.getByRole('button', { name: 'Schedule' })).toBeInTheDocument();
  });
});
