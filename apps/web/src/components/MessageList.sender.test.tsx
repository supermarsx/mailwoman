import { describe, it, expect, beforeEach } from 'vitest';
import { waitFor } from '@solidjs/testing-library';
import { MessageList, senderLabel } from './MessageList.tsx';
import { renderWithApp, mkEmail } from './appHarness.tsx';
import { stripIsolates } from '../i18n/index.ts';

describe('senderLabel', () => {
  it('shows the address beside the display name', () => {
    expect(senderLabel([{ name: 'Alice Example', email: 'alice@example.org' }])).toBe(
      'Alice Example <alice@example.org>',
    );
  });

  it('shows the bare address when there is no name, or the name is the address', () => {
    expect(senderLabel([{ name: null, email: 'alice@example.org' }])).toBe('alice@example.org');
    expect(senderLabel([{ name: '  ', email: 'alice@example.org' }])).toBe('alice@example.org');
    expect(senderLabel([{ name: 'Alice@Example.org', email: 'alice@example.org' }])).toBe('alice@example.org');
  });

  it('puts the real address first when the name claims a different one', () => {
    expect(senderLabel([{ name: 'ceo@bank.example', email: 'x@evil.example' }])).toBe(
      'x@evil.example (ceo@bank.example)',
    );
    expect(senderLabel([{ name: 'Bank Support <help@bank.example>', email: 'x@evil.example' }])).toBe(
      'x@evil.example (Bank Support <help@bank.example>)',
    );
  });

  it('keeps name-first order when the name only repeats the real address', () => {
    expect(senderLabel([{ name: 'Alice (alice@example.org)', email: 'alice@example.org' }])).toBe(
      'Alice (alice@example.org) <alice@example.org>',
    );
  });

  it('names an absent sender', () => {
    expect(senderLabel(null)).toBe('(unknown sender)');
    expect(senderLabel([])).toBe('(unknown sender)');
  });
});

describe('MessageList — the sender cell', () => {
  beforeEach(() => localStorage.clear());

  it('renders the label, address included, for a named and for a spoofing sender', async () => {
    const { app, result } = renderWithApp(() => <MessageList />, {
      emails: [
        mkEmail('a', { from: [{ name: 'Your Bank', email: 'x@evil.example' }] }),
        mkEmail('b', { from: [{ name: 'security@bank.example', email: 'y@evil.example' }] }),
      ],
    });
    await app.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });
    await waitFor(() => expect(app.messages().length).toBe(2));
    const cells = [...result.container.querySelectorAll('.list__sender')].map((el) =>
      stripIsolates(el.textContent ?? ''),
    );
    expect(cells).toEqual(['Your Bank <x@evil.example>', 'y@evil.example (security@bank.example)']);
  });
});
