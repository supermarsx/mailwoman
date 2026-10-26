import { afterEach, describe, expect, it, vi } from 'vitest';
import { analyticsPrivacySignal, analyticsScreen, createAnalyticsReporter } from './analytics.ts';

afterEach(() => vi.useRealTimers());
describe('optional event counts', () => {
  it('admits only fixed screen names and honors both privacy signals', () => {
    expect(analyticsScreen('/mail?token=secret')).toBeNull();
    expect(analyticsScreen('mail')).toBe('mail');
    expect(analyticsPrivacySignal({doNotTrack:'1'})).toBe(true);
    expect(analyticsPrivacySignal({doNotTrack:null,globalPrivacyControl:true})).toBe(true);
  });
  it('does no network work without consent, suppresses repeats, and cancels on opt-out', async () => {
    vi.useFakeTimers();
    const enabled = vi.fn(async () => true);
    const send = vi.fn(async () => {});
    const reporter = createAnalyticsReporter({enabled,send});
    reporter.update(1,'mail',false,false);
    await vi.advanceTimersByTimeAsync(1000);
    expect(enabled).not.toHaveBeenCalled();
    reporter.update(1,'mail',true,false);
    await vi.advanceTimersByTimeAsync(750);
    expect(send).toHaveBeenCalledTimes(1);
    reporter.update(1,'mail',true,false);
    await vi.advanceTimersByTimeAsync(1000);
    expect(send).toHaveBeenCalledTimes(1);
    reporter.update(1,'contacts',true,false);
    reporter.update(1,'contacts',false,false);
    await vi.advanceTimersByTimeAsync(1000);
    expect(send).toHaveBeenCalledTimes(1);
    reporter.stop();
  });
  it('ignores a stale account response and never retries a failed collector attempt', async () => {
    vi.useFakeTimers();
    let finish!: (enabled:boolean) => void;
    const enabled = vi.fn(() => new Promise<boolean>(resolve => { finish = resolve; }));
    const send = vi.fn(async () => { throw new Error('offline'); });
    const reporter = createAnalyticsReporter({enabled,send});
    reporter.update(1,'mail',true,false);
    await vi.advanceTimersByTimeAsync(750);
    reporter.stop(); finish(true);
    await vi.advanceTimersByTimeAsync(1);
    expect(send).not.toHaveBeenCalled();
    reporter.update(2,'mail',true,false);
    await vi.advanceTimersByTimeAsync(750); finish(true);
    await vi.advanceTimersByTimeAsync(10000);
    expect(send).toHaveBeenCalledTimes(1);
    reporter.stop();
  });
});
