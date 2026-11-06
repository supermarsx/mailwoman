import {afterEach, beforeEach, describe, expect, it, vi} from 'vitest';
import {fireEvent, render, screen, waitFor, within} from '@solidjs/testing-library';
import {clearSettings, loadSettings, type Fetcher} from '../api/settings.ts';
import {hasListHeaders, mailingListId, unsubscribeMethods, safeListHref} from '../api/mailingList.ts';
import {badgeMode, badgeVisible} from '../state/readerBadges.ts';
import {markUnsubscribed, listKey, stillReceiving, unsubscribedList, senderKept, UNSUBSCRIBE_GRACE_MS} from '../state/listSubscription.ts';
import {MailingListBadge} from './MailingListBadge.tsx';
const oneClick = vi.hoisted(() => vi.fn());
vi.mock('../api/mailingList.ts', async original => ({...await original<typeof import('../api/mailingList.ts')>(), unsubscribeOneClick: oneClick}));

const base = 'List-Id: "Acme News" <news.acme.example>\r\n';
const ONE_CLICK = `${base}List-Unsubscribe: <https://lists.acme.example/u/abc>, <mailto:leave@acme.example?subject=unsubscribe>\r\nList-Unsubscribe-Post: List-Unsubscribe=One-Click\r\nList-Archive: <https://lists.acme.example/archive>\r\nList-Help: <mailto:help@acme.example>\r\nList-Post: <mailto:post@acme.example>\r\nList-Owner: <javascript:alert(1)>, <mailto:owner@acme.example>`;
const HTTPS_ONLY = `${base}List-Unsubscribe: <https://lists.acme.example/u/abc>`;
const MAIL_ONLY = `${base}List-Unsubscribe: <mailto:leave@acme.example?subject=unsubscribe&body=please>`;
const ID_ONLY = base;
const json = (body: unknown): Response => new Response(JSON.stringify(body), {status: 200, headers: {'content-type': 'application/json'}});
async function load(values: Record<string, unknown>, locked = false): Promise<void> {
  const map = Object.fromEntries(Object.entries(values).map(([key, value]) => [key, {value, source: locked ? 'global' : 'user', locked}]));
  await loadSettings({refresh: true, fetcher: (async () => json({settings: map})) as Fetcher});
}
const props = (headers: string, extra: Partial<Parameters<typeof MailingListBadge>[0]> = {}): Parameters<typeof MailingListBadge>[0] =>
  ({headers, accountId: 'a1', sender: 'news@acme.example', receivedAt: '2026-11-01T10:00:00Z', stats: {count: 3, last: '2026-11-02T10:00:00Z'}, ...extra});
const open = async (): Promise<HTMLElement> => {
  fireEvent.click(await screen.findByTestId('reader-list-badge'));
  return screen.getByRole('dialog');
};

beforeEach(() => { localStorage.clear(); oneClick.mockReset(); });
afterEach(() => { clearSettings(); vi.restoreAllMocks(); });

describe('header parsing', () => {
  it('reads the List-Id and the unsubscribe methods per header combination', () => {
    expect(mailingListId(ONE_CLICK)).toBe('news.acme.example');
    const full = unsubscribeMethods(ONE_CLICK);
    expect(full.oneClick?.href).toBe('https://lists.acme.example/u/abc');
    expect(full.web?.host).toBe('lists.acme.example');
    expect(full.mail?.host).toBe('leave@acme.example');
    const https = unsubscribeMethods(HTTPS_ONLY);
    expect(https.oneClick).toBeNull();
    expect(https.web).not.toBeNull();
    expect(https.mail).toBeNull();
    const mail = unsubscribeMethods(MAIL_ONLY);
    expect(mail.web).toBeNull();
    expect(mail.mail).not.toBeNull();
    expect(unsubscribeMethods(ID_ONLY)).toEqual({oneClick: null, web: null, mail: null});
  });
  it('needs https and a List-Unsubscribe-Post of exactly one value for one-click', () => {
    expect(unsubscribeMethods('List-Unsubscribe: <http://x.example/u>\r\nList-Unsubscribe-Post: List-Unsubscribe=One-Click').oneClick).toBeNull();
    expect(unsubscribeMethods('List-Unsubscribe: <https://x.example/u>\r\nList-Unsubscribe-Post: something=else').oneClick).toBeNull();
  });
  it('a badge needs a List-Id or a List-Unsubscribe', () => {
    expect(hasListHeaders(ID_ONLY)).toBe(true);
    expect(hasListHeaders(MAIL_ONLY)).toBe(true);
    expect(hasListHeaders('Subject: hi')).toBe(false);
  });
  it('refuses executable schemes and credentials as link text', () => {
    expect(safeListHref('javascript:alert(1)')).toBeNull();
    expect(safeListHref('https://u:p@x.example/')).toBeNull();
    expect(safeListHref('https://x.example/a')?.kind).toBe('web');
  });
});

