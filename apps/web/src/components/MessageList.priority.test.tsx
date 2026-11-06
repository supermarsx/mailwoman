import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { cleanup, screen, waitFor } from '@solidjs/testing-library';
import { MessageList } from './MessageList.tsx';
import { renderWithApp, mkEmail } from './appHarness.tsx';
import { clearSettings, loadSettings, type Fetcher } from '../api/settings.ts';
import { highestPriority, priorityOf, showsPriorityMark } from './messagePriority.ts';

const json = (body: unknown): Response => new Response(JSON.stringify(body), { status: 200, headers: { 'content-type': 'application/json' } });
async function marks(value?: string, style?: string): Promise<void> {
  const settings: Record<string, unknown> = {};
  if (value) settings['mail.priorityMarks'] = { value, source: 'user', locked: false };
  if (style) settings['mail.priorityMarks.style'] = { value: style, source: 'user', locked: false };
  await loadSettings({ refresh: true, fetcher: (async () => json({ settings })) as Fetcher });
}
async function rows() {
  const harness = renderWithApp(() => <MessageList />, {
    emails: [
      mkEmail('h', { subject: 'Urgent one', mailwomanPriority: 'high' }),
      mkEmail('l', { subject: 'Whenever', mailwomanPriority: 'low' }),
      mkEmail('n', { subject: 'Plain' }),
    ],
  });
  await harness.app.login({ username: 'me@example.org', password: 'p' });
  await screen.findByText('Plain');
  return harness;
}

describe('message priority helpers', () => {
  it('reads the server property and defaults to normal', () => {
    expect(priorityOf({ mailwomanPriority: 'high' })).toBe('high');
    expect(priorityOf({})).toBe('normal');
    expect(priorityOf(null)).toBe('normal');
  });
  it('takes the highest priority of a conversation', () => {
    expect(highestPriority([{ mailwomanPriority: 'low' }, {}, { mailwomanPriority: 'high' }])).toBe('high');
    expect(highestPriority([{ mailwomanPriority: 'low' }, {}])).toBe('low');
    expect(highestPriority([{}])).toBe('normal');
  });
  it('honours mail.priorityMarks', () => {
    expect(showsPriorityMark('high', 'both')).toBe(true);
    expect(showsPriorityMark('low', 'both')).toBe(true);
    expect(showsPriorityMark('low', 'high')).toBe(false);
    expect(showsPriorityMark('high', 'high')).toBe(true);
    expect(showsPriorityMark('high', 'off')).toBe(false);
    expect(showsPriorityMark('normal', 'both')).toBe(false);
  });
});

describe('MessageList priority marks', () => {
  beforeEach(() => { localStorage.clear(); clearSettings(); });
  afterEach(() => cleanup());
  it('marks high and low rows with an accessible name, not normal ones', async () => {
    await marks();
    const { result } = await rows();
    await waitFor(() => expect(result.container.querySelectorAll('[data-priority]').length).toBe(2));
    const high = result.container.querySelector('[data-priority="high"]')!;
    expect(high.getAttribute('role')).toBe('img');
    expect(high.getAttribute('aria-label')).toBe('High priority');
    expect(result.container.querySelector('[data-priority="low"]')!.getAttribute('aria-label')).toBe('Low priority');
    expect(high.closest('[data-email-id="h"]')).not.toBeNull();
  });
  it('"high only" hides the low mark and "off" hides both', async () => {
    await marks('high');
    const first = await rows();
    await waitFor(() => expect(first.result.container.querySelectorAll('[data-priority]').length).toBe(1));
    expect(first.result.container.querySelector('[data-priority="low"]')).toBeNull();
    cleanup();
    await marks('off');
    const second = await rows();
    expect(second.result.container.querySelectorAll('[data-priority]').length).toBe(0);
  });
  it('the icon+text style also writes the words', async () => {
    await marks('both', 'iconText');
    const { result } = await rows();
    await waitFor(() => expect(result.container.querySelector('[data-priority="high"]')!.textContent).toContain('High priority'));
  });
});
