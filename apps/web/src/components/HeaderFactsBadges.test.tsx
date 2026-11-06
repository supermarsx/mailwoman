import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, render, screen, waitFor } from '@solidjs/testing-library';

const state = vi.hoisted(() => ({ map: {} as Record<string, { value: unknown }> }));
vi.mock('../api/settings.ts', () => ({ settings: () => state.map, loadSettings: vi.fn(async () => undefined) }));

import { HeaderFactsBadges } from './HeaderFactsBadges.tsx';
import { factsFromEmail, hasFacts, isExpired, parseHeaderDate, parseHeaderFacts } from './readerHeaders.ts';

afterEach(() => { cleanup(); state.map = {}; });

const NOW = () => new Date('2026-11-06T12:00:00Z');

describe('parseHeaderFacts', () => {
  it('reads Expires, Reply-By, Sensitivity and Supersedes from a folded header block', () => {
    const raw = 'From: a@b.c\r\nExpires: Tue, 03 Nov 2026\r\n 17:00:00 +0000\r\nReply-By: Wed, 04 Nov 2026 09:00:00 +0000; Thu, 05 Nov 2026 09:00:00 +0000\r\nSensitivity: Company-Confidential\r\nSupersedes: <old@example.com>\r\n\r\nExpires: ignored body line';
    const f = parseHeaderFacts(raw);
    expect(f.expires?.toISOString()).toBe('2026-11-03T17:00:00.000Z');
    expect(f.replyBy?.toISOString()).toBe('2026-11-04T09:00:00.000Z');
    expect(f.sensitivity).toBe('company-confidential');
    expect(f.supersedes).toBe('<old@example.com>');
    expect(isExpired(f, NOW())).toBe(true);
  });
  it('falls back to Expiry-Date and ignores unreadable or unknown values', () => {
    expect(parseHeaderFacts('Expiry-Date: Tue, 03 Dec 2026 17:00:00 +0000\n\n').expires?.getUTCMonth()).toBe(11);
    const f = parseHeaderFacts('Expires: soon\nReply-By: (comment)\nSensitivity: top-secret\nSupersedes: not an id\n\n');
    expect(f).toEqual({});
    expect(hasFacts(f)).toBe(false);
    expect(parseHeaderDate('x'.repeat(200))).toBeUndefined();
  });
  it('reads the values an Email/get returned and cannot be fooled by a line break in one', () => {
    const f = factsFromEmail({ 'header:Sensitivity:asText': 'private\r\nExpires: Tue, 03 Nov 2026 17:00:00 +0000', 'header:Reply-By:asText': null });
    expect(f.sensitivity).toBeUndefined();
    expect(f.expires).toBeUndefined();
    expect(factsFromEmail({ 'header:Expiry-Date:asText': 'Tue, 03 Dec 2026 17:00:00 +0000' }).expires).toBeInstanceOf(Date);
  });
});

describe('HeaderFactsBadges', () => {
  it('shows nothing without an interesting header', () => {
    render(() => <HeaderFactsBadges email={{}} />);
    expect(screen.queryByTestId('header-facts')).toBeNull();
  });

  it('shows sensitivity, reply-by, expiry and supersedes badges', async () => {
    render(() => <HeaderFactsBadges now={NOW} onSearch={() => undefined} email={{
      'header:Sensitivity:asText': 'private', 'header:Reply-By:asText': 'Wed, 04 Dec 2026 09:00:00 +0000',
      'header:Expires:asText': 'Tue, 03 Dec 2026 17:00:00 +0000', 'header:Supersedes:asText': '<old@example.com>',
    }} />);
    expect(await screen.findByTestId('hf-sensitivity')).toHaveTextContent('Private');
    expect(screen.getByTestId('hf-reply-by')).toHaveTextContent('Reply by');
    expect(screen.getByTestId('hf-expires')).toHaveTextContent(/^\s*Expires/);
    expect(screen.getByTestId('hf-expires')).not.toHaveAttribute('data-expired');
    expect(screen.getByTestId('hf-supersedes')).toHaveTextContent('Replaces an earlier message');
  });

  it('marks an expired message and follows the expired-messages setting', async () => {
    state.map = { 'mail.expiry.hideExpired': { value: 'archive-suggestion' } };
    render(() => <HeaderFactsBadges now={NOW} email={{ 'header:Expires:asText': 'Tue, 03 Nov 2026 17:00:00 +0000' }} />);
    const badge = await screen.findByTestId('hf-expires');
    expect(badge).toHaveTextContent(/Expired/);
    expect(badge).toHaveAttribute('data-expired');
    expect(badge).toHaveAttribute('data-mode', 'archive-suggestion');
    expect(screen.getByTestId('hf-archive-suggestion')).toBeInTheDocument();
  });

  it('is hidden by mail.badge.headers = false', () => {
    state.map = { 'mail.badge.headers': { value: false } };
    render(() => <HeaderFactsBadges email={{ 'header:Sensitivity:asText': 'private' }} />);
    expect(screen.queryByTestId('header-facts')).toBeNull();
  });
  it('shows Superseded by a newer message when the lookup finds a replacement', async () => {
    const lookup = vi.fn(async () => true);
    render(() => <HeaderFactsBadges now={NOW} email={{}} messageId="old@example.org" supersededLookup={lookup} />);
    expect(await screen.findByTestId('hf-superseded-by')).toHaveTextContent('Superseded by a newer message');
    expect(lookup).toHaveBeenCalledWith('old@example.org');
  });

  it('shows nothing when no newer message replaces it, or the headers badge is off', async () => {
    const none = vi.fn(async () => false);
    render(() => <HeaderFactsBadges now={NOW} email={{}} messageId="current@example.org" supersededLookup={none} />);
    await waitFor(() => expect(none).toHaveBeenCalled());
    expect(screen.queryByTestId('hf-superseded-by')).toBeNull();
    cleanup();
    state.map = { 'mail.badge.headers': { value: false } };
    const off = vi.fn(async () => true);
    render(() => <HeaderFactsBadges now={NOW} email={{}} messageId="old@example.org" supersededLookup={off} />);
    expect(screen.queryByTestId('hf-superseded-by')).toBeNull();
    expect(off).not.toHaveBeenCalled();
  });
});