describe('badge visibility settings', () => {
  it('defaults to always, and follows mail.badge.list', async () => {
    expect(badgeMode('list')).toBe('always');
    await load({'mail.badge.list': 'notable'});
    expect(badgeVisible('list', false)).toBe(false);
    expect(badgeVisible('list', true)).toBe(true);
    await load({'mail.badge.list': 'never'});
    expect(badgeVisible('list', true)).toBe(false);
  });
  it('renders nothing without list headers, and nothing when the mode is never', async () => {
    const {unmount} = render(() => <MailingListBadge {...props('Subject: hi')}/>);
    expect(screen.queryByTestId('reader-list-badge')).toBeNull();
    unmount();
    await load({'mail.badge.list': 'never'});
    render(() => <MailingListBadge {...props(ONE_CLICK)}/>);
    expect(screen.queryByTestId('reader-list-badge')).toBeNull();
  });
  it('notable shows the badge only once mail keeps arriving after an unsubscribe', async () => {
    await load({'mail.badge.list': 'notable'});
    render(() => <MailingListBadge {...props(ONE_CLICK, {receivedAt: '2026-11-01T10:00:00Z', stats: {count: 1, last: '2026-11-01T10:00:00Z'}})}/>);
    expect(screen.queryByTestId('reader-list-badge')).toBeNull();
  });
  it('the right-click menu offers to hide the list badge', async () => {
    render(() => <MailingListBadge {...props(ONE_CLICK)}/>);
    fireEvent.contextMenu(await screen.findByTestId('reader-list-badge'));
    expect(await screen.findByRole('menuitem', {name: /Hide this badge/})).toBeInTheDocument();
  });
});

describe('the badge', () => {
  it('is a compact neutral button in the badge family with the list name and an accessible tooltip', async () => {
    render(() => <MailingListBadge {...props(ONE_CLICK)}/>);
    const badge = await screen.findByTestId('reader-list-badge');
    expect(badge.tagName).toBe('BUTTON');
    expect(badge).toHaveClass('mw-badge');
    expect(badge).toHaveAttribute('data-tone', 'neutral');
    expect(badge).toHaveAttribute('aria-haspopup', 'dialog');
    expect(badge).toHaveAttribute('aria-expanded', 'false');
    expect(badge).toHaveTextContent('Mailing list · ⁨Acme News⁩');
    expect(badge.getAttribute('data-tooltip')).toBe('Mailing list: ⁨Acme News⁩ · Manage subscription');
    expect(badge.getAttribute('aria-label')).toBe(badge.getAttribute('data-tooltip'));
    expect(badge.querySelectorAll('svg')).toHaveLength(1);
  });
  it('opens with Enter or Space as any button, closes with Escape and returns focus', async () => {
    render(() => <MailingListBadge {...props(ONE_CLICK)}/>);
    const badge = await screen.findByTestId('reader-list-badge');
    badge.focus();
    fireEvent.click(badge);
    const dialog = screen.getByRole('dialog');
    expect(dialog).toHaveAttribute('aria-modal', 'true');
    expect(within(dialog).getByRole('heading', {name: 'Mailing list'})).toBeVisible();
    fireEvent.keyDown(dialog, {key: 'Escape'});
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(badge).toHaveFocus();
  });
});

