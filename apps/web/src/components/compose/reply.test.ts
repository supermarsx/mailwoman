import { describe, it, expect } from 'vitest';
import {
  formatAddress,
  formatAddressList,
  forwardSubject,
  isOwnAddress,
  replyRecipients,
  replySubject,
  replyThreading,
} from './reply.ts';
import { parseRecipients } from '../../api/jmap.ts';
import type { Email, EmailAddress } from '../../api/jmap-types.ts';

const addr = (email: string, name: string | null = null): EmailAddress => ({ name, email });

function mail(over: Partial<Email>): Email {
  return {
    id: 'm1',
    mailboxIds: { inbox: true },
    from: [addr('alice@example.org', 'Alice')],
    to: [addr('me@example.org')],
    subject: 'Hello',
    receivedAt: '2026-01-01T00:00:00Z',
    preview: '',
    ...over,
  };
}

const OWN = { addresses: ['me@example.org', 'alias@corp.example'] };

describe('reply and forward subjects', () => {
  it('adds one Re: and does not stack it', () => {
    expect(replySubject('Hello')).toBe('Re: Hello');
    expect(replySubject('Re: Hello')).toBe('Re: Hello');
    expect(replySubject('RE: re:  Re : Hello')).toBe('Re: Hello');
    expect(replySubject('Fwd: Hello')).toBe('Re: Fwd: Hello');
    expect(replySubject('Regarding Hello')).toBe('Re: Regarding Hello');
    expect(replySubject(null)).toBe('Re:');
  });

  it('adds one Fwd: and does not stack it', () => {
    expect(forwardSubject('Hello')).toBe('Fwd: Hello');
    expect(forwardSubject('Fwd: Hello')).toBe('Fwd: Hello');
    expect(forwardSubject('FW: fwd: Fw: Hello')).toBe('Fwd: Hello');
    expect(forwardSubject('Re: Hello')).toBe('Fwd: Re: Hello');
    expect(forwardSubject('Fwding this')).toBe('Fwd: Fwding this');
  });
});

describe('own addresses', () => {
  it('matches a full address case-insensitively and nothing else', () => {
    expect(isOwnAddress('ME@Example.org', OWN)).toBe(true);
    expect(isOwnAddress('me@other.example', OWN)).toBe(false);
    expect(isOwnAddress('someone@example.org', OWN)).toBe(false);
  });

  it('matches a bare login name against the local part', () => {
    const own = { addresses: ['testuser'] };
    expect(isOwnAddress('testuser@example.org', own)).toBe(true);
    expect(isOwnAddress('testuser2@example.org', own)).toBe(false);
    expect(isOwnAddress('other@testuser', own)).toBe(false);
  });

  it('an empty own entry matches nothing', () => {
    expect(isOwnAddress('@example.org', { addresses: [''] })).toBe(false);
  });
});

describe('reply recipients', () => {
  it('reply goes to the author only', () => {
    const email = mail({ to: [addr('me@example.org'), addr('bob@example.org')], cc: [addr('carol@example.org')] });
    expect(replyRecipients(email, 'reply', OWN)).toEqual({ to: [addr('alice@example.org', 'Alice')], cc: [] });
  });

  it('reply honours Reply-To over From', () => {
    const email = mail({ replyTo: [addr('list@example.org', 'The List')] });
    expect(replyRecipients(email, 'reply', OWN)).toEqual({ to: [addr('list@example.org', 'The List')], cc: [] });
    expect(replyRecipients(email, 'reply-all', OWN)).toEqual({ to: [addr('list@example.org', 'The List')], cc: [] });
  });

  it('reply-all keeps the other To and Cc recipients and leaves out every own address', () => {
    const email = mail({
      to: [addr('ME@example.org', 'Me'), addr('bob@example.org', 'Bob')],
      cc: [addr('carol@example.org'), addr('alias@corp.example'), addr('BOB@example.org'), addr('alice@example.org')],
    });
    expect(replyRecipients(email, 'reply-all', OWN)).toEqual({
      to: [addr('alice@example.org', 'Alice'), addr('bob@example.org', 'Bob')],
      // bob and alice are already in To; the alias is the user's own.
      cc: [addr('carol@example.org')],
    });
  });

  it('a message the user sent is answered to its recipients, not to the user', () => {
    const email = mail({
      from: [addr('me@example.org', 'Me')],
      to: [addr('bob@example.org')],
      cc: [addr('carol@example.org')],
    });
    expect(replyRecipients(email, 'reply', OWN)).toEqual({ to: [addr('bob@example.org')], cc: [] });
    expect(replyRecipients(email, 'reply-all', OWN)).toEqual({
      to: [addr('bob@example.org')],
      cc: [addr('carol@example.org')],
    });
  });

  it('a message with no From and no Reply-To has no reply recipient', () => {
    expect(replyRecipients(mail({ from: null }), 'reply', OWN)).toEqual({ to: [], cc: [] });
  });
});

describe('address field syntax', () => {
  it('writes a bare address, a named one, and quotes a name the field parser would split', () => {
    expect(formatAddress(addr('a@x.org'))).toBe('a@x.org');
    expect(formatAddress(addr('a@x.org', '  '))).toBe('a@x.org');
    expect(formatAddress(addr('a@x.org', 'Alice Example'))).toBe('Alice Example <a@x.org>');
    expect(formatAddress(addr('j@x.org', 'Doe, Jane "JD"'))).toBe('"Doe, Jane \\"JD\\"" <j@x.org>');
  });

  it('round-trips through the composer field parser', () => {
    const list = [addr('j@x.org', 'Doe, Jane; "JD"'), addr('b@y.org'), addr('c@z.org', 'Carol (work)')];
    expect(parseRecipients(formatAddressList(list))).toEqual(list);
  });

  it('drops angle brackets from a display name, so the address is still the address', () => {
    expect(parseRecipients(formatAddress(addr('j@x.org', 'Jane <jane@evil.example>')))).toEqual([
      addr('j@x.org', 'Jane jane@evil.example'),
    ]);
  });
});

describe('reply threading', () => {
  it('answers the original id and appends it to the original references', () => {
    expect(replyThreading({ messageId: 'c@x', references: ['a@x', 'b@x'] })).toEqual({
      inReplyTo: ['c@x'],
      references: ['a@x', 'b@x', 'c@x'],
    });
  });

  it('does not repeat an id the references already hold', () => {
    expect(replyThreading({ messageId: 'c@x', references: ['a@x', 'c@x'] })).toEqual({
      inReplyTo: ['c@x'],
      references: ['a@x', 'c@x'],
    });
  });

  it('carries nothing when the original id is unknown', () => {
    expect(replyThreading(null)).toEqual({ inReplyTo: [], references: [] });
    expect(replyThreading({ messageId: null, references: ['a@x'] })).toEqual({ inReplyTo: [], references: [] });
  });
});
