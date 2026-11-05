import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { clearSettings, loadSettings, type Fetcher } from '../api/settings.ts';
import {
  checkSendAt,
  effectiveRecallSeconds,
  formatCountdown,
  horizonSeconds,
  recallAllowed,
  recallCapSeconds,
  recallChoices,
  sendLaterPresets,
  toLocalInput,
} from './sendDelay.ts';

const json = (body: unknown): Response => new Response(JSON.stringify(body), { status: 200, headers: { 'content-type': 'application/json' } });
async function load(values: Record<string, unknown>): Promise<void> {
  const settings = Object.fromEntries(Object.entries(values).map(([key, value]) => [key, { value, source: 'user', locked: false }]));
  await loadSettings({ refresh: true, fetcher: (async () => json({ settings })) as Fetcher });
}
beforeEach(() => clearSettings());
afterEach(() => clearSettings());

describe('recall window', () => {
  it('defaults to 10 seconds, allowed, capped at 30 minutes, 90 days ahead', () => {
    expect(effectiveRecallSeconds()).toBe(10);
    expect(recallAllowed()).toBe(true);
    expect(recallCapSeconds()).toBe(1800);
    expect(horizonSeconds()).toBe(90 * 86400);
  });
  it('uses the user value within the administrator cap', async () => {
    await load({ 'mail.send.recallDelaySeconds': 120 });
    expect(effectiveRecallSeconds()).toBe(120);
    await load({ 'mail.send.recallDelaySeconds': 600, 'mail.send.recallDelayMaxSeconds': 60 });
    expect(effectiveRecallSeconds()).toBe(60);
  });
  it('is 0 when the administrator turns recall off, or the user picks Off', async () => {
    await load({ 'mail.send.allowRecall': false, 'mail.send.recallDelaySeconds': 30 });
    expect(effectiveRecallSeconds()).toBe(0);
    await load({ 'mail.send.recallDelaySeconds': 0 });
    expect(effectiveRecallSeconds()).toBe(0);
  });
  it('offers the presets under the cap, and the cap itself between presets', () => {
    expect(recallChoices(1800)).toEqual([0, 5, 10, 20, 30, 60, 120, 300, 600, 1800]);
    expect(recallChoices(45)).toEqual([0, 5, 10, 20, 30, 45]);
    expect(recallChoices(0)).toEqual([0]);
  });
  it('formats a countdown as m:ss', () => {
    expect(formatCountdown(25)).toBe('0:25');
    expect(formatCountdown(125.2)).toBe('2:06');
    expect(formatCountdown(-3)).toBe('0:00');
  });
});

describe('send later', () => {
  const horizon = 90 * 86400;
  it('accepts only the future within the horizon', () => {
    const now = new Date(2026, 0, 5, 10, 0);
    expect(checkSendAt(new Date(2026, 0, 5, 11, 0), now, horizon)).toBe('ok');
    expect(checkSendAt(new Date(2026, 0, 5, 10, 0), now, horizon)).toBe('past');
    expect(checkSendAt(new Date('nope'), now, horizon)).toBe('invalid');
    expect(checkSendAt(new Date(2026, 6, 5), now, horizon)).toBe('horizon');
    expect(checkSendAt(new Date(2026, 0, 6), now, 0)).toBe('off');
  });
  it('builds presets in local time', () => {
    const wed = new Date(2026, 0, 7, 9, 30);
    const presets = sendLaterPresets(wed, horizon);
    expect(presets.map((p) => p.id)).toEqual(['hour', 'today', 'tomorrow', 'monday']);
    expect(toLocalInput(presets.find((p) => p.id === 'monday')!.at)).toBe('2026-01-12T09:00');
  });
  it('drops "later today" in the evening and Monday when tomorrow is Monday', () => {
    const sun = new Date(2026, 0, 11, 20, 0);
    expect(sendLaterPresets(sun, horizon).map((p) => p.id)).toEqual(['hour', 'tomorrow']);
  });
  it('drops presets beyond a short horizon', () => {
    const wed = new Date(2026, 0, 7, 9, 30);
    expect(sendLaterPresets(wed, 7200).map((p) => p.id)).toEqual(['hour']);
  });
});