describe('the popup', () => {
  it('one-click: shows the identity, the safe links as text, the methods and the counts', async () => {
    render(() => <MailingListBadge {...props(ONE_CLICK)}/>);
    const dialog = await open();
    expect(dialog).toHaveTextContent('Acme News');
    expect(dialog).toHaveTextContent('news.acme.example');
    expect(dialog).toHaveTextContent('news@acme.example');
    expect(within(dialog).getByTestId('reader-list-methods').querySelectorAll('li')).toHaveLength(3);
    expect(within(dialog).getByTestId('reader-list-methods').querySelector('[data-method="one-click"]')).toHaveTextContent('lists.acme.example');
    expect(within(dialog).getByTestId('reader-list-stats')).toHaveTextContent(/^3 messages from this sender are in the mailbox\./);
    expect(within(dialog).getByTestId('reader-list-status')).toHaveTextContent('No unsubscribe is recorded');
    const links = within(dialog).getByTestId('reader-list-links');
    expect(links).toHaveTextContent('https://lists.acme.example/archive');
    expect(links).toHaveTextContent('help@acme.example');
    expect(links).toHaveTextContent('owner@acme.example');
    expect(links.textContent).not.toContain('javascript');
    const anchors = [...links.querySelectorAll('a')];
    expect(anchors).toHaveLength(1);
    expect(anchors[0]).toHaveAttribute('target', '_blank');
    expect(anchors[0]).toHaveAttribute('rel', 'noopener noreferrer');
    expect(anchors[0]).toHaveAttribute('referrerpolicy', 'no-referrer');
  });
  it('https only: one method, no one-click', async () => {
    render(() => <MailingListBadge {...props(HTTPS_ONLY)}/>);
    const dialog = await open();
    const methods = within(dialog).getByTestId('reader-list-methods');
    expect(methods.querySelectorAll('li')).toHaveLength(1);
    expect(methods.querySelector('[data-method="link"]')).not.toBeNull();
    expect(methods.querySelector('[data-method="one-click"]')).toBeNull();
  });
  it('mailto only: lists the address', async () => {
    render(() => <MailingListBadge {...props(MAIL_ONLY)}/>);
    const dialog = await open();
    expect(within(dialog).getByTestId('reader-list-methods').querySelector('[data-method="mail"]')).toHaveTextContent('leave@acme.example');
  });
  it('no unsubscribe header: only the List-Id information, and Unsubscribe is disabled', async () => {
    render(() => <MailingListBadge {...props(ID_ONLY)}/>);
    const dialog = await open();
    expect(within(dialog).getByTestId('reader-list-methods')).toHaveTextContent('does not offer a way to unsubscribe');
    expect(within(dialog).queryByTestId('reader-list-links')).toBeNull();
    expect(within(dialog).getByTestId('reader-list-unsubscribe')).toBeDisabled();
    expect(dialog).toHaveTextContent('news.acme.example');
  });
  it('offers every action', async () => {
    render(() => <MailingListBadge {...props(ONE_CLICK, {onCleanup: () => undefined, onSenderControl: async () => true, onOpenRules: () => undefined})}/>);
    const dialog = await open();
    const names = within(dialog).getAllByRole('button').map(b => b.textContent);
    for (const name of ['Unsubscribe', 'Unsubscribe and clean up', 'Move future messages to a folder…', 'Mute this list', 'Always keep', 'Block sender', 'Create filter from this list…', 'Report as spam', 'Clean up this sender']) expect(names).toContain(name);
  });
});

describe('unsubscribe flow', () => {
  it('one-click asks first, then calls the server proxy with the https URL and remembers it', async () => {
    const send = oneClick; oneClick.mockResolvedValue({ok: true});
    render(() => <MailingListBadge {...props(ONE_CLICK)}/>);
    const dialog = await open();
    fireEvent.click(within(dialog).getByTestId('reader-list-unsubscribe'));
    expect(send).not.toHaveBeenCalled();
    expect(within(dialog).getByTestId('reader-list-confirm')).toHaveTextContent('does not see your IP address');
    fireEvent.click(within(dialog).getByTestId('reader-list-confirm-send'));
    await waitFor(() => expect(send).toHaveBeenCalledWith('https://lists.acme.example/u/abc'));
    await waitFor(() => expect(unsubscribedList(listKey('a1', 'news.acme.example'))?.method).toBe('one-click'));
    expect(screen.getByTestId('reader-list-badge')).toHaveTextContent('Unsubscribed');
    expect(within(screen.getByRole('dialog')).getByTestId('reader-list-status')).toHaveTextContent(/Unsubscribed on/);
  });
  it('one-click-direct sends without a confirmation; open-link never POSTs', async () => {
    const send = oneClick; oneClick.mockResolvedValue({ok: true});
    await load({'mail.list.unsubscribeMode': 'one-click-direct'});
    const first = render(() => <MailingListBadge {...props(ONE_CLICK)}/>);
    fireEvent.click(within(await open()).getByTestId('reader-list-unsubscribe'));
    await waitFor(() => expect(send).toHaveBeenCalledTimes(1));
    first.unmount();
    localStorage.clear();
    await load({'mail.list.unsubscribeMode': 'open-link'});
    render(() => <MailingListBadge {...props(ONE_CLICK)}/>);
    fireEvent.click(within(await open()).getByTestId('reader-list-unsubscribe'));
    expect(send).toHaveBeenCalledTimes(1);
    expect(screen.getByRole('dialog', {name: 'Unsubscribe'})).toBeInTheDocument();
  });
  it('a failed request is not remembered', async () => {
    oneClick.mockResolvedValue({ok: false, reason: 'rejected'});
    render(() => <MailingListBadge {...props(ONE_CLICK)}/>);
    const dialog = await open();
    fireEvent.click(within(dialog).getByTestId('reader-list-unsubscribe'));
    fireEvent.click(within(dialog).getByTestId('reader-list-confirm-send'));
    await waitFor(() => expect(within(dialog).getByTestId('reader-list-unsubscribe')).not.toBeDisabled());
    expect(unsubscribedList(listKey('a1', 'news.acme.example'))).toBeNull();
  });
  it('https only goes through the destination review, and is remembered only after the user says so', async () => {
    const send = oneClick;
    render(() => <MailingListBadge {...props(HTTPS_ONLY)}/>);
    const dialog = await open();
    fireEvent.click(within(dialog).getByTestId('reader-list-unsubscribe'));
    const review = screen.getByRole('dialog', {name: 'Unsubscribe'});
    expect(review).toHaveTextContent('This leaves Mailwoman');
    expect(send).not.toHaveBeenCalled();
    fireEvent.click(within(review).getByRole('button', {name: /lists\.acme\.example/}));
    const link = within(review).getByRole('link', {name: 'Open in a new tab'});
    expect(link).toHaveAttribute('rel', 'noopener noreferrer');
    expect(unsubscribedList(listKey('a1', 'news.acme.example'))).toBeNull();
    fireEvent.click(within(review).getByRole('button', {name: /Close/}));
    fireEvent.click(await screen.findByRole('button', {name: 'I unsubscribed, remember it'}));
    expect(unsubscribedList(listKey('a1', 'news.acme.example'))?.method).toBe('link');
  });
  it('mailto composes a draft only after review', async () => {
    const compose = vi.fn();
    render(() => <MailingListBadge {...props(MAIL_ONLY, {onCompose: compose})}/>);
    fireEvent.click(within(await open()).getByTestId('reader-list-unsubscribe'));
    const review = screen.getByRole('dialog', {name: 'Unsubscribe'});
    fireEvent.click(within(review).getByRole('button', {name: 'leave@acme.example'}));
    expect(compose).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', {name: 'Review a new message'}));
    expect(compose).toHaveBeenCalledWith(expect.objectContaining({mode: 'new', to: 'leave@acme.example', subject: 'unsubscribe', bodyText: 'please'}));
  });
  it('Unsubscribe and clean up opens the clean up review after a one-click success', async () => {
    oneClick.mockResolvedValue({ok: true});
    const cleanup = vi.fn();
    render(() => <MailingListBadge {...props(ONE_CLICK, {onCleanup: cleanup})}/>);
    const dialog = await open();
    fireEvent.click(within(dialog).getByTestId('reader-list-unsubscribe-clean'));
    expect(cleanup).not.toHaveBeenCalled();
    fireEvent.click(within(dialog).getByTestId('reader-list-confirm-send'));
    await waitFor(() => expect(cleanup).toHaveBeenCalledTimes(1));
  });
  it('Clean up this sender opens the review; Always keep turns it off', async () => {
    const cleanup = vi.fn();
    render(() => <MailingListBadge {...props(ONE_CLICK, {onCleanup: cleanup})}/>);
    const dialog = await open();
    fireEvent.click(within(dialog).getByTestId('reader-list-keep'));
    expect(senderKept('news@acme.example')).toBe(true);
    expect(within(dialog).getByTestId('reader-list-sweep')).toBeDisabled();
    fireEvent.click(within(dialog).getByTestId('reader-list-keep'));
    fireEvent.click(within(dialog).getByTestId('reader-list-sweep'));
    expect(cleanup).toHaveBeenCalledTimes(1);
  });
  it('sender actions call the sender control', async () => {
    const control = vi.fn().mockResolvedValue(true);
    render(() => <MailingListBadge {...props(ONE_CLICK, {onSenderControl: control})}/>);
    const dialog = await open();
    fireEvent.click(within(dialog).getByTestId('reader-list-block'));
    await waitFor(() => expect(control).toHaveBeenCalledWith('block'));
    await waitFor(() => expect(within(dialog).getByTestId('reader-list-spam')).not.toBeDisabled());
    fireEvent.click(within(dialog).getByTestId('reader-list-spam'));
    await waitFor(() => expect(control).toHaveBeenCalledWith('report-junk'));
    await waitFor(() => expect(within(dialog).getByTestId('reader-list-mute')).not.toBeDisabled());
    fireEvent.click(within(dialog).getByTestId('reader-list-mute'));
    await waitFor(() => expect(control).toHaveBeenCalledWith('silence'));
  });
});

describe('unsubscribed state', () => {
  it('persists per account and List-Id and shows in the badge and the tooltip', async () => {
    markUnsubscribed(listKey('a1', 'news.acme.example'), 'one-click', Date.parse('2026-10-30T10:00:00Z'));
    expect(JSON.parse(localStorage.getItem('mw.list.unsubscribed')!)['a1|news.acme.example'].method).toBe('one-click');
    render(() => <MailingListBadge {...props(ONE_CLICK, {receivedAt: '2026-10-31T10:00:00Z', stats: {count: 1, last: '2026-10-31T10:00:00Z'}})}/>);
    const badge = await screen.findByTestId('reader-list-badge');
    expect(badge).toHaveAttribute('data-state', 'unsubscribed');
    expect(badge).toHaveTextContent('Unsubscribed');
    expect(badge.getAttribute('data-tooltip')).toContain('Unsubscribed on');
    expect(listKey('a2', 'news.acme.example')).not.toBe(listKey('a1', 'news.acme.example'));
  });
  it('warns when mail arrives after the grace period', async () => {
    markUnsubscribed(listKey('a1', 'news.acme.example'), 'one-click', Date.parse('2026-10-01T10:00:00Z'));
    render(() => <MailingListBadge {...props(ONE_CLICK, {receivedAt: '2026-11-01T10:00:00Z'})}/>);
    const badge = await screen.findByTestId('reader-list-badge');
    expect(badge).toHaveAttribute('data-state', 'ignored');
    expect(badge).toHaveAttribute('data-tone', 'warning');
    expect(badge.getAttribute('data-tooltip')).toContain('still arriving');
    const dialog = await open();
    expect(within(dialog).getByTestId('reader-list-status')).toHaveTextContent('the list may be ignoring your request');
  });
  it('the grace period is three days', () => {
    const at = Date.parse('2026-10-01T00:00:00Z');
    const entry = {at, method: 'one-click' as const};
    expect(stillReceiving(entry, new Date(at + UNSUBSCRIBE_GRACE_MS - 1000).toISOString())).toBe(false);
    expect(stillReceiving(entry, new Date(at + UNSUBSCRIBE_GRACE_MS + 1000).toISOString())).toBe(true);
    expect(stillReceiving(null, '2026-11-01T00:00:00Z')).toBe(false);
  });
  it('Forget clears the record', async () => {
    markUnsubscribed(listKey('a1', 'news.acme.example'), 'link');
    render(() => <MailingListBadge {...props(ONE_CLICK)}/>);
    const dialog = await open();
    fireEvent.click(within(dialog).getByRole('button', {name: 'Forget'}));
    expect(unsubscribedList(listKey('a1', 'news.acme.example'))).toBeNull();
  });
});

describe('the clean up this sender wording', () => {
  it('the action no longer calls itself a review of actions, and says what the review is for', async () => {
    const { readFileSync } = await import('node:fs');
    const mail = readFileSync('locales/en/mail.ftl', 'utf8');
    expect(mail).toMatch(/^mail-sweep-sender = Clean up this sender\r?$/m);
    expect(mail).not.toContain('Review actions for other messages');
    expect(mail).toMatch(/^mail-action-help-sweep = Review the other messages from this sender and choose what to clean up/m);
  });
});
